// splitmix64 + value noise + FNV-1a — identical algorithm in
// terra-c/src/noise.c; worldgen determinism across languages depends on it.
// Never use rand()-style crates here.

pub fn rng_next(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

/// [0, 1), 24-bit mantissa — exact f32, bit-identical to the C side
pub fn rng_float(state: &mut u64) -> f32 {
    (rng_next(state) >> 40) as f32 / 16777216.0
}

/// [min, max] inclusive
pub fn rng_range(state: &mut u64, min: i32, max: i32) -> i32 {
    min + (rng_next(state) % (max - min + 1) as u64) as i32
}

fn lattice(seed: u64, ix: i32) -> f32 {
    let mut s = seed ^ (ix as u32 as u64).wrapping_mul(0x9E3779B97F4A7C15);
    (rng_next(&mut s) >> 40) as f32 / 16777216.0
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

pub fn value_noise1(seed: u64, x: f32) -> f32 {
    let fx = x.floor();
    let ix = fx as i32;
    let t = x - fx;
    lerp(lattice(seed, ix), lattice(seed, ix + 1), smooth(t))
}

/// 4 octaves, lacunarity 2.0, gain 0.5
pub fn fbm1(seed: u64, x: f32) -> f32 {
    let mut total = 0.0f32;
    let mut amp = 1.0f32;
    let mut freq = 1.0f32;
    let mut norm = 0.0f32;
    for o in 0..4u64 {
        total += amp * value_noise1(seed.wrapping_add(o), x * freq);
        norm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    total / norm
}

pub fn fnv1a64(data: &[u8]) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}
