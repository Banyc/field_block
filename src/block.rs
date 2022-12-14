use std::collections::HashMap;

use octets::{Octets, OctetsMut};

use crate::{Def, Field, FieldName, ToBytesError, ToValuesError, Val, ValInfo};

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

    pub fn add_field(&mut self, name: F, def: Def) {
        self.fields.push(Field::new(name, def));
        self.check_rep();
    }

    pub fn to_bytes(
        &self,
        values: &HashMap<F, Val>,
        b: &mut [u8],
    ) -> Result<usize, ToBytesError<F>> {
        let mut b = OctetsMut::with_slice(b);
        return self.to_bytes_(values, &mut b);
    }

    fn to_bytes_(
        &self,
        values: &HashMap<F, Val>,
        b: &mut OctetsMut,
    ) -> Result<usize, ToBytesError<F>> {
        for field in self.fields.iter() {
            let value = values.get(field.name());
            field.to_bytes(value, b)?;
        }
        Ok(b.off())
    }

    pub fn to_values<'buf>(
        &self,
        b: &'buf [u8],
        values: &mut HashMap<F, ValInfo<'buf>>,
    ) -> Result<usize, ToValuesError<F>> {
        let mut b = Octets::with_slice(b);
        return self.to_values_(&mut b, values);
    }

    fn to_values_<'buf>(
        &self,
        b: &mut Octets<'buf>,
        values: &mut HashMap<F, ValInfo<'buf>>,
    ) -> Result<usize, ToValuesError<F>> {
        for field in self.fields.iter() {
            let value = field.to_value(b)?;
            values.insert(field.name().clone(), value);
        }
        Ok(b.off())
    }
}
