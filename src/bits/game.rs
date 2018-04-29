use std::hash::{Hash,Hasher};
use ::fnv64::{self,Fnv64};
use ::game::*;
use std::cmp::min;
use bits::BinConv;

#[derive(Debug)]
pub struct Game {
  // Size of the board (i.e. size = 5 for a 5x5 game)
  size: usize,
  // Current round, where 1 round is a turn for each player
  round: u32,
  // Current player
  player: Player,

  white_reserves: Reserves,
  black_reserves: Reserves,

  left_mask: u64,
  right_mask: u64,
  top_mask: u64,
  bottom_mask: u64,
  full_mask: u64,

  caps: u64,
  walls: u64,
  white: u64,
  black: u64,
  //
  owners: Vec<::bits::Stack>,
  partial_hash: u64,
}

impl Game {
  pub fn new(size: usize) -> Option<Self> {
    if size < 3 || size > 8 { return None }

    let mut left_mask = 1u64;
    for _ in 1..size { left_mask |= left_mask << size; }

    Some(Game {
        size,
        round: 1,
        player: Player::White,
        white_reserves: Reserves::new(size).unwrap(),
        black_reserves: Reserves::new(size).unwrap(),
        left_mask,
        right_mask: left_mask << (size-1),
        top_mask: ((1u64<<size)-1)<<(size*size-size),
        bottom_mask: (1u64<<size)-1,
        full_mask: match size { 0 ... 7 => (1u64<<(size*size))-1, _ => -1i64 as u64 },
        caps: 0,
        walls: 0,
        white: 0,
        black: 0,
        owners: vec![::bits::Stack::new(); (size*size) as usize],
        partial_hash: 0,
    })
  }

  fn grow(&self, val: u64, mask: u64) -> u64 {
    let mut res = val;
    res |= (val >> 1) & (!self.right_mask);
    res |= (val << 1) & (!self.left_mask);
    res |= val >> self.size;
    res |= val << self.size;

    res & mask
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
  fn fix_top(&mut self, idx: usize) {
    self.set_top(idx, Piece::Flat);
    if self.owners[idx].is_empty() {
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

  pub fn hash(&self) -> u64 {
    let mut hasher = Fnv64::new(self.partial_hash);
    hasher.write_u64(self.caps);
    hasher.write_u64(self.walls);
    hasher.write_u8(self.player.binary() as u8);
    hasher.finish()
  }

  pub fn validate(&self, m: &Move) -> MoveValidity {
    match *m {
      Move::Place(loc, piece) => {
        if loc.x as usize >= self.size || loc.y as usize >= self.size { return MoveValidity::InvalidSquare; }
        let idx = self.idx(loc);
        if !self.owners[idx].is_empty() { return MoveValidity::SquareOccupied; }
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
        if self.owners[start_idx].is_empty() { return MoveValidity::DontControlStack; }
        if self.offset(start_idx,dir,range) > (self.size * self.size) { return MoveValidity::EndOutOfBounds; }
        if self.owners[start_idx].get(0) != self.player { return MoveValidity::DontControlStack; }
        let is_cap = self.get_top(start_idx) == Piece::Cap;
        let mut pieces_moved = 0;
        for i in 1 .. range+1 {
          let idx = self.offset(start_idx,dir,i);
          pieces_moved += drop_counts[i as usize - 1];
          if !self.owners[idx].is_empty() {
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
        for i in (1 ..= range).rev() {
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

  // I would prefer to replace this with an iterator since that's
  // a bit more general, but that's rather painful to write without
  // generators here, based on the amount of bookkeeping that would
  // have to be done. This may also be more efficient than an iterator,
  // though that's just speculation
  pub fn foreach_move<E, F: FnMut(Move) -> Result<(),E>>(&self, mut f: F) -> Result<(),E> {
    if self.round == 1 {
      for y in 0..self.size {
        for x in 0..self.size {
          let loc = Loc { x: x as u8, y: y as u8};
          if self.owners[self.idx(loc)].is_empty() {
            f(Move::Place(loc, Piece::Flat))?;
          }
        }
      }
    } else {
      for y in 0..self.size {
        for x in 0..self.size {
          let loc = Loc { x: x as u8, y: y as u8 };
          let idx = self.idx(loc);

          if self.owners[idx].is_empty() {
            if self.reserves(self.player).count(Piece::Flat) > 0 {
              f(Move::Place(loc, Piece::Flat))?;
              f(Move::Place(loc, Piece::Wall))?;
            } else if self.reserves(self.player).count(Piece::Cap) > 0 {
              f(Move::Place(loc, Piece::Cap))?;
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

                if self.owners[self.idx(l)].is_empty() {
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
                f(Move::Move(loc, dir, range, counts, false))?;
              }
              Ok(())
            };

            add_moves(Dir::Up)?;
            add_moves(Dir::Down)?;
            add_moves(Dir::Left)?;
            add_moves(Dir::Right)?;
          }
        }
      }
    }

    Ok(())
  }

  pub fn status(&self) -> Option<Winner> {
    let check_road = |p: u64, e1: u64, e2: u64| {
      let mask = p & !self.walls;
      let mut cur = e1 & mask;
      loop {
        let next = self.grow(cur, mask);
        if (next & e2) != 0 { return true; }
        if next == cur { return false; }
        cur = next;
      }
    };

    // Check opponents road first (since they just moved, so if they made roads for both players
    // they get the win, by the dragon rule
    let opponent = match self.player { Player::White => self.black, Player::Black => self.white };
    if check_road(opponent, self.bottom_mask, self.top_mask)
    || check_road(opponent, self.left_mask, self.right_mask)
    {
      return Some(Winner::Road(self.player.opponent()));
    }

    let player = match self.player { Player::White => self.white, Player::Black => self.black };
    if check_road(player, self.bottom_mask, self.top_mask)
    || check_road(player, self.left_mask, self.right_mask)
    {
      return Some(Winner::Road(self.player));
    }

    if (self.white | self.black) == self.full_mask
    || self.white_reserves.empty()
    || self.black_reserves.empty()
    {
      let wcount = (self.white & !self.walls & !self.caps).count_ones();
      let bcount = (self.black & !self.walls & !self.caps).count_ones();
      if wcount > bcount { return Some(Winner::Flat(Player::White)); }
        else if wcount < bcount { return Some(Winner::Flat(Player::Black)); }
          else { return Some(Winner::Draw); }
    }

    None
  }

  #[inline]
  pub fn round(&self) -> u32 { self.round }
  #[inline]
  pub fn cur_player(&self) -> Player { self.player }
  #[inline]
  pub fn reserves(&self, player: Player) -> &Reserves {
    match player {
      Player::White => &self.white_reserves,
      Player::Black => &self.black_reserves,
    }
  }
}

impl ToString for Game {
  fn to_string(&self) -> String {
    let mut out = String::new();
    for j in (0 .. self.size).rev() {
      let mut prev_empty = 0;
      for i in 0 .. self.size {
        let idx = self.idx(Loc { x: i as u8, y: j as u8 });
        if self.owners[idx].is_empty() {
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
