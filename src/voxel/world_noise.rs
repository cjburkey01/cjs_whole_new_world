use crate::voxel::{Chunk, InChunkPos, Voxel, CHUNK_WIDTH};
use bevy::prelude::*;
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Fbm, MultiFractal, Simplex,
};

#[derive(Resource)]
pub struct WorldNoiseSettings {
    fbm: Fbm<Simplex>,
}

impl WorldNoiseSettings {
    pub fn new(seed: u32) -> Self {
        Self {
            fbm: Fbm::<Simplex>::new(seed).set_frequency(0.01),
        }
    }

    pub fn noise_chunk_heightmap(&self, chunk_pos: IVec3) -> Vec<f64> {
        PlaneMapBuilder::<_, 2>::new(&self.fbm)
            .set_size(CHUNK_WIDTH as usize, CHUNK_WIDTH as usize)
            .set_x_bounds(chunk_pos.x as f64, chunk_pos.x as f64 + 1.0)
            .set_y_bounds(chunk_pos.z as f64, chunk_pos.z as f64 + 1.0)
            .build()
            .into_iter()
            .collect::<Vec<_>>()
    }

    pub fn build_heightmap_chunk(&self, chunk_pos: IVec3) -> Chunk {
        let mut chunk = Chunk::default();
        let heightmap = self.noise_chunk_heightmap(chunk_pos);
        let mut definitely_empty = true;
        for z in 0..CHUNK_WIDTH {
            for x in 0..CHUNK_WIDTH {
                let height = heightmap[(z * CHUNK_WIDTH + x) as usize] * 150.0
                    - (chunk_pos.y * CHUNK_WIDTH as i32) as f64
                    + 10.0;
                for y in 0..(height.max(0.0) as u32).min(CHUNK_WIDTH) {
                    definitely_empty = false;
                    chunk.voxels.set(
                        InChunkPos::new(UVec3::new(x, y, z)).unwrap(),
                        match (y as f64) < (height - 3.0) {
                            true => Voxel::Stone,
                            false => Voxel::Grass,
                        },
                    );
                }
            }
        }
        Chunk {
            definitely_empty,
            ..chunk
        }
    }
}
