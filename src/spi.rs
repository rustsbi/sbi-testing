use riscv::register::scause::Trap;

pub struct SendIpi;
pub type Case = SendIpi;

pub enum Fatel {
    NotExist,
    UnexpectedTrap(Trap),
}

pub fn test(hartid: usize, f: impl Fn(Case) -> bool) -> Result<bool, Fatel> {
    use crate::trap::wait_interrupt;
    use riscv::register::{scause::Interrupt, sie, sstatus};

    if sbi::probe_extension(sbi::EID_TIME) == 0 {
        Err(Fatel::NotExist)?;
    }

    let trap = unsafe {
        sie::set_ssoft();
        sstatus::set_sie();
        sbi::send_ipi(1 << hartid, 0);
        wait_interrupt()
    };
    match trap {
        Trap::Interrupt(Interrupt::SupervisorSoft) => Ok(f(SendIpi)),
        trap => Err(Fatel::UnexpectedTrap(trap)),
    }
}
