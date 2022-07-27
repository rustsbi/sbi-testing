#![no_std]
// #![deny(warnings)]
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

pub enum Extension {
    Base,
    Time,
    Spi,
    Hsm,
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
    Hsm(hsm::Case),
    HsmFatel(hsm::Fatel),
}

pub struct Testing {
    pub hart_id: usize,
    pub hart_mask: usize,
    pub hart_mask_base: usize,
    pub delay: u64,
}

impl Testing {
    pub fn test(self, f: impl Fn(Case) -> bool) -> bool {
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
        match time::test(self.delay, |case| f(Case::Time(case))) {
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
        match spi::test(self.hart_id, |case| f(Case::Spi(case))) {
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
        // hsm ======================================================
        if !f(Case::Begin(Extension::Hsm)) {
            return false;
        }
        match hsm::test(self.hart_id, self.hart_mask, self.hart_mask_base, |case| {
            f(Case::Hsm(case))
        }) {
            Ok(true) => {}
            Ok(false) => return false,
            Err(fatel) => {
                if !f(Case::HsmFatel(fatel)) {
                    return false;
                }
            }
        }
        if !f(Case::End(Extension::Hsm)) {
            return false;
        }
        // finish ====================================================
        true
    }
}
