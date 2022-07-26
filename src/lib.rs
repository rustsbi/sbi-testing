#![no_std]
#![deny(warnings)]
#![feature(naked_functions, asm_sym, asm_const)]

pub extern crate sbi_rt as sbi;

// §4
pub mod base;
// §6
pub mod time;

pub enum Extension {
    Base,
    Time,
}

pub enum Case {
    Begin(Extension),
    End(Extension),
    Base(base::Case),
    BaseFatel(base::Fatel),
    Time(time::Case),
    TimeFatel(time::Fatel),
}

pub fn test(frequency: u64, f: impl Fn(Case) -> bool) -> bool {
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

    if !f(Case::Begin(Extension::Time)) {
        return false;
    }
    match time::test(frequency, |case| f(Case::Time(case))) {
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
    true
}
