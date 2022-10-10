use crate::{base, hsm, spi, time};
use log_crate::*;

/// Automatic SBI testing with logging enabled
pub struct Testing {
    /// The hart ID to test most of single core extensions
    pub hartid: usize,
    /// A list of harts to test Hart State Monitor extension
    pub hart_mask: usize,
    /// Base of hart list to test Hart State Monitor extension
    pub hart_mask_base: usize,
    /// Delay value to test Timer programmer extension
    pub delay: u64,
}

const TARGET: &str = "sbi-testing";

impl Testing {
    /// Start testing process of RISC-V SBI implementation
    pub fn test(self) -> bool {
        let mut result = true;
        base::test(|case| {
            use base::Case::*;
            match case {
                NotExist => panic!("Sbi Base Not Exist"),
                Begin => info!(target: TARGET, "Testing Base"),
                Pass => info!(target: TARGET, "Sbi Base Test Pass"),
                GetSbiSpecVersion(version) => {
                    info!(target: TARGET, "sbi spec version = {version}");
                }
                GetSbiImplId(Ok(name)) => {
                    info!(target: TARGET, "sbi impl = {name}");
                }
                GetSbiImplId(Err(unknown)) => {
                    warn!(target: TARGET, "unknown sbi impl = {unknown:#x}");
                }
                GetSbiImplVersion(version) => {
                    info!(target: TARGET, "sbi impl version = {version:#x}");
                }
                ProbeExtensions(exts) => {
                    info!(target: TARGET, "sbi extensions = {exts}");
                }
                GetMVendorId(id) => {
                    info!(target: TARGET, "mvendor id = {id:#x}");
                }
                GetMArchId(id) => {
                    info!(target: TARGET, "march id = {id:#x}");
                }
                GetMimpId(id) => {
                    info!(target: TARGET, "mimp id = {id:#x}");
                }
            }
        });
        time::test(self.delay, |case| {
            use time::Case::*;
            match case {
                NotExist => {
                    error!(target: TARGET, "Sbi TIME Not Exist");
                    result = false;
                }
                Begin => info!(target: TARGET, "Testing TIME"),
                Pass => info!(target: TARGET, "Sbi TIME Test Pass"),
                Interval { begin: _, end: _ } => {
                    info!(
                        target: TARGET,
                        "read time register successfuly, set timer +1s"
                    );
                }
                ReadFailed => {
                    error!(target: TARGET, "csrr time failed");
                    result = false;
                }
                TimeDecreased { a, b } => {
                    error!(target: TARGET, "time decreased: {a} -> {b}");
                    result = false;
                }
                SetTimer => {
                    info!(target: TARGET, "timer interrupt delegate successfuly");
                }
                UnexpectedTrap(trap) => {
                    error!(
                        target: TARGET,
                        "expect trap at supervisor timer, but {trap:?} was caught"
                    );
                    result = false;
                }
            }
        });
        spi::test(self.hartid, |case| {
            use spi::Case::*;
            match case {
                NotExist => {
                    error!(target: TARGET, "Sbi sPI Not Exist");
                    result = false;
                }
                Begin => info!(target: TARGET, "Testing sPI"),
                Pass => info!(target: TARGET, "Sbi sPI Test Pass"),
                SendIpi => info!(target: TARGET, "send ipi successfuly"),
                UnexpectedTrap(trap) => {
                    error!(
                        target: TARGET,
                        "expect trap at supervisor soft, but {trap:?} was caught"
                    );
                    result = false;
                }
            }
        });
        hsm::test(self.hartid, self.hart_mask, self.hart_mask_base, |case| {
            use hsm::Case::*;
            match case {
                NotExist => {
                    error!(target: TARGET, "Sbi HSM Not Exist");
                    result = false;
                }
                Begin => info!(target: TARGET, "Testing HSM"),
                Pass => info!(target: TARGET, "Sbi HSM Test Pass"),
                HartStartedBeforeTest(id) => warn!(target: TARGET, "hart {id} already started"),
                NoStoppedHart => warn!(target: TARGET, "no stopped hart"),
                BatchBegin(batch) => info!(target: TARGET, "Testing harts: {batch:?}"),
                HartStarted(id) => debug!(target: TARGET, "hart {id} started"),
                HartStartFailed { hartid, ret } => {
                    error!(target: TARGET, "hart {hartid} start failed: {ret:?}");
                    result = false;
                }
                HartSuspendedNonretentive(id) => {
                    debug!(target: TARGET, "hart {id} suspended nonretentive")
                }
                HartResumed(id) => debug!(target: TARGET, "hart {id} resumed"),
                HartSuspendedRetentive(id) => {
                    debug!(target: TARGET, "hart {id} suspended retentive")
                }
                HartStopped(id) => debug!(target: TARGET, "hart {id} stopped"),
                BatchPass(batch) => info!(target: TARGET, "Testing Pass: {batch:?}"),
            }
        });
        result
    }
}
