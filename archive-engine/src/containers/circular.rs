use std::ops::{Index, IndexMut};

pub struct CircularBuf<T, const CAP: usize> {
    start: usize,
    len: usize,
    backing: [Option<T>; CAP],
}

impl<T, const CAP: usize> Index<usize> for CircularBuf<T, CAP> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        let i = self.calc_idx(index);
        assert!(self.backing[i].is_some());
        self.backing[i].as_ref().unwrap()
    }
}

impl<T, const CAP: usize> IndexMut<usize> for CircularBuf<T, CAP> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let i = self.calc_idx(index);
        assert!(self.backing[i].is_some());
        self.backing[i].as_mut().unwrap()
    }
}

impl<T, const CAP: usize> CircularBuf<T, CAP> {
    const NONE: Option<T> = None;
    pub fn new() -> Self {
        Self {
            start: 0,
            len: 0,
            backing: [Self::NONE; CAP],
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn push_back(&mut self, value: T) {
        if self.len == CAP {
            self.pop_front();
        }
        let back = self.len;
        self.len += 1;

        let back = self.calc_idx(back);
        self.backing[back] = Some(value);
    }
    pub fn pop_front(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            let front = self.start;
            self.start += 1;
            if self.start == CAP {
                self.start = 0;
            }

            self.len -= 1;
            assert!(self.backing[front].is_some());
            self.backing[front].take()
        }
    }
    fn calc_idx(&self, index: usize) -> usize {
        if index >= self.len {
            panic!("circbuf oob");
        }

        (index + self.start) % CAP
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut buf = CircularBuf::<i32, 2>::new();

        buf.push_back(0);
        assert_eq!(buf[0], 0);

        buf.push_back(1);
        assert_eq!(buf[0], 0);
        assert_eq!(buf[1], 1);

        buf.push_back(2);
        assert_eq!(buf[0], 1);
        assert_eq!(buf[1], 2);

        assert_eq!(buf.pop_front(), Some(1));
        assert_eq!(buf.pop_front(), Some(2));

        assert_eq!(buf.pop_front(), None);
    }
}
