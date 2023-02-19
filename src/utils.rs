use itertools::MultiPeek;

pub fn take_static<'a, const N: usize, T: Default + Copy, I: Iterator<Item = T>>(
    iter: &'a mut I,
) -> Option<[T; N]> {
    let mut i: usize = 0;
    let mut out_buf: [T; N] = [T::default(); N];
    while i < N {
        if let Some(v) = iter.next() {
            out_buf[i] = v;
            i += 1;
        } else {
            break;
        }
    }
    if i == N {
        Some(out_buf)
    } else {
        None
    }
}

pub fn peek_static<'a, const N: usize, T: Default + Copy, I: Iterator<Item = T>>(
    iter: &'a mut MultiPeek<I>,
) -> Option<[T; N]> {
    let mut i: usize = 0;
    let mut out_buf: [T; N] = [T::default(); N];
    while i < N {
        if let Some(v) = iter.peek() {
            out_buf[i] = *v;
            i += 1;
        } else {
            break;
        }
    }
    if i == N {
        Some(out_buf)
    } else {
        None
    }
}
