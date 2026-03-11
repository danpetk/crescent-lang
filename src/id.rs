#[derive(Debug, Clone, Copy)]
pub struct SymbolID(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct LoopID(pub usize);

impl LoopID {
    pub fn dummy() -> Self {
        LoopID(usize::MAX)
    }

    pub fn next(&mut self) -> Self {
        let current = *self;
        self.0 += 1;
        current
    }
}
