#![allow(unused_variables)]
use std::io::{BufReader, Read};

use crate::chunk_type::ChunkType;
use crate::Error;

pub struct Chunk {
    chunk_type: ChunkType,
    chunk_data: Vec<u8>,
}

impl Chunk {
    pub const METADATA_BYTES: usize = 12;
    pub fn new(chunk_type: ChunkType, chunk_data: Vec<u8>) -> Chunk {
        Chunk {
            chunk_type,
            chunk_data,
        }
    }

    pub fn length(&self) -> u32 {
        self.chunk_data.len() as u32
    }
    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }
    fn data(&self) -> &[u8] {
        self.chunk_data.as_slice()
    }
    fn crc(&self) -> u32 {
        let bytez: Vec<u8> = self
            .chunk_type
            .bytes()
            .iter()
            .chain(self.chunk_data.iter())
            .copied()
            .collect();
        crc::crc32::checksum_ieee(&bytez)
    }

    pub fn data_as_string(&self) -> Result<String, Error> {
        // match String::from_utf8(self.chunk_data.clone()) {
        //     Ok(v) => Ok(v),
        //     Err(e) => Err(Box::new(ChunkError::InvalidStringError))
        // }
        Ok(String::from_utf8(self.chunk_data.clone()).map_err(Box::new)?)
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        self.length()
            .to_be_bytes()
            .iter()
            .chain(self.chunk_type().bytes().iter())
            .chain(self.data().iter())
            .chain(self.crc().to_be_bytes().iter())
            .copied()
            .collect()
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut reader = BufReader::new(value);
        // 4-byte buffer for temp reading
        let mut buf: [u8; 4] = [0; 4];

        // Length of chunk
        reader.read_exact(&mut buf)?;
        let length = usize::try_from(u32::from_be_bytes(buf))?;

        // ChunkType
        reader.read_exact(&mut buf)?;
        let chunk_type: ChunkType = ChunkType::try_from(buf)?;

        // Prepare chunk_data vec and read

        let mut chunk_data: Vec<u8> = vec![0; length];
        reader.read_exact(&mut chunk_data)?;
        if chunk_data.len() != length {
            return Err(Box::new(ChunkError::LengthError(length, chunk_data.len())));
        }

        let new_chunk = Chunk {
            chunk_type,
            chunk_data,
        };

        // Read and check crc
        reader.read_exact(&mut buf)?;
        let crc_provided = u32::from_be_bytes(buf);
        let crc_computed = new_chunk.crc();

        if crc_provided != crc_computed {
            return Err(Box::new(ChunkError::CrcMismatchError(
                crc_provided,
                crc_computed,
            )));
        }

        Ok(new_chunk)
    }
}

impl std::fmt::Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Chunk Type : {}\nData : {}",
            self.chunk_type(),
            self.data_as_string()
                .unwrap_or_else(|_| "[data]".to_string())
        )
    }
}

#[derive(Debug)]
enum ChunkError {
    LengthError(usize, usize),
    CrcMismatchError(u32, u32),
}

impl std::error::Error for ChunkError {}

impl std::fmt::Display for ChunkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChunkError::LengthError(expected, got) => {
                write!(
                    f,
                    "Length Error! Expected {} bytes, Got {} bytes",
                    expected, got
                )
            }
            ChunkError::CrcMismatchError(expected, got) => {
                write!(f, "CRC Mismatch Error! Expected {}, Got {}", expected, got)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();
        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!"
            .as_bytes()
            .to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }
}
