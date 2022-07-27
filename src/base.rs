use sbi_rt::SbiSpecVersion;

pub enum Case {
    GetSbiSpecVersion(SbiSpecVersion),
    GetSbiImplId(Result<&'static str, usize>),
    GetSbiImplVersion(usize),
    ProbeExtensions(Extensions),
    GetMVendorId(usize),
    GetMArchId(usize),
    GetMimpId(usize),
}

pub struct Extensions {
    pub time: bool,
    pub spi: bool,
    pub rfnc: bool,
    pub hsm: bool,
    pub srst: bool,
    pub pmu: bool,
}

impl core::fmt::Display for Extensions {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[Base")?;
        if self.time {
            write!(f, ", TIME")?;
        }
        if self.spi {
            write!(f, ", sPI")?;
        }
        if self.rfnc {
            write!(f, ", RFNC")?;
        }
        if self.hsm {
            write!(f, ", HSM")?;
        }
        if self.srst {
            write!(f, ", SRST")?;
        }
        if self.pmu {
            write!(f, ", PMU")?;
        }
        write!(f, "]")
    }
}

pub struct NotExist;

pub type Fatel = NotExist;

pub fn test(f: impl Fn(Case) -> bool) -> Result<bool, Fatel> {
    if !sbi::probe_extension(sbi::EID_BASE) {
        Err(NotExist)?;
    }
    let pass = f(Case::GetSbiSpecVersion(sbi::get_spec_version()))
        && f(Case::GetSbiImplId(match sbi::get_sbi_impl_id() {
            sbi::impl_id::BBL => Ok("BBL"),
            sbi::impl_id::OPEN_SBI => Ok("OpenSBI"),
            sbi::impl_id::XVISOR => Ok("Xvisor"),
            sbi::impl_id::KVM => Ok("KVM"),
            sbi::impl_id::RUST_SBI => Ok("RustSBI"),
            sbi::impl_id::DIOSIX => Ok("Diosix"),
            sbi::impl_id::COFFER => Ok("Coffer"),
            unknown => Err(unknown),
        }))
        && f(Case::GetSbiImplVersion(sbi::get_sbi_impl_version()))
        && f(Case::ProbeExtensions(Extensions {
            time: sbi::probe_extension(sbi::EID_TIME),
            spi: sbi::probe_extension(sbi::EID_SPI),
            rfnc: sbi::probe_extension(sbi::EID_RFNC),
            hsm: sbi::probe_extension(sbi::EID_HSM),
            srst: sbi::probe_extension(sbi::EID_SRST),
            pmu: sbi::probe_extension(sbi::EID_PMU),
        }))
        && f(Case::GetMVendorId(sbi::get_mvendorid()))
        && f(Case::GetMArchId(sbi::get_marchid()))
        && f(Case::GetMimpId(sbi::get_mimpid()));
    Ok(pass)
}
