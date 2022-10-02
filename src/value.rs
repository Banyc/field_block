#[derive(Debug)]
pub enum Val<'buf> {
    VarInt(u64),
    Bytes(&'buf [u8]),
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

        let vec = vec![1, 2, 3];
        let val = Val::Bytes(&vec);
        assert_eq!(val.varint().unwrap_err(), Error::InvalidType);

        let val = Val::VarInt(0x1234);
        assert_eq!(val.bytes().unwrap_err(), Error::InvalidType);
    }
}
