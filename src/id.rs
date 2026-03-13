// NOTE: Keeping inner types with pub can be risky but we do it because
// its not that big of a deall

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
