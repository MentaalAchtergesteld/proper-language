#[derive(Debug)]
pub struct PeekableCursor<'a, T> {
    items: &'a [T],
    position: usize,
}

impl<'a, T> PeekableCursor<'a, T> {
    pub fn new(items: &'a [T]) -> Self {
        Self { items, position: 0 }
    }

    pub fn is_at_end(&self) -> bool {
        self.position >= self.items.len()
    }

    pub fn peek_n(&self, n: usize) -> Option<&T> {
        self.items.get(self.position + n)
    }

    pub fn peek(&self) -> Option<&T> {
        self.peek_n(0)
    }

    pub fn consume(&mut self) -> Option<&T> {
        if self.is_at_end() { return None; }
        let item = &self.items[self.position];
        self.position += 1;

        Some(item)
    }

    pub fn consume_while(&mut self, predicate: impl Fn(&T) -> bool) -> &[T] {
        let start_pos = self.position;

        let remaining_items = &self.items[start_pos..];

        let run_length = remaining_items
            .iter()
            .position(|item| !predicate(item))
            .unwrap_or(remaining_items.len());

        let end_pos = start_pos + run_length;

        self.position = end_pos;

        &self.items[start_pos..end_pos]
    }

    pub fn from_position(&self) -> &[T] {
        &self.items[self.position..]
    }
}

