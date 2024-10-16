use std::time::Instant;

use crate::logic::SumdokuBoard;

pub struct Game {
    pub board: SumdokuBoard,
    pub mistakes: u8,
    pub cells: Vec<Vec<GameCell>>,
    pub time: Instant,
    undo_state: BoardState,
}

type BoardState = Vec<Vec<Vec<GameCell>>>;

#[derive(Clone)]
pub struct GameCell {
    pub notes: u16,
    pub value: u32,
}

impl GameCell {
    pub fn new() -> Self {
        GameCell { notes: 0, value: 0 }
    }

    pub fn toggle_note(&mut self, v: u8) {
        let mask = 1 << v;
        self.notes ^= mask;
    }

    fn clear_note(&mut self, v: u8) {
        let mask = 1 << v;
        self.notes &= !mask;
    }

    pub fn zero_notes(&mut self) {
        self.notes = 0;
    }
}

impl Game {
    pub fn new() -> Self {
        Game {
            board: SumdokuBoard::new(6),
            mistakes: 0,
            cells: vec![vec![GameCell::new(); 9]; 9],
            time: Instant::now(),
            undo_state: vec![],
        }
    }

    pub fn set_value(&mut self, row: usize, col: usize, value: u32) -> bool {
        self.push_state();
        self.remove_notes(row, col, value);
        self.cells[row][col].value = if self.cells[row][col].value == value {
            0
        } else {
            value
        };
        self.cells[row][col].notes = 0;
        self.board.solution[row][col] == value
    }

    pub fn clear_cell(&mut self, row: usize, col: usize) {
        self.push_state();
        self.cells[row][col].value = 0;
        self.cells[row][col].notes = 0;
    }

    fn remove_notes(&mut self, r: usize, c: usize, v: u32) {
        let br = r / 3;
        let bc = c / 3;
        for i in 0..3 {
            for j in 0..3 {
                self.cells[br + i][bc + j].clear_note(v as u8);
            }
        }
        for i in 0..9 {
            self.cells[r][i].clear_note(v as u8);
            self.cells[i][c].clear_note(v as u8);
        }
    }

    pub fn push_state(&mut self) {
        self.undo_state.push(self.cells.clone());
    }

    pub fn pop_state(&mut self) {
        if let Some(last_state) = self.undo_state.pop() {
            for r in 0..9 {
                for c in 0..9 {
                    self.cells[r][c].value = last_state[r][c].value;
                    self.cells[r][c].notes = last_state[r][c].notes;
                }
            }
        }
    }

    pub fn can_undo(self) -> bool {
        self.undo_state.len() > 0
    }
}
