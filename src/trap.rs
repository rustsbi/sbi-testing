use core::{
    arch::asm,
    mem::{size_of, transmute},
    sync::atomic::{AtomicUsize, Ordering},
};
use riscv::register::scause::Trap;

static TRAP_CAUGHT: AtomicUsize = AtomicUsize::new(0);

#[repr(C)]
struct TrapFrame {
    ra: usize,
    tp: usize,
    t: [usize; 7],
    a: [usize; 8],
}

pub unsafe fn wait_interrupt() -> Trap {
    use riscv::{asm::wfi, register::stvec};

    stvec::write(trap_handler as _, stvec::TrapMode::Direct);
    loop {
        match TRAP_CAUGHT.swap(0, Ordering::AcqRel) {
            0 => wfi(),
            x => {
                use riscv::register::scause::Scause;
                let scause: Scause = transmute(x);
                return scause.cause();
            }
        }
    }
}

#[naked]
unsafe extern "C" fn trap_handler() {
    asm!(".align 2
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
        trap_handler =   sym trap_handler_rust,
        options(noreturn)
    );
}

#[inline(never)]
extern "C" fn trap_handler_rust() {
    use riscv::register::{
        scause::{self, Interrupt as I, Trap as T},
        sepc,
    };
    let cause = scause::read();
    match cause.cause() {
        T::Exception(_) => {
            sepc::write(sepc::read().wrapping_add(4));
        }
        T::Interrupt(I::SupervisorTimer) => {
            sbi_rt::set_timer(u64::MAX);
        }
        T::Interrupt(I::SupervisorSoft) => unsafe {
            asm!("csrw sip, zero");
        },
        _ => {}
    }
    TRAP_CAUGHT.store(unsafe { transmute(cause) }, Ordering::Release);
}
