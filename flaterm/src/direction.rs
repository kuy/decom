use crate::PropValue;

#[derive(Clone, PartialEq)]
pub enum Direction {
    Column,
    Row,
}

impl Default for Direction {
    fn default() -> Self {
        Direction::Row
    }
}

impl From<String> for Direction {
    fn from(value: String) -> Self {
        match value.as_str() {
            "column" => Direction::Column,
            "row" => Direction::Row,
            _ => panic!("Invalid direction value: {}", value),
        }
    }
}

impl From<PropValue> for Direction {
    fn from(prop_value: PropValue) -> Self {
        if let PropValue::LiteralString(str) = prop_value {
            str.into()
        } else {
            panic!("Failed to convert {:?} to Direction", prop_value);
        }
    }
}
