use sha2::{Sha256, Digest};

pub fn decrypt_all_layers(data: &[u8], keys: &[[u8; 32]]) -> Vec<u8> {
    assert!(keys.len() >= 20, "Need at least 20 keys");

    let mut buf = data.to_vec();

    buf = layer_20_integrity_check(&buf);
    buf = layer_19_timing_check(&buf);
    buf = layer_18_unmangle(&buf);
    buf = layer_17_unstring_encrypt(&buf);
    buf = layer_16_defingerprint(&buf);
    buf = layer_15_deconstruct_key(&buf, &keys[14]);
    buf = layer_14_remove_fake(&buf);
    buf = layer_13_deobfuscate_flow(&buf);
    buf = layer_12_unpolymorphic(&buf, &keys[11]);
    buf = layer_11_remove_padding(&buf);
    buf = layer_10_remove_dead_code(&buf);
    buf = layer_09_remove_antidebug(&buf);
    buf = layer_08_depolymorph(&buf, &keys[7]);
    buf = layer_07_verify_chain(&buf, &keys[6]);
    buf = layer_06_xor_decrypt(&buf, &keys[5]);
    buf = layer_05_aes_decrypt(&buf, &keys[4]);
    buf = layer_04_untranspose(&buf);
    buf = layer_03_xor_decrypt(&buf, &keys[2]);
    buf = layer_02_aes_decrypt(&buf, &keys[1]);
    buf = layer_01_xor_decrypt(&buf, &keys[0]);

    buf
}

fn layer_01_xor_decrypt(data: &[u8], key: &[u8; 32]) -> Vec<u8> {
    let mask = derive_layer_mask(key, 0x01);
    data.iter()
        .enumerate()
        .map(|(i, &b)| b ^ mask[i % mask.len()])
        .collect()
}

fn layer_02_aes_decrypt(data: &[u8], key: &[u8; 32]) -> Vec<u8> {
    use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
    use aes_gcm::aead::Aead;

    if data.len() < 28 {
        return data.to_vec();
    }

    let nonce_bytes = &data[..12];
    let ciphertext = &data[12..];
    let nonce = Nonce::from_slice(nonce_bytes);

    match Aes256Gcm::new_from_slice(key) {
        Ok(cipher) => {
            match cipher.decrypt(nonce, ciphertext) {
                Ok(plaintext) => plaintext,
                Err(_) => data.to_vec(),
            }
        }
        Err(_) => data.to_vec(),
    }
}

fn layer_03_xor_decrypt(data: &[u8], key: &[u8; 32]) -> Vec<u8> {
    let mask = derive_layer_mask(key, 0x03);
    data.iter()
        .enumerate()
        .map(|(i, &b)| {
            let shift = (i % 8) as u32;
            b.wrapping_shr(shift) | b.wrapping_shl(8 - shift)
        })
        .zip(mask.iter().cycle())
        .map(|(b, &m)| b ^ m)
        .collect()
}

fn layer_04_untranspose(data: &[u8]) -> Vec<u8> {
    if data.len() < 4 {
        return data.to_vec();
    }

    let block_size = 4;
    let mut result = Vec::with_capacity(data.len());

    for chunk in data.chunks(block_size) {
        let mut block = [0u8; 4];
        for (i, &b) in chunk.iter().enumerate() {
            if i < 4 {
                block[3 - i] = b;
            }
        }
        result.extend_from_slice(&block[..chunk.len()]);
    }

    result
}

fn layer_05_aes_decrypt(data: &[u8], key: &[u8; 32]) -> Vec<u8> {
    use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
    use aes_gcm::aead::Aead;

    if data.len() < 28 {
        return data.to_vec();
    }

    let nonce_bytes = &data[data.len() - 12..];
    let ciphertext = &data[..data.len() - 12];
    let nonce = Nonce::from_slice(nonce_bytes);

    match Aes256Gcm::new_from_slice(key) {
        Ok(cipher) => {
            match cipher.decrypt(nonce, ciphertext) {
                Ok(plaintext) => plaintext,
                Err(_) => data.to_vec(),
            }
        }
        Err(_) => data.to_vec(),
    }
}

fn layer_06_xor_decrypt(data: &[u8], key: &[u8; 32]) -> Vec<u8> {
    let mut result = Vec::with_capacity(data.len());
    let mut prev = key[0];

    for &b in data {
        let decrypted = b ^ prev ^ key[result.len() % 32];
        result.push(decrypted);
        prev = b;
    }

    result
}

fn layer_07_verify_chain(data: &[u8], key: &[u8; 32]) -> Vec<u8> {
    if data.len() < 32 {
        return data.to_vec();
    }

    let (payload, hash_bytes) = data.split_at(data.len() - 32);
    let mut hasher = Sha256::new();
    hasher.update(payload);
    hasher.update(key);
    let computed = hasher.finalize();

    if computed.as_slice() == hash_bytes {
        payload.to_vec()
    } else {
        data.to_vec()
    }
}

fn layer_08_depolymorph(data: &[u8], key: &[u8; 32]) -> Vec<u8> {
    let seed = key.iter().fold(0u32, |acc, &b| acc.wrapping_add(b as u32));
    let mut result = data.to_vec();

    let mut rng_state = seed;
    for i in (1..result.len()).rev() {
        rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        let j = (rng_state as usize) % (i + 1);
        result.swap(i, j);
    }

    result
}

fn layer_09_remove_antidebug(data: &[u8]) -> Vec<u8> {
    let marker = b"DEBUG_MARKER_1234567890ABCDEF";
    if let Some(pos) = data.windows(marker.len()).position(|w| w == marker) {
        let mut result = data[..pos].to_vec();
        result.extend_from_slice(&data[pos + marker.len()..]);
        result
    } else {
        data.to_vec()
    }
}

fn layer_10_remove_dead_code(data: &[u8]) -> Vec<u8> {
    let dead_pattern: &[u8] = &[0xDE, 0xAD, 0xCA, 0xFE, 0xBA, 0xBE];
    let mut result = Vec::new();
    let mut i = 0;

    while i < data.len() {
        if i + dead_pattern.len() <= data.len() && &data[i..i + dead_pattern.len()] == dead_pattern {
            let skip_len = 32.min(data.len() - i);
            i += skip_len;
        } else {
            result.push(data[i]);
            i += 1;
        }
    }

    result
}

fn layer_11_remove_padding(data: &[u8]) -> Vec<u8> {
    if data.len() < 16 {
        return data.to_vec();
    }

    let last_byte = *data.last().unwrap();
    if last_byte > 0 && last_byte <= 16 && (data.len() as u8) >= last_byte {
        let trim_len = data.len() - last_byte as usize;
        let potential_pad = &data[trim_len..];
        if potential_pad.iter().all(|&b| b == last_byte) {
            return data[..trim_len].to_vec();
        }
    }

    data.to_vec()
}

fn layer_12_unpolymorphic(data: &[u8], key: &[u8; 32]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &b)| {
            let key_byte = key[i % 32];
            let shift = key_byte % 8;
            b.wrapping_shl(shift as u32) | b.wrapping_shr((8 - shift) as u32)
        })
        .collect()
}

fn layer_13_deobfuscate_flow(data: &[u8]) -> Vec<u8> {
    let mut result = data.to_vec();
    let mut i = 0;
    while i + 1 < result.len() {
        result.swap(i, i + 1);
        i += 2;
    }
    result
}

fn layer_14_remove_fake(data: &[u8]) -> Vec<u8> {
    if data.len() < 32 {
        return data.to_vec();
    }

    let checksum: u32 = data.iter().map(|&b| b as u32).sum();
    if checksum % 7 == 0 {
        data[..data.len().saturating_sub(32)].to_vec()
    } else {
        data.to_vec()
    }
}

fn layer_15_deconstruct_key(data: &[u8], _key: &[u8; 32]) -> Vec<u8> {
    let magic = b"secret_key_construction_data_bloc";
    if data.len() >= magic.len() {
        let prefix = &data[..magic.len()];
        if prefix.iter().zip(magic.iter()).all(|(a, b)| a == b) {
            return data[magic.len()..].to_vec();
        }
    }
    data.to_vec()
}

fn layer_16_defingerprint(data: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &b)| b.wrapping_sub((i % 16) as u8))
        .collect()
}

fn layer_17_unstring_encrypt(data: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &b)| {
            match i % 4 {
                0 => b ^ 0x55,
                1 => b ^ 0xAA,
                2 => b ^ 0x33,
                _ => b ^ 0xCC,
            }
        })
        .collect()
}

fn layer_18_unmangle(data: &[u8]) -> Vec<u8> {
    if data.len() < 2 {
        return data.to_vec();
    }

    let mut result = Vec::with_capacity(data.len());
    let chunks: Vec<&[u8]> = data.chunks(2).collect();

    for chunk in chunks.iter().rev() {
        result.extend_from_slice(chunk);
    }

    result
}

fn layer_19_timing_check(data: &[u8]) -> Vec<u8> {
    if data.len() < 8 {
        return data.to_vec();
    }

    let verify = &data[data.len() - 8..];
    let payload = &data[..data.len() - 8];

    let expected: Vec<u8> = verify.iter().enumerate().map(|(i, &b)| b.wrapping_add(i as u8)).collect();
    let actual: Vec<u8> = payload.iter().take(8).copied().collect();

    if expected == actual {
        payload.to_vec()
    } else {
        data.to_vec()
    }
}

fn layer_20_integrity_check(data: &[u8]) -> Vec<u8> {
    if data.len() < 64 {
        return data.to_vec();
    }

    let payload = &data[..data.len() - 32];
    let stored_hash = &data[data.len() - 32..];

    let mut hasher = Sha256::new();
    hasher.update(payload);
    let computed = hasher.finalize();

    if computed.as_slice() == stored_hash {
        payload.to_vec()
    } else {
        data.to_vec()
    }
}

pub fn derive_layer_mask(key: &[u8; 32], layer_id: u8) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(key);
    hasher.update([layer_id]);
    let hash = hasher.finalize();

    let mut mask = Vec::with_capacity(32);
    for i in 0..32 {
        mask.push(hash[i ^ (layer_id as usize % 32)]);
    }
    mask
}

pub fn hash_key(user_key: &str, index: usize) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(user_key.as_bytes());
    hasher.update(&(index as u64).to_le_bytes());
    hasher.update(b"LidBridge_Salt_v1");
    let hash = hasher.finalize();

    let mut key = [0u8; 32];
    key.copy_from_slice(&hash);
    key
}
