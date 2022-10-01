use std::borrow::Cow;

use octets::{Octets, OctetsMut};

use crate::{Error, FieldName, Val, ValInfo};

pub struct Field<F>
where
    F: FieldName,
{
    name: F,
    def: Def,
}

impl<F> Field<F>
where
    F: FieldName,
{
    #[must_use]
    pub fn new(name: F, def: Def) -> Self {
        Field { name, def }
    }

    #[must_use]
    pub fn name(&self) -> &F {
        &self.name
    }

    #[must_use]
    pub fn def(&self) -> &Def {
        &self.def
    }

    pub fn to_bytes(&self, value: Option<&Val>, b: &mut OctetsMut) -> Result<(), Error<F>> {
        match self.def() {
            Def::VarInt(x) => {
                let y = value;
                match (x, y) {
                    (U64::Fixed(x), Some(Val::VarInt(y))) => {
                        if *y != *x {
                            return Err(Error::InvalidValue(self.name().clone()));
                        }
                        if let Err(_) = b.put_varint(*y) {
                            return Err(Error::NotEnoughSpace(self.name().clone()));
                        };
                    }
                    (U64::Var, Some(Val::VarInt(y))) => {
                        if let Err(_) = b.put_varint(*y) {
                            return Err(Error::NotEnoughSpace(self.name().clone()));
                        };
                    }
                    (U64::Fixed(x), None) => {
                        if let Err(_) = b.put_varint(*x) {
                            return Err(Error::NotEnoughSpace(self.name().clone()));
                        };
                    }
                    (U64::Var, None) => {
                        return Err(Error::NoValueProvided(self.name().clone()));
                    }
                    (_, _) => {
                        return Err(Error::InvalidValue(self.name().clone()));
                    }
                };
            }
            Def::Bytes(len) => {
                match len {
                    Len::Fixed(len) => match value {
                        Some(Val::Bytes(x)) => {
                            if x.len() != *len {
                                return Err(Error::InvalidValue(self.name().clone()));
                            }
                            if let Err(_) = b.put_bytes(x) {
                                return Err(Error::NotEnoughSpace(self.name().clone()));
                            };
                        }
                        _ => {
                            return Err(Error::NoValueProvided(self.name().clone()));
                        }
                    },
                    Len::Var => {
                        match value {
                            Some(Val::Bytes(x)) => {
                                // length prefix
                                if let Err(_) = b.put_varint(x.len() as u64) {
                                    return Err(Error::NotEnoughSpace(self.name().clone()));
                                };
                                // data
                                if let Err(_) = b.put_bytes(x) {
                                    return Err(Error::NotEnoughSpace(self.name().clone()));
                                };
                            }
                            _ => {
                                return Err(Error::NoValueProvided(self.name().clone()));
                            }
                        }
                    }
                }
            }
            Def::FixedBytes(x) => {
                if let Some(y) = value {
                    match y {
                        Val::Bytes(y) => {
                            if y != x {
                                return Err(Error::InvalidValue(self.name().clone()));
                            }
                        }
                        _ => {
                            return Err(Error::InvalidValue(self.name().clone()));
                        }
                    }
                }
                if let Err(_) = b.put_bytes(x) {
                    return Err(Error::NotEnoughSpace(self.name().clone()));
                };
            }
        }
        Ok(())
    }

    pub fn to_value<'buf>(&self, b: &mut Octets<'buf>) -> Result<ValInfo<'buf>, Error<F>> {
        let pos = b.off();

        match self.def() {
            Def::VarInt(x) => {
                let y = match b.get_varint() {
                    Ok(y) => y,
                    Err(_) => return Err(Error::NotEnoughData(self.name().clone())),
                };
                match x {
                    U64::Fixed(x) => {
                        if *x != y {
                            return Err(Error::InvalidValue(self.name().clone()));
                        }
                        return Ok(ValInfo {
                            value: Val::VarInt(y),
                            pos,
                        });
                    }
                    U64::Var => {
                        return Ok(ValInfo {
                            value: Val::VarInt(y),
                            pos,
                        });
                    }
                };
            }
            Def::Bytes(len) => match len {
                Len::Fixed(len) => {
                    let x = match b.get_bytes(*len) {
                        Ok(x) => x,
                        Err(_) => return Err(Error::NotEnoughData(self.name().clone())),
                    };
                    let value = Val::Bytes(Cow::from(x.buf()));
                    return Ok(ValInfo { value, pos });
                }
                Len::Var => {
                    let x = match b.get_bytes_with_varint_length() {
                        Ok(x) => x,
                        Err(_) => return Err(Error::NotEnoughData(self.name().clone())),
                    };
                    let value = Val::Bytes(Cow::from(x.buf()));
                    return Ok(ValInfo { value, pos });
                }
            },
            Def::FixedBytes(x) => {
                let y = match b.get_bytes(x.len()) {
                    Ok(y) => y,
                    Err(_) => return Err(Error::NotEnoughData(self.name().clone())),
                };
                if y.buf() != x {
                    return Err(Error::InvalidValue(self.name().clone()));
                }
                return Ok(ValInfo {
                    value: Val::Bytes(Cow::from(y.buf())),
                    pos,
                });
            }
        }
    }
}

pub enum Def {
    VarInt(U64),
    Bytes(Len),
    FixedBytes(Vec<u8>),
}

pub enum U64 {
    Var,
    Fixed(u64),
}

pub enum Len {
    Fixed(usize),
    Var,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint() {
        let field = Field::new(Name::VarInt, Def::VarInt(U64::Var));
        {
            let mut buf = [0; 4];
            let mut b = OctetsMut::with_slice(&mut buf);
            {
                let value = Val::Bytes(Cow::from(vec![1]));
                let e = field.to_bytes(Some(&value), &mut b).unwrap_err();
                assert_eq!(e, Error::InvalidValue(Name::VarInt));
            }
            {
                let value = Val::VarInt(0x0102030405);
                let e = field.to_bytes(Some(&value), &mut b).unwrap_err();
                assert_eq!(e, Error::NotEnoughSpace(Name::VarInt));
            }
            {
                let e = field.to_bytes(None, &mut b).unwrap_err();
                assert_eq!(e, Error::NoValueProvided(Name::VarInt));
            }
        }
        {
            let mut buf = vec![0 | 0x80, 1];
            let mut b = Octets::with_slice(&mut buf);
            let e = field.to_value(&mut b).unwrap_err();
            assert_eq!(e, Error::NotEnoughData(Name::VarInt));
        }
    }

    #[test]
    fn test_fixed_varint() {
        let field = Field::new(Name::FixedVarInt, Def::VarInt(U64::Fixed(0x0102030405)));
        {
            let mut buf = [0; 4];
            let mut b = OctetsMut::with_slice(&mut buf);
            {
                let e = field.to_bytes(None, &mut b).unwrap_err();
                assert_eq!(e, Error::NotEnoughSpace(Name::FixedVarInt));
            }
            {
                let value = Val::VarInt(0x0102030405);
                let e = field.to_bytes(Some(&value), &mut b).unwrap_err();
                assert_eq!(e, Error::NotEnoughSpace(Name::FixedVarInt));
            }
            let mut buf = [0; 8];
            let mut b = OctetsMut::with_slice(&mut buf);
            {
                let value = Val::VarInt(0x0102030405);
                field.to_bytes(Some(&value), &mut b).unwrap();
                b = OctetsMut::with_slice(&mut buf);
            }
            {
                let value = Val::VarInt(0x0002030405);
                let e = field.to_bytes(Some(&value), &mut b).unwrap_err();
                assert_eq!(e, Error::InvalidValue(Name::FixedVarInt));
            }
        }
        {
            let mut buf = vec![0, 2, 3, 4, 5];
            let mut b = Octets::with_slice(&mut buf);
            let e = field.to_value(&mut b).unwrap_err();
            assert_eq!(e, Error::InvalidValue(Name::FixedVarInt));
        }
    }

    #[test]
    fn test_bytes_fixed_len() {
        let field = Field::new(Name::BytesFixedLen, Def::Bytes(Len::Fixed(3)));
        {
            let mut buf = [0; 4];
            let mut b = OctetsMut::with_slice(&mut buf);
            {
                let value = Val::Bytes(Cow::from(vec![1]));
                let e = field.to_bytes(Some(&value), &mut b).unwrap_err();
                assert_eq!(e, Error::InvalidValue(Name::BytesFixedLen));
            }
            {
                let e = field.to_bytes(None, &mut b).unwrap_err();
                assert_eq!(e, Error::NoValueProvided(Name::BytesFixedLen));
            }
            let mut buf = [0; 2];
            let mut b = OctetsMut::with_slice(&mut buf);
            {
                let value = Val::Bytes(Cow::from(vec![1, 2, 3]));
                let e = field.to_bytes(Some(&value), &mut b).unwrap_err();
                assert_eq!(e, Error::NotEnoughSpace(Name::BytesFixedLen));
            }
        }
        {
            let mut buf = vec![0, 1];
            let mut b = Octets::with_slice(&mut buf);
            let e = field.to_value(&mut b).unwrap_err();
            assert_eq!(e, Error::NotEnoughData(Name::BytesFixedLen));
        }
    }

    #[test]
    fn test_bytes_var_len() {
        let field = Field::new(Name::BytesVarLen, Def::Bytes(Len::Var));
        {
            let mut buf = [0; 2];
            let mut b = OctetsMut::with_slice(&mut buf);
            {
                let value = Val::Bytes(Cow::from(vec![1, 2, 3]));
                let e = field.to_bytes(Some(&value), &mut b).unwrap_err();
                assert_eq!(e, Error::NotEnoughSpace(Name::BytesVarLen));
            }
            {
                let value = Val::Bytes(Cow::from(vec![0; 1024]));
                let e = field.to_bytes(Some(&value), &mut b).unwrap_err();
                assert_eq!(e, Error::NotEnoughSpace(Name::BytesVarLen));
            }
            {
                let e = field.to_bytes(None, &mut b).unwrap_err();
                assert_eq!(e, Error::NoValueProvided(Name::BytesVarLen));
            }
        }
        {
            let mut buf = vec![2, 1];
            let mut b = Octets::with_slice(&mut buf);
            let e = field.to_value(&mut b).unwrap_err();
            assert_eq!(e, Error::NotEnoughData(Name::BytesVarLen));
        }
    }

    #[test]
    fn test_fixed_bytes() {
        let field = Field::new(Name::FixedBytes, Def::FixedBytes(vec![1, 2, 3]));
        {
            let mut buf = [0; 4];
            let mut b = OctetsMut::with_slice(&mut buf);
            {
                let value = Val::Bytes(Cow::from(vec![1, 2, 3, 4]));
                let e = field.to_bytes(Some(&value), &mut b).unwrap_err();
                assert_eq!(e, Error::InvalidValue(Name::FixedBytes));
            }
            {
                let value = Val::VarInt(1);
                let e = field.to_bytes(Some(&value), &mut b).unwrap_err();
                assert_eq!(e, Error::InvalidValue(Name::FixedBytes));
            }
            let mut buf = [0; 2];
            let mut b = OctetsMut::with_slice(&mut buf);
            {
                let value = Val::Bytes(Cow::from(vec![1, 2, 3]));
                let e = field.to_bytes(Some(&value), &mut b).unwrap_err();
                assert_eq!(e, Error::NotEnoughSpace(Name::FixedBytes));
            }
            {
                let e = field.to_bytes(None, &mut b).unwrap_err();
                assert_eq!(e, Error::NotEnoughSpace(Name::FixedBytes));
            }
        }
        {
            let mut buf = vec![0, 1];
            let mut b = Octets::with_slice(&mut buf);
            let e = field.to_value(&mut b).unwrap_err();
            assert_eq!(e, Error::NotEnoughData(Name::FixedBytes));
        }
        {
            let mut buf = vec![0, 2, 3];
            let mut b = Octets::with_slice(&mut buf);
            let e = field.to_value(&mut b).unwrap_err();
            assert_eq!(e, Error::InvalidValue(Name::FixedBytes));
        }
    }

    #[derive(Debug, PartialEq, Eq, Hash, Clone)]
    enum Name {
        FixedVarInt,
        VarInt,
        BytesFixedLen,
        BytesVarLen,
        FixedBytes,
    }

    impl FieldName for Name {}
}
