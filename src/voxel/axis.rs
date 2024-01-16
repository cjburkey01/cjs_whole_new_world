use super::CHUNK_WIDTH;
use bevy::math::{IVec3, UVec2, UVec3};

pub const SLICE_DIRECTIONS: [SliceDirection; 6] = [
    // Normal towards +Z
    SliceDirection::new(VoxelAxis::PosX, VoxelAxis::PosY),
    // Normal towards -Z
    SliceDirection::new(VoxelAxis::NegX, VoxelAxis::PosY),
    // Normal towards +X
    SliceDirection::new(VoxelAxis::NegZ, VoxelAxis::PosY),
    // Normal towards -X
    SliceDirection::new(VoxelAxis::PosZ, VoxelAxis::PosY),
    // Normal towards -Y
    SliceDirection::new(VoxelAxis::PosX, VoxelAxis::PosZ),
    // Normal towards +Y
    SliceDirection::new(VoxelAxis::NegX, VoxelAxis::PosZ),
];

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum VoxelAxis {
    PosX,
    PosY,
    PosZ,
    NegX,
    NegY,
    NegZ,
}

impl VoxelAxis {
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
            VoxelAxis::PosX => IVec3::X,
            VoxelAxis::PosY => IVec3::Y,
            VoxelAxis::PosZ => IVec3::Z,
            VoxelAxis::NegX => IVec3::NEG_X,
            VoxelAxis::NegY => IVec3::NEG_Y,
            VoxelAxis::NegZ => IVec3::NEG_Z,
        }
    }

    pub const fn negate(self) -> VoxelAxis {
        match self {
            VoxelAxis::PosX => VoxelAxis::NegX,
            VoxelAxis::PosY => VoxelAxis::NegY,
            VoxelAxis::PosZ => VoxelAxis::NegZ,
            VoxelAxis::NegX => VoxelAxis::PosX,
            VoxelAxis::NegY => VoxelAxis::PosY,
            VoxelAxis::NegZ => VoxelAxis::PosZ,
        }
    }

    pub const fn is_positive(self) -> bool {
        matches!(self, Self::PosX | Self::PosY | Self::PosZ)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SliceDirection {
    pub right: VoxelAxis,
    pub up: VoxelAxis,
    normal: VoxelAxis,
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
    pub const fn new(right: VoxelAxis, up: VoxelAxis) -> Self {
        Self {
            right,
            up,
            normal: VoxelAxis::from_ivec3(const_cross(right.to_ivec3(), up.to_ivec3()))
                .expect("how did u get a fucked up normal with ur axes??"),
        }
    }

    pub fn normal(&self) -> VoxelAxis {
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
