use std::ops::Range;

#[derive(Debug, Default, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn combine(&self, other: &Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
    
    pub fn extend(&self, new_end: usize) -> Span {
        Span {
            start: self.start,
            end: new_end
        }
    }
}

impl From<Range<usize>> for Span {
   fn from(value: Range<usize>) -> Self {
       Self { start: value.start, end: value.end }
   } 
}

#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: impl Into<Span>) -> Self {
        Self { value, span: span.into() }
    }
}
