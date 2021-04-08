
// use std::sync::Arc;
use std::rc::Rc;

pub type Ptr<T> = Rc<T>;

pub mod ops;
pub mod node;
pub mod expr;
pub mod item;
pub mod token;
pub mod debug;
pub mod symbol;

pub use expr::*;
pub use item::*;
pub use node::*;
