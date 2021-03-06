﻿use nom::{self,ErrorKind, digit, IResult, Needed, types::CompleteStr};
use nom::error_to_list;
use ::game::{self,Loc,Move,Dir,Piece,Player};

const OUT_OF_RANGE_CHAR_CODE : u32 = 1;
const TOO_MANY_DROPS_CODE : u32 = 2;
const NUM_PIECES_MISMATCH_CODE : u32 = 3;

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub enum TakAnnotation {
  Tak,
  Tinue,
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub enum SubjAnnotation {
  Questionable,
  Surprising,
  Blunder,
  VerySurprising,
  QuestionableSurprising,
  SurprisingQuestionable,
}

#[derive(Clone)]
#[derive(Debug)]
pub struct AnnotatedMove {
  pub m: Move,
  pub annotation: (Option<TakAnnotation>, Option<SubjAnnotation>),
}

// TODO: Figure out if we should remove player1 etc tags
// TODO: Figure out what should be parsed into the struct
//       and what should be left as tags
#[derive(Debug)]
pub struct Ptn {
  pub player1: String,
  pub player2: String,
  pub size: usize,
  pub result: Option<game::Winner>,
  pub tags: Vec<Tag>,
  pub moves: Vec<AnnotatedMove>
}

#[derive(Debug)]
pub struct Tag {
  name: String,
  value: String,
}

#[derive(Debug)]
pub enum ErrorType {
  InvalidChar(char),
  EndOfFile,
  TooManyDrops,
  InvalidPieceCount,
  NoDrops,
}

#[derive(Debug)]
pub struct ParseError {
  err_type: ErrorType
}

pub fn parse_move(input: &str) -> Option<Move> {
  match parse_move_internal(input.trim()) {
    Ok(("", m)) => Some(m),
    Ok((s,_)) => {
      println!("Not exact match, remaining characters: {}", s);
      None
    },
    Err(e) => {
      println!("Error: {:?}", e);
      None
    }
  }
}

pub fn parse(input: &str) -> Option<Ptn> {
  None
}

macro_rules! parse_char_num (
  ($input:expr, $lower:expr, $upper:expr, zero: $base:expr) => ({
    let mut chars = $input.chars();
    match chars.next() {
      None => Err(ParseError { err_type: ErrorType::EndOfFile }),
      Some(c @ $lower ..= $upper) => Ok((chars.as_str(), (c as u8) - ($base as u8))),
      Some(c) => Err(ParseError { err_type: ErrorType::InvalidChar(c) }),
    }
  });
);

pub fn parse_square(input: &str) -> Result<(&str,Loc),ParseError> {
  let (remaining,x) =
    parse_char_num!(input, 'a','h', zero: 'a')
    .or_else(|_| parse_char_num!(input, 'A','H', zero: 'A'))?;
  let (remaining,y) = parse_char_num!(remaining, '1','8', zero: '1')?;
  Ok((remaining, Loc{x,y}))
}

fn parse_piece_type(input: &str) -> Result<(&str,Piece),ParseError> {
  let mut chars = input.chars();
  match chars.next() {
    None => Err(ParseError { err_type: ErrorType::EndOfFile }),
    Some('f') | Some('F') => Ok((chars.as_str(), Piece::Flat)),
    Some('s') | Some('S') => Ok((chars.as_str(), Piece::Wall)),
    Some('c') | Some('C') => Ok((chars.as_str(), Piece::Cap)),
    Some(c) => Err(ParseError { err_type: ErrorType::InvalidChar(c) }),
  }
}

fn parse_direction(input: &str) -> Result<(&str,Dir),ParseError> {
  let mut chars = input.chars();
  match chars.next() {
    None => Err(ParseError { err_type: ErrorType::EndOfFile }),
    Some('+') => Ok((chars.as_str(), Dir::Up)),
    Some('-') => Ok((chars.as_str(), Dir::Down)),
    Some('<') => Ok((chars.as_str(), Dir::Left)),
    Some('>') => Ok((chars.as_str(), Dir::Right)),
    Some(c) => Err(ParseError { err_type: ErrorType::InvalidChar(c) }),
  }
}

fn parse_movement(input: &str) -> Result<(&str,Move),ParseError> {
  let (remaining, num_pieces) = parse_char_num!(input, '1','8', zero: '0').unwrap_or((input, 1));
  let (remaining, square) = parse_square(remaining)?;
  let (remaining, dir) = parse_direction(remaining)?;
  let range = remaining.bytes().take_while(|c| *c >= b'1' && *c <= b'8').count();
  let (drops_str, remaining) = remaining.split_at(range);

  if range == 0 {
    Ok((remaining, Move::Move(square, dir, 1, [num_pieces,0,0,0,0,0,0], false)))
  } else if range <= 7 {
    let mut drops = [0u8;7];
    for (i, c) in drops_str.char_indices() {
      drops[i] = c as u8 - b'0';
    }

    if num_pieces != drops.iter().sum() {
      println!("num_pieces={}, drops={:?}", num_pieces, drops);
      return Err(ParseError { err_type: ErrorType::InvalidPieceCount });
    }

    Ok((remaining, Move::Move(square, dir, range as u8, drops, false)))
  } else {
    Err(ParseError { err_type: ErrorType::TooManyDrops })
  }
}

fn parse_placement(input: &str) -> Result<(&str,Move),ParseError> {
  let (remaining, piece) = parse_piece_type(input).unwrap_or((input, Piece::Flat));
  let (remaining, square) = parse_square(remaining)?;
  Ok((remaining, Move::Place(square, piece)))
}

fn parse_move_internal(input: &str) -> Result<(&str,Move),ParseError> {
  match parse_movement(input) {
    Ok(res) => Ok(res),
    Err(e @ ParseError { err_type: ErrorType::TooManyDrops }) |
    Err(e @ ParseError { err_type: ErrorType::InvalidPieceCount }) => Err(e),
    Err(_) => parse_placement(input),
  }
}

fn expect_char(input: &str, ch: char) -> Result<&str,ParseError> {
  let mut chars = input.chars();
  match chars.next() {
    Some(c) if c == ch => Ok(chars.as_str()),
    Some(c) => Err(ParseError { err_type: ErrorType::InvalidChar(c) }),
    None => Err(ParseError { err_type: ErrorType::EndOfFile }),
  }
}

fn parse_key(input: &str) -> Result<(&str,String),ParseError> {
  if input.is_empty() {
    return Err(ParseError { err_type: ErrorType::EndOfFile });
  }

  let key_len = input.bytes().take_while(|c| match *c {
    b'a' ..= b'z' | b'A' ..= b'Z' | b'_' => true,
    _ => false
  }).count();

  if key_len == 0 {
    return Err(ParseError { err_type: ErrorType::InvalidChar(input.chars().next().unwrap())})
  }

  let (key, remaining) = input.split_at(key_len);
  Ok((remaining, key.to_ascii_lowercase()))
}

fn parse_value(input: &str) -> Result<(&str,String),ParseError> {
  let remaining = expect_char(input, '"')?;
  let value_len = remaining.bytes().take_while(|c| *c != b'"').count();
  let (value, remaining) = remaining.split_at(value_len);
  let remaining = expect_char(remaining, '"')?;
  Ok((remaining, value.to_string()))
}

fn parse_tag(input: &str) -> Result<(&str,Tag),ParseError> {
  let remaining = expect_char(input, '[')?;
  let (remaining, name) = parse_key(remaining.trim_left())?;
  let (remaining, value) = parse_value(remaining.trim_left())?;
  let remaining = expect_char(remaining.trim_left(), ']')?;
  Ok((remaining, Tag { name, value }))
}

fn parse_winner(input: &str) -> Result<(&str, game::Winner), ErrorType> {
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

  Ok((chars.as_str(), res))
}

/*
macro_rules! char_to_number (
  ($i:expr, $lower:expr, $upper:expr, base: $base:expr) => ({
    let mut chars = $i.chars();
    match chars.next() {
      None             => Err(nom::Err::Incomplete(Needed::Size(1))),
      Some(c @ $lower ..= $upper)  => Ok((CompleteStr(chars.as_str()), (c as u8) - ($base as u8))),
      Some(_) => Err(nom::Err::Error(error_position!($i, ErrorKind::Custom(OUT_OF_RANGE_CHAR_CODE)))),
    }
  });
);

// Insert a failure if the given condition evaluates to true
macro_rules! failure_if (
  ($i:expr, $cond:expr, $code:expr) => ({
    cond_with_error!($i, $cond, |_| -> IResult<CompleteStr,()> {
      Err(nom::Err::Failure(error_position!($i, ErrorKind::Custom($code))))
    })
  })
);

fn eat_ws(input: CompleteStr) -> IResult<CompleteStr, ()> {
  value!(input, (), take_while!(char::is_whitespace))
}

// -------------------------------------------
// ------------- Move Parsing ----------------
// -------------------------------------------

// Board square parsing (i.e. a1,a2,a3,....,h8)
// Note: a1 -> Loc { x: 0, y: 0 }, a2 -> Loc { x: 0, y: 1 }, etc.
named!(pub parse_square(CompleteStr) -> Loc,
  do_parse!(
    x: alt!( char_to_number!('a','h', base: 'a')
           | char_to_number!('A','H', base: 'A')) >>
    y: char_to_number!('1','8', base: '1') >>
    (Loc { x, y })
  )
);

// PTN format piece type
named!(parse_piece_type(CompleteStr) -> Piece,
  alt!( one_of!("fF") => { |_| Piece::Flat }
      | one_of!("sS") => { |_| Piece::Wall }
      | one_of!("cC") => { |_| Piece::Cap })
);

// PTN format movement direction
named!(parse_direction(CompleteStr) -> Dir,
  alt!( char!('+') => { |_| Dir::Up }
      | char!('-') => { |_| Dir::Down }
      | char!('<') => { |_| Dir::Left }
      | char!('>') => { |_| Dir::Right })
);

named!(parse_num_pieces(CompleteStr) -> u8,
  do_parse!(
    n: opt!(char_to_number!('1','8', base: '0')) >>
    (n.unwrap_or(1))
  )
);

named!(parse_drops(CompleteStr) -> (u8,[u8;7]),
  do_parse!(
    drops: take_while!(|c| c >= '1' && c <= '8') >>
    failure_if!(drops.len() > 7, TOO_MANY_DROPS_CODE) >>
    ({
      let mut d = [0u8; 7];
      for (i, c) in drops.char_indices() {
        d[i] = c as u8 - b'0';
      }
      (drops.len() as u8, d)
    })
  )
);

// Movement move
named!(movement(CompleteStr) -> Move,
  do_parse!(
    num_pieces: parse_num_pieces >>
    square: parse_square >>
    dir: parse_direction >>
    drops: parse_drops >>
    // Move is invalid
    failure_if!(drops.0 > 0 && drops.1.iter().sum::<u8>() != num_pieces, NUM_PIECES_MISMATCH_CODE) >>
    ({
      let range = drops.0;
      let drops = drops.1;

      if range == 0 {
        // No drops were included, use default (all to adjacent square)
        Move::Move(square, dir, 1, [num_pieces,0,0,0,0,0,0], false)
      } else {
        Move::Move(square, dir, range, drops, false)
      }
    })
  )
);

// Placement move
named!(placement(CompleteStr) -> Move,
  do_parse!(
    piece: opt!(parse_piece_type) >>
    square: parse_square >>
    (Move::Place(square, piece.unwrap_or(Piece::Flat)))
  )
);

// Parse a full move
// Either a placement or a movement
named!(parse_move_internal(CompleteStr) -> Move,
  alt!(movement | placement)
);

// Parsing for move annotations (tak / tinue)
named!(tak_eval(CompleteStr) -> TakAnnotation,
  alt!( tag!("''") => { |_| TakAnnotation::Tinue }
      | tag!("'") => { |_| TakAnnotation::Tak })
);

// Parsing for move annotations (subjective eval)
named!(subj_eval(CompleteStr) -> SubjAnnotation,
  alt!( tag!("??") => { |_| SubjAnnotation::Blunder }
      | tag!("?!") => { |_| SubjAnnotation::QuestionableSurprising }
      | tag!("?") => { |_| SubjAnnotation::Questionable }
      | tag!("!!") => { |_| SubjAnnotation::VerySurprising }
      | tag!("!?") => { |_| SubjAnnotation::SurprisingQuestionable }
      | tag!("!") => { |_| SubjAnnotation::Surprising })
);

named!(annotated_move(CompleteStr) -> AnnotatedMove,
  do_parse!(
    parsed_move: parse_move_internal >>
    annotation: alt!(
      do_parse!(t: opt!(preceded!(eat_ws, tak_eval)) >>
                s: opt!(preceded!(eat_ws, subj_eval)) >> (t, s)) |
      do_parse!(s: opt!(preceded!(eat_ws, subj_eval)) >>
                t: opt!(preceded!(eat_ws, tak_eval)) >> (t, s))
    ) >>

    (AnnotatedMove { m: parsed_move, annotation: annotation })
  )
);

// -------------------------------------------
// ------------- Tag Parsing -----------------
// -------------------------------------------

fn is_tag_char(c: char) -> bool {
  match c {
    'a' ..= 'z' | 'A' ..= 'Z' | '0' ..= '9' | '_' => true,
    _ => false
  }
}

named!(tag(CompleteStr) -> Tag,
  do_parse!(
    tag!("[") >>
    eat_ws >>
    name: take_while!(is_tag_char) >>
    eat_ws >>
    value: delimited!(tag!("\""), is_not!("\""), tag!("\"")) >>
    eat_ws >>
    tag!("]") >> (
      Tag {
        name: name.to_ascii_lowercase(),
        value: value.to_string()
      }
    )
  )
);

// -------------------------------------------
// ------------- Tag Parsing -----------------
// -------------------------------------------

named!(comment(CompleteStr) -> (), value!((), delimited!(char!('{'),opt!(is_not!("}")),char!('}'))));

named!(result(CompleteStr) -> game::Winner,
  alt!( tag!("R-0") => { |_| game::Winner::Road(Player::White) }
      | tag!("0-R") => { |_| game::Winner::Road(Player::Black) }
      | tag!("F-0") => { |_| game::Winner::Flat(Player::White) }
      | tag!("0-F") => { |_| game::Winner::Flat(Player::Black) }
      | tag!("1-0") => { |_| game::Winner::Other(Player::White) }
      | tag!("0-1") => { |_| game::Winner::Other(Player::Black) }
      | tag!("1/2-1/2") => { |_| game::Winner::Draw })
);

fn body(input: CompleteStr) -> IResult<CompleteStr, Vec<AnnotatedMove>> {
  let mut moves = Vec::new();
  value!(input, moves, separated_list!(eat_ws,
    do_parse!(
      many1!(digit) >> tag!(".") >>
      many_m_n!(1,2, do_parse!(
        eat_ws >>
        opt!(comment) >>
        eat_ws >>
        tap!(a_move: annotated_move => moves.push(a_move.clone())) >> ()
      )) >>
      opt!(preceded!(eat_ws,comment)) >>
      ()
    )
  ))
}

pub fn parse_move(input: &str) -> Option<Move> {
  match exact!(CompleteStr(input.trim()), parse_move_internal) {
    Ok((_, m)) => Some(m),
    Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
      println!("ERROR LIST {:?}", error_to_list(&e));
      None
    }
    Err(nom::Err::Incomplete(n)) => {
      println!("INCOMPELTE {:?}", n);
      None
    },
  }
}

pub fn parse(input: &str) -> Option<Ptn> {
  let result = do_parse!(CompleteStr(input),
    eat_ws >>
    tags: separated_list!(eat_ws, tag) >>
    eat_ws >>
    moves: body >>
    eat_ws >>
    eof!() >>
    ({
      let mut notation = Ptn { player1: String::new(), player2: String::new(), size: 0, result: None, tags, moves };

      for tag in &notation.tags {
        if tag.name == "player1" {
          notation.player1 = tag.value.clone();
        }
        else if tag.name == "player2" {
          notation.player2 = tag.value.clone();
        }
        else if tag.name == "size" {
          if let Ok(size) = tag.value.parse::<usize>() {
            notation.size = size;
          }
        } else if tag.name == "result" {
          if let Ok((_,result)) = result(CompleteStr(&tag.value)) {
            notation.result = Some(result);
          }
        }
      }
      notation
    })
  );

  match result {
    Ok((_,ptn)) => {
      if ptn.size < 3 || ptn.size > 8 { None }
      else { Some(ptn) }
    }
    Err(nom::Err::Error(c)) | Err(nom::Err::Failure(c)) => {
      println!("ERROR LIST {:?}", error_to_list(&c));
      None
    }
    _ => {
      println!("INCOMPLETE");
      None
    }
  }
}
*/

pub fn to_string(m: &Move) -> String {
  match m {
    Move::Place(loc, piece) => {
      let piecestr = match piece {
        Piece::Cap => "C",
        Piece::Wall => "S",
        Piece::Flat => "",
      };
      let x = (loc.x + b'a') as char;
      let y = (loc.y + b'1') as char;
      format!("{}{}{}", piecestr, x, y)
    },
    Move::Move(loc, dir, range, drops, _) => {
      let piece_count : u8 = drops.iter().sum();
      let x = (loc.x + b'a') as char;
      let y = (loc.y + b'1') as char;
      let dir_str = match dir {
        Dir::Up => "+",
        Dir::Down => "-",
        Dir::Left => "<",
        Dir::Right => ">",
      };
      let mut string = String::new();
      if piece_count != 1 { string.push((b'0'+piece_count) as char); }
      string.push(x);
      string.push(y);
      string.push_str(dir_str);
      if *range > 1 {
        for drop in drops {
          if *drop == 0 { break; }
          string.push((b'0' + *drop) as char);
        }
      }
      string
    },
  }
}

#[cfg(test)]
mod test {
}
