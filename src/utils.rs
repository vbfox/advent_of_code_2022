pub struct CharSliceIterator<'a> {
    s: &'a str,
    index: usize,
}

impl<'a> CharSliceIterator<'a> {
    fn new(s: &'a str) -> Self {
        Self { s, index: 0 }
    }
}

impl<'a> Iterator for CharSliceIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.s.len() {
            let slice = &self.s[self.index..=self.index];
            self.index += 1;
            Some(slice)
        } else {
            None
        }
    }
}

pub trait CharSliceExt {
    fn char_slices(&self) -> CharSliceIterator;
}

impl<T: AsRef<str>> CharSliceExt for T {
    fn char_slices(&self) -> CharSliceIterator {
        CharSliceIterator::new(self.as_ref())
    }
}
