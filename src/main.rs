#[cfg(test)]
#[macro_use]
extern crate hex_literal;

pub mod types;
pub mod bitcoin;
pub mod manifoldchain;
pub mod optchain;
pub mod tests;

use crate::{
    // bitcoin::start as bitcoin_start,
    // manifoldchain::start as manifoldchain_start,
    optchain::start as optchain_start,
};

fn main() {
    //run_bitcoin();
    // run_manifoldchain();
    optchain_start();

}
