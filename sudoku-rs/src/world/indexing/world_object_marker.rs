use std::fmt::Debug;
use std::hash::Hash;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CellMarker;
impl WorldObject for CellMarker {}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct GridMarker;
impl WorldObject for GridMarker {}

pub trait WorldObject
where
    Self: Ord + Hash + Clone + Copy + Debug + Default + Send + Sync + 'static,
{
}
