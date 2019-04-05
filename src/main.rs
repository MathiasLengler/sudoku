use sudoku::cell::OptionCell;
use sudoku::position::Position;
use sudoku::Sudoku;

fn main() {
    let mut sudoku = Sudoku::<OptionCell>::new(3);

    let mut debug_value = 0;
    for y in 0..9 {
        for x in 0..9 {
            sudoku.set(Position { x, y }, OptionCell(Some(debug_value)));
            debug_value += 1;
        }
    }

    println!("{}", sudoku);
    dbg!(sudoku.has_conflict());

    sudoku.set(Position {
        x: 2,
        y: 2,
    }, OptionCell(Some(0)));

    println!("{}", sudoku);
    dbg!(sudoku.has_conflict());
}
