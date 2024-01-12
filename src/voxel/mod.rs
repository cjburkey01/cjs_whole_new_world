pub mod world_noise;

use crate::plugin::voxel_material::ATTRIBUTE_ATLAS_INDEX;
use bevy::{
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};
use bitvec::prelude::*;
use std::ops::{Deref, DerefMut};

// TODO: WHY IS AN EXTRA QUAD GENERATED WHEN THE NORMAL IS NEGATIVE??

pub const CHUNK_WIDTH: u32 = 32;
pub const CHUNK_SQUARE: u32 = CHUNK_WIDTH * CHUNK_WIDTH;
pub const CHUNK_CUBE: u32 = CHUNK_SQUARE * CHUNK_WIDTH;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Axis {
    PosX,
    PosY,
    PosZ,
    NegX,
    NegY,
    NegZ,
}

impl Axis {
    pub fn from_ivec3(ivec3: IVec3) -> Option<Self> {
        match ivec3 {
            IVec3::X => Some(Self::PosX),
            IVec3::Y => Some(Self::PosY),
            IVec3::Z => Some(Self::PosZ),
            IVec3::NEG_X => Some(Self::NegX),
            IVec3::NEG_Y => Some(Self::NegY),
            IVec3::NEG_Z => Some(Self::NegZ),
            _ => None,
        }
    }

    pub fn to_ivec3(self) -> IVec3 {
        match self {
            Axis::PosX => IVec3::X,
            Axis::PosY => IVec3::Y,
            Axis::PosZ => IVec3::Z,
            Axis::NegX => IVec3::NEG_X,
            Axis::NegY => IVec3::NEG_Y,
            Axis::NegZ => IVec3::NEG_Z,
        }
    }

    pub fn negate(self) -> Axis {
        Self::from_ivec3(-self.to_ivec3()).unwrap()
    }

    pub fn is_positive(self) -> bool {
        matches!(self, Self::PosX | Self::PosY | Self::PosZ)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SliceDirection {
    pub right: Axis,
    pub up: Axis,
}

impl SliceDirection {
    pub fn normal(&self) -> Axis {
        Axis::from_ivec3(self.right.to_ivec3().cross(self.up.to_ivec3()))
            .expect("how did u get a fucked up normal with ur axes??")
    }

    pub fn transform(&self, slice_depth: u32, slice_pos: UVec2) -> Option<UVec3> {
        let pos = self.exclusive_transform(slice_depth, slice_pos);

        match pos.min_element() < 0 {
            false => Some(pos.as_uvec3()),
            true => None,
        }
    }

    pub fn exclusive_transform(&self, slice_depth: u32, slice_pos: UVec2) -> IVec3 {
        let mut pos = self.right.to_ivec3() * slice_pos.x as i32
            + self.up.to_ivec3() * slice_pos.y as i32
            + self.normal().to_ivec3() * slice_depth as i32;

        if !self.right.is_positive() {
            pos += self.right.negate().to_ivec3() * (CHUNK_WIDTH as i32 - 1);
        }
        if !self.up.is_positive() {
            pos += self.up.negate().to_ivec3() * (CHUNK_WIDTH as i32 - 1);
        }
        if !self.normal().is_positive() {
            pos += self.normal().negate().to_ivec3() * (CHUNK_WIDTH as i32 - 1);
        }

        pos
    }
}

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
    #[allow(unused)]
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

    pub fn atlas_index(&self) -> u32 {
        match self {
            Voxel::Air => 0,
            Voxel::Stone => 1,
            Voxel::Grass => 0,
        }
    }
}

pub struct VoxelContainer(pub Box<[Voxel; CHUNK_CUBE as usize]>);

impl Default for VoxelContainer {
    fn default() -> Self {
        Self(Box::new([default(); CHUNK_CUBE as usize]))
    }
}

impl Deref for VoxelContainer {
    type Target = [Voxel];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

impl DerefMut for VoxelContainer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut_slice()
    }
}

impl VoxelContainer {
    #[allow(unused)]
    pub fn from_voxel(voxel: Voxel) -> Self {
        Self(Box::new([voxel; CHUNK_CUBE as usize]))
    }

    #[allow(unused)]
    pub fn from_voxels(voxels: Vec<Voxel>) -> Option<Self> {
        match voxels.len() as u32 {
            CHUNK_CUBE => Some(Self(voxels.try_into().ok()?)),
            _ => None,
        }
    }

    pub fn at(&self, pos: InChunkPos) -> Voxel {
        self.0[pos.index()]
    }

    pub fn set(&mut self, pos: InChunkPos, voxel: Voxel) {
        self.0[pos.index()] = voxel;
    }
}

#[derive(Default)]
pub struct Chunk {
    voxels: VoxelContainer,
    pub definitely_empty: bool,
}

impl Chunk {
    #[allow(unused)]
    pub fn new(voxels: VoxelContainer) -> Self {
        Self {
            voxels,
            ..default()
        }
    }

    #[allow(unused)]
    pub fn at(&self, pos: InChunkPos) -> Voxel {
        self.voxels.at(pos)
    }

    #[allow(unused)]
    pub fn set(&mut self, pos: InChunkPos, voxel: Voxel) {
        self.voxels.set(pos, voxel);
        if voxel != Voxel::Air {
            self.definitely_empty = false;
        }
    }

    pub fn generate_mesh(&self) -> Mesh {
        let mut tmp_mesh = TmpMesh::default();

        if !self.definitely_empty {
            let mesh_directions = [
                // Normal towards +Z
                SliceDirection {
                    right: Axis::PosX,
                    up: Axis::PosY,
                },
                // Normal towards -Z
                SliceDirection {
                    right: Axis::NegX,
                    up: Axis::PosY,
                },
                // Normal towards +X
                SliceDirection {
                    right: Axis::NegZ,
                    up: Axis::PosY,
                },
                // Normal towards -X
                SliceDirection {
                    right: Axis::PosZ,
                    up: Axis::PosY,
                },
                // Normal towards -Y
                SliceDirection {
                    right: Axis::PosX,
                    up: Axis::PosZ,
                },
                // Normal towards +Y
                SliceDirection {
                    right: Axis::NegX,
                    up: Axis::PosZ,
                },
            ];

            for dir in mesh_directions {
                for z in 0..CHUNK_WIDTH {
                    self.mesh_slice(
                        dir,
                        z,
                        &mut tmp_mesh,
                        if z < CHUNK_WIDTH - 1 {
                            self.get_solid_bits_slice(dir, z + 1)
                        } else {
                            None
                        },
                    );
                }
            }
        }

        tmp_mesh.build()
    }

    fn get_solid_bits_slice(
        &self,
        slice_direction: SliceDirection,
        slice_depth: u32,
    ) -> Option<BitVec> {
        let mut bit_slice = BitVec::repeat(false, CHUNK_CUBE as usize);
        for y in 0..CHUNK_WIDTH {
            let slice_row_index = y * CHUNK_WIDTH;
            for x in 0..CHUNK_WIDTH {
                let voxel = self.voxels.at(InChunkPos::new(
                    slice_direction.transform(slice_depth, UVec2::new(x, y))?,
                )?);
                let slice_index = slice_row_index + x;
                bit_slice.set(slice_index as usize, voxel.does_cull_as_solid());
            }
        }
        Some(bit_slice)
    }

    fn mesh_slice(
        &self,
        slice_direction: SliceDirection,
        slice_depth: u32,
        mesh: &mut TmpMesh,
        previous_slice_bits: Option<BitVec>,
    ) {
        fn emit_quad(
            slice_direction: SliceDirection,
            slice_depth: u32,
            mut quad: Quad,
            voxels: &[Voxel],
            slice_bits: &mut BitVec,
            previous_slice_bits: &Option<BitVec>,
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
                let slice_row_index = y_check * CHUNK_WIDTH;
                for x_check in quad.start.x..quad.end_excl.x {
                    let slice_index = (slice_row_index + x_check) as usize;
                    let in_pos = InChunkPos::new(
                        slice_direction
                            .transform(slice_depth, UVec2::new(x_check, y_check))
                            .unwrap(),
                    )
                    .unwrap();
                    if voxels[in_pos.index()] != quad.voxel
                        || slice_bits[slice_index]
                        || previous_slice_bits
                            .as_ref()
                            .map(|b| b[slice_index])
                            .unwrap_or(false)
                    {
                        break 'outer;
                    }
                }

                for x_check in quad.start.x..quad.end_excl.x {
                    let slice_index = (slice_row_index + x_check) as usize;
                    slice_bits.set(slice_index, true);
                }
                quad.end_excl.y += 1;
            }

            mesh.add_quad(slice_direction, slice_depth, quad);
        }

        let mut slice_bits = BitVec::repeat(false, CHUNK_SQUARE as usize);

        for y in 0..CHUNK_WIDTH {
            let slice_row_index = CHUNK_WIDTH * y;
            let mut current_quad: Option<Quad> = None;
            for x in 0..CHUNK_WIDTH {
                let slice_index = (slice_row_index + x) as usize;

                let pos = InChunkPos::new(
                    slice_direction
                        .transform(slice_depth, UVec2::new(x, y))
                        .unwrap(),
                )
                .unwrap();
                let voxel = self.voxels.at(pos);

                // If the slice bit for this pos is `true`
                if slice_bits[slice_index]
                    || previous_slice_bits
                        .as_ref()
                        .map(|b| b[slice_index])
                        .unwrap_or(false)
                {
                    // We have to do this because the slice_bit might be false
                    // if the previous_slice_bit is true
                    slice_bits.set(slice_index, true);

                    // If the current quad is `Some`
                    if let Some(quad) = current_quad.take() {
                        // Perform quad emit.
                        emit_quad(
                            slice_direction,
                            slice_depth,
                            quad,
                            self.voxels.0.as_slice(),
                            &mut slice_bits,
                            &previous_slice_bits,
                            mesh,
                        );
                    }
                    // Go to next position
                    continue;
                }

                // Set slice bit for this pos to `true`
                slice_bits.set(slice_index, true);

                // If the current quad voxel is `Some`
                if let Some(mut quad) = current_quad.take() {
                    // If the voxel at this position is the same type as the current quad
                    if quad.voxel == voxel {
                        // Increment quad max end x
                        quad.end_excl.x += 1;
                        // Put the current quad back
                        current_quad = Some(quad);
                        continue;
                    } else {
                        // Perform quad emit.
                        emit_quad(
                            slice_direction,
                            slice_depth,
                            quad,
                            self.voxels.0.as_slice(),
                            &mut slice_bits,
                            &previous_slice_bits,
                            mesh,
                        );

                        // If the voxel at this position is not air
                        if voxel != Voxel::Air {
                            // Set current voxel to `Some` with this type
                            current_quad = Some(Quad::new(UVec2::new(x, y), voxel));
                        }
                    }
                } else if voxel != Voxel::Air {
                    // If no current quad and this voxel isn't air, make a new
                    // one.
                    current_quad = Some(Quad::new(UVec2::new(x, y), voxel));
                }
            }

            // After the X loop, emit the current quad if it is `Some`
            if let Some(quad) = current_quad.take() {
                emit_quad(
                    slice_direction,
                    slice_depth,
                    quad,
                    self.voxels.0.as_slice(),
                    &mut slice_bits,
                    &previous_slice_bits,
                    mesh,
                );
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
    norms: Vec<Vec3>,
    atlas_indices: Vec<u32>,
}

impl TmpMesh {
    pub fn add_quad(&mut self, slice_dir: SliceDirection, slice_depth: u32, quad: Quad) {
        // We are at the zero Z, moving sideways along positive-X and
        // vertically along positive-Y.
        // Positive X cross positive Y is positive Z, which we can
        // consider the the normal, making forward -Z. From this
        // perspective, we are meshing the quads located along Z=1.
        // When the normal is negative, AKA, when the cross product of
        // the basis vectors is positive, we must offset the output
        // vertices by one, as we're viewing these quads from their
        // opposite side.
        // Also, we need to go counter-clockwise as if the viewport is
        // looking along negative-Z, so that's a thing to look out for.
        // i guess.
        // When we extend this behavior to other directions, we'll need
        // to take all this shit into account.

        let Quad {
            start,
            end_excl: end,
            voxel,
        } = quad;

        let start_ind = self.verts.len() as u16;

        // Add vertices
        {
            let start_vert = start;
            let end_vert = end;
            let low_right_vert = UVec2::new(end.x, start.y);
            let high_left_vert = UVec2::new(start.x, end.y);
            self.verts.append(
                &mut [start_vert, end_vert, low_right_vert, high_left_vert]
                    .into_iter()
                    // TODO: set Z to 0.0 if cross product of bases is positive.
                    .map(|v| {
                        let mut pos = slice_dir.exclusive_transform(slice_depth, v).as_vec3();

                        if !slice_dir.right.is_positive() {
                            pos -= slice_dir.right.to_ivec3().as_vec3();
                        }
                        if !slice_dir.up.is_positive() {
                            pos -= slice_dir.up.to_ivec3().as_vec3();
                        }

                        match slice_dir.normal().is_positive() {
                            true => pos + slice_dir.normal().to_ivec3().as_vec3(),
                            false => pos,
                        }
                    })
                    .collect(),
            );
        }

        let size = (end.as_vec2() - start.as_vec2()).abs();
        // Add UVs for each vertex
        {
            let epsilon = Vec2::splat(0.001);
            let (min_uv, max_uv) = (Vec2::ZERO, size); //voxel.uv_min_max();
            let high_left_uv = min_uv + epsilon;
            let low_right_uv = max_uv - epsilon;
            let end_uv = Vec2::new(low_right_uv.x, high_left_uv.y);
            let start_uv = Vec2::new(high_left_uv.x, low_right_uv.y);
            self.uvs
                .append(&mut vec![start_uv, end_uv, low_right_uv, high_left_uv]);
        }

        // Add normals
        self.norms.append(
            &mut [slice_dir.normal().to_ivec3().as_vec3()]
                .into_iter()
                .cycle()
                .take(4)
                .collect(),
        );

        // Add indices to make a quad
        self.inds.append(
            &mut [0u16, 1, 3, 0, 2, 1]
                .into_iter()
                .map(|i| start_ind + i)
                .collect(),
        );

        self.atlas_indices
            .append(&mut [voxel.atlas_index()].into_iter().cycle().take(4).collect());
    }

    pub fn build(self) -> Mesh {
        let Self {
            verts,
            inds,
            uvs,
            norms,
            atlas_indices,
        } = self;

        Mesh::new(PrimitiveTopology::TriangleList)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, verts)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, norms)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
            .with_inserted_attribute(ATTRIBUTE_ATLAS_INDEX, atlas_indices)
            .with_indices(Some(Indices::U16(inds)))
    }
}
