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

pub enum Extension {
    Base,
    Time,
    Spi,
}

pub enum Case {
    Begin(Extension),
    End(Extension),
    Base(base::Case),
    BaseFatel(base::Fatel),
    Time(time::Case),
    TimeFatel(time::Fatel),
    Spi(spi::Case),
    SpiFatel(spi::Fatel),
}

pub fn test(hartid: usize, delay: u64, f: impl Fn(Case) -> bool) -> bool {
    // base =====================================================
    if !f(Case::Begin(Extension::Base)) {
        return false;
    }
    match base::test(|case| f(Case::Base(case))) {
        Ok(true) => {}
        Ok(false) => return false,
        Err(fatel) => {
            if !f(Case::BaseFatel(fatel)) {
                return false;
            }
        }
    }
    if !f(Case::End(Extension::Base)) {
        return false;
    }
    // time =====================================================
    if !f(Case::Begin(Extension::Time)) {
        return false;
    }
    match time::test(delay, |case| f(Case::Time(case))) {
        Ok(true) => {}
        Ok(false) => return false,
        Err(fatel) => {
            if !f(Case::TimeFatel(fatel)) {
                return false;
            }
        }
    }
    if !f(Case::End(Extension::Time)) {
        return false;
    }
    // spi ======================================================
    if !f(Case::Begin(Extension::Spi)) {
        return false;
    }
    match spi::test(hartid, |case| f(Case::Spi(case))) {
        Ok(true) => {}
        Ok(false) => return false,
        Err(fatel) => {
            if !f(Case::SpiFatel(fatel)) {
                return false;
            }
        }
    }
    if !f(Case::End(Extension::Spi)) {
        return false;
    }
    // finish ====================================================
    true
}
