#![no_std]
#![deny(warnings)]
#![feature(naked_functions, asm_sym, asm_const)]

pub extern crate sbi_rt as sbi;

mod trap;

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
