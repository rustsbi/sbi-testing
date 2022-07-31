use riscv::register::scause::Trap;

pub enum Case {
    NotExist,
    Begin,
    Interval { begin: u64, end: u64 },
    TimeDecreased { a: u64, b: u64 },
    SetTimer,
    UnexpectedTrap(Trap),
    Pass,
}

pub fn test(delay: u64, f: impl Fn(Case)) {
    use crate::trap::wait_interrupt;
    use riscv::register::{scause::Interrupt, sie, sstatus, time};

    if !sbi::probe_extension(sbi::EID_TIME) {
        f(Case::NotExist);
        return;
    }
    f(Case::Begin);
    let begin = time::read64();
    let end = time::read64();
    if begin >= end {
        f(Case::TimeDecreased { a: begin, b: end });
        return;
    }
    f(Case::Interval { begin, end });

    sbi::set_timer(time::read64() + delay);
    let trap = unsafe {
        sie::set_stimer();
        sstatus::set_sie();
        wait_interrupt()
    };
    match trap {
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            f(Case::SetTimer);
            f(Case::Pass);
        }
        trap => {
            f(Case::UnexpectedTrap(trap));
        }
    }
}
