﻿use crate::thread::Thread;
use riscv::register::{
    scause::Interrupt,
    scause::{self, Trap},
    sie,
};

pub enum Case {
    NotExist,
    Begin,
    SendIpi,
    UnexpectedTrap(Trap),
    Pass,
}

pub fn test(hart_id: usize, mut f: impl FnMut(Case)) {
    if sbi::probe_extension(sbi::Timer).is_unavailable() {
        f(Case::NotExist);
        return;
    }

    fn ipi(hart_id: usize) -> ! {
        sbi::send_ipi(1 << hart_id, 0);
        // 必须立即触发中断，即使是一个指令的延迟，也会触发另一个异常
        unsafe { core::arch::asm!("unimp", options(noreturn, nomem)) };
    }

    f(Case::Begin);
    let mut stack = [0usize; 32];
    let mut thread = Thread::new(ipi as _);
    *thread.sp_mut() = stack.as_mut_ptr_range().end as _;
    *thread.a_mut(0) = hart_id;
    unsafe {
        sie::set_ssoft();
        thread.execute();
    }
    match scause::read().cause() {
        Trap::Interrupt(Interrupt::SupervisorSoft) => {
            f(Case::SendIpi);
            f(Case::Pass);
        }
        trap => {
            f(Case::UnexpectedTrap(trap));
        }
    }
}
