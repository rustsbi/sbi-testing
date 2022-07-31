use core::sync::atomic::{AtomicU32, Ordering};
use sbi::SbiRet;

pub enum Case {
    NotExist,
    Begin,
    NoSecondaryHart,
    HartStarted(usize),
    HartStartFailed { hartid: usize, ret: SbiRet },
    Pass,
}

pub fn test(primary_hart_id: usize, mut hart_mask: usize, hart_mask_base: usize, f: impl Fn(Case)) {
    // 不支持 HSM 扩展
    if !sbi::probe_extension(sbi::EID_HSM) {
        f(Case::NotExist);
        return;
    }
    f(Case::Begin);
    // 分批测试
    let mut batch = [0usize; TEST_BATCH_SIZE];
    let mut batch_count = 0;
    let mut batch_size = 0;
    let mut hartid = hart_mask_base;
    while hart_mask != 0 {
        if hartid != primary_hart_id {
            // 副核在测试前必须处于停止状态
            if sbi::hart_get_status(hartid) == STOPPED {
                batch[batch_size] = hartid;
                batch_size += 1;
                // 收集一个批次，执行测试
                if batch_size == TEST_BATCH_SIZE {
                    match test_batch(&batch) {
                        Ok(()) => {
                            batch_count += 1;
                            batch_size = 0;
                        }
                        Err((hartid, ret)) => {
                            f(Case::HartStartFailed { hartid, ret });
                            return;
                        }
                    }
                }
            }
            // 副核不在停止状态
            else {
                f(Case::HartStarted(hartid));
            }
        }
        let distance = hart_mask.trailing_zeros() + 1;
        hart_mask >>= distance;
        hartid += distance as usize;
    }
    // 为不满一批次的核执行测试
    if batch_size > 0 {
        match test_batch(&batch[..batch_size]) {
            Ok(()) => f(Case::Pass),
            Err((hartid, ret)) => f(Case::HartStartFailed { hartid, ret }),
        }
    }
    // 所有批次通过测试
    else if batch_count > 0 {
        f(Case::Pass);
    }
    // 没有找到能参与测试的副核
    else {
        f(Case::NoSecondaryHart)
    }
}

const STARTED: SbiRet = SbiRet {
    error: sbi::RET_SUCCESS,
    value: sbi::HART_STATE_STARTED,
};

const STOPPED: SbiRet = SbiRet {
    error: sbi::RET_SUCCESS,
    value: sbi::HART_STATE_STOPPED,
};

const SUSPENDED: SbiRet = SbiRet {
    error: sbi::RET_SUCCESS,
    value: sbi::HART_STATE_SUSPENDED,
};

const TEST_BATCH_SIZE: usize = 4;
static mut STACK: [ItemPerHart; TEST_BATCH_SIZE] = [ItemPerHart::ZERO; TEST_BATCH_SIZE];

#[repr(C, align(512))]
struct ItemPerHart {
    stage: AtomicU32,
    signal: AtomicU32,
    stack: [u8; 504],
}

const STAGE_IDLE: u32 = 0;
const STAGE_STARTED: u32 = 1;
const STAGE_RESUMED: u32 = 2;

impl ItemPerHart {
    const ZERO: Self = Self {
        stage: AtomicU32::new(STAGE_IDLE),
        signal: AtomicU32::new(0),
        stack: [0; 504],
    };

    fn reset(&mut self) -> *const ItemPerHart {
        self.stage.store(STAGE_IDLE, Ordering::Relaxed);
        self as _
    }

    fn wait_start(&self) {
        while self.stage.load(Ordering::Relaxed) != STAGE_STARTED {
            core::hint::spin_loop();
        }
    }

    fn wait_resume(&self) {
        while self.stage.load(Ordering::Relaxed) != STAGE_RESUMED {
            core::hint::spin_loop();
        }
    }

    fn send_signal(&self) {
        self.signal.store(1, Ordering::Release);
    }

    fn wait_signal(&self) {
        while self
            .signal
            .compare_exchange(1, 0, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
    }
}

/// 测试一批核
fn test_batch(batch: &[usize]) -> Result<(), (usize, SbiRet)> {
    // 初始这些核都是停止状态，测试 start
    for (i, hartid) in batch.iter().copied().enumerate() {
        let ptr = unsafe { STACK[i].reset() };
        let ret = sbi::hart_start(hartid, test_entry as _, ptr as _);
        if ret.error != sbi::RET_SUCCESS {
            return Err((hartid, ret));
        }
    }
    // 测试不可恢复休眠
    for (i, hartid) in batch.iter().copied().enumerate() {
        let item = unsafe { &mut STACK[i] };
        // 等待完成启动
        while sbi::hart_get_status(hartid) != STARTED {
            core::hint::spin_loop();
        }
        // 等待信号
        item.wait_start();
        // 发出信号
        item.send_signal();
        // 等待完成休眠
        while sbi::hart_get_status(hartid) != SUSPENDED {
            core::hint::spin_loop();
        }
    }
    // 全部唤醒
    let mut mask = 1usize;
    for hartid in &batch[1..] {
        mask |= 1 << (hartid - batch[0]);
    }
    sbi::send_ipi(mask, batch[0]);
    // 测试可恢复休眠
    for (i, hartid) in batch.iter().copied().enumerate() {
        let item = unsafe { &mut STACK[i] };
        // 等待完成恢复
        while sbi::hart_get_status(hartid) != STARTED {
            core::hint::spin_loop();
        }
        // 等待信号
        item.wait_resume();
        // 发出信号
        item.send_signal();
        // 等待完成休眠
        while sbi::hart_get_status(hartid) != SUSPENDED {
            core::hint::spin_loop();
        }
        // 单独恢复
        for _ in 0..0x1000 {
            core::hint::spin_loop();
        }
        sbi::send_ipi(1, hartid);
        // 等待关闭
        while sbi::hart_get_status(hartid) != STOPPED {
            core::hint::spin_loop();
        }
    }
    Ok(())
}

/// 测试用启动入口
#[naked]
unsafe extern "C" fn test_entry(hartid: usize, opaque: *mut ItemPerHart) -> ! {
    core::arch::asm!(
        "csrw sie, zero",   // 关中断
        "call {set_stack}", // 设置栈
        "j    {rust_main}", // 进入 rust
        set_stack = sym set_stack,
        rust_main = sym rust_main,
        options(noreturn),
    )
}

#[naked]
unsafe extern "C" fn set_stack(hart_id: usize, ptr: *const ItemPerHart) {
    core::arch::asm!("addi sp, a1, 512", "ret", options(noreturn));
}

#[inline(never)]
extern "C" fn rust_main(hart_id: usize, opaque: *mut ItemPerHart) -> ! {
    let item = unsafe { &mut *opaque };
    match item.stage.compare_exchange(
        STAGE_IDLE,
        STAGE_STARTED,
        Ordering::AcqRel,
        Ordering::Acquire,
    ) {
        Ok(_) => {
            item.wait_signal();
            let ret = sbi::hart_suspend(
                sbi::HART_SUSPEND_TYPE_NON_RETENTIVE,
                test_entry as _,
                opaque as _,
            );
            unreachable!("suspend [{hart_id}] but {ret:?}")
        }
        Err(STAGE_STARTED) => {
            item.stage.store(STAGE_RESUMED, Ordering::Release);
            item.wait_signal();
            let _ = sbi::hart_suspend(
                sbi::HART_SUSPEND_TYPE_RETENTIVE,
                test_entry as _,
                opaque as _,
            );
            let ret = sbi::hart_stop();
            unreachable!("suspend [{hart_id}] but {ret:?}")
        }
        Err(_) => unreachable!(),
    }
}
