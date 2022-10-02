# Field Block

A definition language for buffer parsing.

## Usage

Adding this crate to a project:

```bash
cargo add field_block
```

Defining fields for a buffer:

```rust
fn get_block() -> Block<Name> {
    let mut block = Block::new();
    block.add_field(
        Name::FixedVarInt, //
        Def::VarInt(U64::Fixed(0xdeadbeef)),
    );
    block.add_field(
        Name::VarInt, //
        Def::VarInt(U64::Var),
    );
    block.add_field(
        Name::BytesFixedLen, //
        Def::Bytes(Len::Fixed(1)),
    );
    block.add_field(
        Name::BytesVarLen, //
        Def::Bytes(Len::Var),
    );
    block.add_field(
        Name::FixedBytes, //
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
```

Encoding a buffer:

```rust
let block = get_block();

let mut values = HashMap::new();
values.insert(Name::VarInt, Val::VarInt(0x1234));
let vec = vec![1];
values.insert(Name::BytesFixedLen, Val::Bytes(&vec));
let vec = vec![1, 2, 3];
values.insert(Name::BytesVarLen, Val::Bytes(&vec));

let mut vec = vec![0; 1024];

let end = block.to_bytes(&values, &mut vec).unwrap();
```

Decoding a buffer:

```rust
let block = get_block();

let vec = vec![0 | 0xc0, 0, 0, 0, 0xde, 0xad, 0xbe, 0xef, 0x12 | 0x40, 0x34, 1, 3, 1, 2, 3, 0xba, 0xad, 0xf0, 0x0d];

let mut values = HashMap::new();

let end = block.to_values(&vec, &mut values).unwrap();

let ValInfo { value, pos } = values.get(&Name::VarInt).unwrap();
let value = value.varint().unwrap();

println!("Field VarInt has a value {} at pos {}", value, pos);
```

See unit tests for examples.
