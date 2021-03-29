#![feature(const_generics)]
#![allow(incomplete_features)]
#![cfg_attr(not(test), no_std)]


use bare_metal_modulo::{ModNumC, MNum, ModNumIterator};
use pluggable_interrupt_os::vga_buffer::{BUFFER_WIDTH, BUFFER_HEIGHT, plot, ColorCode, Color};
use pc_keyboard::{DecodedKey, KeyCode};
use num::traits::AsPrimitive;


/// Enums Cell, Dir, Status and structs Position and RowColIter
/// are taken directly from Dr. Ferrer's ghost_hunter.

const UPDATE_FREQUENCY: usize = 3;
const WIDTH: usize = BUFFER_WIDTH;
const HEIGHT: usize = BUFFER_HEIGHT;

#[derive(Debug,Copy,Clone,Eq,PartialEq)]
#[repr(u8)]
pub enum Cell {
    Empty,
    Chicken,
    CannonZone
}

#[derive(Copy,Clone,Eq,PartialEq,Debug)]
pub enum Status {
    Normal,
    Over
}

#[derive(Debug,Copy,Clone,Eq,PartialEq)]
#[repr(u8)]
pub enum Dir {
    N, S, E, W
}

impl Dir {
    fn reverse(&self) -> Dir {
        match self {
            Dir::N => Dir::S,
            Dir::S => Dir::N,
            Dir::E => Dir::W,
            Dir::W => Dir::E
        }
    }

    fn left(&self) -> Dir {
        match self {
            Dir::N => Dir::W,
            Dir::S => Dir::E,
            Dir::E => Dir::N,
            Dir::W => Dir::S
        }
    }

    fn right(&self) -> Dir {
        match self {
            Dir::N => Dir::E,
            Dir::S => Dir::W,
            Dir::E => Dir::S,
            Dir::W => Dir::N
        }
    }
}

//issues with <WIDTH, HEIGHT> constants, so made simpler
#[derive(Debug,Copy,Clone,Eq,PartialEq)]
pub struct Position {
    col: i16, row: i16
}

impl Position {
    pub fn is_legal(&self) -> bool {
        0 <= self.col && self.col < WIDTH as i16 && 0 <= self.row && self.row < HEIGHT as i16
    }

    pub fn row_col(&self) -> (usize, usize) {
        (self.row as usize, self.col as usize)
    }

    pub fn neighbor(&self, d: Dir) -> Position{
        match d {
            Dir::N => Position {row: self.row - 1, col: self.col},
            Dir::S => Position {row: self.row + 1, col: self.col},
            Dir::E => Position {row: self.row,     col: self.col + 1},
            Dir::W => Position {row: self.row,     col: self.col - 1}
        }
    }
}

#[derive(Copy,Debug,Clone,Eq,PartialEq)]
pub struct Cannon {
    cannon: char,
    pos: Position,
    dx: ModNumC<usize, BUFFER_WIDTH>,
}

impl Cannon {
    fn new() -> Self {
        Cannon {
            cannon: '^',
            pos: Position {col: (BUFFER_WIDTH / 2) as i16, row: (BUFFER_HEIGHT - 1) as i16 },
            dx: ModNumC::new(0)
        }
    }
    fn set_pos(&mut self, position: Position) {
        self.pos = position;
    }


    // /// Do both handle_raws in Game??
    // fn handle_raw(&mut self, key: KeyCode) {
    //     match key {
    //         KeyCode::ArrowLeft => {
    //             self.dx -= 1;
    //         }
    //         KeyCode::ArrowRight => {
    //             self.dx += 1;
    //         }
    //         _ => {}
    //     }
    // }
}

#[derive(Copy,Debug,Clone,Eq,PartialEq)]
pub struct Chicken {
    pos: Position,
    //chick_array: [Position; WIDTH*(HEIGHT-2)],
    dir: Dir,
    active: bool
}

impl Chicken {
    fn new(position: Position) -> Self{
        Chicken{
            pos: position,
            //chick_array: [Position{ col: 0, row: 0 }; WIDTH*(HEIGHT-2)],
            dir: Dir::S,
            active: true
        }
    }
}

#[derive(Copy,Debug,Clone,Eq,PartialEq)]
pub struct Rockets {
    pos: Position,
    //rockets: [Position; WIDTH*HEIGHT],
    dir: Dir,
    active: bool
}

impl Rockets {
    fn new(position: Position) -> Self {
        Rockets {
            ////gets its pos from cannon
            pos: position,
            //array of positions is for rockets that are active at once, last position in array is one coming out of cannon?
            //rockets: [Position{ col: 0, row: 0 }; WIDTH*HEIGHT],
            dir: Dir::N,
            active: false //Only active on keypress
        }
    }
    // fn handle_raw(&mut self, key: KeyCode) {
    //     match key {
    //         KeyCode::Spacebar => {
    //             self.active = true;
    //         }
    //         _ => {}
    //     }
    // }
}

#[derive(Copy,Debug,Clone,Eq,PartialEq)]
pub struct Game {
    cannon: Cannon,
    chickens: [Chicken; WIDTH*HEIGHT],
    //rockets: [Rockets; WIDTH*(HEIGHT-2)],

    col: ModNumC<usize, WIDTH>,
    row: ModNumC<usize, HEIGHT>,
    cells: [[Cell; WIDTH]; HEIGHT],
    countdown: usize,
    status: Status,
    score: u16
}

impl Game {
    pub fn new() -> Self {
        let mut game = Game {
            cannon: Cannon::new(),
            chickens: [Chicken::new(Position { col: 0, row: 0 }); WIDTH * HEIGHT],
            //rockets: [Rockets::new(); WIDTH*(HEIGHT-2)],

            col: ModNumC::new(WIDTH),
            row: ModNumC::new(HEIGHT),
            cells: [[Cell::Empty; WIDTH]; HEIGHT],
            //pos: Position { col: 0, row: 0 },
            countdown: UPDATE_FREQUENCY,
            status: Status::Normal,
            score: 0
        };
        game.reset();
        game
    }

    fn reset(&mut self) {
        self.status = Status::Normal;
        self.score = 0;
    }

    fn score(&self) -> u16 { self.score }

    fn translate_icon(&mut self, row: usize, col: usize, icon: char) {
        match icon {
            '&' => self.cells[row][col] = Cell::Chicken,
            '^' => {
                self.cannon.set_pos(Position { col: col as i16, row: row as i16 });
            },
            _ => panic!("Unrecognized character: '{}'", icon)
        }
    }

    fn column_iter(&self) -> impl Iterator<Item=usize> {
        // ModNumIterator::new(self.col)
        //     .take(self.num_letters.a())
        //     .map(|m| m.a())
        ModNumIterator::new(self.col)
            .take(self.chickens.len())
            .map(|m| m.a())
    }

    pub fn tick(&mut self) {
        self.clear_current();
        self.update_location();
        self.draw_current();
    }

    fn clear_current(&self) {
        //plot(' ', self.cannon.pos.col.as_(), self.cannon.pos.row.as_(), ColorCode::new(Color::Black, Color::Black));
        for x in self.column_iter() {
            plot(' ', x, self.row.a(), ColorCode::new(Color::Black, Color::Black));
            plot(' ', x, self.cannon.pos.row.as_(), ColorCode::new(Color::Black, Color::Black));
        }
    }

    fn update_location(&mut self) {
        let mut x: usize = self.cannon.pos.col.as_();
        x += self.cannon.dx.a();
    }

    fn draw_current(&mut self) {
        let mut count = 0;
        for (_i, x) in self.column_iter().enumerate() {
            //plot(self.letters[i], x, self.row.a(), ColorCode::new(Color::Cyan, Color::Black));
            if count % 3 == 0 && count < WIDTH {
                plot(
                    '&',
                    x,
                    self.row.a(),
                    ColorCode::new(Color::Red, Color::Black)
                );
                self.chickens[count] = Chicken::new(Position { col: x as i16, row: self.row.a() as i16 });
            }
            count += 1;
        }
        plot('^', self.cannon.pos.col.as_(), self.cannon.pos.row.as_(), ColorCode::new(Color::Cyan, Color::Black));
        // if UPDATE_FREQUENCY == 0 {
        //     for chick in self.chickens {
        //
        //     }
    }

    pub fn countdown_complete(&mut self) -> bool {
        if self.countdown == 0 {
            self.countdown = UPDATE_FREQUENCY;
            true
        } else {
            self.countdown -= 1;
            false
        }
    }

    pub fn key(&mut self, key: DecodedKey) {
        // match self.status {
        //     Status::Over => {
        //         match key {
        //             DecodedKey::RawKey(KeyCode::S) | DecodedKey::Unicode('s') => self.reset(),
        //             _ => {}
        //         }
        //     }
        //     _ => {
        //         let key = key2dir(key);
        //         if key.is_some() {
        //             self.last_key = key;
        //         }
        //     }
        // }
        match key {
            DecodedKey::RawKey(code) => self.handle_raw(code),
            //DecodedKey::Unicode(c) => self.handle_unicode(c),
            _ => {}
        }
    }

    fn handle_raw(&mut self, key: KeyCode) {
        match key {
            KeyCode::ArrowLeft => {
                //self.cannon.dx -= 1;
                self.cannon.pos.col -= 1;
            }
            KeyCode::ArrowRight => {
                //self.cannon.dx += 1;
                self.cannon.pos.col += 1;
            }
            KeyCode::Spacebar => {
                //self.rockets.rockets
            }
            _ => {}
        }
    }
}


pub struct RowColIter {
    row: usize, col: usize
}

impl Iterator for RowColIter{
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        if self.row < HEIGHT {
            let result = Some(Position {row: self.row as i16, col: self.col as i16});
            self.col += 1;
            if self.col == WIDTH {
                self.col = 0;
                self.row += 1;
            }
            result
        } else {
            None
        }
    }
}

