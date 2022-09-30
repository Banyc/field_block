use std::{borrow::Cow, collections::HashMap};

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

    pub fn to_bytes(&self, values: &HashMap<F, Val>, b: &mut OctetsMut) -> Result<(), Error<F>> {
        match self.def() {
            Def::VarInt(x) => {
                let y = values.get(self.name());
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
                    Len::Fixed(len) => match values.get(self.name()) {
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
                        match values.get(self.name()) {
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
                if let Some(y) = values.get(self.name()) {
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

    pub fn to_value<'buf>(
        &self,
        b: &mut Octets<'buf>,
        values: &mut HashMap<F, ValInfo<'buf>>,
    ) -> Result<(), Error<F>> {
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
                    }
                    U64::Var => {
                        values.insert(
                            self.name().clone(),
                            ValInfo {
                                value: Val::VarInt(y),
                                pos,
                            },
                        );
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
                    values.insert(self.name().clone(), ValInfo { value, pos });
                }
                Len::Var => {
                    let x = match b.get_bytes_with_varint_length() {
                        Ok(x) => x,
                        Err(_) => return Err(Error::NotEnoughData(self.name().clone())),
                    };
                    let value = Val::Bytes(Cow::from(x.buf()));
                    values.insert(self.name().clone(), ValInfo { value, pos });
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
            }
        }
        Ok(())
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
