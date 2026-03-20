#[derive(Debug, Clone, Copy)]
pub struct LoopID(pub usize);

impl LoopID {
    pub fn next(&mut self) -> Self {
        let current = *self;
        self.0 += 1;
        current
    }
}
