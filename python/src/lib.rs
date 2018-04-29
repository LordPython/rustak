#![feature(proc_macro, specialization, const_fn)]

extern crate rustak;
extern crate pyo3;

use pyo3::prelude::*;
use std::panic;


#[py::modinit(pyrustak)]
fn mod_init(_: Python, m: &PyModule) -> PyResult<()> {
  m.add_class::<Game>()?;
  m.add_class::<Move>()?;

  Ok(())
}

#[py::class]
struct Move {
  m: rustak::game::Move,
  t: PyToken
}

#[py::methods]
impl Move {
  #[new]
  fn __new__(obj: &PyRawObject, ptn: &str) -> PyResult<()> {
    panic::catch_unwind(|| {
      rustak::ptn::parse_move(ptn).map_or(
        Err(exc::Exception::new("error parsing move")),
        |m| { obj.init(|t| Move { m, t }) }
      )
    }).unwrap_or(Err(exc::Exception::new("unexpected panic")))
  }
}

#[py::proto]
impl PyObjectProtocol for Move {
  fn __repr__(&self) -> PyResult<String> {
    Ok(rustak::ptn::to_string(&self.m))
  }

  fn __richcmp__(&self, other: &Move, op: CompareOp) -> PyResult<bool> {
    match op {
      CompareOp::Eq => Ok(self.m == other.m),
      CompareOp::Ne => Ok(self.m != other.m),
      _ => Err(exc::NotImplementedError.into()),
    }

  }

  fn __hash__(&self) -> PyResult<isize> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    self.m.hash(&mut hasher);
    Ok(hasher.finish() as isize)
  }
}

#[py::class]
struct Game {
  game: rustak::bits::Game,
}

#[py::methods]
impl Game {
  #[new]
  fn __new__(obj: &PyRawObject, size: usize) -> PyResult<()> {
    panic::catch_unwind(move || {
      rustak::game::new(size).map_or(
        Err(exc::Exception::new(format!("Invalid board size {}", size))),
        |game| {
          obj.init(|_| Game { game })
        }
      )
    }).unwrap_or(Err(exc::Exception::new("unexpected panic")))
  }

  fn execute(&mut self, m: &mut Move) -> PyResult<()> {
    panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
      match self.game.validate(&m.m) {
        rustak::game::MoveValidity::Valid => {
          self.game.execute(&mut m.m);
          Ok(())
        },
        val => Err(exc::Exception::new(format!("{:?}", val)))
      }
    })).unwrap_or(Err(exc::Exception::new("unexpected panic")))
  }

  fn moves(&self) -> PyResult<Py<PyList>> {
    panic::catch_unwind(move || {
      let gil = Python::acquire_gil();
      let py = gil.python();
      let list = PyList::new::<()>(py, &[]);
      self.game.foreach_move(|m| -> PyResult<()> {
        let py_move = py.init(|t| Move{m,t})?;
        list.append(py_move)?;
        Ok(())
      })?;
      Ok(list.into())
    }).unwrap_or(Err(exc::Exception::new("unexpected panic")))
  }
}

#[py::proto]
impl PyObjectProtocol for Game {
  fn __repr__(&self) -> PyResult<String> {
    Ok(self.game.to_string())
  }

  fn __hash__(&self) -> PyResult<isize> {
    Ok(rustak::game::Game::hash(&self.game) as isize)
  }
}
