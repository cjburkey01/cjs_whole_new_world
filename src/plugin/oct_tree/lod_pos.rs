use bevy::prelude::*;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct LodPos {
    pub level: u8,
    pub pos: IVec3,
}

#[allow(unused)]
impl LodPos {
    pub fn to_level(&self, level: u8) -> Self {
        let diff = level as i16 - self.level as i16;
        let diff_pow = 1 << diff.abs() as usize;
        match diff {
            // No change
            0 => *self,
            // Increase in level decreases position
            d if d > 0 => Self {
                level,
                pos: self.pos.div_euclid(IVec3::splat(diff_pow)),
            },
            // Decrease in level increases position
            _ => Self {
                level,
                pos: self.pos * diff_pow,
            },
        }
    }

    pub fn parent(&self) -> Self {
        Self {
            level: self.level + 1,
            pos: self.pos.div_euclid(IVec3::splat(2)),
        }
    }

    pub fn start_child(&self) -> Option<Self> {
        self.level.checked_sub(1).map(|level| Self {
            level,
            pos: self.pos * 2,
        })
    }

    pub fn children(&self) -> Option<[LodPos; 8]> {
        self.start_child().map(|start_pos| {
            [
                start_pos,
                LodPos {
                    level: start_pos.level,
                    pos: start_pos.pos + IVec3::X,
                },
                LodPos {
                    level: start_pos.level,
                    pos: start_pos.pos + IVec3::Y,
                },
                LodPos {
                    level: start_pos.level,
                    pos: start_pos.pos + IVec3::Y + IVec3::X,
                },
                LodPos {
                    level: start_pos.level,
                    pos: start_pos.pos + IVec3::Z,
                },
                LodPos {
                    level: start_pos.level,
                    pos: start_pos.pos + IVec3::Z + IVec3::X,
                },
                LodPos {
                    level: start_pos.level,
                    pos: start_pos.pos + IVec3::Z + IVec3::Y,
                },
                LodPos {
                    level: start_pos.level,
                    pos: start_pos.pos + IVec3::Z + IVec3::Y + IVec3::X,
                },
            ]
        })
    }
}
