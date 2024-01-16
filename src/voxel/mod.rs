pub mod world_noise;

use crate::plugin::voxel_material::ATTRIBUTE_HACK_VERT;
use bevy::{
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};
use bitvec::prelude::*;
use itertools::iproduct;
use std::ops::{Deref, DerefMut};

pub const CHUNK_WIDTH: u32 = 31;
pub const CHUNK_SQUARE: u32 = CHUNK_WIDTH * CHUNK_WIDTH;
pub const CHUNK_CUBE: u32 = CHUNK_SQUARE * CHUNK_WIDTH;

pub const SLICE_DIRECTIONS: [SliceDirection; 6] = [
    // Normal towards +Z
    SliceDirection::new(Axis::PosX, Axis::PosY),
    // Normal towards -Z
    SliceDirection::new(Axis::NegX, Axis::PosY),
    // Normal towards +X
    SliceDirection::new(Axis::NegZ, Axis::PosY),
    // Normal towards -X
    SliceDirection::new(Axis::PosZ, Axis::PosY),
    // Normal towards -Y
    SliceDirection::new(Axis::PosX, Axis::PosZ),
    // Normal towards +Y
    SliceDirection::new(Axis::NegX, Axis::PosZ),
];

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
    pub const fn from_ivec3(ivec3: IVec3) -> Option<Self> {
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

    pub const fn to_ivec3(self) -> IVec3 {
        match self {
            Axis::PosX => IVec3::X,
            Axis::PosY => IVec3::Y,
            Axis::PosZ => IVec3::Z,
            Axis::NegX => IVec3::NEG_X,
            Axis::NegY => IVec3::NEG_Y,
            Axis::NegZ => IVec3::NEG_Z,
        }
    }

    pub const fn negate(self) -> Axis {
        match self {
            Axis::PosX => Axis::NegX,
            Axis::PosY => Axis::NegY,
            Axis::PosZ => Axis::NegZ,
            Axis::NegX => Axis::PosX,
            Axis::NegY => Axis::PosY,
            Axis::NegZ => Axis::PosZ,
        }
    }

    pub const fn is_positive(self) -> bool {
        matches!(self, Self::PosX | Self::PosY | Self::PosZ)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SliceDirection {
    pub right: Axis,
    pub up: Axis,
    normal: Axis,
}

/// See [IVec3::cross]
const fn const_cross(lhs: IVec3, rhs: IVec3) -> IVec3 {
    IVec3 {
        x: lhs.y * rhs.z - rhs.y * lhs.z,
        y: lhs.z * rhs.x - rhs.z * lhs.x,
        z: lhs.x * rhs.y - rhs.x * lhs.y,
    }
}

impl SliceDirection {
    pub const fn new(right: Axis, up: Axis) -> Self {
        Self {
            right,
            up,
            normal: Axis::from_ivec3(const_cross(right.to_ivec3(), up.to_ivec3()))
                .expect("how did u get a fucked up normal with ur axes??"),
        }
    }

    pub fn normal(&self) -> Axis {
        self.normal
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
    Grass,
    Dirt,
}

impl Voxel {
    pub fn does_cull_as_solid(&self) -> bool {
        *self != Voxel::Air
    }

    pub fn atlas_index(&self) -> u32 {
        match *self {
            Voxel::Air => 0,
            Voxel::Stone => 1,
            Voxel::Grass => 0,
            Voxel::Dirt => 2,
        }
    }
}

pub struct VoxelContainer(pub Box<[Voxel; CHUNK_CUBE as usize]>);

impl Default for VoxelContainer {
    fn default() -> Self {
        Self(Box::new([default(); CHUNK_CUBE as usize]))
    }
}

impl Clone for VoxelContainer {
    fn clone(&self) -> Self {
        Self(Box::new(*self.0.as_ref()))
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
pub struct NeighborChunkSlices {
    pos_x: BitVec,
    pos_y: BitVec,
    pos_z: BitVec,
    neg_x: BitVec,
    neg_y: BitVec,
    neg_z: BitVec,
}

impl NeighborChunkSlices {
    pub fn get_in_direction(&self, direction: Axis) -> &BitVec {
        match direction {
            Axis::PosX => &self.pos_x,
            Axis::PosY => &self.pos_y,
            Axis::PosZ => &self.pos_z,
            Axis::NegX => &self.neg_x,
            Axis::NegY => &self.neg_y,
            Axis::NegZ => &self.neg_z,
        }
    }

    pub fn get_in_direction_mut(&mut self, direction: Axis) -> &mut BitVec {
        match direction {
            Axis::PosX => &mut self.pos_x,
            Axis::PosY => &mut self.pos_y,
            Axis::PosZ => &mut self.pos_z,
            Axis::NegX => &mut self.neg_x,
            Axis::NegY => &mut self.neg_y,
            Axis::NegZ => &mut self.neg_z,
        }
    }
}

#[derive(Clone, Default)]
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

    pub fn generate_mesh(&self, neighbors: NeighborChunkSlices) -> Mesh {
        let mut tmp_mesh = TmpMesh::default();

        if !self.definitely_empty {
            for (dir, z) in iproduct!(SLICE_DIRECTIONS, 0..CHUNK_WIDTH) {
                self.mesh_slice(
                    dir,
                    z,
                    &mut tmp_mesh,
                    if z < CHUNK_WIDTH - 1 {
                        self.get_solid_bits_slice(dir, z + 1)
                    } else {
                        Some(neighbors.get_in_direction(dir.normal()).clone())
                    },
                );
            }
        }

        tmp_mesh.build()
    }

    // TODO: THIS IS A VERY HOT FUNCTION!
    //       IT'S TAKING ABOUT 33% OF ALL TIME!
    pub fn get_solid_bits_slice(
        &self,
        slice_direction: SliceDirection,
        slice_depth: u32,
    ) -> Option<BitVec> {
        let mut bit_slice = BitVec::repeat(false, CHUNK_CUBE as usize);
        for (y, x) in iproduct!(0..CHUNK_WIDTH, 0..CHUNK_WIDTH) {
            let voxel = self.voxels.at(InChunkPos::new(
                slice_direction.transform(slice_depth, UVec2::new(x, y))?,
            )?);
            let slice_index = y * CHUNK_WIDTH + x;
            bit_slice.set(slice_index as usize, voxel.does_cull_as_solid());
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
pub struct Quad {
    pub start: UVec2,
    pub end_excl: UVec2,
    pub voxel: Voxel,
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
pub struct TmpMesh {
    verts: Vec<Vec3>,
    hacks: Vec<UVec2>,
    inds: Vec<u16>,
}

fn iter_to_array<Element, const N: usize>(mut iter: impl Iterator<Item = Element>) -> [Element; N] {
    // Here I use `()` to make array zero-sized -> no real use in runtime.
    // `map` creates new array, which we fill by values of iterator.
    let res: [_; N] = std::array::from_fn(|_| iter.next().unwrap());
    // Ensure that iterator finished
    assert!(iter.next().is_none());
    res
}

impl TmpMesh {
    /// Hack layout:
    ///                       Negative normal?
    ///                              \_/
    /// U32:(xxxxxx,yy)(yyyy,zzzz)(zz,nnnn,uu)(uuuvvvvv)
    ///      ^----^ ^------^ ^------^  ^-^ ^---^  ^---^
    ///        X       Y         Z    Norml  U      V
    /// 3x6 bits = 0-32** for each position component
    /// 3 bits for each axis of normal, 1 bit for negative.
    /// 2x5 bits = 0-31 for quad size to determine UV
    /// 4 bytes for U32 to represent atlas index
    pub fn build_hack_verts(
        slice_dir: SliceDirection,
        slice_depth: u32,
        quad: Quad,
    ) -> [(Vec3, u32); 4] {
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

        let Quad {
            start,
            end_excl: end,
            ..
        } = quad;

        let start_vert = start;
        let end_vert = end;
        let low_right_vert = UVec2::new(end.x, start.y);
        let high_left_vert = UVec2::new(start.x, end.y);

        let size = (end.as_ivec2() - start.as_ivec2()).abs().as_uvec2();
        let (high_left_uv, low_right_uv) = (UVec2::ZERO, size);
        let end_uv = UVec2::new(low_right_uv.x, high_left_uv.y);
        let start_uv = UVec2::new(high_left_uv.x, low_right_uv.y);

        let normal = slice_dir.normal().to_ivec3();
        let normal_neg = normal.min_element() < 0;
        let normal = normal.abs().as_uvec3();
        let normal_bits = (normal.x << 2) | (normal.y << 1) | (normal.z);

        iter_to_array(
            [
                (start_vert, start_uv),
                (end_vert, end_uv),
                (low_right_vert, low_right_uv),
                (high_left_vert, high_left_uv),
            ]
            .into_iter()
            .map(|(v, uv)| {
                let mut pos = slice_dir.exclusive_transform(slice_depth, v);

                if !slice_dir.right.is_positive() {
                    pos -= slice_dir.right.to_ivec3();
                }
                if !slice_dir.up.is_positive() {
                    pos -= slice_dir.up.to_ivec3();
                }
                if slice_dir.normal().is_positive() {
                    pos += slice_dir.normal().to_ivec3()
                }

                let pos = pos.as_uvec3();

                let mut hack = (pos.x << 26) | (pos.y << 20) | (pos.z << 14);
                if normal_neg {
                    hack |= 1 << 13;
                }
                (
                    pos.as_vec3(),
                    hack | (normal_bits << 10) | (uv.x << 5) | uv.y,
                )
            }),
        )
    }

    pub fn add_quad(&mut self, slice_dir: SliceDirection, slice_depth: u32, quad: Quad) {
        let start_ind = self.hacks.len() as u16;

        let verts_hacks = Self::build_hack_verts(slice_dir, slice_depth, quad);

        self.verts
            .append(&mut verts_hacks.map(|(vert, _)| vert).to_vec());

        self.hacks.append(
            &mut verts_hacks
                .map(|(_, hack)| UVec2::new(hack, quad.voxel.atlas_index()))
                .to_vec(),
        );

        // Add indices to make a quad
        self.inds.append(
            &mut [0u16, 1, 3, 0, 2, 1]
                .into_iter()
                .map(|i| start_ind + i)
                .collect(),
        );
    }

    pub fn build(self) -> Mesh {
        let Self { verts, inds, hacks } = self;

        Mesh::new(PrimitiveTopology::TriangleList)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, verts)
            .with_inserted_attribute(ATTRIBUTE_HACK_VERT, hacks)
            .with_indices(Some(Indices::U16(inds)))
    }
}
