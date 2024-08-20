use candid::{CandidType, Decode, Encode};
use serde::Deserialize;

/// Encodes a Candid type to bytes
pub fn encode<T: CandidType>(item: &T) -> Vec<u8> {
    Encode!(item).expect("failed to encode item to candid")
}

/// Decodes a Candid type from bytes
pub fn decode<'a, T: CandidType + Deserialize<'a>>(bytes: &'a [u8]) -> T {
    Decode!(bytes, T).expect("failed to decode item from candid")
}