use std::sync::Arc;
pub mod solver;
pub mod web;

use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Down,
    Across,
}

impl Direction {
    fn cross(&self) -> Direction {
        match &self {
            Direction::Across => Direction::Down,
            Direction::Down => Direction::Across,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

pub struct InMemoryEntry {
    cells: Vec<InMemoryCell>,
}

impl InMemoryEntry {
    pub async fn write(&self, value: String) {
        for (cell, c) in self.cells.iter().zip(value.chars()) {
            cell.write(c).await
        }
    }

    pub async fn clear(&self) {
        for c in self.cells.iter() {
            c.clear().await
        }
    }

    pub async fn chars(&self) -> Vec<Option<char>> {
        let mut v = vec![];
        for c in self.cells.iter() {
            v.push(c.value().await);
        }
        v
    }
}

#[derive(Debug, Clone)]
/// Cell whose value is kept in memory
pub struct InMemoryCell {
    value: Arc<RwLock<Option<char>>>,
    position: Position,
}

impl InMemoryCell {
    pub fn new(position: Position, value: Option<char>) -> Self {
        Self {
            value: Arc::new(RwLock::new(value)),
            position,
        }
    }
    async fn value(&self) -> Option<char> {
        *self.value.read().await
    }

    async fn write(&self, c: char) {
        let mut guard = self.value.write().await;
        let _ = guard.insert(c);
    }

    async fn clear(&self) {
        self.value.write().await.take();
    }
}

/// Get the positions of the Cells for the given Clue
fn positions_for_clue(
    clue: &Clue,
    width: usize,
    height: usize,
    shaded_squares: &[Position],
) -> Vec<Position> {
    let mut position = clue.position;
    let mut cell_positions = vec![];
    loop {
        // Stop conditions are either the edge of the puzzle or a shaded cell
        if position.column >= width || position.row >= height || shaded_squares.contains(&position)
        {
            break cell_positions;
        }
        // Add this Cell to the Entry
        cell_positions.push(position);
        // Look at the next cell
        match clue.direction {
            Direction::Across => position.column += 1,
            Direction::Down => position.row += 1,
        }
    }
}

//
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Clue {
    pub number: usize,
    pub direction: Direction,
    pub text: String,
    pub position: Position,
}

#[cfg(test)]
mod tests {
    use crate::{positions_for_clue, Clue, Position};

    #[test]
    fn test_positions_for_clue() {
        let across = positions_for_clue(
            &Clue {
                direction: crate::Direction::Across,
                position: crate::Position { row: 0, column: 0 },
                number: 0,
                text: String::new(),
            },
            2,
            2,
            &[Position { row: 0, column: 1 }],
        );
        assert_eq!(across.len(), 1);
        let down = positions_for_clue(
            &Clue {
                direction: crate::Direction::Down,
                position: crate::Position { row: 0, column: 0 },
                number: 0,
                text: String::new(),
            },
            2,
            2,
            &[Position { row: 0, column: 1 }],
        );
        assert_eq!(down.len(), 2);
    }
}
