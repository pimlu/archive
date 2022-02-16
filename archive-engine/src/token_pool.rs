use std::{cmp::Reverse, collections::BinaryHeap, num::Wrapping};

use serde::{Deserialize, Serialize};

type Generation = Wrapping<u32>;
pub type PoolKey = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PoolToken {
    key: PoolKey,
    generation: Generation,
}
impl PoolToken {
    pub fn key(&self) -> PoolKey {
        self.key
    }
    // allows you to search BTreeMaps for whether a token exists
    pub fn range_for_key(key: PoolKey) -> std::ops::Range<PoolToken> {
        let generation = Wrapping(0);
        let lo = PoolToken { key, generation };
        let hi = PoolToken {
            key: key + 1,
            generation,
        };
        lo..hi
    }
}

#[derive(Default)]
pub struct TokenPool {
    generations: Vec<Generation>,
    freed: BinaryHeap<Reverse<PoolKey>>,
}

impl TokenPool {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn alloc(&mut self) -> PoolToken {
        // if the freed entires min heap has an entry, pop the index
        if let Some(Reverse(key)) = self.freed.pop() {
            let generation = self.generations[key as usize];
            return PoolToken { key, generation };
        }

        // otherwise, bump the pool larger and set generation to 0
        let idx: PoolKey = self.generations.len().try_into().unwrap();
        let generation = Wrapping(0);
        self.generations.push(generation);

        PoolToken {
            key: idx,
            generation,
        }
    }

    pub fn free(&mut self, entry: PoolToken) {
        let PoolToken { key, generation } = entry;
        let gen_ref = &mut self.generations[key as usize];
        assert_eq!(generation, *gen_ref, "double freed you dummy");

        // increment the generation since we are reusing an index
        *gen_ref += Wrapping(1);

        self.freed.push(Reverse(key));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indices() {
        let mut pool = TokenPool::new();
        let a = pool.alloc();
        let b = pool.alloc();
        let c = pool.alloc();
        assert_eq!([a.key, b.key, c.key], [0, 1, 2]);

        pool.free(b);

        let d = pool.alloc();
        let e = pool.alloc();

        assert_eq!(d.key, b.key);
        assert_eq!(e.key, 3);
    }

    #[test]
    fn test_allocs_lowest() {
        let mut pool = TokenPool::new();
        let a = pool.alloc();
        let _b = pool.alloc();
        let c = pool.alloc();

        pool.free(c);
        pool.free(a);

        let d = pool.alloc();
        assert_eq!(d.key, 0);
    }

    #[test]
    #[should_panic(expected = "double freed you dummy")]
    fn test_double_free() {
        let mut pool = TokenPool::new();
        let a = pool.alloc();
        pool.free(a);
        pool.free(a);
    }
}
