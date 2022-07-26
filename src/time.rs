use core::{
    mem::transmute,
    sync::atomic::{AtomicUsize, Ordering},
};
use riscv::{
    asm::wfi,
    register::{
        scause::{self, Interrupt, Scause, Trap},
        sepc, sie, sstatus, stvec, time,
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

static TRAP_CAUGHT: AtomicUsize = AtomicUsize::new(0);

unsafe fn wait_interrupt() -> Trap {
    stvec::write(time_interrupt_handler as _, stvec::TrapMode::Direct);
    sie::set_stimer();
    sstatus::set_sie();
    loop {
        let cause = TRAP_CAUGHT.load(Ordering::Acquire);
        if cause != 0 {
            let cause: Scause = transmute(cause);
            return cause.cause();
        }
        wfi();
    }
}

#[repr(C)]
struct TrapFrame {
    ra: usize,
    tp: usize,
    t: [usize; 7],
    a: [usize; 8],
}

#[naked]
unsafe extern "C" fn time_interrupt_handler() {
    use core::mem::size_of;
    core::arch::asm!(
        "   .align 2
            addi sp, sp, -1*{frame_size}
            sd   ra,  0*{usize_size}(sp)
            sd   tp,  1*{usize_size}(sp)

            sd   t0,  2*{usize_size}(sp)
            sd   t1,  3*{usize_size}(sp)
            sd   t2,  4*{usize_size}(sp)
            sd   t3,  5*{usize_size}(sp)
            sd   t4,  6*{usize_size}(sp)
            sd   t5,  7*{usize_size}(sp)
            sd   t6,  8*{usize_size}(sp)

            sd   a0,  9*{usize_size}(sp)
            sd   a1, 10*{usize_size}(sp)
            sd   a2, 11*{usize_size}(sp)
            sd   a3, 12*{usize_size}(sp)
            sd   a4, 13*{usize_size}(sp)
            sd   a5, 14*{usize_size}(sp)
            sd   a6, 15*{usize_size}(sp)
            sd   a7, 16*{usize_size}(sp)

            mv   a0, sp
            call {trap_handler}

            ld   ra,  0*{usize_size}(sp)
            ld   tp,  1*{usize_size}(sp)

            ld   t0,  2*{usize_size}(sp)
            ld   t1,  3*{usize_size}(sp)
            ld   t2,  4*{usize_size}(sp)
            ld   t3,  5*{usize_size}(sp)
            ld   t4,  6*{usize_size}(sp)
            ld   t5,  7*{usize_size}(sp)
            ld   t6,  8*{usize_size}(sp)

            ld   a0,  9*{usize_size}(sp)
            ld   a1, 10*{usize_size}(sp)
            ld   a2, 11*{usize_size}(sp)
            ld   a3, 12*{usize_size}(sp)
            ld   a4, 13*{usize_size}(sp)
            ld   a5, 14*{usize_size}(sp)
            ld   a6, 15*{usize_size}(sp)
            ld   a7, 16*{usize_size}(sp)

            addi sp, sp, {frame_size}
            sret
        ",
        frame_size   = const size_of::<TrapFrame>(),
        usize_size   = const size_of::<usize>(),
        trap_handler =   sym trap_handler,
        options(noreturn)
    );
}

#[inline(never)]
extern "C" fn trap_handler() {
    let cause = scause::read();
    match cause.cause() {
        Trap::Exception(_) => {
            sepc::write(sepc::read().wrapping_add(4));
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            sbi_rt::set_timer(u64::MAX);
        }
        _ => {}
    }
    TRAP_CAUGHT.store(unsafe { transmute(cause) }, Ordering::Release);
}
