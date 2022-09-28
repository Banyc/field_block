#[derive(Debug)]
pub enum FieldValue {
    VarInt(i64),
    Bytes(Vec<u8>),
}

#[derive(Debug)]
pub struct FieldValueInfo {
    pub value: FieldValue,
    pub pos: usize,
}
