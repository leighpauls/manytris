use crate::shapes::{Orientation, Shape, TetrominoLocation};

pub const W: i32 = 10;
pub const H: i32 = 10;

pub const PREVIEW_H: i32 = 2;

const TOTAL_BLOCKS: usize = (W * H) as usize;

#[derive(Clone, Eq, PartialEq)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone)]
pub struct Tetromino {
    shape: Shape,
    loc: TetrominoLocation,
    orientation: Orientation,
}

pub enum Shift {
    Left,
    Right,
}

struct Field {
    occupied: [bool; TOTAL_BLOCKS],
}

pub struct GameState {
    field: Field,
    active: Option<Tetromino>,
}

pub enum BlockState {
    Empty,
    Occupied,
    Active,
}

impl Pos {
    fn to_buffer_idx(&self) -> usize {
        (self.y * W + self.x) as usize
    }

    fn out_of_bounds(&self) -> bool {
        self.x < 0 || self.x >= W || self.y < 0
    }
}

impl Tetromino {
    pub fn new() -> Tetromino {
        let shape = Shape::I;
        Tetromino {
            loc: shape.starting_tetromino_location(),
            shape,
            orientation: Orientation::Up,
        }
    }

    fn get_blocks(&self) -> [Pos; 4] {
        let rels = self.shape.relative_positions(&self.orientation);
        rels.map(|rp| Pos {
            x: self.loc.0 + rp.0,
            y: self.loc.1 + rp.1,
        })
    }

    fn contains(&self, p: &Pos) -> bool {
        self.get_blocks().contains(p)
    }

    /// Returns a new Tetromino, dropped 1 space, if valid.
    fn down(&self) -> Option<Tetromino> {
        let mut t = self.clone();
        t.loc.1 -= 1;
        for p in &t.get_blocks() {
            if p.out_of_bounds() {
                return None;
            }
        }
        Some(t)
    }

    fn cw(&self) -> Option<Tetromino> {
        let mut new_t = self.clone();
        new_t.orientation = new_t.orientation.cw();

        if new_t.out_of_bounds() {
            None
        } else {
            Some(new_t)
        }
    }

    fn out_of_bounds(&self) -> bool {
        for p in self.get_blocks() {
            if p.out_of_bounds() {
                return true;
            }
        }
        false
    }
}

impl Shift {
    /// Return a shifted clone of the Tetromino, if valid.
    fn apply(&self, t: &Tetromino) -> Option<Tetromino> {
        let mut new_t = t.clone();
        match self {
            Self::Left => new_t.loc.0 -= 1,
            Self::Right => new_t.loc.0 += 1,
        }

        if new_t.out_of_bounds() {
            None
        } else {
            Some(new_t)
        }
    }
}

impl Field {
    fn new() -> Field {
        Field {
            occupied: [false; TOTAL_BLOCKS],
        }
    }

    fn apply_tetrominio(&mut self, t: &Tetromino) {
        for block_pos in &t.get_blocks() {
            self.occupied[block_pos.to_buffer_idx()] = true
        }
    }

    fn is_occupied(&self, pos: &Pos) -> bool {
        self.occupied[pos.to_buffer_idx()]
    }

    fn is_valid(&self, t: &Tetromino) -> bool {
        for p in t.get_blocks() {
            if self.is_occupied(&p) {
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

    fn lock_active_tetromino(&mut self) {
        if let Some(t) = self.active.take() {
            self.field.apply_tetrominio(&t)
        }
    }

    pub fn down(&mut self) -> Option<()> {
        let t = self.active.as_mut()?;
        match t.down() {
            Some(new_t) if self.field.is_valid(&new_t) => {
                *t = new_t;
            }
            _ => {
                self.lock_active_tetromino();
            }
        }
        Some(())
    }

    pub fn drop(&mut self) {
        while self.active.is_some() {
            self.down();
        }
    }

    pub fn shift(&mut self, dir: Shift) -> Option<()> {
        let t = self.active.as_mut()?;
        let new_t = dir.apply(t)?;
        if self.field.is_valid(&new_t) {
            *t = new_t;
            return Some(());
        }
        None
    }

    pub fn cw(&mut self) -> Option<()> {
        let t = self.active.as_mut()?;
        *t = t.cw()?;

        Some(())
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
        let bottom: String = ['-'; (W + 2) as usize].iter().collect();
        println!("{}", bottom);
    }

    pub fn check_block(&self, p: &Pos) -> BlockState {
        if let Some(ref t) = self.active {
            if t.contains(p) {
                return BlockState::Active;
            }
        }
        if self.field.is_occupied(p) {
            BlockState::Occupied
        } else {
            BlockState::Empty
        }
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
            '_'
        }
    }
}
