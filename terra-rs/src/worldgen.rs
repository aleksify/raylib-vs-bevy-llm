use crate::consts::*;
use crate::noise::*;
use crate::world::{Tile, TileWorld};

// Generation runs in fixed stages, each on its own splitmix64 stream
// (seed+0..3) so adding calls to one stage never shifts another stage's
// randomness. Stage order and iteration order must match terra-c exactly.

pub fn generate(seed: u64) -> TileWorld {
    let w = WORLD_W as usize;
    let h = WORLD_H as usize;
    let mut tiles = vec![Tile::Air as u8; w * h];

    let mut surface = vec![0i32; w];
    let mut cave_min = vec![0i32; w];

    // 1+2: heightmap (pure fbm of column x) + strata
    let mut r_strata = seed;
    for x in 0..w {
        let hn = fbm1(seed, x as f32 * 0.015);
        let sy = (WORLD_H as f32 * 0.25 + hn * (WORLD_H as f32 * 0.20)) as i32;
        surface[x] = sy;
        let dirt_len = rng_range(&mut r_strata, 6, 10);
        cave_min[x] = sy + dirt_len + 11; // skip 10 tiles below the dirt layer
        for y in sy as usize..h {
            tiles[y * w + x] = if y as i32 == sy {
                Tile::Grass as u8
            } else if y as i32 <= sy + dirt_len {
                Tile::Dirt as u8
            } else {
                Tile::Stone as u8
            };
        }
    }

    // 3: caves — cellular automata on the stone layer, 45% fill, 5 passes,
    // 4/5 rule (3x3 count incl. self, >=5 -> wall), OOB counts as wall
    let mut walls = vec![1u8; w * h];
    let mut next = vec![1u8; w * h];
    let mut r_caves = seed.wrapping_add(1);
    for y in 0..h {
        for x in 0..w {
            walls[y * w + x] = if (y as i32) >= cave_min[x] {
                (rng_float(&mut r_caves) < 0.45) as u8
            } else {
                1
            };
        }
    }
    for _pass in 0..5 {
        for y in 0..h as i32 {
            for x in 0..w as i32 {
                let i = (y * WORLD_W + x) as usize;
                if y < cave_min[x as usize] {
                    next[i] = 1;
                    continue;
                }
                let mut n = 0;
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let (nx, ny) = (x + dx, y + dy);
                        if nx < 0 || ny < 0 || nx >= WORLD_W || ny >= WORLD_H {
                            n += 1;
                        } else {
                            n += walls[(ny * WORLD_W + nx) as usize] as i32;
                        }
                    }
                }
                next[i] = (n >= 5) as u8;
            }
        }
        std::mem::swap(&mut walls, &mut next);
    }
    for y in 0..h {
        for x in 0..w {
            if y as i32 >= cave_min[x] && walls[y * w + x] == 0 {
                tiles[y * w + x] = Tile::Air as u8;
            }
        }
    }

    let mut world = TileWorld { tiles };

    // 4: ore — 200 drunkard's walks through stone
    let mut r_ore = seed.wrapping_add(2);
    for _ in 0..200 {
        let mut x = rng_range(&mut r_ore, 0, WORLD_W - 1);
        let mut y = rng_range(&mut r_ore, 0, WORLD_H - 1);
        let len = rng_range(&mut r_ore, 4, 10);
        for _ in 0..len {
            if world.tile_at(x, y) == Tile::Stone as u8 {
                world.set_tile(x, y, Tile::Ore as u8);
            }
            match rng_range(&mut r_ore, 0, 3) {
                0 => x += 1,
                1 => x -= 1,
                2 => y += 1,
                _ => y -= 1,
            }
        }
    }

    // 5: trees — ~10% of grass columns, min 4 apart
    let mut r_trees = seed.wrapping_add(3);
    let mut last_tree = -100i32;
    for x in 2..WORLD_W - 2 {
        if x - last_tree < 4 {
            continue;
        }
        if world.tile_at(x, surface[x as usize]) != Tile::Grass as u8 {
            continue;
        }
        if rng_float(&mut r_trees) >= 0.10 {
            continue;
        }
        last_tree = x;
        let sy = surface[x as usize];
        let trunk = rng_range(&mut r_trees, 4, 7);
        for t in 1..=trunk {
            world.set_tile(x, sy - t, Tile::Wood as u8);
        }
        let cy = sy - trunk - 1;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if world.tile_at(x + dx, cy + dy) == Tile::Air as u8 {
                    world.set_tile(x + dx, cy + dy, Tile::Leaves as u8);
                }
            }
        }
    }

    world
}

/// First solid tile y in column
pub fn surface_y(world: &TileWorld, tx: i32) -> i32 {
    for y in 0..WORLD_H {
        if world.is_solid(tx, y) {
            return y;
        }
    }
    WORLD_H / 2
}
