use sbi::{ExtensionInfo, Version};
use sbi_spec::base::impl_id;

pub enum Case {
    NotExist,
    Begin,
    Pass,
    GetSbiSpecVersion(Version),
    GetSbiImplId(Result<&'static str, usize>),
    GetSbiImplVersion(usize),
    ProbeExtensions(Extensions),
    GetMVendorId(usize),
    GetMArchId(usize),
    GetMimpId(usize),
}

pub struct Extensions {
    pub time: ExtensionInfo,
    pub spi: ExtensionInfo,
    pub rfnc: ExtensionInfo,
    pub hsm: ExtensionInfo,
    pub srst: ExtensionInfo,
    pub pmu: ExtensionInfo,
}

impl core::fmt::Display for Extensions {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[Base")?;
        if self.time.is_available() {
            write!(f, ", TIME")?;
        }
        if self.spi.is_available() {
            write!(f, ", sPI")?;
        }
        if self.rfnc.is_available() {
            write!(f, ", RFNC")?;
        }
        if self.hsm.is_available() {
            write!(f, ", HSM")?;
        }
        if self.srst.is_available() {
            write!(f, ", SRST")?;
        }
        if self.pmu.is_available() {
            write!(f, ", PMU")?;
        }
        write!(f, "]")
    }
}

pub fn test(mut f: impl FnMut(Case)) {
    if sbi::probe_extension(sbi::Base).is_unavailable() {
        f(Case::NotExist);
        return;
    }
    f(Case::Begin);
    f(Case::GetSbiSpecVersion(sbi::get_spec_version()));
    f(Case::GetSbiImplId(match sbi::get_sbi_impl_id() {
        impl_id::BBL => Ok("BBL"),
        impl_id::OPEN_SBI => Ok("OpenSBI"),
        impl_id::XVISOR => Ok("Xvisor"),
        impl_id::KVM => Ok("KVM"),
        impl_id::RUST_SBI => Ok("RustSBI"),
        impl_id::DIOSIX => Ok("Diosix"),
        impl_id::COFFER => Ok("Coffer"),
        unknown => Err(unknown),
    }));
    f(Case::GetSbiImplVersion(sbi::get_sbi_impl_version()));
    f(Case::ProbeExtensions(Extensions {
        time: sbi::probe_extension(sbi::Timer),
        spi: sbi::probe_extension(sbi::Ipi),
        rfnc: sbi::probe_extension(sbi::Fence),
        hsm: sbi::probe_extension(sbi::Hsm),
        srst: sbi::probe_extension(sbi::Reset),
        pmu: sbi::probe_extension(sbi::Pmu),
    }));
    f(Case::GetMVendorId(sbi::get_mvendorid()));
    f(Case::GetMArchId(sbi::get_marchid()));
    f(Case::GetMimpId(sbi::get_mimpid()));
    f(Case::Pass);
}
