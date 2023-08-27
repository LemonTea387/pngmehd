use crate::Error;
use std::{fmt::Display, str::FromStr};

#[derive(PartialEq, Eq, Debug)]
pub struct ChunkType {
    bytez: [u8; 4],
}

impl Display for ChunkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            self.bytez[0] as char,
            self.bytez[1] as char,
            self.bytez[2] as char,
            self.bytez[3] as char
        )
    }
}

impl ChunkType {
    pub fn bytes(&self) -> [u8; 4] {
        self.bytez
    }

    /// Returns the is critical of this [`ChunkType`].
    /// Critical chunks have their 5th bit of first byte be un-set (0) instead of set (1) which
    /// indicates ancillary.
    fn is_critical(&self) -> bool {
        !is_bit_set(self.bytez[0], 5)
    }

    /// Returns the inverse of private flag
    /// Public chunks are signified by 5th bit of second byte to be un-set(0) instead of set (1)
    /// which indicates private
    fn is_public(&self) -> bool {
        !is_bit_set(self.bytez[1], 5)
    }

    /// 5th bit is 0 for third bytes to be reserved
    fn is_reserved_bit_valid(&self) -> bool {
        !is_bit_set(self.bytez[2], 5)
    }

    /// 5th bit is set (1) for safe to copy
    fn is_safe_to_copy(&self) -> bool {
        is_bit_set(self.bytez[3], 5)
    }

    fn is_valid(&self) -> bool {
        let validChars = self.is_valid_characters(); 
        let validBit = self.is_reserved_bit_valid();
        println!("Is valid chars {validChars}, validBit {validBit}");
        
        self.is_valid_characters() && self.is_reserved_bit_valid()
    }

    fn is_valid_characters(&self) -> bool {
        self.bytez
            .iter()
            .all(|&b| b.is_ascii_alphabetic())
    }
}

/// Checking if bit n in byte x is set, if n is greater than 7, it will return false regardless.
fn is_bit_set(x: u8, n: u8) -> bool {
    if n >= 8 || (x >> n) & 1 == 0 {
        return false;
    }
    true
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = Error;

    fn try_from(value: [u8; 4]) -> Result<Self, Self::Error> {
        let chunk_type = ChunkType { bytez: value };
        match chunk_type.is_valid() {
            true => Ok(chunk_type),
            false => Err(Box::new(ChunkTypeError::InvalidBytesError)),
        }
    }
}

impl FromStr for ChunkType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytez = s.as_bytes();
        if bytez.len() != 4 {
            return Err(Box::new(ChunkTypeError::ByteLengthError(bytez.len())));
        }
        let bytez: [u8; 4] = [bytez[0], bytez[1], bytez[2], bytez[3]];

        let chunk = ChunkType { bytez };

        println!("{}", chunk);

        match chunk.is_valid_characters() {
            true => Ok(chunk),
            false => Err(Box::new(ChunkTypeError::InvalidBytesError)),
        }
    }
}

#[derive(Debug)]
enum ChunkTypeError {
    ByteLengthError(usize),
    InvalidBytesError,
}

impl std::error::Error for ChunkTypeError {}

impl Display for ChunkTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChunkTypeError::ByteLengthError(size) => {
                write!(f, "Expected 4 bytes but got {} instead.", size)
            }
            ChunkTypeError::InvalidBytesError => {
                write!(f, "Bytes are invalid as a chunk type!")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_valid());

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}
