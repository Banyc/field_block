use std::{collections::HashMap, sync::Arc};

use bytes::BytesMut;

use crate::{Error, Field, FieldName, FieldValue};

pub struct Block<F>
where
    F: FieldName,
{
    fields: Vec<Field<F>>,
}

impl<F> Block<F>
where
    F: FieldName,
{
    pub fn check_rep() {}

    pub fn new() -> Self {
        Block { fields: Vec::new() }
    }

    pub fn add_field(&mut self, field: Field<F>) {
        self.fields.push(field);
    }

    pub fn into_arc(self) -> Arc<Block<F>> {
        Arc::new(self)
    }

    pub fn to_bytes(
        &self,
        values: &HashMap<F, FieldValue>,
        b: &mut BytesMut,
    ) -> Result<(), Error<F>> {
        for field in self.fields.iter() {
            field.to_bytes(values, b)?;
        }
        Ok(())
    }

    pub fn to_values(
        &self,
        b: &mut BytesMut,
        values: &mut HashMap<F, FieldValue>,
    ) -> Result<(), Error<F>> {
        for field in self.fields.iter() {
            field.to_values(b, values)?;
        }
        Ok(())
    }
}
