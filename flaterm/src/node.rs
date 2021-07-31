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

    pub fn prop<T>(&self, prop_key: &str) -> Option<T>
    where
        T: From<PropValue>,
    {
        self.props
            .iter()
            .find_map(|(key, prop)| match (prop, key.as_str() == prop_key) {
                (PropValue::LiteralString(_str), true) => Some(prop.clone().into()),
                (PropValue::LiteralNumber(_num), true) => Some(prop.clone().into()),
                _ => None,
            })
    }
}
