use nom;
use ::game::{self,Loc,Move,Dir,Piece};
use std::cmp::min;
use time::Tm;
use std::str::from_utf8;

#[derive(Debug)]
pub struct Ptn {
  player1: String,
  player2: String,
  date: Tm,
  size: u8,
  result: game::Result,
  moves: Vec<Move>
}

#[derive(Debug)]
pub struct Tag {
  name: String,
  value: String,
}

macro_rules! check(
  ($input:expr, $submac:ident!( $($args:tt)* )) => (

    {
      let mut failed = false;
      for &idx in $input {
        if !$submac!(idx, $($args)*) {
            failed = true;
            break;
        }
      }
      if failed {
        nom::IResult::Error(nom::Err::Position(nom::ErrorKind::Custom(20),$input))
      } else {
        nom::IResult::Done(&b""[..], $input)
      }
    }
  );
  ($input:expr, $f:expr) => (
    check!($input, call!($f));
  );
);

macro_rules! char_between(
    ($input:expr, $min:expr, $max:expr) => (
        {
        fn f(c: u8) -> bool { c >= ($min as u8) && c <= ($max as u8)}
        flat_map!($input, take!(1), check!(f))
        }
    );
);

named!(pub parse_square <Loc>, 
  chain!(
    x: alt!( char_between!('a','h') => {|c: &[u8]| c[0]-b'a'} 
           | char_between!('A', 'H') => {|c: &[u8]| c[0]-b'A'}) ~
    y: char_between!('1', '8'), ||
    { Loc { x: x, y: y[0]-b'1' } }
  )
);

named!(movement <Move>,
  chain!(
    num_pieces: opt!(char_between!('1', '8')) ~
    square: parse_square ~
    dir: one_of!(b"+-<>") ~
    drops: many0!(char_between!('1', '8')), || {
      let range = drops.len();
      let dir = match dir {
        '+' => Dir::Up,
        '-' => Dir::Down,
        '<' => Dir::Left,
        /*'>'*/ _  => Dir::Right,
      };

      let mut d = [0u8; 7];
      if drops.len() == 0 {
        d[0] = num_pieces.map(|x| x[0]-b'0').unwrap_or(1);
      }
      for i in 0 .. min(7, range) {
        d[i] = drops[i][0] as u8 - b'0';
      }

      Move::Move { start: square, dir: dir, range: range as u8, drop_counts: d }
    }
  )
);

named!(piece_type(&[u8]) -> Piece,
  alt!( one_of!("fF") => { |_| Piece::Flat }
      | one_of!("sS") => { |_| Piece::Wall }
      | one_of!("cC") => { |_| Piece::Cap })
);

named!(placement(&[u8]) -> Move,
  chain!(
    piece: opt!(piece_type) ~
    square: parse_square, || {
      Move::Place(square, piece.unwrap_or(Piece::Flat))
    }
  )
);

fn is_tag_char(c: u8) -> bool {
  match c {
    b'a' ... b'z' | b'A' ... b'Z' | b'0' ... b'9' | b'_' => true,
    _ => false
  }
}

named!(pub ptn_tag(&[u8]) -> Tag,
  chain!(
    tag!(b"[") ~
    name: take_while!(is_tag_char) ~
    many0!(one_of!(b" \t")) ~
    value: delimited!(char!('"'), is_not!(b"\""), char!('"')) ~
    tag!(b"]"), || {
      Tag {
        name: from_utf8(name).unwrap().to_string(),
        value: from_utf8(value).unwrap().to_string()
      }
    }
  )
);

named!(pub parse_move <Move>,
  alt!(placement | movement)
);
