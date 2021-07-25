pub struct Block {
    pub children: Vec<Block>,
}

impl Block {
    pub fn new() -> Self {
        Self { children: vec![] }
    }
}

pub struct Text {
    pub content: String,
}

impl Text {
    pub fn new(content: String) -> Self {
        Self { content }
    }
}

#[derive(Debug)]
pub struct Node {
    pub ty: String,
    pub children: Vec<Node>,
}

impl Node {
    pub fn new(ty: String) -> Self {
        Self {
            ty,
            children: vec![],
        }
    }
}
