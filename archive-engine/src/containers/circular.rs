use std::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
pub struct CircularBuf<T, const CAP: usize> {
    start: usize,
    len: usize,
    backing: [Option<T>; CAP],
}
impl<T, const CAP: usize> Default for CircularBuf<T, CAP> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const CAP: usize> Index<usize> for CircularBuf<T, CAP> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<T, const CAP: usize> IndexMut<usize> for CircularBuf<T, CAP> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).unwrap()
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
    pub fn start(&self) -> usize {
        self.start
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

        let back = self.calc_idx(back).unwrap();
        self.backing[back] = Some(value);
    }
    pub fn pop_front(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            let front = self.start % CAP;
            self.start += 1;

            self.len -= 1;
            assert!(self.backing[front].is_some());
            self.backing[front].take()
        }
    }
    pub fn get(&self, index: usize) -> Option<&T> {
        let i = self.calc_idx(index)?;
        assert!(self.backing[i].is_some());
        Some(self.backing[i].as_ref().unwrap())
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        let i = self.calc_idx(index)?;
        assert!(self.backing[i].is_some());
        Some(self.backing[i].as_mut().unwrap())
    }
    fn calc_idx(&self, index: usize) -> Option<usize> {
        if index >= self.len {
            None
        } else {
            Some((index + self.start) % CAP)
        }
    }
}

// infers absolute indices from indices modulo VCAP.
// stores the last CAP entires in a CircularBuf.
// assumes CAP <<< VCAP.
#[derive(Debug, Clone)]
pub struct RollingBuf<T, const CAP: usize, const VCAP: usize> {
    backing: Box<CircularBuf<Option<T>, CAP>>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RollingBufError {
    TooOld,
    OutOfBounds,
}
impl<T, const CAP: usize, const VCAP: usize> Default for RollingBuf<T, CAP, VCAP> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const CAP: usize, const VCAP: usize> RollingBuf<T, CAP, VCAP> {
    const fn check_caps() {
        assert!(
            CAP * 8 <= VCAP,
            "VCAP is too small to detect rollovers correctly"
        );
    }
    fn calc_true_index(&self, rolling_index: usize) -> Result<usize, RollingBufError> {
        if rolling_index >= VCAP {
            return Err(RollingBufError::OutOfBounds);
        }
        let start = self.backing.start;
        let rolling_start = start % VCAP;
        let block_start = start - rolling_start;

        let mut rollover = 0;
        if rolling_index < rolling_start {
            if rolling_start >= VCAP - CAP && rolling_index <= CAP {
                rollover = VCAP;
            } else {
                return Err(RollingBufError::TooOld);
            }
        }
        let true_index = block_start + rollover + rolling_index;
        Ok(true_index)
    }

    pub fn index(&self, rolling_index: usize) -> Result<Option<&T>, RollingBufError> {
        let start = self.backing.start;
        let true_index = self.calc_true_index(rolling_index)?;
        let data = self.backing.get(true_index - start);
        if let Some(data) = data {
            Ok(data.as_ref())
        } else {
            // this happens when the index > len()
            Err(RollingBufError::OutOfBounds)
        }
    }
    pub fn new() -> Self {
        Self::check_caps();
        RollingBuf {
            backing: Box::new(CircularBuf::new()),
        }
    }
    pub fn add(&mut self, rolling_index: usize, value: T) -> Result<usize, RollingBufError> {
        let true_index = self.calc_true_index(rolling_index)?;
        // if you skip this far forward, something else went wrong
        if true_index > self.backing.start + 2 * CAP {
            return Err(RollingBufError::OutOfBounds);
        }
        // while true_index can't fit in the backing, jump forward
        while self.backing.start + self.backing.len <= true_index {
            self.backing.push_back(None);
        }
        let start = self.backing.start;
        self.backing[true_index - start] = Some(value);
        Ok(true_index)
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;

    #[test]
    fn test_circular_basic() {
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

    #[test]
    fn test_rollover_circular_access() {
        let mut buf = RollingBuf::<i32, 2, 16>::new();

        buf.add(0, 0).unwrap();

        assert_eq!(*buf.index(0).unwrap().unwrap(), 0);

        buf.add(1, 1).unwrap();

        assert_eq!(*buf.index(0).unwrap().unwrap(), 0);
        assert_eq!(*buf.index(1).unwrap().unwrap(), 1);

        buf.add(2, 2).unwrap();

        assert_eq!(*buf.index(2).unwrap().unwrap(), 2);

        assert_eq!(buf.index(0), Err(RollingBufError::TooOld));
        assert_eq!(buf.index(3), Err(RollingBufError::OutOfBounds));
    }

    #[test]
    fn test_rollover_wraparound() {
        let mut buf = RollingBuf::<i32, 2, 16>::new();
        for i in 0..=20 {
            buf.add(i as usize % 16, i).unwrap();
        }

        assert_eq!(buf.index(18 % 16), Err(RollingBufError::TooOld));
        assert_eq!(*buf.index(19 % 16).unwrap().unwrap(), 19);
        assert_eq!(*buf.index(20 % 16).unwrap().unwrap(), 20);
        assert_eq!(buf.index(21 % 16), Err(RollingBufError::OutOfBounds));
    }

    #[test]
    fn panic_stress_test() {
        use rand::SeedableRng;
        let mut rng = rand_pcg::Pcg32::seed_from_u64(123);

        for _i in 0..4 {
            let mut buf = RollingBuf::<usize, 2, 16>::new();

            for j in 0..32 {
                let diff: i32 = rng.gen_range(-3..3);
                let idx = j + diff;
                let idx = idx as usize;
                let _ = buf.add(idx, idx);

                let diff: i32 = rng.gen_range(-6..3);
                let idx = j + diff;
                let idx = idx as usize;
                let _ = buf.index(idx);
            }
        }
    }

    #[test]
    #[should_panic(expected = "VCAP is too small to detect rollovers correctly")]
    fn bad_cap() {
        let _ = RollingBuf::<i32, 2, 4>::new();
    }
}
