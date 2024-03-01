#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub enum LodState {
    #[default]
    None,
    Loading,
    Ready,
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub enum LodNeededState {
    #[default]
    Deleted,
    Render,
}