use std::collections::HashMap;

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
    fn check_rep(&self) {}

    #[must_use]
    pub fn new() -> Self {
        let self_ = Block { fields: Vec::new() };
        self_.check_rep();
        self_
    }

    pub fn add_field(&mut self, field: Field<F>) {
        self.fields.push(field);
        self.check_rep();
    }

    pub fn to_bytes(
        &self,
        values: &HashMap<F, FieldValue>,
        b: &mut [u8],
    ) -> Result<usize, Error<F>> {
        let mut b = OctetsMut::with_slice(b);
        return self.to_bytes_(values, &mut b);
    }

    fn to_bytes_(
        &self,
        values: &HashMap<F, FieldValue>,
        b: &mut OctetsMut,
    ) -> Result<usize, Error<F>> {
        for field in self.fields.iter() {
            field.to_bytes(values, b)?;
        }
        Ok(b.off())
    }

    pub fn to_values(
        &self,
        b: &[u8],
        values: &mut HashMap<F, FieldValueInfo>,
    ) -> Result<usize, Error<F>> {
        let mut b = Octets::with_slice(b);
        return self.to_values_(&mut b, values);
    }

    fn to_values_(
        &self,
        b: &mut Octets,
        values: &mut HashMap<F, FieldValueInfo>,
    ) -> Result<usize, Error<F>> {
        for field in self.fields.iter() {
            field.to_value(b, values)?;
        }
        Ok(b.off())
    }
}
