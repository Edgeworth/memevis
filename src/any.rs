use std::fmt;
use std::hash::Hash;

use derive_more::{Display, From, Into};
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Copy,
    Clone,
    Display,
    From,
    Into,
    Serialize,
    Deserialize,
)]
pub struct Any;

pub trait Basic = Copy + Clone + Ord + PartialOrd + Eq + PartialEq + Hash + fmt::Debug + Default;
