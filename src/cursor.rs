pub struct Cursor<'a, T> {
    content: &'a [T],
}

impl<'a, T> Cursor<'a, T> {
    pub fn new(content: &'a [T]) -> Self {
        Self { content }
    }

    pub fn peek(&self, offset: usize) -> Option<&T> {
        self.content.get(offset)
    } 

    pub fn chop(&mut self, n: usize) -> Option<&'a [T]> {
        if n > self.content.len() { return None };

        let chopped = &self.content[..n];
        self.content = &self.content[n..];
        Some(chopped)
    }

    pub fn chop_while<P>(&mut self, mut predicate: P) -> Option<&[T]> where P: FnMut(&T) -> bool {
        let mut n = 0;
        while n < self.content.len() && predicate(&self.content[n]) {
            n += 1;
        }
        self.chop(n)
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}
