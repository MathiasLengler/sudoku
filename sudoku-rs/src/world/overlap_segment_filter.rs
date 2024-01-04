#[allow(clippy::struct_excessive_bools)]
#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct OverlapSegmentFilter {
    pub(crate) top_left: bool,
    pub(crate) top: bool,
    pub(crate) top_right: bool,
    pub(crate) left: bool,
    pub(crate) middle: bool,
    pub(crate) right: bool,
    pub(crate) bottom_left: bool,
    pub(crate) bottom: bool,
    pub(crate) bottom_right: bool,
}

impl OverlapSegmentFilter {
    // Row-major order of segments
    pub(crate) fn contains_index(&self, index: u8) -> bool {
        match index {
            0 => self.top_left,
            1 => self.top,
            2 => self.top_right,
            3 => self.left,
            4 => self.middle,
            5 => self.right,
            6 => self.bottom_left,
            7 => self.bottom,
            8 => self.bottom_right,
            _ => false,
        }
    }
}
