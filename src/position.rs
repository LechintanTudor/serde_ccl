#[derive(Clone, Copy, Default, Debug)]
pub(crate) struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    #[inline]
    #[must_use]
    pub fn is_default(&self) -> bool {
        self.line == 0 && self.column == 0
    }
}
