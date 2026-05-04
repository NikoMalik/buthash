pub mod hash;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        assert_ne!(hash::low_level_hash(0, b""), 0);
    }

    #[test]
    fn test_hello() {
        assert_ne!(hash::low_level_hash(0, b"hello"), 0);
    }

    #[test]
    fn test_hash() {
        #[derive(Clone, Copy, bytemuck::NoUninit)]
        #[repr(C)] // packed layout
        struct MyKey {
            a: u64,
            b: u32,
            c: u32, //pad
        }

        let key = MyKey { a: 1, b: 2, c: 3 };
        let h = hash::hash_inline(&key, 0);
        let h2 = hash::hash_inline(b"hello", 0);

        dbg!("hash1: {}", h);
        dbg!("hash2: {}", h2);
    }
}
