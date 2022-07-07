use rand::RngCore;

pub fn new_cred_id() -> String {
    let mut id_array: [u8; 24] = [0; 24];
    rand::thread_rng().fill_bytes(&mut id_array);
    base64::encode(&id_array)
}
