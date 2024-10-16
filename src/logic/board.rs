use rand::prelude::*;

#[derive(Debug)]
pub struct SumdokuBoard {
    pub solution: Vec<Vec<u32>>,
    pub cages: Vec<Cage>,
}

#[derive(Debug)]
pub struct Cell {
    pub row: usize,
    pub col: usize,
}

impl Cell {
    pub fn new(row: usize, col: usize) -> Self {
        Cell { row, col }
    }
}

#[derive(Debug)]
pub struct Cage {
    pub sum: u32,
    pub cells: Vec<Cell>,
    pub lines: Option<Vec<((i32,i32),(i32,i32))>>,
}

impl Cage {
    pub fn is_adjacent(&self, cell1: &Cell, cell2: &Cell) -> bool {
        (cell1.row == cell2.row && (cell1.col as isize - cell2.col as isize).abs() == 1)
            || (cell1.col == cell2.col && (cell1.row as isize - cell2.row as isize).abs() == 1)
    }

    pub fn is_cross_join(&self, cell: &Cell, other: &Cell) -> bool {
        (cell.row as isize - other.row as isize).abs()==1 &&
        (cell.col as isize - other.col as isize).abs()==1
    }
}

fn is_valid(board: &Vec<Vec<u32>>, row: usize, col: usize, num: u32) -> bool {
    // Check if the number is already in the row
    for x in 0..9 {
        if board[row][x] == num {
            return false;
        }
    }

    // Check if the number is already in the column
    for x in 0..9 {
        if board[x][col] == num {
            return false;
        }
    }

    // Check if the number is in the 3x3 subgrid
    let start_row = 3 * (row / 3);
    let start_col = 3 * (col / 3);
    for i in 0..3 {
        for j in 0..3 {
            if board[i + start_row][j + start_col] == num {
                return false;
            }
        }
    }

    true
}

fn find_empty_cell(b: &Vec<Vec<u32>>) -> Option<(usize, usize)> {
    for r in 0..9 {
        for c in 0..9 {
            if b[r][c] == 0 {
                return Some((r, c));
            }
        }
    }
    None
}

fn solve(b: &mut Vec<Vec<u32>>, rng: &mut ThreadRng) -> bool {
    if let Some((row, col)) = find_empty_cell(b) {
        let mut numbers: Vec<u32> = (1..=9).collect();
        numbers.shuffle(&mut thread_rng());

        for &num in numbers.iter() {
            if is_valid(b, row, col, num) {
                b[row][col] = num;
                if solve(b, rng) {
                    return true;
                }
                b[row][col] = 0; // Backtrack
            }
        }
        return false;
    }
    true
}

fn generate_solution(rng: &mut ThreadRng) -> Vec<Vec<u32>> {
    let mut board = vec![vec![0u32; 9]; 9];
    solve(&mut board, rng);
    board
}

fn generate_cages(max_cage_size: usize) -> Vec<Cage> {
    let mut total = 81;
    let mut rng = rand::thread_rng();
    let mut result = Vec::new();
    let mut visited = vec![vec![false; 9]; 9];
    let dirs = [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)];
    while total > 0 {
        let mut cells = Vec::new();
        let (mut x, mut y) = find_available_cell(&visited);
        if x == -1 {
            break;
        }
        let r = x as usize;
        let c = y as usize;
        cells.push(Cell::new(r, c));
        visited[r][c] = true;
        let approx_size: usize = rng.gen::<usize>() % max_cage_size;
        for _i in 0..approx_size {
            let dir = rng.gen::<usize>() % 4;
            let (dr, dc) = dirs[dir];
            let (nr, nc) = (x + dr, y + dc);
            if try_assign(nr, nc, &mut visited, &mut cells) {
                continue;
            }
            let mut added = false;
            for (dr, dc) in dirs {
                if try_assign(x + dr, y + dc, &mut visited, &mut cells) {
                    added = true;
                    x += dr;
                    y += dc;
                    break;
                }
            }
            if !added {
                break;
            }
        }
        cells.sort_by(|a,b| {
            let score_a = a.row*100+a.col;
            let score_b = b.row*100+b.col;
            score_a.cmp(&score_b)
        });
        total -= cells.len();
        result.push(Cage {
            sum: 0,
            cells,
            lines: None,
        });
    }
    result
}

fn try_assign(r: i32, c: i32, v: &mut Vec<Vec<bool>>, ce: &mut Vec<Cell>) -> bool {
    if r >= 0 && r < 9 && c >= 0 && c < 9 && !v[r as usize][c as usize] {
        ce.push(Cell::new(r as usize, c as usize));
        v[r as usize][c as usize] = true;
        return true;
    }
    false
}

fn find_available_cell(visited: &Vec<Vec<bool>>) -> (i32, i32) {
    for r in 0..9 {
        for c in 0..9 {
            if !visited[r][c] {
                return (r as i32, c as i32);
            }
        }
    }
    (-1, -1)
}

fn add_cages(board: &mut SumdokuBoard, max_cage_size: usize) {
    for cage in generate_cages(max_cage_size) {
        let mut sum = 0;
        for cell in &cage.cells {
            sum += board.solution[cell.row][cell.col];
        }
        board.cages.push(Cage {
            sum,
            cells: cage.cells,
            lines: None,
        });
    }
}

impl SumdokuBoard {
    pub fn new(max_cage_size: usize) -> Self {
        let mut rng = rand::thread_rng();
        let solution = generate_solution(&mut rng);
        let mut board = SumdokuBoard {
            solution,
            cages: Vec::new(),
        };
        add_cages(&mut board, max_cage_size);
        board
    }
}
