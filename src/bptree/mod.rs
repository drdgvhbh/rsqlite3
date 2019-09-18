use serde::ser::Serialize;
use std::cmp::Ord;
use std::fmt::{Debug, Display};
use std::hash::Hash;

mod bp_tree;
mod bp_tree_node;
mod entry;
mod serializer;

pub trait Key = Hash + Serialize + Eq + Ord + Display + Debug + Clone + Display;
pub trait Value = Serialize + Eq + Debug + Clone;

pub use bp_tree::BPTree;
pub use entry::Entry;
pub use serializer::Serializer;
