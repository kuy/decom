use std::collections::BTreeMap;

#[derive(Debug, Default)]
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

#[derive(Debug)]
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
