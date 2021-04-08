

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u32);

impl NodeId{
    pub fn new() -> Self{
        Self(0)
    }
    pub fn shift(&mut self) -> Self{
        self.0 += 1;
        self.clone()
    }
}