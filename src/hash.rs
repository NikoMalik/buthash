use bytemuck::NoUninit;

pub trait Hashable {
    fn hash_with_seed(&self, seed: u64) -> u64;
}

impl<T: NoUninit> Hashable for T {
    #[inline(always)]
    fn hash_with_seed(&self, seed: u64) -> u64 {
        low_level_hash(seed, bytemuck::bytes_of(self))
    }
}

impl Hashable for [u8] {
    #[inline(always)]
    fn hash_with_seed(&self, seed: u64) -> u64 {
        low_level_hash(seed, self)
    }
}

#[inline(always)]
pub fn hash_inline<T: Hashable + ?Sized>(value: &T, seed: u64) -> u64 {
    value.hash_with_seed(seed)
}

const SALT: [u64; 5] = [
    0xa0761d6478bd642f,
    0xe7037ed1a0b428db,
    0x8ebc6af09c88c6e3,
    0x589965cc75374cc3,
    0x1d8e4e27c47d124f,
];

#[inline(always)]
fn mul128(lhs: u64, rhs: u64) -> u64 {
    let product = (lhs as u128) * (rhs as u128);
    (product as u64) ^ ((product >> 64) as u64)
}

#[inline(always)]
fn read_u64(ptr: *const u8) -> u64 {
    unsafe { (ptr as *const u64).read_unaligned() }
}

#[inline(always)]
fn read_u32(ptr: *const u8) -> u64 {
    unsafe { (ptr as *const u32).read_unaligned() as u64 }
}

#[inline(always)]
pub fn low_level_hash_u64(seed: u64, input: u64) -> u64 {
    let state = seed ^ SALT[0];
    let mixed = (input ^ SALT[1]).wrapping_mul(state ^ SALT[2]);
    mixed ^ (mixed >> 32)
}

#[inline(always)]
pub fn low_level_hash(seed: u64, input: &[u8]) -> u64 {
    let mut state = seed ^ SALT[0];
    let starting_len = input.len();
    let mut ptr = input.as_ptr();
    let mut len = input.len();

    if len > 64 {
        let mut dup = [state, state];
        while len > 64 {
            for i in 0..2usize {
                let base = unsafe { ptr.add(i * 32) };
                let (c0, c1, c2, c3) = unsafe {
                    (
                        read_u64(base),
                        read_u64(base.add(8)),
                        read_u64(base.add(16)),
                        read_u64(base.add(24)),
                    )
                };
                let mix1 = mul128(c0 ^ SALT[i * 2 + 1], c1 ^ dup[i]);
                let mix2 = mul128(c2 ^ SALT[i * 2 + 2], c3 ^ dup[i]);
                dup[i] = mix1.rotate_left(32);
                dup[i] ^= mix2.rotate_left(32);
            }
            ptr = unsafe { ptr.add(64) };
            len -= 64;
        }
        state = dup[0] ^ dup[1];
    }

    while len > 16 {
        let (c0, c1) = unsafe { (read_u64(ptr), read_u64(ptr.add(8))) };
        let mixed = mul128(c0 ^ SALT[1], c1 ^ state);
        state = mixed.rotate_left(32);
        ptr = unsafe { ptr.add(16) };
        len -= 16;
    }

    let mut chunk = [0u64; 2];
    if len > 8 {
        chunk[0] = read_u64(ptr);
        chunk[1] = unsafe { read_u64(ptr.add(len - 8)) };
    } else if len > 3 {
        chunk[0] = read_u32(ptr);
        chunk[1] = unsafe { read_u32(ptr.add(len - 4)) };
    } else if len > 0 {
        let inp = unsafe { std::slice::from_raw_parts(ptr, len) };
        chunk[0] = ((inp[0] as u64) << 16) | ((inp[len / 2] as u64) << 8) | (inp[len - 1] as u64);
    }

    let mut mixed = mul128(chunk[0] ^ SALT[1], chunk[1] ^ state);
    mixed = mixed.rotate_left(32);
    mixed = mul128(mixed, (starting_len as u64) ^ SALT[1]);
    mixed.rotate_left(32)
}
