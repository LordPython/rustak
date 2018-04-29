mod game;
pub use self::game::Game;

mod stack;
pub use self::stack::*;

#[cfg(feature = "stack_128")]
type StackRepr = u128;

#[cfg(not(feature = "stack_128"))]
type StackRepr = u64;

trait BinConv {
  fn binary(&self) -> StackRepr;
  fn from_binary(bin: StackRepr) -> Self;
}

use ::game::Player;
impl BinConv for Player {
  fn binary(&self) -> StackRepr { match *self { Player::White => 0, Player::Black => 1 } }
  fn from_binary(bin: StackRepr) -> Self { match bin { 0 => Player::White, _ => Player::Black } }
}