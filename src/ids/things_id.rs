use std::{fmt, str::FromStr};

use rand::random;
use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
    de::{self, Visitor},
};
use sha1::{Digest, Sha1};
use uuid::Uuid;

/// A Things 3 entity identifier.
///
/// Internally stored as canonical 16 bytes (SHA1-truncated UUID digest).
/// Hyphenated UUIDs and compact base58 IDs are accepted at parse-time.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ThingsId([u8; 16]);

impl ThingsId {
    pub fn random() -> Self {
        let uuid = Uuid::from_bytes(random());
        ThingsId(uuid_to_bytes(&uuid))
    }

    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    pub fn starts_with(&self, prefix: &str) -> bool {
        self.to_string().starts_with(prefix)
    }
}

impl fmt::Display for ThingsId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (buf, len) = base58_encode_fixed(&self.0);
        let encoded = std::str::from_utf8(&buf[..len]).expect("base58 output must be ASCII");
        f.write_str(encoded)
    }
}

impl AsRef<[u8; 16]> for ThingsId {
    fn as_ref(&self) -> &[u8; 16] {
        &self.0
    }
}

impl From<ThingsId> for String {
    fn from(id: ThingsId) -> Self {
        id.to_string()
    }
}

impl From<&ThingsId> for String {
    fn from(id: &ThingsId) -> Self {
        id.to_string()
    }
}

impl TryFrom<String> for ThingsId {
    type Error = ParseThingsIdError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse::<ThingsId>()
    }
}

impl TryFrom<&str> for ThingsId {
    type Error = ParseThingsIdError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse::<ThingsId>()
    }
}

impl Default for ThingsId {
    fn default() -> Self {
        Self([0u8; 16])
    }
}

impl FromStr for ThingsId {
    type Err = ParseThingsIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseThingsIdError(s.to_owned()));
        }
        if let Ok(uuid) = Uuid::parse_str(s) {
            return Ok(ThingsId(uuid_to_bytes(&uuid)));
        }
        if s.len() > 22 {
            return Err(ParseThingsIdError(s.to_owned()));
        }
        let decoded = base58_decode(s).ok_or_else(|| ParseThingsIdError(s.to_owned()))?;
        if decoded.len() != 16 {
            return Err(ParseThingsIdError(s.to_owned()));
        }
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&decoded);
        Ok(ThingsId(bytes))
    }
}

impl Serialize for ThingsId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ThingsId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ThingsIdVisitor;

        impl Visitor<'_> for ThingsIdVisitor {
            type Value = ThingsId;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "a Things ID string (compact base58 or hyphenated UUID)")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<ThingsId, E> {
                v.parse().map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_str(ThingsIdVisitor)
    }
}

/// Error returned when a string cannot be parsed as a [`ThingsId`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseThingsIdError(String);

impl fmt::Display for ParseThingsIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid Things ID: {:?}", self.0)
    }
}

impl std::error::Error for ParseThingsIdError {}

const BASE58_ALPHABET: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

/// Encode 16 bytes into base58 ASCII, writing into a stack-allocated
/// `[u8; 22]` buffer. Returns the buffer and the number of valid bytes.
pub(crate) fn base58_encode_fixed(raw: &[u8; 16]) -> ([u8; 22], usize) {
    let mut digits = [0u8; 22];
    let mut len = 0usize;

    for &byte in raw {
        let mut carry = byte as u32;
        for digit in digits[..len].iter_mut() {
            let value = (*digit as u32) * 256 + carry;
            *digit = (value % 58) as u8;
            carry = value / 58;
        }
        while carry > 0 {
            digits[len] = (carry % 58) as u8;
            len += 1;
            carry /= 58;
        }
    }

    let leading_ones = raw.iter().take_while(|&&b| b == 0).count();
    let total = leading_ones + len;
    debug_assert!(
        total <= 22,
        "base58_encode_fixed: output length {total} > 22"
    );

    let mut out = [0u8; 22];
    for b in out[..leading_ones].iter_mut() {
        *b = BASE58_ALPHABET[0];
    }
    for (i, &d) in digits[..len].iter().rev().enumerate() {
        out[leading_ones + i] = BASE58_ALPHABET[d as usize];
    }

    (out, total.max(1))
}

fn base58_digit(byte: u8) -> Option<u8> {
    BASE58_ALPHABET
        .iter()
        .position(|&c| c == byte)
        .map(|i| i as u8)
}

fn base58_decode(input: &str) -> Option<Vec<u8>> {
    if input.is_empty() {
        return Some(Vec::new());
    }

    let bytes = input.as_bytes();
    let mut leading_ones = 0usize;
    for b in bytes {
        if *b == b'1' {
            leading_ones += 1;
        } else {
            break;
        }
    }

    let mut decoded: Vec<u8> = Vec::new();
    for &ch in bytes.iter().skip(leading_ones) {
        let mut carry = base58_digit(ch)? as u32;
        for byte in &mut decoded {
            let value = (*byte as u32 * 58) + carry;
            *byte = (value & 0xff) as u8;
            carry = value >> 8;
        }
        while carry > 0 {
            decoded.push((carry & 0xff) as u8);
            carry >>= 8;
        }
    }

    let mut out = Vec::with_capacity(leading_ones + decoded.len());
    out.extend(std::iter::repeat_n(0u8, leading_ones));
    for byte in decoded.iter().rev() {
        out.push(*byte);
    }
    Some(out)
}

fn uuid_to_bytes(uuid: &Uuid) -> [u8; 16] {
    let canonical = uuid.to_string().to_uppercase();
    let digest = Sha1::digest(canonical.as_bytes());
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    const LEGACY_UUID: &str = "3C6BBD49-8D11-4FFF-8B0E-B8F33FA9C00A";
    const LEGACY_UUID_LOWER: &str = "3c6bbd49-8d11-4fff-8b0e-b8f33fa9c00a";
    fn compact_for_legacy() -> String {
        ThingsId::from_str(LEGACY_UUID).unwrap().to_string()
    }

    #[test]
    fn parse_legacy_uuid_uppercase() {
        let id: ThingsId = LEGACY_UUID.parse().unwrap();
        assert_eq!(id.to_string(), compact_for_legacy());
        assert_eq!(id.to_string().len(), 22);
    }

    #[test]
    fn parse_legacy_uuid_lowercase() {
        let upper: ThingsId = LEGACY_UUID.parse().unwrap();
        let lower: ThingsId = LEGACY_UUID_LOWER.parse().unwrap();
        assert_eq!(upper, lower, "UUID parsing must be case-insensitive");
    }

    #[test]
    fn parse_compact_preserved() {
        let compact = compact_for_legacy();
        let id: ThingsId = compact.parse().unwrap();
        assert_eq!(id.to_string(), compact);
    }

    #[test]
    fn empty_string_is_error() {
        let err = "".parse::<ThingsId>();
        assert!(err.is_err());
    }

    #[test]
    fn display_roundtrip() {
        let id: ThingsId = LEGACY_UUID.parse().unwrap();
        let displayed = id.to_string();
        let reparsed: ThingsId = displayed.parse().unwrap();
        assert_eq!(id, reparsed);
    }

    #[test]
    fn random_is_unique() {
        let ids: HashSet<String> = (0..20).map(|_| ThingsId::random().to_string()).collect();
        assert_eq!(ids.len(), 20, "random IDs should be unique");
    }

    #[test]
    fn random_is_compact_length() {
        let id = ThingsId::random();
        let len = id.to_string().len();
        assert!((1..=22).contains(&len), "compact ID length must be 1..=22");
    }

    #[test]
    fn serde_roundtrip_compact() {
        let id: ThingsId = LEGACY_UUID.parse().unwrap();
        let json = serde_json::to_string(&id).unwrap();
        let back: ThingsId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn serde_deserialize_from_legacy_uuid() {
        let json = format!("\"{}\"", LEGACY_UUID);
        let id: ThingsId = serde_json::from_str(&json).unwrap();
        assert_eq!(id.to_string().len(), 22);
        assert_eq!(id.to_string(), compact_for_legacy());
    }

    #[test]
    fn into_string() {
        let id: ThingsId = LEGACY_UUID.parse().unwrap();
        let s: String = id.clone().into();
        assert_eq!(s, id.to_string());
    }

    #[test]
    fn as_ref_bytes() {
        let id: ThingsId = LEGACY_UUID.parse().unwrap();
        let r: &[u8; 16] = id.as_ref();
        assert_eq!(r, id.as_bytes());
    }

    #[test]
    fn rejects_invalid_compact_id() {
        assert!("not-a-things-id".parse::<ThingsId>().is_err());
        assert!("0OIl".parse::<ThingsId>().is_err());
        assert!(
            "123456789ABCDEFGHJKLMNPQRSTUVWXYZ"
                .parse::<ThingsId>()
                .is_err()
        );
    }

    #[test]
    fn base58_roundtrip_for_internal_bytes() {
        let samples = [
            [0u8; 16],
            [255u8; 16],
            uuid_to_bytes(&Uuid::parse_str(LEGACY_UUID).unwrap()),
        ];
        for sample in samples {
            let (buf, len) = base58_encode_fixed(&sample);
            let encoded = std::str::from_utf8(&buf[..len]).unwrap().to_owned();
            let decoded = base58_decode(&encoded).unwrap();
            assert_eq!(decoded, sample);
        }
    }

    #[test]
    fn base58_encode_fixed_matches_display_encoding() {
        let mut samples: Vec<ThingsId> = vec![
            ThingsId([0u8; 16]),
            ThingsId([255u8; 16]),
            LEGACY_UUID.parse().unwrap(),
        ];
        for _ in 0..20 {
            samples.push(ThingsId::random());
        }

        for id in &samples {
            let (buf, len) = base58_encode_fixed(id.as_bytes());
            let fixed = std::str::from_utf8(&buf[..len]).unwrap().to_owned();
            let expected = id.to_string();
            assert_eq!(fixed, expected, "mismatch for {:?}", id.as_bytes());
        }
    }

    #[test]
    fn base58_encode_fixed_preserves_sort_order() {
        let ids: Vec<ThingsId> = (0..50).map(|_| ThingsId::random()).collect();
        let mut by_fixed: Vec<String> = ids
            .iter()
            .map(|id| {
                let (buf, len) = base58_encode_fixed(id.as_bytes());
                std::str::from_utf8(&buf[..len]).unwrap().to_owned()
            })
            .collect();
        by_fixed.sort();

        let mut by_string: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
        by_string.sort();

        assert_eq!(
            by_fixed, by_string,
            "base58_encode_fixed sort order != to_string sort order"
        );
    }
}
