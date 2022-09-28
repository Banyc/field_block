use std::collections::HashMap;

use octets::{Octets, OctetsMut};

use crate::{Error, FieldName, FieldValue, FieldValueInfo};

pub struct Field<F>
where
    F: FieldName,
{
    name: F,
    definition: FieldDefinition,
}

impl<F> Field<F>
where
    F: FieldName,
{
    pub fn new(name: F, definition: FieldDefinition) -> Self {
        Field { name, definition }
    }

    pub fn name(&self) -> &F {
        &self.name
    }

    pub fn definition(&self) -> &FieldDefinition {
        &self.definition
    }

    pub fn to_bytes(
        &self,
        values: &HashMap<F, FieldValue>,
        b: &mut OctetsMut,
    ) -> Result<(), Error<F>> {
        match self.definition() {
            FieldDefinition::VarInt(x) => {
                let y = values.get(self.name());
                match (x, y) {
                    (Some(x), Some(FieldValue::VarInt(y))) => {
                        if *y != *x {
                            return Err(Error::InvalidValue(self.name().clone()));
                        }
                        if let Err(_) = b.put_varint(*y) {
                            return Err(Error::NotEnoughSpace(self.name().clone()));
                        };
                    }
                    (None, Some(FieldValue::VarInt(y))) => {
                        if let Err(_) = b.put_varint(*y) {
                            return Err(Error::NotEnoughSpace(self.name().clone()));
                        };
                    }
                    (Some(x), None) => {
                        if let Err(_) = b.put_varint(*x) {
                            return Err(Error::NotEnoughSpace(self.name().clone()));
                        };
                    }
                    (None, None) => {
                        return Err(Error::NoValueProvided(self.name().clone()));
                    }
                    (_, _) => {
                        return Err(Error::InvalidValue(self.name().clone()));
                    }
                };
            }
            FieldDefinition::Bytes(len) => {
                match len {
                    FieldLen::Fixed(len) => match values.get(self.name()) {
                        Some(FieldValue::Bytes(x)) => {
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
                    FieldLen::Var => {
                        match values.get(self.name()) {
                            Some(FieldValue::Bytes(x)) => {
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
            FieldDefinition::FixedBytes(x) => {
                if let Some(y) = values.get(self.name()) {
                    match y {
                        FieldValue::Bytes(y) => {
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

    pub fn to_values(
        &self,
        b: &mut Octets,
        values: &mut HashMap<F, FieldValueInfo>,
    ) -> Result<(), Error<F>> {
        let pos = b.off();

        match self.definition() {
            FieldDefinition::VarInt(x) => {
                match b.get_varint() {
                    Ok(y) => match x {
                        Some(x) => {
                            if *x != y {
                                return Err(Error::InvalidValue(self.name().clone()));
                            }
                        }
                        None => {
                            values.insert(
                                self.name().clone(),
                                FieldValueInfo {
                                    value: FieldValue::VarInt(y),
                                    pos,
                                },
                            );
                        }
                    },
                    Err(_) => return Err(Error::NotEnoughData(self.name().clone())),
                };
            }
            FieldDefinition::Bytes(len) => match len {
                FieldLen::Fixed(len) => {
                    let x = match b.get_bytes(*len) {
                        Ok(x) => x,
                        Err(_) => return Err(Error::NotEnoughData(self.name().clone())),
                    };
                    let value = FieldValue::Bytes(x.to_vec());
                    values.insert(self.name().clone(), FieldValueInfo { value, pos });
                }
                FieldLen::Var => {
                    let x = match b.get_bytes_with_varint_length() {
                        Ok(x) => x,
                        Err(_) => return Err(Error::NotEnoughData(self.name().clone())),
                    };
                    let value = FieldValue::Bytes(x.to_vec());
                    values.insert(self.name().clone(), FieldValueInfo { value, pos });
                }
            },
            FieldDefinition::FixedBytes(x) => {
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

pub enum FieldDefinition {
    VarInt(Option<u64>),
    Bytes(FieldLen),
    FixedBytes(Vec<u8>),
}

pub enum FieldLen {
    Fixed(usize),
    Var,
}
