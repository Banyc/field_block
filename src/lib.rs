mod block;
mod field;
mod value;

use std::hash::Hash;

pub use block::*;
pub use field::*;
pub use value::*;

pub trait FieldName: PartialEq + Eq + Hash + Clone {}

#[derive(Debug, PartialEq)]
pub enum ToBytesError<F>
where
    F: FieldName,
{
    NoValueProvided(F),
    InvalidValue(F),
    NotEnoughSpace(F),
}

#[derive(Debug, PartialEq)]
pub enum ToValuesError<F>
where
    F: FieldName,
{
    InvalidValue(F),
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
        values.insert(Name::VarInt, Val::VarInt(0x1234));
        let vec = vec![1];
        values.insert(Name::BytesFixedLen, Val::Bytes(&vec));
        let vec = vec![1, 2, 3];
        values.insert(Name::BytesVarLen, Val::Bytes(&vec));

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

        let ValInfo { value, pos } = values.get(&Name::VarInt).unwrap();
        assert_eq!(value.varint().unwrap(), 0x1234);
        assert_eq!(*pos, 8);

        let ValInfo { value, pos } = values.get(&Name::BytesFixedLen).unwrap();
        assert_eq!(value.bytes().unwrap(), vec![1]);
        assert_eq!(*pos, 10);

        let ValInfo { value, pos } = values.get(&Name::BytesVarLen).unwrap();
        assert_eq!(value.bytes().unwrap(), vec![1, 2, 3]);
        assert_eq!(*pos, 11);
    }

    fn get_block() -> Block<Name> {
        let mut block = Block::new();
        block.add_field(Name::FixedVarInt, Def::VarInt(U64::Fixed(0xdeadbeef)));
        block.add_field(Name::VarInt, Def::VarInt(U64::Var));
        block.add_field(Name::BytesFixedLen, Def::Bytes(Len::Fixed(1)));
        block.add_field(Name::BytesVarLen, Def::Bytes(Len::Var));
        block.add_field(
            Name::FixedBytes,
            Def::FixedBytes(vec![0xba, 0xad, 0xf0, 0x0d]),
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
