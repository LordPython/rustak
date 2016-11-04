#[derive(Debug)]
pub struct Loc {
  pub x: u8,
  pub y: u8,
}

#[derive(Debug)]
pub enum Dir {
  Up,
  Down,
  Left,
  Right
}

#[derive(Debug)]
pub enum Piece {
  Flat, Wall, Cap
}

#[derive(Debug)]
pub enum Move {
  Place(Loc, Piece),
  Move { start: Loc, dir: Dir, range: u8, drop_counts: [u8; 7] }
}

#[derive(Debug)]
pub enum Player {
  White,
  Black,
}

#[derive(Debug)]
pub enum Result {
  Road(Player),  // Road win
  Flat(Player),  // Flat win
  Other(Player), // Win by forfeit or time
  Draw,
}
