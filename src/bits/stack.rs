use game::Player;
use std::iter::{FromIterator,IntoIterator};
use std::hash::{Hash,Hasher};
use bits::BinConv;

#[derive(Debug,Clone,Copy)]
pub struct Stack {
  owners: ::bits::StackRepr,
  size: u8,
}

impl Stack {
  pub fn new() -> Self {
    Stack { owners: 0, size: 0 }
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.size as usize
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.size == 0
  }

  #[inline]
  pub fn get(&self, index: usize) -> Player {
    Player::from_binary((self.owners >> index) & 1)
  }

  #[inline]
  pub fn pop_stack(&mut self, n: u8) -> Stack {
    let o = self.owners & ((1<<n)-1);
    self.owners >>= n;
    self.size -= n;
    Stack { owners: o, size: n }
  }

  #[inline]
  pub fn push_stack(&mut self, s: Stack) {
    self.owners = (self.owners << s.size) | s.owners;
    self.size += s.size;
  }

  #[inline]
  pub fn push(&mut self, p: Player) {
    self.owners = (self.owners << 1) | p.binary();
    self.size += 1;
  }
}

impl Default for Stack {
  fn default() -> Self {
    Self::new()
  }
}

impl Hash for Stack {
  #[inline]
  fn hash<H: Hasher>(&self, hasher: &mut H) {
    #[cfg(feature="stack_128")]
    hasher.write_u128(self.owners);
    #[cfg(not(feature="stack_128"))]
    hasher.write_u64(self.owners);

    hasher.write_u8(self.size);
  }
}

impl FromIterator<Player> for Stack {
  fn from_iter<T>(iter: T) -> Self
    where T: IntoIterator<Item=Player>
  {
    let mut s = Stack::new();
    for p in iter {
      s.push(p);
    }
    s
  }
}

pub struct StackIter<'a> {
  front: usize,
  back: usize,
  stack: &'a Stack,
}

impl<'a> Iterator for StackIter<'a> {
  type Item = Player;
  fn next(&mut self) -> Option<Player> {
    if self.front < self.back {
      let res = Some(self.stack.get(self.front));
      self.front += 1;
      res
    } else {
      None
    }
  }
}

impl<'a> DoubleEndedIterator for StackIter<'a> {
  fn next_back(&mut self) -> Option<Player> {
    if self.front < self.back {
      self.back -= 1;
      Some(self.stack.get(self.back))
    } else {
      None
    }
  }
}

impl<'a> IntoIterator for &'a Stack {
  type Item = Player;
  type IntoIter = StackIter<'a>;
  fn into_iter(self) -> StackIter<'a> {
    StackIter { front: 0, back: self.len(), stack: self }
  }
}
