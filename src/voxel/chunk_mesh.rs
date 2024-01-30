use super::{
    Chunk, InChunkPos, NeighborChunkSlices, SliceDirection, Voxel, CHUNK_SQUARE, CHUNK_WIDTH,
    SLICE_DIRECTIONS,
};
use crate::plugin::voxel_material::ATTRIBUTE_HACK_VERT;
use bevy::{
    math::{UVec2, Vec3},
    prelude::Mesh,
    render::mesh::{Indices, PrimitiveTopology},
};
use bevy_rapier3d::prelude::Collider;
use bitvec::prelude::BitVec;
use itertools::iproduct;

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
pub struct TmpChunkMesh {
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

impl TmpChunkMesh {
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

    pub fn build(self) -> Option<(Collider, Mesh)> {
        let Self { verts, inds, hacks } = self;

        let mut collider_inds = Vec::with_capacity(inds.len() / 3);
        for i in 0..collider_inds.capacity() {
            collider_inds.push([
                inds[3 * i] as u32,
                inds[3 * i + 1] as u32,
                inds[3 * i + 2] as u32,
            ]);
        }

        match inds.is_empty() {
            true => None,
            false => Some((
                Collider::trimesh(verts.clone(), collider_inds),
                Mesh::new(PrimitiveTopology::TriangleList)
                    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, verts)
                    .with_inserted_attribute(ATTRIBUTE_HACK_VERT, hacks)
                    .with_indices(Some(Indices::U16(inds))),
            )),
        }
    }
}

pub fn generate_mesh(chunk: &Chunk, neighbors: NeighborChunkSlices) -> Option<(Collider, Mesh)> {
    let mut tmp_mesh = TmpChunkMesh::default();

    if !chunk.definitely_empty {
        for (dir, z) in iproduct!(SLICE_DIRECTIONS, 0..CHUNK_WIDTH) {
            mesh_slice(
                chunk,
                dir,
                z,
                &mut tmp_mesh,
                if z < CHUNK_WIDTH - 1 {
                    chunk.get_solid_bits_slice(dir, z + 1)
                } else {
                    Some(neighbors.get_in_direction(dir.normal()).clone())
                },
            );
        }
    }

    tmp_mesh.build()
}

fn mesh_slice(
    chunk: &Chunk,
    slice_direction: SliceDirection,
    slice_depth: u32,
    mesh: &mut TmpChunkMesh,
    previous_slice_bits: Option<BitVec>,
) {
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
            let voxel = chunk.at(pos);

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
                        chunk.as_slice(),
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
                        chunk.as_slice(),
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
                chunk.as_slice(),
                &mut slice_bits,
                &previous_slice_bits,
                mesh,
            );
        }
    }
}

fn emit_quad(
    slice_direction: SliceDirection,
    slice_depth: u32,
    mut quad: Quad,
    voxels: &[Voxel],
    slice_bits: &mut BitVec,
    previous_slice_bits: &Option<BitVec>,
    mesh: &mut TmpChunkMesh,
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
