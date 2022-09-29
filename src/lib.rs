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
    NotEnoughData(F),
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_to_bytes() {
        let block = get_block();

        let mut values = HashMap::new();
        values.insert(Name::VarInt, FieldValue::VarInt(0x1234));
        values.insert(Name::BytesFixedLen, FieldValue::Bytes(vec![1]));
        values.insert(Name::BytesVarLen, FieldValue::Bytes(vec![1, 2, 3]));

        let mut vec = vec![0; 1024];

        let end = block.to_bytes(&values, &mut vec).unwrap();

        assert_eq!(end, 19);

        assert_eq!(
            &vec[..end],
            &vec![
                // fixed varint
                0 | 0xc0,
                0,
                0,
                0,
                0xde,
                0xad,
                0xbe,
                0xef,
                // varint
                0x12 | 0x40,
                0x34,
                // bytes with fixed len
                1,
                // bytes with var len
                3,
                1,
                2,
                3,
                // fixed bytes
                0xba,
                0xad,
                0xf0,
                0x0d
            ]
        );
    }

    #[test]
    fn test_to_values() {
        let block = get_block();

        let vec = vec![
            // fixed varint
            0 | 0xc0,
            0,
            0,
            0,
            0xde,
            0xad,
            0xbe,
            0xef,
            // varint
            0x12 | 0x40,
            0x34,
            // bytes with fixed len
            1,
            // bytes with var len
            3,
            1,
            2,
            3,
            // fixed bytes
            0xba,
            0xad,
            0xf0,
            0x0d,
        ];

        let mut values = HashMap::new();

        let end = block.to_values(&vec, &mut values).unwrap();

        assert_eq!(end, vec.len());

        match values.get(&Name::VarInt) {
            Some(FieldValueInfo {
                value: FieldValue::VarInt(x),
                pos,
            }) => {
                assert_eq!(*x, 0x1234);
                assert_eq!(*pos, 8);
            }
            _ => panic!(),
        };
        match values.get(&Name::BytesFixedLen) {
            Some(FieldValueInfo {
                value: FieldValue::Bytes(x),
                pos,
            }) => {
                assert_eq!(*x, vec![1]);
                assert_eq!(*pos, 10);
            }
            _ => {
                panic!();
            }
        }
        match values.get(&Name::BytesVarLen) {
            Some(FieldValueInfo {
                value: FieldValue::Bytes(x),
                pos,
            }) => {
                assert_eq!(*x, vec![1, 2, 3]);
                assert_eq!(*pos, 11);
            }
            _ => {
                panic!();
            }
        }
    }

    fn get_block() -> Block<Name> {
        let mut block = Block::new();
        block.add_field(Name::FixedVarInt, FieldDef::VarInt(Some(0xdeadbeef)));
        block.add_field(Name::VarInt, FieldDef::VarInt(None));
        block.add_field(Name::BytesFixedLen, FieldDef::Bytes(FieldLen::Fixed(1)));
        block.add_field(Name::BytesVarLen, FieldDef::Bytes(FieldLen::Var));
        block.add_field(
            Name::FixedBytes,
            FieldDef::FixedBytes(vec![0xba, 0xad, 0xf0, 0x0d]),
        );
        block
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
