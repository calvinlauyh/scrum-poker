use sha2::{Digest, Sha256};

pub fn hash(message: &str, salts: &[&str]) -> String {
    let mut hasher = Sha256::new();

    hasher.input(message);
    for salt in salts.iter() {
        hasher.input(salt);
    }

    hex::encode(hasher.result().as_slice())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn hash_should_hash_message_with_salts() {
        assert_eq!(
            hash("apple", &vec!["1", "2", "3"]),
            String::from("599a4410e2af69d1585f16d82d4b5f0abf3ad09fa42b9d55d7b7a50671ccf8c1")
        );
    }

}
