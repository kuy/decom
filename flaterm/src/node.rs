use crate::PropValue;
use std::collections::BTreeMap;

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
