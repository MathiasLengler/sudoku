/// World grids overlap in both directions.
/// This creates distinct segments, which this enum refers to.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum OverlapSegment {
    TopLeft,
    Top,
    TopRight,
    Left,
    Middle,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct OverlapSegments<T> {
    pub top_left: T,
    pub top: T,
    pub top_right: T,
    pub left: T,
    pub middle: T,
    pub right: T,
    pub bottom_left: T,
    pub bottom: T,
    pub bottom_right: T,
}

impl<T> OverlapSegments<T> {
    pub fn into_iter_filtered(self, filter: OverlapSegmentFilter) -> impl Iterator<Item = T> {
        self.into_iter()
            .zip(filter)
            .filter_map(|(segment, is_contained)| is_contained.then_some(segment))
    }
}

impl<T> IntoIterator for OverlapSegments<T> {
    type Item = T;

    type IntoIter = std::array::IntoIter<T, 9>;

    fn into_iter(self) -> Self::IntoIter {
        let Self {
            top_left,
            top,
            top_right,
            left,
            middle,
            right,
            bottom_left,
            bottom,
            bottom_right,
        } = self;

        [
            top_left,
            top,
            top_right,
            left,
            middle,
            right,
            bottom_left,
            bottom,
            bottom_right,
        ]
        .into_iter()
    }
}

pub type OverlapSegmentFilter = OverlapSegments<bool>;

impl OverlapSegmentFilter {
    pub fn contains(self, segment: OverlapSegment) -> bool {
        match segment {
            OverlapSegment::TopLeft => self.top_left,
            OverlapSegment::Top => self.top,
            OverlapSegment::TopRight => self.top_right,
            OverlapSegment::Left => self.left,
            OverlapSegment::Middle => self.middle,
            OverlapSegment::Right => self.right,
            OverlapSegment::BottomLeft => self.bottom_left,
            OverlapSegment::Bottom => self.bottom,
            OverlapSegment::BottomRight => self.bottom_right,
        }
    }
}
