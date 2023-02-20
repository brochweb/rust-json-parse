#[derive(Debug)]
pub struct SliceIter<'a, T: Copy> {
    slice: &'a [T],
    index: usize,
}

pub trait CopyIter<'a>: Iterator {
    fn peek_copy(&self) -> Option<Self::Item>;

    /// Copies and returns an array
    ///
    /// Returns `None` if the slice is too short
    fn peek_many<const N: usize>(&self) -> Option<[Self::Item; N]>;

    fn peek_many_ref(&self, len: usize) -> Option<&'a [Self::Item]>;

    fn take_many<const N: usize>(&mut self) -> Option<[Self::Item; N]>;

    fn ignore_next(&mut self);

    fn ignore_many(&mut self, num: usize);
}

impl<'a, T: Copy> SliceIter<'a, T> {
    pub fn new(slice: &'a [T]) -> Self {
        Self { slice, index: 0 }
    }
}

impl<'a, T: Copy> CopyIter<'a> for SliceIter<'a, T> {
    fn peek_copy(&self) -> Option<T> {
        if self.index < self.slice.len() {
            Some(self.slice[self.index])
        } else {
            None
        }
    }
    fn peek_many<const N: usize>(&self) -> Option<[T; N]> {
        if self.index + N <= self.slice.len() {
            Some(self.slice[self.index..self.index + N].try_into().unwrap())
        } else {
            None
        }
    }
    fn peek_many_ref(&self, len: usize) -> Option<&'a [T]> {
        if self.index + len <= self.slice.len() {
            Some(&self.slice[self.index..self.index + len])
        } else {
            None
        }
    }
    fn take_many<const N: usize>(&mut self) -> Option<[T; N]> {
        if self.index + N <= self.slice.len() {
            let ret = Some(self.slice[self.index..self.index + N].try_into().unwrap());
            self.index += N;
            return ret;
        } else {
            None
        }
    }
    fn ignore_next(&mut self) {
        if self.index < self.slice.len() {
            self.index += 1;
        }
    }
    fn ignore_many(&mut self, num: usize) {
        if self.index + num <= self.slice.len() {
            self.index += num;
        } else {
            self.index = self.slice.len();
        }
    }
}

impl<'a, T: Copy> From<&'a [T]> for SliceIter<'a, T> {
    fn from(slice: &'a [T]) -> Self {
        Self { slice, index: 0 }
    }
}

impl<'a, T: Copy> Iterator for SliceIter<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.slice.len() {
            let ret = Some(self.slice[self.index]);
            self.index += 1;
            return ret;
        } else {
            None
        }
    }
}
