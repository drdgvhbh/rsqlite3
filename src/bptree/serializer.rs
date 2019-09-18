use super::{Entry, Key, Value};
use rmp_serde;

#[derive(Clone)]
pub enum Serializer {
    RMP,
    Mock,
}

impl Serializer {
    pub fn serialize<K: Key, V: Value>(&self, entries: &Vec<Entry<K, V>>) -> Vec<u8> {
        match self {
            Serializer::Mock => {
                let mut buf = Vec::new();
                for _ in entries {
                    buf.push(1 as u8);
                }

                buf
            }
            Serializer::RMP => {
                let mut buf = Vec::new();
                entries
                    .serialize(&mut rmp_serde::Serializer::new(&mut buf))
                    .unwrap();

                buf
            }
        }
    }
}
