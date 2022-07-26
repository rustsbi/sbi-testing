use riscv::{
    asm::wfi,
    register::{
        scause::{Interrupt, Trap},
        sie, sstatus, time,
    },
};

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
    if sbi::probe_extension(sbi::EID_TIME) == 0 {
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
    match unsafe { wait_interrupt() } {
        Trap::Interrupt(Interrupt::SupervisorTimer) => Ok(f(Case::SetTimer)),
        trap => Err(Fatel::UnexpectedTrap(trap)),
    }
}

unsafe fn wait_interrupt() -> Trap {
    crate::trap::set_stvec();
    sie::set_stimer();
    sstatus::set_sie();
    loop {
        if let Some(cause) = crate::trap::last_trap() {
            return cause;
        }
        wfi();
    }
}
