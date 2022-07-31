use crate::trap::wait_interrupt;
use riscv::register::scause::Trap;
use riscv::register::{scause::Interrupt, sie, sstatus};

pub enum Case {
    NotExist,
    Begin,
    SendIpi,
    UnexpectedTrap(Trap),
    Pass,
}

pub fn test(hart_id: usize, f: impl Fn(Case)) {
    if !sbi::probe_extension(sbi::EID_TIME) {
        f(Case::NotExist);
        return;
    }

    f(Case::Begin);
    let trap = unsafe {
        sie::set_ssoft();
        sstatus::set_sie();
        sbi::send_ipi(1 << hart_id, 0);
        wait_interrupt()
    };
    match trap {
        Trap::Interrupt(Interrupt::SupervisorSoft) => {
            f(Case::SendIpi);
            f(Case::Pass);
        }
        trap => {
            f(Case::UnexpectedTrap(trap));
        }
    }
}
