#[derive(Debug)]
pub struct Node {
    pub name: String,
    pub children: Vec<Node>,
}

impl Node {
    pub fn new(ty: String) -> Self {
        Self {
            name: ty,
            children: vec![],
        }
    }
}
