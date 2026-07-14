use super::decrypt::hash_key;

pub fn derive_keys(passwords: &[String]) -> Result<[[u8; 32]; 20], String> {
    if passwords.len() < 20 {
        return Err(format!(
            "Need exactly 20 passwords, got {}.\nEnter all 20 passwords.",
            passwords.len()
        ));
    }

    let mut keys = [[0u8; 32]; 20];
    for (i, password) in passwords.iter().enumerate() {
        keys[i] = hash_key(password, i);
    }

    Ok(keys)
}

pub fn derive_master_keys(master: &str) -> [[u8; 32]; 20] {
    let mut keys = [[0u8; 32]; 20];
    for i in 0..20 {
        keys[i] = hash_key(master, i);
    }
    keys
}

pub fn validate_key(key: &str, index: usize) -> Result<(), String> {
    if key.trim().is_empty() {
        return Err(format!("Key {} cannot be empty", index + 1));
    }
    if key.len() < 8 {
        return Err(format!(
            "Key {} too short ({} chars). Minimum 8 characters.",
            index + 1,
            key.len()
        ));
    }
    Ok(())
}
