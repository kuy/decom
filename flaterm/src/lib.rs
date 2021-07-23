pub struct Block {
    children: Vec<Block>,
}

impl Block {
    pub fn new() -> Self {
        Self { children: vec![] }
    }
}
