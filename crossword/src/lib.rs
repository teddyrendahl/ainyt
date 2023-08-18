use std::{cell::RefCell, rc::Rc};

use chatgpt::prelude::ChatGPT;

/// Generate a ChatGPT prompt for a given Clue
fn prompt_for_clue<const W: usize, const H: usize>(clue: &Clue, grid: &Grid<W, H>) -> String {
    let cells = grid.cells_for_clue(clue);
    let pattern = cells
        .iter()
        .map(|cell| match cell.fill() {
            Fill::Empty => '_',
            Fill::Filled(c) => c,
            Fill::Shaded => panic!("Clue pattern should not have shaded cell"),
        })
        .collect::<String>();
    format!(
        "What is the answer to the crossword clue \"{}\". \
             The answer is {} letters long with the pattern {}? \
             Respond with just the answer",
        clue.text,
        cells.len(),
        pattern
    )
}

pub struct GPTSolver(ChatGPT);

impl GPTSolver {
    pub fn new(api_key: String) -> chatgpt::Result<Self> {
        Ok(Self(ChatGPT::new(api_key)?))
    }
    pub async fn solve<const W: usize, const H: usize>(
        &self,
        grid: &Grid<W, H>,
        clues: &[Clue],
    ) -> chatgpt::Result<()> {
        let mut conversation = self.0.new_conversation();
        for clue in clues {
            let prompt = prompt_for_clue(clue, grid);
            let cells = grid.cells_for_clue(clue);
            println!("{}", prompt);
            let answer = conversation
                .send_message(prompt)
                .await?
                .message()
                .content
                .clone();
            println!("{}", answer);
            if cells.len() != answer.len() {
                println!(
                    "Answer {} does not match clue length of {}",
                    answer,
                    cells.len()
                )
            } else {
                grid.enter_answer(clue, answer)
            }
        }
        Ok(())
    }
}

// Description of the entire Grid
pub struct Grid<const W: usize, const H: usize> {
    pub cells: [[Cell; W]; H],
}

impl<const W: usize, const H: usize> Grid<W, H> {
    // Get a Cell at a specific Position
    fn cell_at(&self, position: Position) -> Cell {
        self.cells[position.row][position.column].clone()
    }

    /// Get all the fillable cells for the Clue
    fn cells_for_clue(&self, clue: &Clue) -> Vec<Cell> {
        let mut cells = vec![self.cell_at(clue.position)];
        let mut position = clue.position;
        loop {
            // Look at the next cell
            match clue.direction {
                Direction::Across => position.column += 1,
                Direction::Down => position.row += 1,
            }

            // Stop conditions are either the edge of the puzzle or a shaded cell
            if position.column >= W || position.row >= H {
                break cells;
            }
            let next_cell = self.cell_at(position);
            if matches!(next_cell.fill(), Fill::Shaded) {
                break cells;
            }
            cells.push(next_cell);
        }
    }

    /// Enter an answer for the Clue
    fn enter_answer(&self, clue: &Clue, answer: String) {
        let cells = self.cells_for_clue(clue);
        // TODO: This should be a proper error
        //       Maybe we want to allow for partial entry?
        assert_eq!(cells.len(), answer.len());
        for (cell, c) in cells.iter().zip(answer.chars()) {
            cell.write_to(c.to_ascii_uppercase())
        }
    }

    /// Show the current status of the grid
    pub fn show(&self) {
        for row in self.cells.iter() {
            let r: String = row
                .iter()
                .map(|c| match c.fill() {
                    Fill::Empty => '_',
                    Fill::Filled(c) => c,
                    Fill::Shaded => 'X',
                })
                .collect();
            println!("{}", r)
        }
    }
}

pub enum Direction {
    Down,
    Across,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Fill {
    Shaded,
    Empty,
    Filled(char),
}

#[derive(Clone, Copy)]
pub struct Position {
    pub row: usize,
    pub column: usize,
}

#[derive(Clone)]
pub struct Cell {
    fill: Rc<RefCell<Fill>>,
}

impl Cell {
    /// Create a new empty Cell
    pub fn empty() -> Self {
        Cell {
            fill: Rc::new(RefCell::new(Fill::Empty)),
        }
    }

    /// Create a new shaded Cell
    pub fn shaded() -> Self {
        Cell {
            fill: Rc::new(RefCell::new(Fill::Shaded)),
        }
    }

    /// Get the current Fill value
    fn fill(&self) -> Fill {
        *self.fill.borrow()
    }

    /// Write an answer into our Grid
    pub fn write_to(&self, c: char) {
        self.fill.replace_with(|&mut old| {
            assert!(!matches!(old, Fill::Shaded));
            Fill::Filled(c)
        });
    }
}

//
pub struct Clue {
    pub number: usize,
    pub direction: Direction,
    pub text: String,
    pub position: Position,
}

#[cfg(test)]
mod tests {
    use crate::{Cell, Clue, Fill, Grid};

    #[test]
    fn test_get_clue_cells() {
        let grid = Grid {
            cells: [
                [Cell::empty(), Cell::shaded()],
                [Cell::empty(), Cell::empty()],
            ],
        };
        let across_clue = Clue {
            direction: crate::Direction::Across,
            position: crate::Position { row: 0, column: 0 },
            number: 0,
            text: String::new(),
        };
        let down_clue = Clue {
            direction: crate::Direction::Down,
            position: crate::Position { row: 0, column: 0 },
            number: 1,
            text: String::new(),
        };
        assert_eq!(grid.cells_for_clue(&across_clue).len(), 1);
        assert_eq!(grid.cells_for_clue(&down_clue).len(), 2)
    }

    #[test]
    fn test_cell_entry() {
        let grid = Grid {
            cells: [
                [Cell::empty(), Cell::shaded()],
                [Cell::empty(), Cell::empty()],
            ],
        };
        let across_clue = Clue {
            direction: crate::Direction::Across,
            position: crate::Position { row: 0, column: 0 },
            number: 0,
            text: String::new(),
        };
        let down_clue = Clue {
            direction: crate::Direction::Down,
            position: crate::Position { row: 0, column: 0 },
            number: 1,
            text: String::new(),
        };
        grid.enter_answer(&across_clue, "A".into());
        assert_eq!(
            grid.cells_for_clue(&across_clue)[0].fill(),
            Fill::Filled('A')
        );
        assert_eq!(grid.cells_for_clue(&down_clue)[0].fill(), Fill::Filled('A'));
        assert_eq!(grid.cells_for_clue(&down_clue)[1].fill(), Fill::Empty)
    }
}
