#[derive(Debug)]
pub enum FieldValue {
    I32(i32),
    VarInt(i64),
    Bytes(Vec<u8>),
}
