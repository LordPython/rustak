use fnv64::{self,Fnv64};
use std::hash::{Hash,Hasher};
use std::cmp::min;

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct Loc {
  pub x: u8,
  pub y: u8,
}

impl Loc {
  fn offset(self, dir: Dir, dist: u8) -> Self {
    match dir {
      Dir::Up => Loc { x: self.x, y: self.y.wrapping_add(dist) },
      Dir::Down => Loc { x: self.x, y: self.y.wrapping_sub(dist) },
      Dir::Left => Loc { x: self.x.wrapping_sub(dist), y: self.y },
      Dir::Right => Loc { x: self.x.wrapping_add(dist), y: self.y },
    }
  }
}

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum Dir {
  Up,
  Down,
  Left,
  Right
}

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum Piece {
  Flat, Wall, Cap
}

#[derive(Debug,Clone)]
pub enum Move {
  Place(Loc, Piece),
  Move(Loc, Dir, u8, [u8; 7], bool),
}

#[derive(Debug,Clone)]
pub struct ValidMove {
  m: Move,
}

#[derive(Debug,Clone,Copy,PartialEq)]
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

  #[inline]
  pub fn binary(self) -> u128 {
    match self {
      Player::White => 0,
      Player::Black => 1,
    }
  }

  #[inline]
  pub fn from_binary(bin: u128) -> Player {
    match bin {
      0 => Player::White,
      _ => Player::Black,
    }
  }
}

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum Result {
  Road(Player),  // Road win
  Flat(Player),  // Flat win
  Other(Player), // Win by forfeit or time
  Draw,
}

#[derive(Debug)]
pub struct Reserves {
  flats: u8,
  caps: u8,
}

impl Reserves {
  fn new(size: usize) -> Option<Self> {
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

  fn add(&mut self, p: Piece) {
    match p {
      Piece::Cap => self.caps += 1,
      _ => self.flats += 1,
    };
  }

  fn remove(&mut self, p: Piece) {
    match p {
      Piece::Cap => self.caps -= 1,
      _ => self.flats -= 1,
    };
  }
}

pub mod bits {
  #[derive(Debug)]
  pub struct Constants {
    pub size: u64,
    pub left: u64,
    pub right: u64,
    pub top: u64,
    pub bottom: u64,
    pub mask: u64,
  }

  impl Constants {
    pub fn new(size: usize) -> Self {
      let mut left = 1u64;
      for _ in 1..size {
        left |= left << size;
      }

      Constants {
        size: size as u64,
        left: left,
        right: left<<size-1,
        top: ((1u64<<size)-1)<<(size*size-size),
        bottom: (1u64<<size)-1,
        mask: match size { 0 ... 7 => (1u64<<(size*size))-1, _ => -1i64 as u64 },
      }
    }

    pub fn grow(&self, val: u64, mask: u64) -> u64 {
      let mut res = val;
      res |= (val >> 1) & (!self.right);
      res |= (val << 1) & (!self.left);
      res |= val >> self.size;
      res |= val << self.size;
      return res & mask;
    }

    pub fn format(&self, val: u64) -> String {
      let mut s = String::with_capacity((self.size*self.size) as usize);
      for j in (0u64 .. self.size).rev() {
        for i in 0u64 .. self.size {
          s.push(match (val >> (i + self.size*j))&1 {
            0 => '.',
            _ => '#',
          });
        }
        s.push('\n');
      }
      s
    }
  }

  #[derive(Debug,Clone,Copy)]
  pub struct Stack {
    owners: u128,
    size: u8,
  }

  use super::Player;
  use std::iter::{FromIterator,IntoIterator};
  use std::hash::{Hash,Hasher};

  impl Stack {
    pub fn new() -> Self {
      Stack { owners: 0, size: 0 }
    }

    #[inline]
    pub fn len(&self) -> usize {
      self.size as usize
    }

    #[inline]
    pub fn get(&self, index: usize) -> Player {
      Player::from_binary((self.owners >> index) & 1)
    }

    #[inline]
    pub fn pop_stack(&mut self, n: u8) -> Stack {
      let o = self.owners & ((1<<n)-1);
      self.owners >>= n;
      self.size -= n;
      Stack { owners: o, size: n }
    }

    #[inline]
    pub fn push_stack(&mut self, s: Stack) {
      self.owners = (self.owners << s.size) | s.owners;
      self.size += s.size;
    }

    #[inline]
    pub fn push(&mut self, p: Player) {
      self.owners = (self.owners << 1) | p.binary();
      self.size += 1;
    }
  }

  impl Hash for Stack {
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
      hasher.write_u128(self.owners);
      hasher.write_u8(self.size);
    }
  }

  impl FromIterator<Player> for Stack {
    fn from_iter<T>(iter: T) -> Self 
       where T: IntoIterator<Item=Player>
    {
      let mut s = Stack::new();
      for p in iter {
        s.push(p);
      }
      s
    }
  }

  pub struct StackIter<'a> {
    front: usize,
    back: usize,
    stack: &'a Stack,
  }

  impl<'a> Iterator for StackIter<'a> {
    type Item = Player;
    fn next(&mut self) -> Option<Player> {
      if self.front < self.back {
        let res = Some(self.stack.get(self.front));
        self.front += 1;
        res
      } else {
        None
      }
    }
  }

  impl<'a> DoubleEndedIterator for StackIter<'a> {
    fn next_back(&mut self) -> Option<Player> {
      if self.front < self.back {
        self.back -= 1;
        Some(self.stack.get(self.back))
      } else {
        None
      }
    }
  }

  impl<'a> IntoIterator for &'a Stack {
    type Item = Player;
    type IntoIter = StackIter<'a>;
    fn into_iter(self) -> StackIter<'a> {
      StackIter { front: 0, back: self.len(), stack: self }
    }
  }
}

#[derive(Debug)]
pub struct Game {
  size: usize,
  pub round: u32,
  player: Player,
  white_reserves: Reserves,
  black_reserves: Reserves,
  pub caps: u64,
  pub walls: u64,
  pub white: u64,
  pub black: u64,
  pub owners: Vec<bits::Stack>,
  pub c: bits::Constants,
  partial_hash: u64,
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

impl Game {
  pub fn new(size: usize) -> Option<Self> {
    match size {
      3 ... 8 => Some(Game {
        size: size,
        round: 1,
        player: Player::White,
        white_reserves: Reserves::new(size).unwrap(),
        black_reserves: Reserves::new(size).unwrap(),
        caps: 0,
        walls: 0,
        white: 0,
        black: 0,
        owners: vec![bits::Stack::new(); (size*size) as usize],
        c: bits::Constants::new(size),
        partial_hash: 0,
      }),
      _ => None,
    }
  }

  #[inline]
  fn idx(&self, loc: Loc) -> usize {
    loc.x as usize + self.size * loc.y as usize
  }

  #[inline]
  fn offset(&self, idx: usize, dir: Dir, dist: u8) -> usize {
    match dir {
      Dir::Up => idx+self.size*dist as usize,
      Dir::Down => idx-self.size*dist as usize,
      Dir::Left => idx-dist as usize,
      Dir::Right => idx+dist as usize,
    }
  }

  #[inline]
  fn reserves_mut(&mut self, player: Player) -> &mut Reserves {
    match player {
      Player::White => &mut self.white_reserves,
      Player::Black => &mut self.black_reserves,
    }
  }

  #[inline]
  fn reserves(&self, player: Player) -> &Reserves {
    match player {
      Player::White => &self.white_reserves,
      Player::Black => &self.black_reserves,
    }
  }

  #[inline]
  fn fix_top(&mut self, idx: usize) {
    self.set_top(idx, Piece::Flat);
    if self.owners[idx].len() == 0 {
      self.white &= !(1<<idx);
      self.black &= !(1<<idx);
    } else {
      match self.owners[idx].get(0) {
        Player::White => {
          self.white |= 1<<idx;
          self.black &= !(1<<idx);
        },
        Player::Black => {
          self.white &= !(1<<idx);
          self.black |= 1<<idx;
        },
      }
    }
  }

  #[inline]
  fn transfer(&mut self, from: usize, to: usize, amount: u8) {
    let pieces = self.owners[from].pop_stack(amount);
    self.owners[to].push_stack(pieces);
    self.fix_top(from);
    self.fix_top(to);
  }

  fn update_hash(&mut self, idx: usize) {
    let mut hasher = Fnv64::new(fnv64::BASE[idx]);
    self.owners[idx].hash(&mut hasher);
    self.partial_hash ^= hasher.finish();
  }

  pub fn hash(&self) -> u64 {
    let mut hasher = Fnv64::new(self.partial_hash);
    //hasher.write_u64(self.white); // Redundant?
    //hasher.write_u64(self.black); // Redundant?
    hasher.write_u64(self.caps);
    hasher.write_u64(self.walls);
    hasher.write_u8(self.player.binary() as u8);
    hasher.finish()
  }

  #[inline]
  fn get_top(&self, idx: usize) -> Piece {
    if self.caps & (1<<idx) != 0 {
      Piece::Cap
    } else if self.walls & (1<<idx) != 0 {
      Piece::Wall
    } else {
      Piece::Flat
    }
  }

  #[inline]
  fn set_top(&mut self, idx: usize, p: Piece) {
    match p {
      Piece::Cap => self.caps |= 1<<idx,
      Piece::Wall => self.walls |= 1<<idx,
      Piece::Flat => {
        self.caps &= !(1<<idx);
        self.walls &= !(1<<idx);
      }
    }
  }

  pub fn validate(&self, m: &Move) -> MoveValidity {
    match *m {
      Move::Place(loc, piece) => {
        if loc.x as usize >= self.size || loc.y as usize >= self.size { return MoveValidity::InvalidSquare; }
        let idx = self.idx(loc);
        if self.owners[idx].len() != 0 { return MoveValidity::SquareOccupied; }
        if self.reserves(self.player).count(piece) == 0 { return MoveValidity::NotEnough(piece); }
        if self.round == 1 && piece != Piece::Flat {
          return MoveValidity::MustPlaceFlatFirstRound;
        }
        MoveValidity::Valid
      },
      Move::Move(start, dir, range, ref drop_counts, _) => {
        if start.x as usize >= self.size || start.y as usize >= self.size { return MoveValidity::InvalidSquare; }
        if self.round == 1 { return MoveValidity::MustPlaceFlatFirstRound; }
        let start_idx = self.idx(start);
        if self.owners[start_idx].len() == 0 { return MoveValidity::DontControlStack; }
        if self.offset(start_idx,dir,range) > (self.size * self.size) { return MoveValidity::EndOutOfBounds; }
        if self.owners[start_idx].get(0) != self.player { return MoveValidity::DontControlStack; }
        let is_cap = self.get_top(start_idx) == Piece::Cap;
        let mut pieces_moved = 0;
        for i in 1 .. range+1 {
          let idx = self.offset(start_idx,dir,i);
          pieces_moved += drop_counts[i as usize - 1];
          if self.owners[idx].len() > 0 {
            match self.get_top(idx) {
              Piece::Cap => return MoveValidity::CantMoveIntoCap,
              Piece::Wall => {
                if !is_cap {
                  return MoveValidity::NeedCapToSmash;
                }
                if drop_counts[i as usize - 1] != 1 {
                  return MoveValidity::CapMustSmashAlone;
                }
                if i != range {
                  return MoveValidity::SmashMustBeLast;
                }
              },
              _ => {},
            }
          }
          if pieces_moved > self.owners[start_idx].len() as u8 {
            return MoveValidity::NotEnoughPieces;
          }
          if pieces_moved > self.size as u8 {
            return MoveValidity::CarryLimit;
          }
        }
        MoveValidity::Valid
      },
    }
  }

  pub fn execute(&mut self, m: &mut Move) {
    match *m {
      Move::Place(loc, piece) => {
        let idx = self.idx(loc);
        self.update_hash(idx);
        let player = if self.round == 1 { self.player.opponent() } else { self.player };
        self.reserves_mut(player).remove(piece);
        self.owners[idx].push(player);
        self.fix_top(idx);
        self.set_top(idx, piece);
        self.update_hash(idx);
      },
      Move::Move(start, dir, range, ref drop_counts, ref mut smash) => {
        let start_idx = self.idx(start);
        let end_idx = self.offset(start_idx, dir, range);
        self.update_hash(start_idx);
        *smash = self.walls & (1<<end_idx) != 0;
        let top = self.get_top(start_idx);
        for i in (1 .. range+1).rev() {
          let idx = self.offset(start_idx,dir,i);
          self.update_hash(idx);
          self.transfer(start_idx, idx, drop_counts[(i-1) as usize]);
          self.update_hash(idx);
        }
        self.set_top(end_idx, top);
        self.update_hash(start_idx);
      },
    }

    self.player = self.player.opponent();
    if self.player == Player::White {
      self.round += 1;
    }
  }

  pub fn undo(&mut self, m: &Move) {
    if self.player == Player::White {
      self.round -= 1;
    }
    self.player = self.player.opponent();

    match *m {
      Move::Place(loc, piece) => {
        let idx = self.idx(loc);
        self.update_hash(idx);
        let player = if self.round == 1 { self.player } else { self.player.opponent() };
        self.reserves_mut(player).add(piece);
        let _ = self.owners[idx].pop_stack(1);
        self.walls &= !(1<<idx);
        self.caps &= !(1<<idx);
        self.white &= !(1<<idx);
        self.black &= !(1<<idx);
        self.update_hash(idx);
      },
      Move::Move(start, dir, range, ref drop_counts, smash) => {
        let start_idx = self.idx(start);
        let end_idx = self.offset(start_idx, dir, range);
        let top = self.get_top(end_idx);
        self.update_hash(start_idx);
        for i in 1 .. range+1 {
          let idx = self.offset(start_idx,dir,i);
          self.update_hash(idx);
          self.transfer(idx, start_idx, drop_counts[(i-1) as usize]);
          self.update_hash(idx);
        }
        self.set_top(start_idx, top);
        if smash { self.set_top(end_idx, Piece::Wall); }
        self.update_hash(start_idx);
      },
    }
  }

  pub fn moves(&self) -> Vec<Move> {
    // TODO: Decide how to set starting capacity
    let mut moves = Vec::with_capacity(50);

    if self.round == 1 {
      for y in 0..self.size {
        for x in 0..self.size {
          let loc = Loc { x: x as u8, y: y as u8};
          if self.owners[self.idx(loc)].len() == 0 {
            moves.push(Move::Place(loc, Piece::Flat));
          }
        }
      }
    } else {
      for y in 0..self.size {
        for x in 0..self.size {
          let loc = Loc { x: x as u8, y: y as u8 };
          let idx = self.idx(loc);

          if self.owners[idx].len() == 0 {
            if self.reserves(self.player).count(Piece::Flat) > 0 {
              moves.push(Move::Place(loc, Piece::Flat));
              moves.push(Move::Place(loc, Piece::Wall));
            } else if self.reserves(self.player).count(Piece::Cap) > 0 {
              moves.push(Move::Place(loc, Piece::Cap));
            }
          } else if self.owners[idx].get(0) == self.player {
            let mut add_moves = |dir: Dir| {
              let mut max_dist = 0;
              let mut smash = false;
              let mut l = loc;
              loop {
                l = l.offset(dir, 1);
                if l.x >= self.size as u8 || l.y >= self.size as u8 {
                  break;
                }

                if self.owners[self.idx(l)].len() == 0 {
                  max_dist += 1;
                  continue;
                }
                match self.get_top(self.idx(l)) {
                  Piece::Cap => break,
                  Piece::Wall => {
                    smash = self.get_top(idx) == Piece::Cap;
                    if smash { max_dist += 1; }
                    break;
                  },
                  Piece::Flat => {},
                }
                max_dist += 1;
              }

              let mobile_pieces = min(self.size, self.owners[idx].len());
              let dist = min(mobile_pieces, max_dist);
              use tables::{drop_counts, DropCount};
              for &DropCount(range, counts) in drop_counts(mobile_pieces, dist, smash) {
                moves.push(Move::Move(loc, dir, range, counts, false));
              }
            };

            add_moves(Dir::Up);
            add_moves(Dir::Down);
            add_moves(Dir::Left);
            add_moves(Dir::Right);
          }
        }
      }
    }

    moves
  }

  pub fn game_over(&self) -> Option<Result> {
    let check_road = |p: u64, e1: u64, e2: u64| {
      let mask = p & !self.walls;
      let mut cur = e1 & mask;
      loop {
        let next = self.c.grow(cur, mask);
        if (next & e2) != 0 { return true; }
        if next == cur { return false; }
        cur = next;
      }
    };

    // Check opponents road first (since they just moved, so if they made roads for both players
    // they get the win, by the dragon rule
    let opponent = match self.player { Player::White => self.black, Player::Black => self.white };
    if check_road(opponent, self.c.bottom, self.c.top)
    || check_road(opponent, self.c.left, self.c.right)
    {
      return Some(Result::Road(self.player.opponent()));
    }

    let player = match self.player { Player::White => self.white, Player::Black => self.black };
    if check_road(player, self.c.bottom, self.c.top)
    || check_road(player, self.c.left, self.c.right)
    {
      return Some(Result::Road(self.player));
    }

    if (self.white | self.black) == self.c.mask
    || self.white_reserves.empty()
    || self.black_reserves.empty()
    {
      let wcount = (self.white & !self.walls & !self.caps).count_ones();
      let bcount = (self.black & !self.walls & !self.caps).count_ones();
      if wcount > bcount { return Some(Result::Flat(Player::White)); }
      else if wcount < bcount { return Some(Result::Flat(Player::Black)); }
      else { return Some(Result::Draw); }
    }

    None
  }
}

impl ToString for Game {
  fn to_string(&self) -> String {
    let mut out = String::new();
    for j in (0 .. self.size).rev() {
      let mut prev_empty = 0;
      for i in 0 .. self.size {
        let idx = self.idx(Loc { x: i as u8, y: j as u8 });
        if self.owners[idx].len() == 0 {
          prev_empty += 1;
          if i == (self.size - 1) {
            out.push('x');
            if prev_empty > 1 { out.push_str(prev_empty.to_string().as_str()); }
          }
        } else {
          if prev_empty > 0 {
            out.push('x');
            if prev_empty > 1 { out.push_str(prev_empty.to_string().as_str()); }
            out.push(',');
            prev_empty = 0;
          }
          for p in self.owners[idx].into_iter().rev() {
            match p {
              Player::White => out.push('1'),
              Player::Black => out.push('2'),
            }
          }
          match self.get_top(idx) {
            Piece::Cap => out.push('C'),
            Piece::Wall => out.push('S'),
            _ => {},
          }
          if i < (self.size - 1) {
            out.push(',');
          }
        }

      }
      if j > 0 {
        out.push('/');
      }
    }

    out.push(' ');
    out.push(match self.player { Player::White => '1', Player::Black => '2' });
    out.push(' ');
    out.push_str(self.round.to_string().as_str());
    out
  }
}
