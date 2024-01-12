use bevy::{
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};
use bitvec::prelude::*;
use std::ops::Deref;

pub const CHUNK_WIDTH: u32 = 16;
pub const CHUNK_SQUARE: u32 = CHUNK_WIDTH * CHUNK_WIDTH;
pub const CHUNK_CUBE: u32 = CHUNK_SQUARE * CHUNK_WIDTH;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct InChunkPos(UVec3);

impl InChunkPos {
    pub fn new(pos: UVec3) -> Option<Self> {
        match pos.max_element() < CHUNK_WIDTH {
            true => Some(Self(pos)),
            false => None,
        }
    }

    pub fn index(&self) -> usize {
        (CHUNK_SQUARE * self.z + CHUNK_WIDTH * self.y + self.x) as usize
    }

    pub fn inner(&self) -> UVec3 {
        self.0
    }
}

impl Deref for InChunkPos {
    type Target = UVec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Voxel {
    #[default]
    Air,
    Stone,
    Grass,
}

impl Voxel {
    pub fn does_cull_as_solid(&self) -> bool {
        match *self {
            Voxel::Air => false,
            Voxel::Stone => true,
            Voxel::Grass => true,
        }
    }

    pub fn uv_min_max(&self) -> (Vec2, Vec2) {
        // TODO: THIS
        (Vec2::ZERO, Vec2::ONE)
    }
}

pub type ChunkVoxels = [Voxel; CHUNK_CUBE as usize];

pub struct Chunk {
    bits: BitVec,
    voxels: Box<ChunkVoxels>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            bits: BitVec::repeat(false, CHUNK_CUBE as usize),
            voxels: Box::new([default(); CHUNK_CUBE as usize]),
        }
    }

    pub fn from_voxels(voxels: Vec<Voxel>) -> Option<Self> {
        match voxels.len() as u32 {
            CHUNK_CUBE => Some(Self {
                bits: BitVec::from_iter(voxels.iter().map(Voxel::does_cull_as_solid)),
                voxels: voxels.try_into().ok()?,
            }),
            _ => None,
        }
    }

    pub fn at(&self, pos: InChunkPos) -> Voxel {
        self.voxels[pos.index()]
    }

    pub fn set(&mut self, pos: InChunkPos, voxel: Voxel) {
        let index = pos.index();
        self.bits.set(index, voxel.does_cull_as_solid());
        self.voxels[index] = voxel;
    }

    pub fn generate_mesh(&self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

        let mut tmp_mesh = TmpMesh::default();

        self.mesh_slice(&mut tmp_mesh);

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, tmp_mesh.verts);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, tmp_mesh.uvs);
        mesh.set_indices(Some(Indices::U16(tmp_mesh.inds)));

        mesh
    }

    fn mesh_slice(&self, mesh: &mut TmpMesh) {
        fn emit_quad(
            z: u32,
            mut quad: Quad,
            voxels: &[Voxel],
            slice_bits: &mut BitVec,
            mesh: &mut TmpMesh,
        ) {
            // Loop through each y value between y+1 and CHUNK_WIDTH:
            //   Loop from quad.start.x up to quad.end.x:
            //     If any voxels are not equal to the quad's voxel type:
            //       Break the outer loop
            //
            //   Set the slice_bits to true between quad.start.x and
            //   quad.end.x on row y.
            //   Increment the quad's end.y value

            'outer: for y_check in (quad.start.y + 1)..CHUNK_WIDTH {
                for x_check in quad.start.x..quad.end_excl.x {
                    let in_pos = InChunkPos::new(UVec3::new(x_check, y_check, z)).unwrap();
                    if voxels[in_pos.index()] != quad.voxel {
                        break 'outer;
                    }
                }

                for x_check in quad.start.x..quad.end_excl.x {
                    slice_bits.set(x_check as usize, true);
                }
                quad.end_excl.y += 1;
            }

            mesh.add_quad(quad);
        }

        let mut slice_bits: BitVec<usize, Lsb0> = BitVec::repeat(false, CHUNK_SQUARE as usize);

        let z = 0;
        for y in 0..CHUNK_WIDTH {
            let row = CHUNK_WIDTH * y;
            let mut current_quad: Option<Quad> = None;
            for x in 0..CHUNK_WIDTH {
                // If the slice bit for this pos is `true`:
                //   If the current quad is `Some`:
                //     Perform quad emit.
                //   Continue loop;
                //
                // Set slice bit for this pos to `true`.
                // If the current quad voxel is `Some`:
                //   If the voxel at this position is the same type as the current quad:
                //     Increment quad max end x.
                //     Continue loop;
                //   Otherwise:
                //     Perform quad emit.
                //     If the voxel at this position is not air:
                //       Set current voxel to `Some` with this type.
                // Otherwise:
                //   If the voxel at this position is not air:
                //     Set current voxel to `Some` with this type.
                //
                // After the X loop, emit the current quad if it is `Some`

                let slice_pos = (row + x) as usize;

                let pos = InChunkPos::new(UVec3::new(x, y, z)).unwrap();
                let voxel = self.voxels[pos.index()];

                if slice_bits[slice_pos] {
                    if let Some(quad) = current_quad.take() {
                        emit_quad(z, quad, self.voxels.as_slice(), &mut slice_bits, mesh);
                    }
                    continue;
                }

                slice_bits.set(slice_pos, true);

                if let Some(mut quad) = current_quad.take() {
                    if quad.voxel == voxel {
                        quad.end_excl.x += 1;
                        // Put the current quad back
                        current_quad = Some(quad);
                        continue;
                    } else {
                        emit_quad(z, quad, self.voxels.as_slice(), &mut slice_bits, mesh);

                        if voxel != Voxel::Air {
                            current_quad = Some(Quad::new(UVec2::new(x, y), voxel));
                        }
                    }
                } else if voxel != Voxel::Air {
                    current_quad = Some(Quad::new(UVec2::new(x, y), voxel));
                }
            }

            if let Some(quad) = current_quad.take() {
                emit_quad(z, quad, self.voxels.as_slice(), &mut slice_bits, mesh);
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Quad {
    start: UVec2,
    end_excl: UVec2,
    voxel: Voxel,
}

impl Quad {
    pub fn new(pos: UVec2, voxel: Voxel) -> Self {
        Self {
            start: pos,
            end_excl: pos + UVec2::ONE,
            voxel,
        }
    }
}

#[derive(Default)]
struct TmpMesh {
    verts: Vec<Vec3>,
    inds: Vec<u16>,
    uvs: Vec<Vec2>,
}

impl TmpMesh {
    pub fn add_quad(&mut self, quad: Quad) {
        // TODO:
        //       We are at the zero Z, moving sideways along positive-X and
        //       vertically along positive-Y.
        //       Positive X cross positive Y is positive Z, which we can
        //       consider the the normal, making forward -Z. From this
        //       perspective, we are meshing the quads located along Z=1.
        //       When the normal is negative, AKA, when the cross product of
        //       the basis vectors is positive, we must offset the output
        //       vertices by one, as we're viewing these quads from their
        //       opposite side.
        //       Also, we need to go counter-clockwise as if the viewport is
        //       looking along negative-Z, so that's a thing to look out for.
        //       i guess.
        //       When we extend this behavior to other directions, we'll need
        //       to take all this shit into account.

        // TODO: set Z to 0.0 if cross product of bases is positive.
        let Quad {
            start,
            end_excl: end,
            voxel,
        } = quad;

        let start_ind = self.verts.len() as u16;

        // Add vertices
        {
            let start_vert = start.as_vec2();
            let end_vert = end.as_vec2();
            let low_right_vert = UVec2::new(end.x, start.y).as_vec2();
            let high_left_vert = UVec2::new(start.x, end.y).as_vec2();
            self.verts.append(
                &mut [start_vert, end_vert, low_right_vert, high_left_vert]
                    .into_iter()
                    .map(|v| v.extend(1.0))
                    .collect(),
            );
        }

        // Add indices to make a quad
        self.inds.append(
            &mut [0u16, 1, 3, 0, 2, 1]
                .into_iter()
                .map(|i| start_ind + i)
                .collect(),
        );

        // Add UVs for each vertex
        {
            let (high_left_uv, low_right_uv) = voxel.uv_min_max();
            let end_uv = Vec2::new(low_right_uv.x, high_left_uv.y);
            let start_uv = Vec2::new(high_left_uv.x, low_right_uv.y);
            self.uvs
                .append(&mut vec![start_uv, end_uv, low_right_uv, high_left_uv]);
        }
    }
}
