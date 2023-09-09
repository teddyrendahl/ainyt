use std::{cell::RefCell, rc::Rc};
pub mod solver;
pub mod web;

use serde::{Deserialize, Serialize};

// Description of the entire Grid
#[derive(Clone)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<Cell>>,
}

impl Grid {
    /// Utility to check that all fillable cells have values entered in them
    pub fn filled(&self) -> bool {
        self.cells.iter().flatten().all(|c| match c {
            Cell::Shaded => true,
            Cell::Fillable(f) if f.value().is_some() => true,
            Cell::Fillable(_) => false,
        })
    }
    // Get a Cell at a specific Position
    fn cell_at(&self, position: Position) -> Cell {
        self.cells[position.row][position.column].clone()
    }

    /// Get all the fillable cells for the Clue
    fn cells_for_clue(&self, clue: &Clue) -> Vec<(Position, Fill)> {
        let mut cells = vec![];
        let mut position = clue.position;
        loop {
            // Stop conditions are either the edge of the puzzle or a shaded cell
            if position.column >= self.width || position.row >= self.height {
                break cells;
            }
            let next_cell = self.cell_at(position);
            match next_cell {
                Cell::Shaded => break cells,
                Cell::Fillable(f) => cells.push((position, f)),
            }

            // Look at the next cell
            match clue.direction {
                Direction::Across => position.column += 1,
                Direction::Down => position.row += 1,
            }
        }
    }

    /// Clear all answers from the Grid
    fn clear(&mut self) {
        for c in self.cells.iter_mut().flatten() {
            if let Cell::Fillable(f) = c {
                f.clear()
            }
        }
    }

    /// Enter an answer for the Clue
    fn enter_answer(&self, clue: &Clue, answer: String) {
        let cells = self.cells_for_clue(clue);
        // TODO: This should be a proper error
        //       Maybe we want to allow for partial entry?
        assert_eq!(cells.len(), answer.len());
        for ((_, mut cell), c) in cells.into_iter().zip(answer.chars()) {
            cell.write_to(c.to_ascii_uppercase())
        }
    }

    /// Get the current answer for
    pub fn answer_for(&self, clue: &Clue) -> String {
        self.cells_for_clue(clue)
            .iter()
            .map(|(_, f)| f.value().unwrap_or('_'))
            .collect()
    }

    /// Show the current status of the grid
    pub fn show(&self) {
        for row in self.cells.iter() {
            let r: String = row
                .iter()
                .map(|c| match c {
                    Cell::Fillable(x) => x.value().unwrap_or('_'),
                    Cell::Shaded => 'X',
                })
                .collect();
            println!("{}", r)
        }
    }

    /// Find all all the Clues that Cross with a given Clue
    pub fn crosses(&self, clue: &Clue, clues: &[Clue]) -> Vec<Clue> {
        let positions: Vec<Position> = self
            .cells_for_clue(clue)
            .into_iter()
            .map(|(p, _)| p)
            .collect();
        clues
            .iter()
            .filter_map(|c| {
                if c.direction != clue.direction
                    && self
                        .cells_for_clue(c)
                        .iter()
                        .any(|(p, _)| positions.contains(p))
                {
                    Some(c.clone())
                } else {
                    None
                }
            })
            .rev()
            .collect()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Direction {
    Down,
    Across,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
/// Zero-indexed grid Position
pub struct Position {
    pub row: usize,
    pub column: usize,
}

impl Position {
    pub fn from_cell_id(id: usize, columns: usize) -> Self {
        Position {
            row: id / columns,
            column: id % columns,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Cell {
    Shaded,
    Fillable(Fill),
}

#[derive(Debug, Clone)]
pub struct Fill(Rc<RefCell<Option<char>>>);

impl Fill {
    /// Create a new empty Fillable cell
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(None)))
    }

    /// Current fill value
    pub fn value(&self) -> Option<char> {
        *self.0.borrow()
    }

    /// Write an answer into our Grid
    pub fn write_to(&mut self, c: char) {
        let _ = self.0.borrow_mut().insert(c);
    }

    pub fn clear(&mut self) {
        self.0.borrow_mut().take();
    }
}

#[derive(Serialize, Deserialize)]
pub struct Puzzle {
    pub width: usize,
    pub height: usize,
    pub clues: Vec<Clue>,
    pub shaded_squares: Vec<Position>,
}

impl From<&Puzzle> for Grid {
    fn from(value: &Puzzle) -> Self {
        Grid {
            width: value.width,
            height: value.height,
            cells: (0..value.height)
                .map(|row| {
                    (0..value.width)
                        .map(|column| {
                            if value.shaded_squares.contains(&Position { row, column }) {
                                Cell::Shaded
                            } else {
                                Cell::empty()
                            }
                        })
                        .collect::<Vec<Cell>>()
                })
                .collect::<Vec<Vec<Cell>>>(),
        }
    }
}

impl Default for Fill {
    fn default() -> Self {
        Self::new()
    }
}

impl Cell {
    /// Create a new empty Cell
    pub fn empty() -> Self {
        Cell::Fillable(Fill::new())
    }
}

//
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clue {
    pub number: usize,
    pub direction: Direction,
    pub text: String,
    pub position: Position,
    pub answer: Option<String>,
}

#[cfg(test)]
mod tests {
    use crate::{Cell, Clue, Direction, Grid, Position};

    #[test]
    fn test_get_clue_cells() {
        let grid = Grid {
            width: 2,
            height: 2,
            cells: vec![
                vec![Cell::empty(), Cell::Shaded],
                vec![Cell::empty(), Cell::empty()],
            ],
        };
        let across_clue = Clue {
            direction: crate::Direction::Across,
            position: crate::Position { row: 0, column: 0 },
            number: 0,
            text: String::new(),
            answer: None,
        };
        let down_clue = Clue {
            direction: crate::Direction::Down,
            position: crate::Position { row: 0, column: 0 },
            number: 1,
            text: String::new(),
            answer: None,
        };
        assert_eq!(grid.cells_for_clue(&across_clue).len(), 1);
        assert_eq!(grid.cells_for_clue(&down_clue).len(), 2)
    }

    #[test]
    fn test_cell_entry() {
        let grid = Grid {
            width: 2,
            height: 2,
            cells: vec![
                vec![Cell::empty(), Cell::Shaded],
                vec![Cell::empty(), Cell::empty()],
            ],
        };
        let across_clue = Clue {
            direction: crate::Direction::Across,
            position: crate::Position { row: 0, column: 0 },
            number: 0,
            text: String::new(),
            answer: None,
        };
        let down_clue = Clue {
            direction: crate::Direction::Down,
            position: crate::Position { row: 0, column: 0 },
            number: 1,
            text: String::new(),
            answer: None,
        };
        grid.enter_answer(&across_clue, "A".into());
        assert_eq!(grid.cells_for_clue(&across_clue)[0].1.value(), Some('A'));
        assert_eq!(grid.cells_for_clue(&down_clue)[0].1.value(), Some('A'));
        assert_eq!(grid.cells_for_clue(&down_clue)[1].1.value(), None)
    }

    #[test]
    fn test_position_from_cell_id() {
        let p = Position::from_cell_id(3, 5);
        assert_eq!(p.row, 0);
        assert_eq!(p.column, 3);

        let p = Position::from_cell_id(6, 5);
        assert_eq!(p.row, 1);
        assert_eq!(p.column, 1);
    }

    #[test]
    fn test_crosses() {
        let grid = Grid {
            width: 2,
            height: 2,
            cells: vec![
                vec![Cell::empty(), Cell::empty()],
                vec![Cell::empty(), Cell::empty()],
            ],
        };
        let clue = Clue {
            direction: crate::Direction::Across,
            position: crate::Position { row: 0, column: 0 },
            number: 0,
            text: String::new(),
            answer: None,
        };

        let crosses = grid.crosses(
            &clue,
            &[
                clue.clone(),
                Clue {
                    direction: crate::Direction::Across,
                    position: crate::Position { row: 1, column: 0 },
                    number: 0,
                    text: String::new(),
                    answer: None,
                },
                Clue {
                    direction: crate::Direction::Down,
                    position: crate::Position { row: 0, column: 0 },
                    number: 1,
                    text: String::new(),
                    answer: None,
                },
                Clue {
                    direction: crate::Direction::Down,
                    position: crate::Position { row: 0, column: 1 },
                    number: 1,
                    text: String::new(),
                    answer: None,
                },
            ],
        );
        assert!(crosses.iter().all(|c| c.direction == Direction::Down));
        assert_eq!(crosses.len(), 2);
    }
}
