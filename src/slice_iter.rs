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

    fn peek_at_copy(&self, index: usize) -> Option<Self::Item>;

    fn peek_many_ref(&self, len: usize) -> Option<&'a [Self::Item]>;

    fn ignore_next(&mut self);

    fn ignore_many(&mut self, num: usize);

    fn take_many<const N: usize>(&mut self) -> Option<[Self::Item; N]> {
        let ret = self.peek_many::<N>();
        self.ignore_many(N);
        return ret;
    }

    fn take_many_ref(&mut self, len: usize) -> Option<&'a [Self::Item]> {
        let ret = self.peek_many_ref(len);
        self.ignore_many(len);
        return ret;
    }

    fn take_while<F: Fn(Self::Item) -> bool>(&mut self, pred: F) -> &'a [Self::Item] {
        let mut len = 0;
        while let Some(val) = self.peek_at_copy(len) {
            if pred(val) {
                len += 1;
            } else {
                break;
            }
        }
        self.take_many_ref(len).unwrap_or(&[])
    }

    fn take_while_chunked<
        const N: usize,
        F1: Fn([Self::Item; N]) -> bool,
        F2: Fn(Self::Item) -> bool,
    >(
        &mut self,
        pred1: F1,
        pred2: F2,
    ) -> &'a [Self::Item];
}

impl<'a, T: Copy> SliceIter<'a, T> {
    pub fn new(slice: &'a [T]) -> Self {
        Self { slice, index: 0 }
    }
}

impl<'a, T: Copy> CopyIter<'a> for SliceIter<'a, T> {
    #[inline]
    fn peek_copy(&self) -> Option<T> {
        self.slice.get(self.index).map(|v| *v)
    }

    fn peek_at_copy(&self, index: usize) -> Option<Self::Item> {
        self.slice.get(self.index + index).map(|v| *v)
    }

    #[inline]
    fn peek_many<const N: usize>(&self) -> Option<[T; N]> {
        self.slice
            .get(self.index..self.index + N)
            .map(|v| v.try_into().unwrap())
    }

    fn peek_many_ref(&self, len: usize) -> Option<&'a [T]> {
        self.slice.get(self.index..self.index + len)
    }

    #[inline]
    fn take_many<const N: usize>(&mut self) -> Option<[T; N]> {
        self.peek_many::<N>().map(|v| {
            self.index += N;
            v
        })
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

    #[inline]
    fn take_while_chunked<
        const N: usize,
        F1: Fn([Self::Item; N]) -> bool,
        F2: Fn(Self::Item) -> bool,
    >(
        &mut self,
        pred1: F1,
        pred2: F2,
    ) -> &'a [Self::Item] {
        let op_slice = &self.slice[self.index..];
        let mut len = 0;
        while let Some(chunk) = self.peek_many::<N>() {
            if pred1(chunk) {
                len += N;
                self.index += N;
            } else {
                break;
            }
        }
        while let Some(itm) = self.peek_copy() {
            if pred2(itm) {
                len += 1;
                self.index += 1;
            } else {
                break;
            }
        }
        return &op_slice[0..len];
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
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.slice.len() - self.index;
        (len, Some(len))
    }
}
