use sbi::SbiRet;

pub enum Case {
    HartStarted(usize),
}

pub enum Fatel {
    NotExist,
    NoSecondaryHart,
    HartStartFailed { hartid: usize, ret: SbiRet },
}

pub fn test(
    primary_hart_id: usize,
    mut hart_mask: usize,
    hart_mask_base: usize,
    f: impl Fn(Case) -> bool,
) -> Result<bool, Fatel> {
    // 不支持 HSM 扩展
    if !sbi::probe_extension(sbi::EID_HSM) {
        Err(Fatel::NotExist)?;
    }
    // 常量
    const MAX_HART_COUNT: usize = 8;
    const STARTED: SbiRet = SbiRet {
        error: sbi::RET_SUCCESS,
        value: sbi::HART_STATE_STARTED,
    };
    const STOPPED: SbiRet = SbiRet {
        error: sbi::RET_SUCCESS,
        value: sbi::HART_STATE_STOPPED,
    };
    const STACK_SIZE: usize = 512;
    static mut STACK: [[u8; STACK_SIZE]; MAX_HART_COUNT] = [[0u8; STACK_SIZE]; MAX_HART_COUNT];
    // 找到所有参与测试的副核
    let mut harts = [0usize; MAX_HART_COUNT];
    let mut hart_count = 0;
    for i in 0..usize::BITS as usize {
        if hart_mask & 1 == 1 {
            let hartid = hart_mask_base + i;
            if hartid != primary_hart_id {
                // 副核在测试前必须处于停止状态
                if sbi::hart_get_status(hartid) != STOPPED {
                    harts[hart_count] = hartid;
                    hart_count += 1;
                    // 名额已满
                    if hart_count == MAX_HART_COUNT {
                        break;
                    }
                }
                // 副核不在停止状态
                else if !f(Case::HartStarted(hartid)) {
                    return Ok(false);
                }
            }
        }
        hart_mask >>= 1;
    }
    // 没有找到能参与测试的副核
    if hart_count == 0 {
        Err(Fatel::NoSecondaryHart)?;
    }
    // 启动副核
    for (i, hartid) in harts[..hart_count].iter().copied().enumerate() {
        let opaque = Opaque {
            entry: start_rust_main as _,
            stack: unsafe { STACK[i] }.as_ptr() as _,
        };
        let ret = sbi::hart_start(hartid, test_entry as _, &opaque as *const _ as _);
        if ret.error != sbi::RET_SUCCESS {
            Err(Fatel::HartStartFailed { hartid, ret })?;
        }
        while sbi::hart_get_status(hartid) != STARTED {
            core::hint::spin_loop();
        }
    }

    todo!()
}

#[repr(C)]
struct Opaque {
    entry: usize,
    stack: usize,
}

#[naked]
unsafe extern "C" fn test_entry(hartid: usize, opaque: *const Opaque) -> ! {
    core::arch::asm!(
        "csrw sie, zero",  // 关中断
        "ld   sp,  8(a1)", // 设置栈
        "ld   a1,  0(a1)", // 设置入口
        "jr   a1",         // 进入 rust
        options(noreturn),
    )
}

extern "C" fn start_rust_main(hart_id: usize) -> ! {
    //     STARTED.wait().wait();
    let ret = sbi::hart_suspend(
        sbi::HART_SUSPEND_TYPE_NON_RETENTIVE,
        test_entry as _,
        resume_rust_main as _,
    );
    unreachable!("suspend [{hart_id}] but {ret:?}");
}

extern "C" fn resume_rust_main(hart_id: usize) -> ! {
    // unreachable!("suspend [{hart_id}] but {ret:?}");
    todo!()
}
