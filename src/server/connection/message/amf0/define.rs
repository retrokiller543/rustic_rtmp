use indexmap::IndexMap;

#[derive(PartialEq, Clone, Debug)]
pub enum Amf0ValueType {
    Number(f64),
    Boolean(bool),
    UTF8String(String),
    Object(IndexMap<String, Amf0ValueType>),
    Null,
    EcmaArray(IndexMap<String, Amf0ValueType>),
    LongUTF8String(String),
    END,
}