use std::collections::HashMap;

use bytes::{Buf, BufMut, BytesMut};

use crate::{Error, FieldName, FieldValue};

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
        b: &mut BytesMut,
    ) -> Result<(), Error<F>> {
        match self.definition() {
            FieldDefinition::I32(x) => match x {
                Some(x) => {
                    b.put_i32(*x);
                }
                None => match values.get(self.name()) {
                    Some(FieldValue::I32(x)) => {
                        b.put_i32(*x);
                    }
                    _ => {
                        return Err(Error::NoValueProvided(self.name().clone()));
                    }
                },
            },
            FieldDefinition::VarInt(_x) => {
                return Err(Error::NotImpl(self.name().clone()));
            }
            FieldDefinition::Bytes(len) => {
                match len {
                    FieldLen::Fixed(len) => match values.get(self.name()) {
                        Some(FieldValue::Bytes(x)) => {
                            if x.len() != *len {
                                return Err(Error::InvalidValue(self.name().clone()));
                            }
                            b.put_slice(x);
                        }
                        _ => {
                            return Err(Error::NoValueProvided(self.name().clone()));
                        }
                    },
                    FieldLen::Var => {
                        match values.get(self.name()) {
                            Some(FieldValue::Bytes(x)) => {
                                // length prefix
                                b.put_u32(x.len() as u32);
                                // data
                                b.put_slice(x);
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
                b.put_slice(x);
            }
        }
        Ok(())
    }

    pub fn to_values(
        &self,
        b: &mut BytesMut,
        values: &mut HashMap<F, FieldValue>,
    ) -> Result<(), Error<F>> {
        match self.definition() {
            FieldDefinition::I32(x) => match x {
                Some(x) => {
                    if *x != b.get_i32() {
                        return Err(Error::InvalidValue(self.name().clone()));
                    }
                }
                None => {
                    values.insert(self.name().clone(), FieldValue::I32(b.get_i32()));
                }
            },
            FieldDefinition::VarInt(_x) => {
                return Err(Error::NotImpl(self.name().clone()));
            }
            FieldDefinition::Bytes(len) => match len {
                FieldLen::Fixed(len) => {
                    let mut x = vec![0; *len];
                    b.copy_to_slice(&mut x);
                    values.insert(self.name().clone(), FieldValue::Bytes(x));
                }
                FieldLen::Var => {
                    let len = b.get_u32() as usize;
                    let mut x = vec![0; len];
                    b.copy_to_slice(&mut x);
                    values.insert(self.name().clone(), FieldValue::Bytes(x));
                }
            },
            FieldDefinition::FixedBytes(x) => {
                let mut y = vec![0; x.len()];
                b.copy_to_slice(&mut y);
                if y != *x {
                    return Err(Error::InvalidValue(self.name().clone()));
                }
            }
        }
        Ok(())
    }
}

pub enum FieldDefinition {
    I32(Option<i32>),
    VarInt(Option<i64>),
    Bytes(FieldLen),
    FixedBytes(Vec<u8>),
}

pub enum FieldLen {
    Fixed(usize),
    Var,
}
