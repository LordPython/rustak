use nom::{self};
use ::game::{Loc,Move,Dir,Piece};

macro_rules! check(
  ($input:expr, $submac:ident!( $($args:tt)* )) => (

    {
      let mut failed = false;
      for idx in $input.chars() {
        if !$submac!(idx, $($args)*) {
            failed = true;
            break;
        }
      }
      if failed {
        nom::IResult::Error(nom::ErrorKind::Custom(20))
      } else {
        nom::IResult::Done(&""[..], $input)
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
        fn f(c: char) -> bool { c >= $min && c <= $max}
        flat_map!($input, take!(1), check!(f))
        }
    );
);

named!(pub parse_square(&str) -> Loc,
  do_parse!(
    x: alt!( char_between!('a','h') => {|c: &str| c.chars().next().unwrap() as u8 - b'a'}
           | char_between!('A','H') => {|c: &str| c.chars().next().unwrap() as u8 - b'A'}) >>
    y: char_between!('1','8') >>
    (Loc { x: x, y: y.chars().next().unwrap() as u8 - b'1' })
  )
);

#[inline(always)]
named!(flat(&str) -> Piece, value!(Piece::Flat));

named!(piece_type(&str) -> Piece,
  alt_complete!( tag_s!("W") => { |_| Piece::Wall }
               | tag_s!("C") => { |_| Piece::Cap }
               | flat)
);

named!(ws(&str) -> &str,
  is_a_s!(" \t\n\r")
);

named!(dropcount(&str) -> u8,
  map_res!(is_a_s!("1234567890"), |s: &str| { s.parse::<u8>() })
);

named!(placement(&str) -> Move,
  do_parse!(
    tag_s!("P") >> ws >>
    sq: parse_square >>
    pt: opt!(preceded!(ws, piece_type)) >>
    (Move::Place(sq, pt.unwrap_or(Piece::Flat)))
  )
);

named!(movement(&str) -> Move,
  do_parse!(
    tag_s!("M") >> ws >> start: parse_square >> ws >> end: parse_square >>
    drops: separated_list!(ws, dropcount) >> ({
      let dx = end.x as i16 - start.x as i16;
      let dy = end.y as i16 - start.y as i16;
      let dir = if dx > 0 {
        Dir::Right
      } else if dx < 0 {
        Dir::Left
      } else if dy > 0 {
        Dir::Up
      } else {
        Dir::Down
      };
      let mut dc = [0u8; 7];
      let mut i = 0;
      for drop in &drops {
        if i >= 7 {
          break;
        }
        dc[i] = *drop;
        i+=1;
      }
      Move::Move(start, dir, drops.len() as u8, dc, false)
    })
  )
);

named!(parse_move(&str) -> Move,
  alt!(placement | movement)
);
