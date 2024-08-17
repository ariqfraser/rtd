use hex;

pub fn info_str_to_bytes(info_hash: &str) -> [u8; 20] {
    let bytes = hex::decode(info_hash).expect("Could not decode hash");
    let validated: [u8; 20] = bytes.try_into().expect("Failed to conver info hash to [u8;20]");
    validated
}

pub fn info_bytes_to_string(bytes: &[u8; 20]) -> String {
    hex::encode(bytes)
}