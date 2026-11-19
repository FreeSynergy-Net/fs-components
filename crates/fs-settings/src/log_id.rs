//! The ids of the instance log: `UUIDv7` without a counter over the log, so an id says nothing about how many lines lie
//! between two ids (review K6-a2 B1: a bot sees only the visible lines, but their ids must not count the hidden ones).
//!
//! Pattern: Factory (the next id from the previous one and the clock).

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};

use fs_types::{ObjectId, Utc};

/// The random part of a `UUIDv7`: `rand_a` (12 bits) and `rand_b` (62 bits).
const RAND_BITS: u32 = 74;
/// Headroom a fresh value leaves below the top, so a run of increments within one millisecond cannot overflow.
const FRESH_BITS: u32 = RAND_BITS - 2;
/// The largest exponent of an increment (bits): increments are log-uniform in `1..2^MAX_STEP_BITS`.
const MAX_STEP_BITS: u32 = 40;

/// 64 random bits from the keys the standard library draws from the operating system for every `RandomState`.
fn random() -> u64 {
    RandomState::new().build_hasher().finish()
}

fn random128() -> u128 {
    (u128::from(random()) << 64) | u128::from(random())
}

/// A random increment, log-uniform: its size is spread over 40 bits, so the sum over `k` skipped lines is dominated by
/// its largest step and a gap says next to nothing about `k` (a uniform step would make the mean gap grow with `k`).
fn step() -> u128 {
    let bits = 1 + u32::try_from(random() % u64::from(MAX_STEP_BITS)).unwrap_or(0);
    1 + (u128::from(random()) >> (64 - bits))
}

fn pack(ms: u64, rand: u128) -> ObjectId {
    let mut b = [0u8; 16];
    b[..6].copy_from_slice(&ms.to_be_bytes()[2..]);
    let a = u16::try_from((rand >> 62) & 0x0FFF).unwrap_or(0);
    b[6] = 0x70 | u8::try_from(a >> 8).unwrap_or(0);
    b[7] = u8::try_from(a & 0xFF).unwrap_or(0);
    let low = u64::try_from(rand & ((1u128 << 62) - 1)).unwrap_or(0);
    b[8..].copy_from_slice(&(low | (0x8000_0000_0000_0000)).to_be_bytes());
    ObjectId(b)
}

fn unpack(id: &ObjectId) -> (u64, u128) {
    let b = &id.0;
    let mut m = [0u8; 8];
    m[2..].copy_from_slice(&b[..6]);
    let a = (u128::from(b[6] & 0x0F) << 8) | u128::from(b[7]);
    let mut l = [0u8; 8];
    l.copy_from_slice(&b[8..]);
    (
        u64::from_be_bytes(m),
        (a << 62) | u128::from(u64::from_be_bytes(l) & ((1u64 << 62) - 1)),
    )
}

/// The id of the next line, written `at` (UTC): above `prev` (ids rise with the order of writing), and made of the time
/// and random bits only, never of the length of the log. Within one millisecond the id is `prev` plus a random step.
pub(crate) fn next_id(at: Utc, prev: Option<&ObjectId>) -> ObjectId {
    let ms = u64::try_from(at.millis()).unwrap_or(0) & 0xFFFF_FFFF_FFFF;
    if let Some((p_ms, p_rand)) = prev.map(unpack)
        && p_ms >= ms
    {
        let next = p_rand + step();
        if next >> RAND_BITS == 0 {
            return pack(p_ms, next);
        }
        return pack(p_ms + 1, random128() >> (128 - FRESH_BITS));
    }
    pack(ms, random128() >> (128 - FRESH_BITS))
}
