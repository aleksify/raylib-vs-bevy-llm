#ifndef TERRA_NOISE_H
#define TERRA_NOISE_H

#include <stdint.h>
#include <stddef.h>

// splitmix64 — identical algorithm in terra-rs/src/noise.rs; worldgen
// determinism across languages depends on it. Never use rand() here.
uint64_t RngNext(uint64_t *state);
float RngFloat(uint64_t *state);                 // [0, 1), 24-bit mantissa
int RngRange(uint64_t *state, int min, int max); // [min, max] inclusive

float ValueNoise1(uint64_t seed, float x);       // [0, 1)
float Fbm1(uint64_t seed, float x);              // 4 octaves, lac 2.0, gain 0.5

uint64_t Fnv1a64(const uint8_t *data, size_t n);

#endif // TERRA_NOISE_H
