use crossword::{Cell, Clue, Grid, Solver};

fn main() {
    let grid = Grid {
        cells: [
            [
                Cell::shaded(),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::shaded(),
            ],
            [
                Cell::shaded(),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::shaded(),
            ],
            [
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
            ],
            [
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
            ],
            [
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
                Cell::empty(),
            ],
        ],
    };
    let clues = vec![
        Clue {
            number: 1,
            direction: crossword::Direction::Across,
            position: crossword::Position { row: 0, column: 1 },
            text: "Channel with on-air pledge drives".into(),
        },
        Clue {
            number: 4,
            direction: crossword::Direction::Across,
            position: crossword::Position { row: 1, column: 1 },
            text: "\"Say what!?\"".into(),
        },
        Clue {
            number: 5,
            direction: crossword::Direction::Across,
            position: crossword::Position { row: 2, column: 0 },
            text: "With 7-Across, street sign with a red circle and white bar".into(),
        },
        Clue {
            number: 7,
            direction: crossword::Direction::Across,
            position: crossword::Position { row: 3, column: 0 },
            text: "See 5-Across".into(),
        },
        Clue {
            number: 8,
            direction: crossword::Direction::Across,
            position: crossword::Position { row: 4, column: 0 },
            text: "Like eating a melting ice cream cone".into(),
        },
        Clue {
            number: 1,
            direction: crossword::Direction::Down,
            position: crossword::Position { row: 0, column: 1 },
            text: "Object that has moved from the wall to your pocket".into(),
        },
        Clue {
            number: 2,
            direction: crossword::Direction::Down,
            position: crossword::Position { row: 0, column: 2 },
            text: "Short hits, in baseball".into(),
        },
        Clue {
            number: 3,
            direction: crossword::Direction::Down,
            position: crossword::Position { row: 0, column: 3 },
            text: "Vans, e.g.".into(),
        },
        Clue {
            number: 5,
            direction: crossword::Direction::Down,
            position: crossword::Position { row: 2, column: 0 },
            text: "Blue on an electoral map: Abbr.".into(),
        },
        Clue {
            number: 6,
            direction: crossword::Direction::Down,
            position: crossword::Position { row: 2, column: 4 },
            text: "Make an effort".into(),
        },
    ];
    let solver = Solver;
    solver.solve(&grid, &clues);
}
