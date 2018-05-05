#![feature(test)]
#![feature(const_fn)]
extern crate test;

#[macro_use]
extern crate nom;
extern crate time;

#[cfg(test)]
extern crate sqlite;

pub mod bits;
pub mod game;
pub mod ptn;
pub mod tps;
pub mod playtak;
pub mod tables;

mod fnv64 {
  use std::hash::Hasher;

  pub static BASE : [u64; 64] = [
    0x7c3b_57bf_837d_04e6, 0xd924_4b92_8d93_faea,
    0x6fdb_5f71_960d_526d, 0xc43e_9c5a_95f7_0002,
    0xba6a_7d0e_5b20_e0d0, 0x3fa9_0333_21f2_cb46,
    0x87ac_bf43_3b10_e9fb, 0xe2cf_e649_9c46_8b89,
    0x86dc_f3bb_0e86_9a7a, 0xd5a6_8467_30ac_08f3,
    0x450b_c1f6_c2cc_f7d8, 0xf917_6056_3e5a_70a3,
    0xed1b_7d46_71f8_247e, 0xe825_6b87_3bac_c9d8,
    0x27c8_2854_21c1_1f8f, 0x2eb7_4959_fb1e_0b1f,
    0x8624_d91e_6328_3c54, 0xf777_7ce8_134a_64ce,
    0x1cff_8bb4_887b_613e, 0xc07b_d839_514f_90d7,
    0xf7c7_d7e3_305b_2653, 0xb6e3_6f88_edb8_d9e2,
    0x3dc1_9ad1_fad5_f094, 0xa6c2_3347_f47f_3739,
    0x4aec_0a16_4c2a_c2f7, 0xd8cb_953d_783a_5372,
    0xc875_5ec8_0405_5012, 0x3fb6_0577_25e9_4dda,
    0x61b7_1a74_521a_203d, 0xb062_c38e_c447_2232,
    0xa25c_6d97_86a4_d298, 0x4d6c_497f_2e00_901a,
    0x0dea_728a_d7fe_460f, 0xfec7_196d_45a0_7561,
    0xe9b5_fe05_091f_d05f, 0xb89f_49ba_a733_7191,
    0x190c_d409_07ad_c68f, 0x2318_1f04_0b6c_0fdc,
    0xdbce_4b65_e1d4_66ae, 0x4d98_8803_29ac_3ffe,
    0x93e7_8ff7_f174_c251, 0x1b14_b126_0bb0_c1bd,
    0xde99_8fcb_7872_78c3, 0xbcaa_0c55_7792_41af,
    0x4582_ebd8_da8d_c5e0, 0xcf9b_bb89_f4df_ddbc,
    0x85f9_95fd_0c26_7cdb, 0x37f8_8d55_1623_6a07,
    0x74f5_588e_1bd2_22d3, 0x312f_6f21_0bc6_dbe8,
    0xf4b2_0eed_1152_ebd1, 0xe8b0_42a6_9a3e_8bdb,
    0xb65f_db0f_9290_9433, 0x7aff_3584_5c48_f21c,
    0x1fb0_3462_fcf1_f412, 0xb65d_3df2_d5a9_b05a,
    0xc5dd_5ad6_eaad_a6b7, 0xb7e5_528b_517b_bfb2,
    0x7bee_ba46_b26e_3efd, 0x7cc2_d49e_df17_9f04,
    0xdb77_22d0_4b07_a322, 0xd507_e2e6_a6ef_4350,
    0xdd84_f103_bcda_57b8, 0x3a0a_b8ef_41e4_aa26,
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
        self.hash ^= u64::from(*byte);
        self.hash = self.hash.wrapping_mul(0x0000_0100_0000_01B3);
      }
    }
  }
}

/*
fn main() {
    println!("{:?}", playtak::parse_moves("  M A1 A3   1 2, P B3 C"));
    //let c = game::bits::Constants::new(5);
    //println!("0x{:x}", c.mask);

    let ptnmove = "6a5+123";
    //println!("{:?}: {:?}", ptnmove, ptn::parse_move(ptnmove));
    //println!("Sa2: {:?}", ptn::parse_move("Sa2"));
    //println!("D3: {:?}", ptn::parse_move("D3"));
    //println!("Cb7: {:?}", ptn::parse_move("Cb7"));
    //println!("{:?}: {:?}", "1. a3>1 a5\n2. b2 b1", ptn::body(&b"1. 1 a3>1 a5\n2. b2 b1"[..]));
    //println!("{:?}: {:?}", "BODY", ptn::parse("1. a3>1 a5 x"));
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
    let file = include_bytes!("../test.ptn");
    //let p = ptn::parse(file);
    //println!("PTN:\n{:?}",p);

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
    */

    let file = include_str!("../test.ptn");
    let p = ptn::parse(file);
    if let Some(mut p) = p {
      let mut g = game::new(p.size).unwrap();
      let mut plies = 0;
      for m in p.moves.iter_mut() {
        for gen_move in g.moves() {
          let valid = g.validate(&gen_move);
          if valid != game::MoveValidity::Valid {
            panic!("Invalid move: {:?}", valid);
          }
        }

        if let Some(res) = g.status() {
          /*
          println!("white:\n{}\nblack:\n{}\ncaps:\n{}\nwalls:\n{}",
          g.c.format(g.white),
          g.c.format(g.black),
          g.c.format(g.caps),
          g.c.format(g.walls));
          */
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

      println!("Result: {:?}", g.status());

      println!("-----------------UNDOING-----------------");

      for m in p.moves.iter().rev() {
        println!("Move: {:?}", m.m);
        g.undo(&m.m);
        println!("Board: {}", g.to_string());
      }
    }
}
*/

#[cfg(test)]
mod tests {
  use test::Bencher;

  /*
  #[bench]
  fn test_ptn(b: &mut Bencher) {
    let file = include_str!("../test.ptn");
    b.iter(|| ::ptn::parse(file));
  }
  */

  #[bench]
  fn test_move(b: &mut Bencher) {
    b.iter(|| ::ptn::parse_move("3c3+12"));
  }
}
