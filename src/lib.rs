#![deny(unsafe_code)]
#![allow(dead_code)]
#![doc = include_str!("../README.md")]

pub use crate::crdt::*;

mod crdt;
