#[macro_use]
extern crate nom;
extern crate time;

mod game;
mod ptn;
mod tps;

fn main() {
    println!("Hello, world!");

    let m = ptn::parse_move(&b"3a3<12"[..]);
    println!("{:?}", m);
    let tag = ptn::ptn_tag(&b"[testing \"dlkjlk\"]"[..]);
    println!("{:?}", tag);
}
