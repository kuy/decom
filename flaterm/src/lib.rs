use std::collections::BTreeMap;

mod renderer;
pub use renderer::render;

#[derive(Clone, Debug, Default)]
pub struct Node {
    pub name: String,
    pub children: Vec<Node>,
    pub props: BTreeMap<String, PropValue>,
}

impl Node {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug)]
pub enum PropValue {
    LiteralString(String),
    LiteralNumber(i32),
    Expression(String),
    None,
}

impl Default for PropValue {
    fn default() -> Self {
        Self::None
    }
}

impl From<&str> for PropValue {
    fn from(value: &str) -> Self {
        Self::LiteralString(String::from(value))
    }
}

impl From<i32> for PropValue {
    fn from(value: i32) -> Self {
        Self::LiteralNumber(value)
    }
}

impl From<PropValue> for String {
    fn from(prop_value: PropValue) -> Self {
        if let PropValue::LiteralString(str) = prop_value {
            str
        } else {
            panic!("Failed to convert {:?} to String", prop_value);
        }
    }
}

impl From<PropValue> for u16 {
    fn from(prop_value: PropValue) -> Self {
        if let PropValue::LiteralNumber(num) = prop_value {
            num as u16
        } else {
            panic!("Failed to convert {:?} to u16", prop_value);
        }
    }
}
