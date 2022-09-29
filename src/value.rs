use std::borrow::Cow;

#[derive(Debug)]
pub enum FieldValue<'buf> {
    VarInt(u64),
    Bytes(Cow<'buf, [u8]>),
}

impl<'buf> FieldValue<'buf> {
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

    pub fn into_bytes(self) -> Result<Cow<'buf, [u8]>, Error> {
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
pub struct FieldValueInfo<'buf> {
    pub value: FieldValue<'buf>,
    pub pos: usize,
}
