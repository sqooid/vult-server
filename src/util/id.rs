use rand::RngCore;

pub fn random_b64(bytes: usize) -> String {
    let mut id_array: Vec<u8> = vec![0; bytes];
    rand::thread_rng().fill_bytes(&mut id_array);
    base64::encode(&id_array)
}
