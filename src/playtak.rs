use game::{self, Move, Loc, Piece, Dir, Player};

#[derive(Debug)]
pub enum ErrorType {
  InvalidChar(char),
  InvalidMoveSquares(Loc,Loc),
  EndOfFile,
}

#[derive(Debug)]
pub enum ParseResult<'a, T> {
  Done(T, &'a str),
  Error(ErrorType),
}

#[inline]
fn is_ws(c: char) -> bool {
  match c {
    ' ' | '\t' | '\n' | 'r' => true,
    _ => false,
  }
}

#[inline]
fn ws(input: &str) -> &str {
  let mut i = input.chars();
  while i.clone().next().map(is_ws).unwrap_or(false) { i.next(); }
  i.as_str()
}

#[inline]
fn parse_square(input: &str) -> Result<(Loc, &str),ErrorType> {
  use self::ErrorType::*;

  let mut chars = input.chars();

  let x = match chars.next()  {
    Some(c @ 'a' ... 'h') => c as u8 - b'a',
    Some(c @ 'A' ... 'H') => c as u8 - b'A',
    Some(c) => return Err(InvalidChar(c)),
    None    => return Err(EndOfFile),
  };

  let y = match chars.next() {
    Some(c @ '1' ... '8') => c as u8 - b'1',
    Some(c) => return Err(InvalidChar(c)),
    None    => return Err(EndOfFile),
  };

  Ok((Loc { x, y }, chars.as_str()))
}

#[inline]
fn parse_piecetype(input: &str) -> Result<(Piece, &str),ErrorType> {
  use self::ErrorType::*;
  let mut chars = input.chars();
  let piece = match chars.next() {
    Some('c') | Some('C') => Piece::Cap,
    Some('w') | Some('W') => Piece::Wall,
    Some(c) => return Err(InvalidChar(c)),
    None => return Err(EndOfFile),
  };

  Ok((piece, chars.as_str()))
}

#[inline]
fn parse_dropcounts(input: &str, num: u8) -> Result<([u8; 7], &str), ErrorType> {
  use self::ErrorType::*;
  let mut i = input;
  let mut dropcounts = [0u8; 7];
  for idx in 0 .. num {
    let mut chars = ws(i).chars();
    dropcounts[idx as usize] = match chars.next() {
      Some(c @ '1' ... '8') => c as u8 - b'0',
      Some(c) => return Err(InvalidChar(c)),
      None => return Err(EndOfFile),
    };
    i = chars.as_str();
  }

  Ok((dropcounts, i))
}

#[inline]
pub fn parse_move(input: &str) -> Result<(Move, &str),ErrorType> {
  use self::ErrorType::*;
  let mut chars = input.chars();

  match chars.next() {
    Some('p') | Some('P') => {
      let (square, input) = parse_square(ws(chars.as_str()))?;
      if let Ok((piece, input)) = parse_piecetype(ws(input)) {
        Ok((Move::Place(square, piece), input))
      } else {
        Ok((Move::Place(square, Piece::Flat), input))
      }
    },
    Some('m') | Some('M') => {
      let (start, input) = parse_square(ws(chars.as_str()))?;
      let (end, input) = parse_square(ws(input))?;
      let dx = end.x as i8 - start.x as i8;
      let dy = end.y as i8 - start.y as i8;
      let (dir, range) = if dy > 0 && dx == 0 {
        (Dir::Up, dy as u8)
      } else if dy < 0 && dx == 0 {
        (Dir::Down, -dy as u8)
      } else if dx > 0 && dy == 0 {
        (Dir::Right, dx as u8)
      } else if dx < 0 && dy == 0 {
        (Dir::Left, -dx as u8)
      } else {
        return Err(InvalidMoveSquares(start,end));
      };
      let (dropcounts, input) = parse_dropcounts(input, range)?;
      Ok((Move::Move(start, dir, range, dropcounts, false), input))
    },
    Some(c) => Err(InvalidChar(c)),
    None    => Err(EndOfFile),
  }
}

pub fn parse_moves(input: &str) -> Result<(Vec<Move>, &str), ErrorType> {
  let mut moves = Vec::new();
  let mut res = match parse_move(ws(input)) {
    Ok(res) => res,
    Err(_) => return Ok((moves, input)),
  };
  moves.push(res.0);
  loop {
    let mut chars = ws(res.1).chars();
    match chars.next() {
      Some(',') => {
        res = parse_move(ws(chars.as_str()))?;
        moves.push(res.0);
      },
      _ => break,
    }
  }

  Ok((moves, res.1))
}

pub fn parse_result(input: &str) -> Result<(game::Winner, &str), ErrorType> {
  use self::ErrorType::*;
  let mut chars = input.chars();
  let res = match chars.next() {
    Some('1') => {
      match chars.next() {
        Some('-') => {
          match chars.next() {
            Some('0') => game::Winner::Other(Player::White),
            Some(c) => return Err(InvalidChar(c)),
            None => return Err(EndOfFile),
          }
        },
        Some('/') => {
          match chars.next() {
            Some('2') => {},
            Some(c) => return Err(InvalidChar(c)),
            None => return Err(EndOfFile),
          }
          match chars.next() {
            Some('-') => {},
            Some(c) => return Err(InvalidChar(c)),
            None => return Err(EndOfFile),
          }
          match chars.next() {
            Some('1') => {},
            Some(c) => return Err(InvalidChar(c)),
            None => return Err(EndOfFile),
          }
          match chars.next() {
            Some('/') => {},
            Some(c) => return Err(InvalidChar(c)),
            None => return Err(EndOfFile),
          }
          match chars.next() {
            Some('2') => game::Winner::Draw,
            Some(c) => return Err(InvalidChar(c)),
            None => return Err(EndOfFile),
          }
        },
        Some(c) => return Err(InvalidChar(c)),
        None => return Err(EndOfFile),
      }
    },
    Some('0') => {
      match chars.next() {
        Some('-') => {},
        Some(c) => return Err(InvalidChar(c)),
        None => return Err(EndOfFile),
      }
      match chars.next() {
        Some('R') => game::Winner::Road(Player::Black),
        Some('F') => game::Winner::Flat(Player::Black),
        Some('1') => game::Winner::Other(Player::Black),
        Some(c) => return Err(InvalidChar(c)),
        None => return Err(EndOfFile),
      }
    },
    Some('R') => {
      match chars.next() {
        Some('-') => {},
        Some(c) => return Err(InvalidChar(c)),
        None => return Err(EndOfFile),
      }
      match chars.next() {
        Some('0') => game::Winner::Road(Player::White),
        Some(c) => return Err(InvalidChar(c)),
        None => return Err(EndOfFile),
      }
    },
    Some('F') => {
      match chars.next() {
        Some('-') => {},
        Some(c) => return Err(InvalidChar(c)),
        None => return Err(EndOfFile),
      }
      match chars.next() {
        Some('0') => game::Winner::Flat(Player::White),
        Some(c) => return Err(InvalidChar(c)),
        None => return Err(EndOfFile),
      }
    },
    Some(c) => return Err(InvalidChar(c)),
    None => return Err(EndOfFile),
  };

  Ok((res, chars.as_str()))
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;
  use game::{self, MoveValidity};
  use sqlite;
  use super::parse_moves;
  use super::parse_result;

  // Games that have the wrong result recorded from when playtak did not properly implement the
  // dragon rule
  const PLAYTAK_DRAGON_RULE_BUG_GAMES : [i64; 13] = [3172,4932,6037,6249,14270,15070,15527,16082,16325,17091,17316,17405,17532];
  // Games that have extra moves past when the game has been won, and the database has the wrong
  // result recorded
  const PLAYTAK_INCORRECT_RESULT_GAMES : [i64; 7] = [380, 3018, 9329, 15296, 54675, 81952, 116539];
  //
  const PLAYTAK_UNKNOWN_PROBLEM_GAMES: [i64; 3] = [9013,9449,9598];

  #[test]
  fn test_moves() {
    let mut dbfile = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dbfile.push("games_anon.db.1");
    let connection = sqlite::open(dbfile).unwrap();
    // Games between sTAKbot1 and sTAKbot2 have been filtered out as it seems a number of them
    // contain invalid moves
    let mut cursor = connection.prepare("
      SELECT size, notation, result, id 
      FROM games
      WHERE (player_white != \"sTAKbot1\" or player_black != \"sTAKbot2\")
        and (player_white != \"sTAKbot2\" or player_black != \"sTAKbot1\")
    ").unwrap().cursor();

    while let Some(row) = cursor.next().unwrap() {
      let size = row[0].as_integer().unwrap();
      let moves_str = row[1].as_string().unwrap();
      let result_str = row[2].as_string().unwrap();
      let id = row[3].as_integer().unwrap();

      if result_str.starts_with("0-0") {
        // Not sure why there exist games with this result.
        // It's not a valid result
        // Perhaps these were offered draws?
        continue;
      }
      let mut moves = match parse_moves(moves_str) {
        Ok((moves, _)) => moves,
        Err(e) => panic!("Could not parse moves '{}', error: {:?}", moves_str, e),
      };
      let result = match parse_result(result_str) {
        Ok((result, _)) => result,
        Err(e) => panic!("Could not parse result '{}', error: {:?}", result_str, e),
      };
      let mut g = game::new(size as usize).unwrap();
      for m in moves.iter_mut() {
        if let Some(_) = g.status() {
          break;
        }
        if let MoveValidity::Valid = g.validate(m) {
          g.execute(m);
        } else {
          panic!("Error during simulated game(id={})\nMove {:?} not valid\nBoard State: \n{}\nMoves str:\n{}", id, m, g.to_string(), moves_str);
        }
      }

      match result {
        game::Winner::Other(_) => {},
        _ => {
          if PLAYTAK_UNKNOWN_PROBLEM_GAMES.contains(&id) {
            continue;
          }

          if PLAYTAK_INCORRECT_RESULT_GAMES.contains(&id) {
            continue;
          }

          if PLAYTAK_DRAGON_RULE_BUG_GAMES.contains(&id) {
            continue;
          }
          let simulated = g.status();
          if simulated.is_none() {
            if result == game::Winner::Draw { continue; }
            panic!("Simulated game (id={}) did not terminate, result should have been {:?}\nFinal board state:\n{}", id, result, g.to_string());
          }

          if result != simulated.unwrap() {
            panic!("Simulated game (id={}) had incorrect result {:?}, should have been {:?}\nFinal board state:\n{}", id, simulated, result, g.to_string());
          }
        },
      }
    }
  }
}

