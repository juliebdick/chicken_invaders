#![feature(const_generics)]
#![allow(incomplete_features)]
#![cfg_attr(not(test), no_std)]


use bare_metal_modulo::{ModNumC, MNum, ModNumIterator};
use pluggable_interrupt_os::vga_buffer::{BUFFER_WIDTH, BUFFER_HEIGHT, plot, ColorCode, Color};
use pc_keyboard::{DecodedKey, KeyCode};
use num::traits::AsPrimitive;
//use pluggable_interrupt_os::println;


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
}

#[derive(Copy,Debug,Clone,Eq,PartialEq)]
pub struct Chicken {
    pos: Position,
    dir: Dir,
    active: bool
}

impl Chicken {
    fn new(position: Position) -> Self{
        Chicken{
            pos: position,
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
            pos: position,
            //rockets: [Position{ col: 0, row: 0 }; WIDTH*HEIGHT],
            dir: Dir::N,
            active: false //Only active on keypress
        }
    }
}

#[derive(Copy,Debug,Clone,Eq,PartialEq)]
pub struct Game {
    cannon: Cannon,
    chickens: [[Chicken; WIDTH];HEIGHT],
    //rockets: [Rockets; WIDTH];HEIGHT],
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
            chickens: [[Chicken::new(Position { col: 0, row: 0 }); WIDTH]; HEIGHT],
            //rockets: [Rockets::new(); WIDTH];HEIGHT],

            col: ModNumC::new(WIDTH),
            row: ModNumC::new(HEIGHT),
            cells: [[Cell::Empty; WIDTH]; HEIGHT],
            countdown: UPDATE_FREQUENCY,
            status: Status::Normal,
            score: 0
        };
        game.reset();
        game
    }
    fn get_chick_pos(self, chick: Chicken) -> Position {
        chick.pos
    }
    fn set_chick_pos(self, mut chick: Chicken, position: Position) {
        chick.pos = position
    }

    fn reset(&mut self) {
        self.status = Status::Normal;
        self.score = 0;
    }

    fn column_iter(&self) -> impl Iterator<Item=usize> {
        ModNumIterator::new(self.col)
            .take(WIDTH)
            .map(|m| m.a())
    }

    pub fn tick(&mut self) {
        self.clear_current();
        self.update_location();
        self.draw_current();
    }

    fn clear_current(&self) {
        for x in self.column_iter() {
            plot(' ', x, self.row.a(), ColorCode::new(Color::Black, Color::Black));
            plot(' ', x, self.cannon.pos.row.as_(), ColorCode::new(Color::Black, Color::Black));
        }
    }

    fn update_location(&mut self) {
        self.cannon.pos.col += self.cannon.dx.a() as i16;
        if self.cannon.pos.col >= BUFFER_WIDTH as i16 {
            self.cannon.pos.col = 0
        } else if self.cannon.pos.col < 0 {
            self.cannon.pos.col = BUFFER_WIDTH as i16 - 1;
        }
    }

    fn draw_current(&mut self) {
        for (_i, x) in self.column_iter().enumerate() {
            if x % 3 == 0 && x < WIDTH {
                plot(
                    '&',
                    x,
                    self.row.a(),
                    ColorCode::new(Color::Red, Color::Black)
                );
                self.chickens[self.row.a()][x] = Chicken::new(Position{ col: x as i16, row: self.row.a() as i16 });
            }
        }

        plot('^', self.cannon.pos.col.as_(), self.cannon.pos.row.as_(), ColorCode::new(Color::Cyan, Color::Black));

        // Move chickens down.
        if self.countdown_complete() {
            for (x, chick) in self.chickens.iter().enumerate() {
                let new_row = self.get_chick_pos(chick[x]).row - 1;
                self.set_chick_pos(chick[x], Position{ col: x as i16, row: new_row as i16 })
            }
        }
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
        match self.status {
            Status::Over => {
                match key {
                    DecodedKey::RawKey(KeyCode::S) | DecodedKey::Unicode('s') => self.reset(),
                    _ => {}
                }
            }
            _ => {}
        }
        match key {
            DecodedKey::RawKey(code) => self.handle_raw(code),
            //DecodedKey::Unicode(c) => self.handle_unicode(c),
            _ => {}
        }
    }

    fn handle_raw(&mut self, key: KeyCode) {
        match key {
            KeyCode::ArrowLeft => {
                self.cannon.pos.col -= 1;
            }
            KeyCode::ArrowRight => {
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