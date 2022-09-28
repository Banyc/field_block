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
    NotEnoughSpace(F),
    NotImpl(F),
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use super::*;

    #[test]
    fn test_to_bytes() {
        let block = get_block();

        let mut values = HashMap::new();
        values.insert(TestName::BytesFixedLen, FieldValue::Bytes(vec![1]));
        values.insert(TestName::BytesVarLen, FieldValue::Bytes(vec![1, 2, 3]));

        let mut vec = vec![0; 1024];
        block.to_bytes(&values, &mut vec).unwrap();

        assert_eq!(&vec[..9], &vec![1, 3, 1, 2, 3, 0xba, 0xad, 0xf0, 0x0d]);
    }

    #[test]
    fn test_to_values() {
        let block = get_block();

        let vec = vec![1, 3, 1, 2, 3, 0xba, 0xad, 0xf0, 0x0d];

        let mut values = HashMap::new();
        block.to_values(&vec, &mut values).unwrap();

        match values.get(&TestName::BytesFixedLen) {
            Some(FieldValueInfo {
                value: FieldValue::Bytes(x),
                pos,
            }) => {
                assert_eq!(*x, vec![1]);
                assert_eq!(*pos, 0);
            }
            _ => {
                panic!();
            }
        }
        match values.get(&TestName::BytesVarLen) {
            Some(FieldValueInfo {
                value: FieldValue::Bytes(x),
                pos,
            }) => {
                assert_eq!(*x, vec![1, 2, 3]);
                assert_eq!(*pos, 1);
            }
            _ => {
                panic!();
            }
        }
    }

    fn get_block() -> Arc<Block<TestName>> {
        let mut block = Block::new();
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
            FieldDefinition::FixedBytes(vec![0xba, 0xad, 0xf0, 0x0d]),
        ));
        block.into_arc()
    }

    #[derive(Debug, PartialEq, Eq, Hash, Clone)]
    enum TestName {
        BytesFixedLen,
        BytesVarLen,
        FixedBytes,
    }

    impl FieldName for TestName {}
}
