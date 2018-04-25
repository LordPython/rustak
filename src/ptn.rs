//use nom::{digit, IResult};
use ::game::{self,Loc,Move,Dir,Piece,Player};
use time::Tm;
use time;
use std::str::from_utf8;

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

#[derive(Debug)]
pub struct Ptn {
  pub player1: String,
  pub player2: String,
  pub date: Tm,
  pub size: usize,
  pub result: Option<game::Result>,
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

fn parse_square<'a>(input: &'a str) -> Result<(Loc, &'a str),ErrorType> {
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

  Ok((Loc { x: x, y: y }, chars.as_str()))
}

fn parse_piececount<'a>(input: &'a str) -> Result<(u8, &'a str),ErrorType> {
  use self::ErrorType::*;
  let mut chars = input.chars();

  let piececount = match chars.next() {
    Some(c @ '1'...'8') => c as u8 - b'0',
    Some(c)   => return Err(InvalidChar(c)),
    None      => return Err(EndOfFile),
  };

  Ok((piececount, chars.as_str()))
}

fn parse_dir<'a>(input: &'a str) -> Result<(Dir, &'a str),ErrorType> {
  use self::ErrorType::*;
  let mut chars = input.chars();

  let dir = match chars.next() {
    Some('+') => Dir::Up,
    Some('-') => Dir::Down,
    Some('<') => Dir::Left,
    Some('>') => Dir::Right,
    Some(c)   => return Err(InvalidChar(c)),
    None      => return Err(EndOfFile),
  };

  Ok((dir, chars.as_str()))
}

fn parse_piecetype<'a>(input: &'a str) -> Result<(Piece, &'a str),ErrorType> {
  use self::ErrorType::*;
  let mut chars = input.chars();

  let piece = match chars.next() {
    Some('f') | Some('F') => Piece::Flat,
    Some('s') | Some('S') => Piece::Wall,
    Some('c') | Some('C') => Piece::Cap,
    Some(c) => return Err(InvalidChar(c)),
    None    => return Err(EndOfFile),
  };

  Ok((piece, chars.as_str()))
}

fn parse_dropcounts<'a>(input: &'a str) -> Result<(u8,[u8;7], &'a str),ErrorType> {
  use self::ErrorType::*;
  let mut drops = [0;7];
  let mut chars = input.char_indices();
  let mut range = 0u8;

  while let Some((i, next_char)) = chars.next() {
    if i >= 7 {
      return Err(TooManyDrops);
    }

    drops[i] = match next_char {
      c @ '1' ... '8' => c as u8 - b'0',
      c => break,
    };

    range = i as u8 + 1;
  }

  if range > 0 {
    Ok((range, drops, chars.as_str()))
  } else {
    Err(NoDrops)
  }
}

fn parse_placement<'a>(input: &'a str) -> Result<(Move, &'a str),ErrorType> {
  if let Ok((square,input)) = parse_square(input) {
    Ok((Move::Place(square,Piece::Flat), input))
  } else {
    let (piece,input) = parse_piecetype(input)?;
    let (square,input) = parse_square(input)?;
    Ok((Move::Place(square,piece), input))
  }
}

fn parse_movement<'a>(input: &'a str) -> Result<(Move, &'a str),ErrorType> {
  use self::ErrorType::*;

  let (piececount,input) = parse_piececount(input).unwrap_or((1, input));
  let (square,input) = parse_square(input)?;
  let (dir,input) = parse_dir(input)?;
  match parse_dropcounts(input) {
    Ok((range,drops,input)) => {
      if piececount != drops.iter().sum() {
        return Err(InvalidPieceCount);
      }

      Ok((Move::Move(square, dir, range, drops, false), input))
    }
    Err(NoDrops) => {
      Ok((Move::Move(square, dir, 1, [piececount, 0, 0, 0, 0, 0, 0], false), input))
    },
    Err(e) => Err(e),
  }
}

pub fn parse_move<'a>(input: &'a str) -> Result<(Move, &'a str),ErrorType> {
  parse_movement(input).or_else(|_| parse_placement(input))
}

fn parse_annotated_move<'a>(input: &'a str) -> Result(AnnotatedMove, &'a str),ErrorType> {
  let (m, input) = parse_move(input)?;
  let input = input.trim_left();


  Ok((AnnotatedMove { m: m, annotation: (None, None) }, input))
}

/*
// -------------------------------------------
// ------------- Move Parsing ----------------
// -------------------------------------------

// Board square parsing (i.e. a1,a2,a3,....,h8)
named!(pub parse_square(&[u8]) -> Loc, 
  do_parse!(
    x: alt!( one_of!(&b"abcdefgh"[..]) => {|c| c as u8 - b'a'}
           | one_of!(&b"ABCDEFGH"[..]) => {|c| c as u8 - b'A'}) >>
    y: one_of!(&b"12345678"[..]) >>
    (Loc { x: x, y: y as u8 -b'1' })
  )
);

#[inline(always)]
named!(flat(&[u8]) -> Piece, value!(Piece::Flat));

// PTN format piece type
named!(piece_type(&[u8]) -> Piece,
  alt_complete!( one_of!(&b"fF"[..]) => { |_| Piece::Flat }
               | one_of!(&b"sS"[..]) => { |_| Piece::Wall }
               | one_of!(&b"cC"[..]) => { |_| Piece::Cap }
               | flat)
);

// PTN format movement direction
named!(movement_direction(&[u8]) -> Dir,
  alt!( tag!("+") => { |_| Dir::Up }
      | tag!("-") => { |_| Dir::Down }
      | tag!("<") => { |_| Dir::Left }
      | tag!(">") => { |_| Dir::Right })
);

// Movement move
named!(movement(&[u8]) -> Move,
  do_parse!(
    num_pieces: opt!(one_of!(&b"12345678"[..])) >>
    square: parse_square >>
    dir: movement_direction >>
    drops: opt!(complete!(is_a!(&b"12345678"[..]))) >>
    ({
      let mut d = [0u8; 7];
      let mut range = 1;

      match drops {
        Some(drops) => {
          range = drops.len();
          let mut i = 0;
          for c in drops {
            if i > 7 { break; }
            d[i] = c - b'0';
            i += 1;
          }
        },
        None => {
          d[0] = num_pieces.map(|x| x as u8 -b'0').unwrap_or(1);
        },
      }

      Move::Move(square, dir, range as u8, d, false)
    })
  )
);

// Placement move
named!(placement(&[u8]) -> Move,
/*
  do_parse!(
    piece: opt!(piece_type) >>
    square: parse_square >>
    (Move::Place(square, piece.unwrap_or(Piece::Flat)))
  )
  */
  /*
  alt!(do_parse!(
    piece: piece_type >>
    square: parse_square >>
    (Move::Place(square, piece))
  ) | parse_square => { |square| Move::Place(square, Piece::Flat) })
  */
  alt!(
    parse_square => { |square| Move::Place(square, Piece::Flat) }
    |
    do_parse!(
      piece: piece_type >>
      square: parse_square >>
      (Move::Place(square, piece))
    )
  )
);

// Parse a full move
// Either a placement or a movement
named!(pub parse_move(&[u8]) -> Move,
  //alt!(placement | movement)
  alt!(complete!(movement) | placement)
);

// Parsing for move annotations (tak / tinue)
named!(tak_eval(&[u8]) -> TakAnnotation,
  alt_complete!( tag!("''") => { |_| TakAnnotation::Tinue }
               | tag!("'") => { |_| TakAnnotation::Tak })
);

// Parsing for move annotations (subjective eval)
named!(subj_eval(&[u8]) -> SubjAnnotation,
  alt_complete!( tag!("??") => { |_| SubjAnnotation::Blunder }
               | tag!("?!") => { |_| SubjAnnotation::QuestionableSurprising }
               | tag!("?") => { |_| SubjAnnotation::Questionable }
               | tag!("!!") => { |_| SubjAnnotation::VerySurprising }
               | tag!("!?") => { |_| SubjAnnotation::SurprisingQuestionable }
               | tag!("!") => { |_| SubjAnnotation::Surprising })
);

// This has to be split out to make things compile ...
// Just a parser that returns (None,None) as an evaluation
#[inline(always)]
named!(no_eval(&[u8]) -> (Option<TakAnnotation>, Option<SubjAnnotation>),
  value!((None,None))
);

named!(annotation(&[u8]) -> (Option<TakAnnotation>, Option<SubjAnnotation>),
  alt_complete!(
    do_parse!(t: tak_eval >> s: opt!(subj_eval) >> (Some(t), s)) |
    do_parse!(s: subj_eval >> t: opt!(tak_eval) >> (t, Some(s)))
  )
);

named!(pub annotated_move(&[u8]) -> AnnotatedMove,
  do_parse!(
    parsed_move: parse_move >>
    annotation: opt!(complete!(annotation)) >>
    (AnnotatedMove { m: parsed_move, annotation: annotation.unwrap_or((None,None)) })
  )
);

// -------------------------------------------
// ------------- Tag Parsing -----------------
// -------------------------------------------

fn is_tag_char(c: u8) -> bool {
  match c {
    b'a' ... b'z' | b'A' ... b'Z' | b'0' ... b'9' | b'_' => true,
    _ => false
  }
}

named!(tag(&[u8]) -> Tag,
  do_parse!(
    tag!(b"[") >>
    many0!(one_of!(&b" \t"[..])) >>
    name: take_while!(is_tag_char) >>
    many0!(one_of!(&b" \t"[..])) >>
    value: delimited!(char!('"'), is_not!(&b"\""[..]), char!('"')) >>
    tag!(b"]") >> (
      Tag {
        name: from_utf8(name).unwrap().to_string(),
        value: from_utf8(value).unwrap().to_string()
      }
    )
  )
);

// -------------------------------------------
// ------------- Tag Parsing -----------------
// -------------------------------------------

named!(comment(&[u8]) -> (), map!(delimited!(char!('{'),opt!(is_not!("}")),char!('}')), |_| {}));

named!(result(&[u8]) -> game::Result,
  alt!( tag!("R-0") => { |_| game::Result::Road(Player::White) }
      | tag!("0-R") => { |_| game::Result::Road(Player::Black) }
      | tag!("F-0") => { |_| game::Result::Flat(Player::White) }
      | tag!("0-F") => { |_| game::Result::Flat(Player::Black) }
      | tag!("1-0") => { |_| game::Result::Other(Player::White) }
      | tag!("0-1") => { |_| game::Result::Other(Player::Black) }
      | tag!("1/2-1/2") => { |_| game::Result::Draw })
);

fn body(input: &[u8]) -> IResult<&[u8], Vec<AnnotatedMove>> {
  let mut moves = Vec::new();
  value!(input, moves, separated_list!(is_a!("\n\r"),
    do_parse!(
      many1!(digit) >> tag!(".") >>
      many_m_n!(1,2, do_parse!(
        //many0!(one_of!(&" \t"[..])) >>
        is_a!(&" \t"[..]) >>
        //opt!(comment) >>
        //many0!(one_of!(&" \t"[..])) >>
        tap!(a_move: annotated_move => moves.push(a_move.clone())) >> ()
      )) >>
      //many0!(complete!(one_of!(&" \t"[..]))) >>
      //opt!(complete!(do_parse!(comment >> many0!(one_of!(&" \t"[..])) >> ()))) >>
      ()
    )
  ))
}

named!(pub body_eof(&[u8]) -> Vec<AnnotatedMove>,
  do_parse!(moves: body /*>> eof!()*/ >> (moves))
);

pub fn parse(input: &[u8]) -> Option<Ptn> {
  let result = do_parse!(input,
    tags: separated_list!(is_a!(" \t\n\r"), tag) >>
    opt!(is_a!(" \t\n\r")) >>
    moves: body >>
    opt!(complete!(is_a!(" \t\n\r"))) >>
    //eof!() >>
    ({
      let mut notation = Ptn { player1: String::new(), player2: String::new(), date: time::now(), size: 0, result: None, tags: tags, moves: moves };
      let mut date = String::new();
      for tag in &notation.tags {
          if tag.name == "Player1" { notation.player1 = tag.value.clone(); }
          else if tag.name == "Player2" { notation.player2 = tag.value.clone(); }
          else if tag.name == "Size" { if let Ok(size) = tag.value.parse::<usize>() { notation.size = size; } }
          else if tag.name == "Result" { 
            if let IResult::Done(_,result) = result(tag.value.as_bytes()) {
              notation.result = Some(result);
            }
          }
          else if tag.name == "Date" {
            date.push_str(&tag.value);
            date.push_str(";");
          }
          else if tag.name == "Time" {
            date.push_str(&tag.value);
          }
      }
      if let Ok(tm) = time::strptime(&date, "%Y.%m.%d;%H:%M:%S") {
        notation.date = tm;
      }
      notation
    })
  );

  println!("NOM RES {:?}", result);

  if let IResult::Done(_,ptn) = result {
    if ptn.size < 3 || ptn.size > 8 { None }
    else { Some(ptn) }
  } else {
    None
  }
}
*/
