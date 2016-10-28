use nom;
use ::game::{Loc,Move,Dir,Piece};
use std::cmp::min;

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
      for i in 0 .. min(7, range) {
        d[i] = drops[i][0] as u8 - b'0';
      }

      Move::Move { start: square, dir: dir, range: range as u8, drop_counts: d }
    }
  )
);

named!(placement <Move>,
  chain!(
    piece: opt!(alt!( one_of!("fF") => { |_| Piece::Flat }
                    | one_of!("sS") => { |_| Piece::Wall } 
                    | one_of!("cC") => { |_| Piece::Cap })) ~
    square: parse_square, || {
      Move::Place(square, piece.unwrap_or(Piece::Flat))
    }
  )
);

named!(pub parse_move <Move>,
  alt!(placement | movement)
);