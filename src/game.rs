pub use ::bits::Game;

#[derive(Debug,Clone,Copy,PartialEq,Hash)]
pub enum Player {
  White,
  Black,
}

impl Player {
  #[inline]
  pub fn opponent(self) -> Player {
    match self {
      Player::White => Player::Black,
      Player::Black => Player::White,
    }
  }
}

#[derive(Debug,Clone,Copy,PartialEq,Hash)]
pub enum Piece {
  Flat, Wall, Cap
}

#[derive(Debug,Clone,Copy,PartialEq,Hash)]
pub enum Dir {
  Up,
  Down,
  Left,
  Right
}

#[derive(Debug,Clone,Copy,PartialEq,Hash)]
pub struct Loc {
  pub x: u8,
  pub y: u8,
}

impl Loc {
  pub fn offset(self, dir: Dir, dist: u8) -> Self {
    match dir {
      Dir::Up => Loc { x: self.x, y: self.y.wrapping_add(dist) },
      Dir::Down => Loc { x: self.x, y: self.y.wrapping_sub(dist) },
      Dir::Left => Loc { x: self.x.wrapping_sub(dist), y: self.y },
      Dir::Right => Loc { x: self.x.wrapping_add(dist), y: self.y },
    }
  }
}

#[derive(Debug,Clone,PartialEq,Hash)]
pub enum Move {
  Place(Loc, Piece),
  Move(Loc, Dir, u8, [u8; 7], bool),
}

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum MoveValidity {
  Valid,
  InvalidSquare,
  SquareOccupied,
  DontControlStack,
  NotEnoughPieces,
  NeedCapToSmash,
  CapMustSmashAlone,
  SmashMustBeLast,
  CantMoveIntoCap,
  NotEnough(Piece),
  MustPlaceFlatFirstRound,
  EndOutOfBounds,
  CarryLimit,
}

#[derive(Debug,Clone,Copy)]
pub struct Reserves {
  flats: u8,
  caps: u8,
}

impl Reserves {
  pub fn new(size: usize) -> Option<Self> {
    match size {
      3 => Some(Reserves { flats: 10, caps: 0 }),
      4 => Some(Reserves { flats: 15, caps: 0 }),
      5 => Some(Reserves { flats: 21, caps: 1 }),
      6 => Some(Reserves { flats: 30, caps: 1 }),
      7 => Some(Reserves { flats: 40, caps: 2 }),
      8 => Some(Reserves { flats: 50, caps: 2 }),
      _ => None,
    }
  }

  pub fn empty(&self) -> bool {
    self.flats == 0 && self.caps == 0
  }

  pub fn count(&self, p: Piece) -> u8 {
    match p {
      Piece::Cap => self.caps,
      _          => self.flats,
    }
  }

  pub fn add(&mut self, p: Piece) {
    match p {
      Piece::Cap => self.caps += 1,
      _ => self.flats += 1,
    };
  }

  pub fn remove(&mut self, p: Piece) {
    match p {
      Piece::Cap => self.caps -= 1,
      _ => self.flats -= 1,
    };
  }
}

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum Winner {
  Road(Player),
  Flat(Player),
  Other(Player), // Win by forfeit or time
  Draw,
}

pub fn new(size: usize) -> Option<::bits::Game> {
  Game::new(size)
}

