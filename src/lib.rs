mod block;
mod field;
mod value;

use std::hash::Hash;

pub use block::*;
pub use field::*;
pub use value::*;

pub trait FieldName: PartialEq + Eq + Hash + Clone + Sized {}

#[derive(Debug)]
pub enum Error<F>
where
    F: FieldName,
{
    NoValueProvided(F),
    InvalidValue(F),
    NotImpl(F),
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use bytes::{BufMut, BytesMut};

    use super::*;

    #[test]
    fn test_to_bytes() {
        let block = get_block();

        let mut values = HashMap::new();
        values.insert(TestName::I32, FieldValue::I32(1));
        values.insert(TestName::BytesFixedLen, FieldValue::Bytes(vec![1]));
        values.insert(TestName::BytesVarLen, FieldValue::Bytes(vec![1, 2, 3]));

        let mut b = BytesMut::new();
        block.to_bytes(&values, &mut b).unwrap();

        assert_eq!(
            b.to_vec(),
            vec![0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 3, 1, 2, 3, 1, 2, 3]
        );
    }

    #[test]
    fn test_to_values() {
        let block = get_block();

        let mut b = BytesMut::new();
        b.put_slice(&vec![
            0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 3, 1, 2, 3, 1, 2, 3,
        ]);

        let mut values = HashMap::new();
        block.to_values(&mut b, &mut values).unwrap();

        match values.get(&TestName::I32) {
            Some(FieldValue::I32(x)) => {
                assert_eq!(*x, 1);
            }
            _ => {
                panic!();
            }
        }
        match values.get(&TestName::BytesFixedLen) {
            Some(FieldValue::Bytes(x)) => {
                assert_eq!(*x, vec![1]);
            }
            _ => {
                panic!();
            }
        }
        match values.get(&TestName::BytesVarLen) {
            Some(FieldValue::Bytes(x)) => {
                assert_eq!(*x, vec![1, 2, 3]);
            }
            _ => {
                panic!();
            }
        }
    }

    fn get_block() -> Arc<Block<TestName>> {
        let mut block = Block::new();
        block.add_field(Field::new(
            TestName::FixedI32,
            FieldDefinition::I32(Some(1)),
        ));
        block.add_field(Field::new(TestName::I32, FieldDefinition::I32(None)));
        block.add_field(Field::new(
            TestName::BytesFixedLen,
            FieldDefinition::Bytes(FieldLen::Fixed(1)),
        ));
        block.add_field(Field::new(
            TestName::BytesVarLen,
            FieldDefinition::Bytes(FieldLen::Var),
        ));
        block.add_field(Field::new(
            TestName::FixedBytes,
            FieldDefinition::FixedBytes(vec![1, 2, 3]),
        ));
        block.into_arc()
    }

    #[derive(Debug, PartialEq, Eq, Hash, Clone)]
    enum TestName {
        FixedI32,
        I32,
        BytesFixedLen,
        BytesVarLen,
        FixedBytes,
    }

    impl FieldName for TestName {}
}
