//! RISC-V Supervisor Binary Interface test suite

#![no_std]
#![deny(missing_docs)]
#![feature(naked_functions, asm_sym, asm_const)]

mod thread;

pub extern crate sbi_rt as sbi;

#[cfg(feature = "log")]
pub extern crate log_crate as log;

#[cfg(feature = "log")]
mod log_test;

#[cfg(feature = "log")]
pub use log_test::Testing;

// §4
pub mod base;
// §6
pub mod time;
// §7
pub mod spi;
// §8
// pub mod rfnc;
// §9
pub mod hsm;
// §10
// pub mod srst;
// §11
// pub mod pmu;
