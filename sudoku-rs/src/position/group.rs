#![allow(dead_code)]

use crate::base::SudokuBase;

use super::Coordinate;

// TODO: naming: Group Pointer|Index|Coordinate
// TODO: implement
// TODO: integrate/interop with Grid/Position etc
struct RowCoordinate<Base: SudokuBase>(Coordinate<Base>);
struct ColumnCoordinate<Base: SudokuBase>(Coordinate<Base>);
struct BoxCoordinate<Base: SudokuBase>(Coordinate<Base>);

// Could be used for indicating the current group in all_group_positions
enum GroupCoordinate<Base: SudokuBase> {
    Row(RowCoordinate<Base>),
    Column(ColumnCoordinate<Base>),
    Box(BoxCoordinate<Base>),
}
