#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u32);

impl Default for NodeId {
    fn default() -> Self {
        Self(0)
    }
}

impl NodeId {
    pub fn shift(&mut self) -> Self {
        self.0 += 1;
        *self
    }
}
