use super::{BiomeTable, Chunk, InChunkPos, Voxel, CHUNK_WIDTH};
use bevy::prelude::*;
use itertools::iproduct;
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Add, Constant, Fbm, Min, MultiFractal, Multiply, NoiseFn, Perlin, Power,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct Chunk2dNoiseValues {
    pub chunk_pos: IVec2,
    pub heightmap: Vec<f64>,
    pub temperature: Vec<f64>,
    pub humidity: Vec<f64>,
}

#[derive(Resource, Clone)]
pub struct WorldNoiseSettings {
    heightmap_noise: Arc<dyn NoiseFn<f64, 2> + Send + Sync>,
    temperature_noise: Arc<dyn NoiseFn<f64, 2> + Send + Sync>,
    humidity_noise: Arc<dyn NoiseFn<f64, 2> + Send + Sync>,
    #[allow(unused)]
    biome_table: BiomeTable,
}

impl WorldNoiseSettings {
    pub fn new(seed: u32, biome_table: BiomeTable) -> Self {
        let offset_seed = seed.wrapping_mul(34857923) ^ 487529837;

        Self {
            heightmap_noise: Arc::new(Add::new(
                Constant::new(10.0),
                Add::new(
                    Multiply::new(
                        Constant::new(90.0),
                        Fbm::<Perlin>::new(seed)
                            .set_frequency(0.02)
                            .set_persistence(0.6),
                    ),
                    Multiply::new(
                        Constant::new(300.0),
                        Min::new(
                            Power::new(
                                Fbm::<Perlin>::new(seed).set_frequency(0.015).set_octaves(1),
                                Constant::new(2.0),
                            ),
                            Power::new(
                                Fbm::<Perlin>::new(offset_seed)
                                    .set_frequency(0.015)
                                    .set_octaves(1),
                                Constant::new(2.0),
                            ),
                        ),
                    ),
                ),
            )),
            temperature_noise: Arc::new(
                Fbm::<Perlin>::new(seed).set_octaves(2).set_frequency(0.003),
            ),
            humidity_noise: Arc::new(
                Fbm::<Perlin>::new(offset_seed)
                    .set_octaves(2)
                    .set_frequency(0.014),
            ),
            biome_table,
        }
    }

    pub fn chunk_2d_noise_fn(
        noise_fn: &(impl NoiseFn<f64, 2> + ?Sized),
        chunk_pos: IVec2,
    ) -> Vec<f64> {
        PlaneMapBuilder::<_, 2>::new(noise_fn)
            .set_size(CHUNK_WIDTH as usize, CHUNK_WIDTH as usize)
            .set_x_bounds(chunk_pos.x as f64, chunk_pos.x as f64 + 1.0)
            .set_y_bounds(chunk_pos.y as f64, chunk_pos.y as f64 + 1.0)
            .build()
            .into_iter()
            .collect::<Vec<_>>()
    }

    pub fn generate_chunk_2d_noise(&self, chunk_pos: IVec2) -> Chunk2dNoiseValues {
        Chunk2dNoiseValues {
            chunk_pos,
            heightmap: Self::chunk_2d_noise_fn(self.heightmap_noise.as_ref(), chunk_pos),
            temperature: Self::chunk_2d_noise_fn(self.temperature_noise.as_ref(), chunk_pos),
            humidity: Self::chunk_2d_noise_fn(self.humidity_noise.as_ref(), chunk_pos),
        }
    }

    pub fn generate_chunk_from_noise(&self, y_level: i32, noise: &Chunk2dNoiseValues) -> Chunk {
        let mut chunk = Chunk::default();
        let heightmap = noise.heightmap.as_slice();

        for (z, x) in iproduct!(0..CHUNK_WIDTH, 0..CHUNK_WIDTH) {
            let height_i = heightmap[(z * CHUNK_WIDTH + x) as usize].round() as i32
                - (y_level * CHUNK_WIDTH as i32);
            let height_u = (height_i.max(0) as u32).min(CHUNK_WIDTH);

            for y in 0..height_u {
                chunk.set(
                    InChunkPos::new(UVec3::new(x, y, z)).unwrap(),
                    match y {
                        y if (y as i32) < (height_i - 2) => Voxel::Stone,
                        y if (y as i32) < (height_i - 1) => Voxel::Dirt,
                        _ => Voxel::Grass,
                    },
                );
            }
        }

        chunk.update_edge_slice_bits();
        chunk
    }
}
