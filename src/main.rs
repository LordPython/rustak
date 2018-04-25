#![feature(test)]
#![feature(const_fn)]
extern crate test;

//#[macro_use]
//extern crate nom;
extern crate time;
extern crate sqlite;

mod game;
mod ptn;
mod tps;
mod playtak;
mod tables;

mod fnv64 {
  use std::hash::Hasher;

  pub static BASE : [u64; 64] = [
    0x7c3b57bf837d04e6, 0xd9244b928d93faea,
    0x6fdb5f71960d526d, 0xc43e9c5a95f70002,
    0xba6a7d0e5b20e0d0, 0x3fa9033321f2cb46,
    0x87acbf433b10e9fb, 0xe2cfe6499c468b89,
    0x86dcf3bb0e869a7a, 0xd5a6846730ac08f3,
    0x450bc1f6c2ccf7d8, 0xf91760563e5a70a3,
    0xed1b7d4671f8247e, 0xe8256b873bacc9d8,
    0x27c8285421c11f8f, 0x2eb74959fb1e0b1f,
    0x8624d91e63283c54, 0xf7777ce8134a64ce,
    0x1cff8bb4887b613e, 0xc07bd839514f90d7,
    0xf7c7d7e3305b2653, 0xb6e36f88edb8d9e2,
    0x3dc19ad1fad5f094, 0xa6c23347f47f3739,
    0x4aec0a164c2ac2f7, 0xd8cb953d783a5372,
    0xc8755ec804055012, 0x3fb6057725e94dda,
    0x61b71a74521a203d, 0xb062c38ec4472232,
    0xa25c6d9786a4d298, 0x4d6c497f2e00901a,
    0x0dea728ad7fe460f, 0xfec7196d45a07561,
    0xe9b5fe05091fd05f, 0xb89f49baa7337191,
    0x190cd40907adc68f, 0x23181f040b6c0fdc,
    0xdbce4b65e1d466ae, 0x4d98880329ac3ffe,
    0x93e78ff7f174c251, 0x1b14b1260bb0c1bd,
    0xde998fcb787278c3, 0xbcaa0c55779241af,
    0x4582ebd8da8dc5e0, 0xcf9bbb89f4dfddbc,
    0x85f995fd0c267cdb, 0x37f88d5516236a07,
    0x74f5588e1bd222d3, 0x312f6f210bc6dbe8,
    0xf4b20eed1152ebd1, 0xe8b042a69a3e8bdb,
    0xb65fdb0f92909433, 0x7aff35845c48f21c,
    0x1fb03462fcf1f412, 0xb65d3df2d5a9b05a,
    0xc5dd5ad6eaada6b7, 0xb7e5528b517bbfb2,
    0x7beeba46b26e3efd, 0x7cc2d49edf179f04,
    0xdb7722d04b07a322, 0xd507e2e6a6ef4350,
    0xdd84f103bcda57b8, 0x3a0ab8ef41e4aa26,
  ];

  pub struct Fnv64 {
    hash: u64,
  }

  impl Fnv64 {
    pub fn new(init: u64) -> Self {
      Fnv64 { hash: init }
    }
  }

  impl Hasher for Fnv64 {
    fn finish(&self) -> u64 {
      self.hash
    }

    fn write(&mut self, bytes: &[u8]) {
      for byte in bytes {
        self.hash ^= *byte as u64;
        self.hash = self.hash.wrapping_mul(0x00000100000001B3);
      }
    }
  }
}

fn main() {
    println!("{:?}", playtak::parse_moves("  M A1 A3   1 2, P B3 C"));
    let c = game::bits::Constants::new(5);
    println!("0x{:x}", c.mask);

    let ptnmove = "6a5+123";
    println!("{:?}: {:?}", ptnmove, ptn::parse_move(ptnmove));
    println!("{:?}: {:?}", "Sa2", ptn::parse_move("Sa2"));
    println!("{:?}: {:?}", "D3", ptn::parse_move("D3"));
    println!("{:?}: {:?}", "Cb7", ptn::parse_move("Cb7"));
    //println!("{:?}: {:?}", "1. a3>1 a5\n2. b2 b1", ptn::body(&b"1. 1 a3>1 a5\n2. b2 b1"[..]));
    //println!("{:?}: {:?}", "BODY", ptn::body(&b"1. a3>1 a5 x"[..]));
    //println!("{:?}: {:?}", "BODY", ptn::body_eof(&b"1. a3>1 a5"[..]));
    /*
    let mut x = 3;
    loop {
      let next = c.grow(x, c.mask);
      if next == x { break; }
      println!("{}\n", c.format(next));
      x = next;
    }
    */

    /*
    let ptnmove = b"a5+123";
    println!("{:?}: {:?}", ptnmove, ptn::annotated_move(&ptnmove[..]));
    println!("{:?}: {:?}", "Sa2", ptn::annotated_move(&b"Sa2"[..]));
    println!("{:?}: {:?}", "D3", ptn::annotated_move(&b"D3"[..]));
    println!("{:?}: {:?}", "Cb7", ptn::annotated_move(&b"Cb7"[..]));
    //println!("{:?}: {:?}", "1. a3>1 a5\n2. b2 b1", ptn::body(&b"1. 1 a3>1 a5\n2. b2 b1"[..]));
    //println!("{:?}: {:?}", "BODY", ptn::body(&b"1. a3>1 a5 x"[..]));
    println!("{:?}: {:?}", "BODY", ptn::body_eof(&b"1. a3>1 a5"[..]));

    let bodyptn = include_bytes!("../body.ptn");
    println!("{:?}: {:?}", "BODYPTN", ptn::body_eof(bodyptn));
    let bodyptn2 = b"1. a1 a6\n2. c4 b4\n3. c3 c5\n4. b3 Cd4\n5. Cd3 a3\n";
    println!("{:?}: {:?}", "BODYPTN2", ptn::body_eof(bodyptn2));

    let file = include_bytes!("../test.ptn");
    let p = ptn::parse(file);
    if let Some(mut p) = p {
      let mut g = game::Game::new(p.size).unwrap();
      let mut plies = 0;
      for m in p.moves.iter_mut() {
        for gen_move in g.moves() {
          let valid = g.validate(&gen_move);
          if valid != game::MoveValidity::Valid {
            panic!("Invalid move: {:?}", valid);
          }
        }

        if let Some(res) = g.game_over() {
          println!("white:\n{}\nblack:\n{}\ncaps:\n{}\nwalls:\n{}",
          g.c.format(g.white),
          g.c.format(g.black),
          g.c.format(g.caps),
          g.c.format(g.walls));
          println!("Game over early! {:?}", res);
          panic!();
        }
        plies += 1;
        println!("Move: {:?}", m.m);
        let valid = g.validate(&m.m);
        if valid != game::MoveValidity::Valid {
          println!("Invalid because {:?}", valid);
          panic!();
        }
        g.execute(&mut m.m);
        println!("Board after {} plies: {}", plies, g.to_string());
      }

      println!("Result: {:?}", g.game_over());

      println!("-----------------UNDOING-----------------");

      for m in p.moves.iter().rev() {
        println!("Move: {:?}", m.m);
        g.undo(&m.m);
        println!("Board: {}", g.to_string());
      }
    }

    */
}

#[cfg(test)]
mod tests {
  use super::*;
  use super::ptn;
  use test::Bencher;

  #[bench]
  fn test_ptn(b: &mut Bencher) {
    //let file = include_bytes!("../test.ptn");
    //b.iter(|| ptn::parse(file));
  }

  #[bench]
  fn test_move(b: &mut Bencher) {
    //b.iter(|| ptn::parse_move(&b"3c3+12"[..]));
  }
}
