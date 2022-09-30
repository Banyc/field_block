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

#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidType,
}

#[derive(Debug)]
pub struct ValInfo<'buf> {
    pub value: Val<'buf>,
    pub pos: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let val = Val::VarInt(0x1234);
        assert_eq!(val.varint().unwrap(), 0x1234);

        let val = Val::Bytes(Cow::from(vec![1, 2, 3]));
        assert_eq!(val.varint().unwrap_err(), Error::InvalidType);

        let val = Val::VarInt(0x1234);
        assert_eq!(val.bytes().unwrap_err(), Error::InvalidType);
        assert_eq!(val.into_bytes().unwrap_err(), Error::InvalidType);

        let val = Val::Bytes(Cow::from(vec![1, 2, 3]));
        assert_eq!(val.into_bytes().unwrap(), vec![1, 2, 3]);
    }
}
