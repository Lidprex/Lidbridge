pub mod decrypt;
pub mod encrypted;
pub mod integrity;
pub mod keys;

use decrypt::decrypt_all_layers;
use encrypted::{ENCRYPTED_SECRET_BLOB, BLOB_VERSION, KEY_COUNT, MAGIC_BYTES};
use integrity::{is_debugger_attached, timing_check, verify_blob_integrity};
use keys::derive_keys;

pub fn get_secret(passwords: &[String]) -> Result<String, String> {
    if is_debugger_attached() {
        return Err("Debugger detected.".to_string());
    }
    if !timing_check() {
        return Err("Timing anomaly detected.".to_string());
    }
    if !verify_blob_integrity(ENCRYPTED_SECRET_BLOB, &MAGIC_BYTES) {
        return Err("Encrypted blob integrity check failed.".to_string());
    }
    if ENCRYPTED_SECRET_BLOB.len() < 4 || ENCRYPTED_SECRET_BLOB[3] != BLOB_VERSION {
        return Err("Unsupported blob version.".to_string());
    }
    let keys = derive_keys(passwords)?;
    let encrypted_data = &ENCRYPTED_SECRET_BLOB[4..];
    let decrypted = decrypt_all_layers(encrypted_data, &keys);
    let secret = String::from_utf8(decrypted)
        .map_err(|_| "Decrypted data is not valid UTF-8. Wrong passwords?".to_string())?;
    Ok(secret)
}

pub fn handle_encrypt_command(args: &[String]) {
    println!();
    println!("LidBridge Secret Encryptor");
    println!("=========================");
    if args.len() < 2 + KEY_COUNT {
        println!("Usage: lidbridge --encrypt <secret> <key1> <key2> ... <key20>");
        println!("You provided {} arguments, need {}.", args.len() - 2, KEY_COUNT);
        return;
    }
    let secret = &args[2];
    let passwords: Vec<String> = args[3..3 + KEY_COUNT].to_vec();
    let keys = match derive_keys(&passwords) {
        Ok(k) => k,
        Err(e) => { println!("Error: {}", e); return; }
    };
    let mut encrypted = secret.as_bytes().to_vec();
    for i in 0..20 {
        let mask = decrypt::derive_layer_mask(&keys[i], (i + 1) as u8);
        encrypted = encrypted.iter().enumerate().map(|(j, &b)| b ^ mask[j % mask.len()]).collect();
    }
    println!();
    println!("pub const ENCRYPTED_SECRET_BLOB: &[u8] = &[");
    print!("    ");
    for (i, byte) in encrypted.iter().enumerate() {
        print!("0x{:02X}, ", byte);
        if (i + 1) % 8 == 0 { print!("\n    "); }
    }
    println!("];");
}
