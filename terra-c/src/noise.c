#include "noise.h"
#include <math.h>

uint64_t RngNext(uint64_t *state)
{
    uint64_t z = (*state += 0x9E3779B97F4A7C15ULL);
    z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9ULL;
    z = (z ^ (z >> 27)) * 0x94D049BB133111EBULL;
    return z ^ (z >> 31);
}

float RngFloat(uint64_t *state)
{
    // Top 24 bits -> exact f32 in [0,1); bit-identical to the Rust side
    return (float)(RngNext(state) >> 40) / 16777216.0f;
}

int RngRange(uint64_t *state, int min, int max)
{
    return min + (int)(RngNext(state) % (uint64_t)(max - min + 1));
}

static float Lattice(uint64_t seed, int32_t ix)
{
    uint64_t s = seed ^ ((uint64_t)(uint32_t)ix * 0x9E3779B97F4A7C15ULL);
    return (float)(RngNext(&s) >> 40) / 16777216.0f;
}

static float Smooth(float t) { return t * t * (3.0f - 2.0f * t); }
static float Lerp(float a, float b, float t) { return a + t * (b - a); }

float ValueNoise1(uint64_t seed, float x)
{
    float fx = floorf(x);
    int32_t ix = (int32_t)fx;
    float t = x - fx;
    return Lerp(Lattice(seed, ix), Lattice(seed, ix + 1), Smooth(t));
}

float Fbm1(uint64_t seed, float x)
{
    float total = 0.0f, amp = 1.0f, freq = 1.0f, norm = 0.0f;
    for (int o = 0; o < 4; o++) {
        total += amp * ValueNoise1(seed + (uint64_t)o, x * freq);
        norm += amp;
        amp *= 0.5f;
        freq *= 2.0f;
    }
    return total / norm;
}

uint64_t Fnv1a64(const uint8_t *data, size_t n)
{
    uint64_t h = 0xcbf29ce484222325ULL;
    for (size_t i = 0; i < n; i++) {
        h ^= data[i];
        h *= 0x100000001b3ULL;
    }
    return h;
}
