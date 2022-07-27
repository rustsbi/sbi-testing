use riscv::register::scause::Trap;

pub enum Case {
    Interval { begin: u64, end: u64 },
    SetTimer,
}

pub enum Fatel {
    NotExist,
    TimeDecreased { a: u64, b: u64 },
    UnexpectedTrap(Trap),
}

pub fn test(frequency: u64, f: impl Fn(Case) -> bool) -> Result<bool, Fatel> {
    use crate::trap::wait_interrupt;
    use riscv::register::{scause::Interrupt, sie, sstatus, time};

    if sbi::probe_extension(sbi::EID_TIME) {
        Err(Fatel::NotExist)?;
    }
    let begin = time::read64();
    let end = time::read64();
    if begin >= end {
        Err(Fatel::TimeDecreased { a: begin, b: end })?;
    }
    if !f(Case::Interval { begin, end }) {
        return Ok(false);
    }

    sbi::set_timer(time::read64() + frequency);
    let trap = unsafe {
        sie::set_stimer();
        sstatus::set_sie();
        wait_interrupt()
    };
    match trap {
        Trap::Interrupt(Interrupt::SupervisorTimer) => Ok(f(Case::SetTimer)),
        trap => Err(Fatel::UnexpectedTrap(trap)),
    }
}
