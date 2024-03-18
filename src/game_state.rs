const W: usize = 10;
const H: usize = 10;

const PREVIEW_H: usize = 2;

const TOTAL_BLOCKS: usize = W * H;

#[derive(Clone, Eq, PartialEq)]
pub struct Pos {
    x: usize,
    y: usize,
}

#[derive(Clone)]
pub struct Tetromino {
    blocks: [Pos; 4],
}

struct Field {
    occupied: [bool; TOTAL_BLOCKS],
}

pub struct GameState {
    field: Field,
    active: Option<Tetromino>,
}

impl Pos {
    fn to_buffer_idx(&self) -> usize {
        self.y * W + self.x
    }
}

impl Tetromino {
    pub fn new() -> Tetromino {
        let y = H - 1;
        Tetromino {
            blocks: [
                Pos { x: 0, y },
                Pos { x: 1, y },
                Pos { x: 2, y },
                Pos { x: 3, y },
            ],
        }
    }

    fn contains(&self, p: &Pos) -> bool {
        self.blocks.contains(p)
    }

    /// Returns a new Tetromino, dropped 1 space, if valid.
    fn down(&self) -> Option<Tetromino> {
        let mut t = self.clone();
        for p in &mut t.blocks {
            if p.y == 0 {
                return None;
            }
            p.y -= 1;
        }
        Some(t)
    }
}

impl Field {
    fn new() -> Field {
        Field {
            occupied: [false; TOTAL_BLOCKS],
        }
    }

    fn apply_tetrominio(&mut self, t: &Tetromino) {
        for block_pos in &t.blocks {
            self.occupied[block_pos.to_buffer_idx()] = true
        }
    }

    fn is_occupied(&self, pos: &Pos) -> bool {
        self.occupied[pos.to_buffer_idx()]
    }

    fn is_valid(&self, t: &Tetromino) -> bool {
        for p in &t.blocks {
            if self.is_occupied(p) {
                return false;
            }
        }
        true
    }
}

impl GameState {
    pub fn new() -> GameState {
        return GameState {
            field: Field::new(),
            active: None,
        };
    }

    /// Try to set a new tetromino, return True is successful.
    pub fn new_active_tetromino(&mut self, t: Tetromino) -> bool {
        if self.field.is_valid(&t) {
            self.active = Some(t);
            true
        } else {
            false
        }
    }

    pub fn lock_active_tetromino(&mut self) {
        if let Some(t) = self.active.take() {
            self.field.apply_tetrominio(&t)
        }
    }

    pub fn down(&mut self) {
        if let Some(ref mut t) = self.active {
            match t.down() {
                Some(new_t) if self.field.is_valid(&new_t) => {
                    *t = new_t;
                }
                _ => {
                    self.lock_active_tetromino();
                }
            }
        }
    }

    pub fn print(&self) {
        for y in (0..H).rev() {
            let mut line = String::from("");
            for x in 0..W {
                line.push(self.char_for_pos(&Pos { x, y }));
            }
            let border = if y < H - PREVIEW_H { "|" } else { " " };
            println!("{}{}{}", border, line, border);
        }
        let bottom: String = ['-'; W + 2].iter().collect();
        println!("{}", bottom);
    }

    fn char_for_pos(&self, p: &Pos) -> char {
        if let Some(ref t) = self.active {
            if t.contains(p) {
                return 'O';
            }
        }
        if self.field.is_occupied(p) {
            'X'
        } else {
            ' '
        }
    }
}
