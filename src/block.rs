use std::{collections::HashMap, sync::Arc};

use octets::{Octets, OctetsMut};

use crate::{Error, Field, FieldName, FieldValue, FieldValueInfo};

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

    pub fn to_bytes(&self, values: &HashMap<F, FieldValue>, b: &mut [u8]) -> Result<(), Error<F>> {
        let mut b = OctetsMut::with_slice(b);
        return self.to_bytes_(values, &mut b);
    }

    fn to_bytes_(
        &self,
        values: &HashMap<F, FieldValue>,
        b: &mut OctetsMut,
    ) -> Result<(), Error<F>> {
        for field in self.fields.iter() {
            field.to_bytes(values, b)?;
        }
        Ok(())
    }

    pub fn to_values(
        &self,
        b: &[u8],
        values: &mut HashMap<F, FieldValueInfo>,
    ) -> Result<(), Error<F>> {
        let mut b = Octets::with_slice(b);
        return self.to_values_(&mut b, values);
    }

    fn to_values_(
        &self,
        b: &mut Octets,
        values: &mut HashMap<F, FieldValueInfo>,
    ) -> Result<(), Error<F>> {
        for field in self.fields.iter() {
            field.to_values(b, values)?;
        }
        Ok(())
    }
}
