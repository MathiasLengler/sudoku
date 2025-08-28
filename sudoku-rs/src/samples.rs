use crate::base::consts::*;
use crate::base::SudokuBase;
use crate::cell::Candidates;
use crate::cell::Cell;
use crate::error::Result;
use crate::generator::{Generator, PruningSettings, PruningTarget};
use crate::grid::Grid;

pub fn grid<Base: SudokuBase>(index: usize) -> Grid<Base> {
    Base::grid_samples().nth(index).unwrap()
}

// TODO: rethink API (unwrap, clone for consumer of specific sudoku)
pub fn base_2() -> Vec<Grid<Base2>> {
    let mut grids = vec![
        vec![
            vec![0, 3, 4, 0],
            vec![4, 0, 0, 2],
            vec![1, 0, 0, 3],
            vec![0, 2, 1, 0],
        ],
        vec![
            vec![1, 0, 4, 0],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
            vec![0, 1, 0, 2],
        ],
        vec![
            vec![0, 0, 1, 0],
            vec![4, 0, 0, 0],
            vec![0, 0, 0, 2],
            vec![0, 3, 0, 0],
        ],
    ]
    .into_iter()
    .map(TryInto::<Grid<Base2>>::try_into)
    .collect::<Result<Vec<_>>>()
    .unwrap();

    for grid in &mut grids {
        grid.fix_all_values();
    }

    grids
}

pub fn base_2_solved() -> Grid<Base2> {
    Grid::<Base2>::try_from(vec![
        vec![2, 3, 4, 1],
        vec![4, 1, 3, 2],
        vec![1, 4, 2, 3],
        vec![3, 2, 1, 4],
    ])
    .unwrap()
}

pub fn base_2_candidates_coordinates() -> Grid<Base2> {
    Grid::<Base2>::with(
        (0..u8::try_from(Base2::CELL_COUNT).unwrap())
            .map(|i| Cell::with_candidates(Candidates::with_integral(i)))
            .collect(),
    )
    .unwrap()
}

pub fn base_3() -> Vec<Grid<Base3>> {
    let mut grids = vec![
        // 11 Star difficulty
        vec![
            vec![8, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![0, 0, 3, 6, 0, 0, 0, 0, 0],
            vec![0, 7, 0, 0, 9, 0, 2, 0, 0],
            vec![0, 5, 0, 0, 0, 7, 0, 0, 0],
            vec![0, 0, 0, 0, 4, 5, 7, 0, 0],
            vec![0, 0, 0, 1, 0, 0, 0, 3, 0],
            vec![0, 0, 1, 0, 0, 0, 0, 6, 8],
            vec![0, 0, 8, 5, 0, 0, 0, 1, 0],
            vec![0, 9, 0, 0, 0, 0, 4, 0, 0],
        ],
    ]
    .into_iter()
    .map(TryInto::<Grid<Base3>>::try_into)
    .collect::<Result<Vec<_>>>()
    .unwrap();

    grids.extend(
        [
            ".5..83.17...1..4..3.4..56.8....3...9.9.8245....6....7...9....5...729..861.36.72.4",
            "2.6.3......1.65.7..471.8.5.5......29..8.194.6...42...1....428..6.93....5.7.....13",
            "..45.21781...9..3....8....46..45.....7.9...128.12.35..4.......935..6.8.7.9.3..62.",
            "59....147...9....8.72....3.7...4.29..2..3.8.68..17..5...5764..9.36..5...1..8....2",
            "9...84.6.6.4..52.7.3..7..8.76...15...53.....1...4.96.31.5.26.9...2.4....8....371.",
        ]
        .into_iter()
        .map(|s| s.parse().unwrap()),
    );

    for grid in &mut grids {
        grid.fix_all_values();
    }

    grids
}

pub fn base_4() -> Vec<Grid<Base4>> {
    let minimal = "001b600e000f0000d00010b00e0008470ce30000d40000a04500000g0700f000000000160ged000af0600002001000050005a0008000100200b00470f050000c0d06000000gce00000870d0041000bc9c000003a0bf000200g005000200007f450000e006ca0097800g0f1c0309804d000009060000000g00ec0080d0000b501".parse().unwrap();
    vec![minimal]
}

pub fn base_5() -> Vec<Grid<Base5>> {
    let easy = "00mo00knd00000a0006e0c000200013000p06c08070ome4d000000dm2600i7ej000l0can0kfc0l7boj00ed0k003000000h0g0j9000hg00m20obndaik7800l000000n0p0jc000ohd0096ge0j930g48o1020006p0c0500lin5000mf0h0bkp0g0l0n91o20040002od0mk00050lf00000hp08001000g00003nbo006a40k0j5000000050nhil0060b02d00003c0il100ojb00005pk0n0e00000g0023068c0o570le0040b090000000lecn40019g0j00aim001b0200f4hp0j0ga0803kl70040200g0c0f0n080k00l0j100d00cka0700d0lg0504oe08000i000gi500h00020c03m800f040lm0jhk10000b040d00nf07e0o70d00e48m310hfk020p000a0060000nl47i30p10800d0000aeo00000e900000a00f20l0gn07i0kb0h0da0000cjenp506o4l1a07040500oef00200g00p0jcmdhpfe06000000inma4073508k".parse().unwrap();
    vec![easy]
}

pub fn minimal<Base: SudokuBase>() -> Grid<Base> {
    Generator::with_pruning(PruningSettings {
        target: PruningTarget::Minimal,
        set_all_direct_candidates: true,
        ..Default::default()
    })
    .generate()
    .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_2() {
        base_2();
    }

    #[test]
    fn test_base_2_solved() {
        assert!(base_2_solved().is_solved());
    }

    #[test]
    fn test_base_2_candidates_coordinates() {
        let grid = base_2_candidates_coordinates();

        let top_left_cell = grid.get((0, 0).try_into().unwrap());
        assert_eq!(*top_left_cell, Cell::with_candidates(Candidates::new()));

        let bottom_right = grid.get((3, 3).try_into().unwrap());
        assert_eq!(*bottom_right, Cell::with_candidates(Candidates::all()));
    }

    #[test]
    fn test_base_3() {
        base_3();
    }
}
