use std::borrow::Cow;

#[derive(Debug)]
pub enum Val<'buf> {
    VarInt(u64),
    Bytes(Cow<'buf, [u8]>),
}

impl<'buf> Val<'buf> {
    pub fn varint(&self) -> Result<u64, Error> {
        match self {
            Val::VarInt(x) => Ok(*x),
            _ => Err(Error::InvalidType),
        }
    }

    pub fn bytes(&self) -> Result<&[u8], Error> {
        match self {
            Val::Bytes(x) => Ok(x),
            _ => Err(Error::InvalidType),
        }
    }

    pub fn into_bytes(self) -> Result<Cow<'buf, [u8]>, Error> {
        match self {
            Val::Bytes(x) => Ok(x),
            _ => Err(Error::InvalidType),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidType,
}

#[derive(Debug)]
pub struct ValInfo<'buf> {
    pub value: Val<'buf>,
    pub pos: usize,
}
