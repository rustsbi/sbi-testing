use crate::thread::Thread;
use riscv::register::scause::{self, Trap};

pub enum Case {
    NotExist,
    Begin,
    Interval { begin: u64, end: u64 },
    ReadFailed,
    TimeDecreased { a: u64, b: u64 },
    SetTimer,
    UnexpectedTrap(Trap),
    Pass,
}

pub fn test(delay: u64, mut f: impl FnMut(Case)) {
    use riscv::register::{scause::Interrupt, sie, time};

    if sbi::probe_extension(sbi::Timer).is_unavailable() {
        f(Case::NotExist);
        return;
    }
    f(Case::Begin);
    let begin: u64;
    let end: u64;
    let mut ok = 0xffusize;
    unsafe {
        core::arch::asm!(
            "   la   {stvec}, 1f
                csrw stvec,   {stvec}
                csrr {begin}, time
                csrr {end},   time
                mv   {ok},    zero
            .align 2
            1:
            ",
            stvec = out(reg) _,
            begin = out(reg) begin,
            end   = out(reg) end,
            ok    = inlateout(reg) ok,
        );
    }
    if ok != 0 {
        f(Case::ReadFailed);
        return;
    }
    if begin >= end {
        f(Case::TimeDecreased { a: begin, b: end });
        return;
    }
    f(Case::Interval { begin, end });

    let mut stack = [0usize; 32];
    let mut thread = Thread::new(riscv::asm::wfi as _);
    *thread.sp_mut() = stack.as_mut_ptr_range().end as _;

    sbi::set_timer(time::read64() + delay);
    unsafe {
        sie::set_stimer();
        thread.execute();
    }
    match scause::read().cause() {
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            sbi::set_timer(u64::MAX);
            f(Case::SetTimer);
            f(Case::Pass);
        }
        trap => {
            f(Case::UnexpectedTrap(trap));
        }
    }
}
