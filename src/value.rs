#[derive(Debug)]
pub enum FieldValue {
    VarInt(u64),
    Bytes(Vec<u8>),
}

impl FieldValue {
    pub fn varint(&self) -> Result<u64, Error> {
        match self {
            FieldValue::VarInt(x) => Ok(*x),
            _ => Err(Error::InvalidType),
        }
    }

    pub fn bytes(&self) -> Result<&[u8], Error> {
        match self {
            FieldValue::Bytes(x) => Ok(x),
            _ => Err(Error::InvalidType),
        }
    }

    pub fn into_bytes(self) -> Result<Vec<u8>, Error> {
        match self {
            FieldValue::Bytes(x) => Ok(x),
            _ => Err(Error::InvalidType),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidType,
}

#[derive(Debug)]
pub struct FieldValueInfo {
    pub value: FieldValue,
    pub pos: usize,
}
