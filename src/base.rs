pub enum Case {
    GetSbiSpecVersion(usize),
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
    if sbi::probe_extension(sbi::EID_BASE) == 0 {
        Err(NotExist)?;
    }
    let pass = f(Case::GetSbiSpecVersion(sbi::get_spec_version()))
        && f(Case::GetSbiImplId(match sbi::get_sbi_impl_id() {
            sbi::impl_id::IMPL_BBL => Ok("BBL"),
            sbi::impl_id::IMPL_OPEN_SBI => Ok("OpenSBI"),
            sbi::impl_id::IMPL_XVISOR => Ok("Xvisor"),
            sbi::impl_id::IMPL_KVM => Ok("KVM"),
            sbi::impl_id::IMPL_RUST_SBI => Ok("RustSBI"),
            sbi::impl_id::IMPL_DIOSIX => Ok("Diosix"),
            sbi::impl_id::IMPL_COFFER => Ok("Coffer"),
            unknown => Err(unknown),
        }))
        && f(Case::GetSbiImplVersion(sbi::get_sbi_impl_version()))
        && f(Case::ProbeExtensions(Extensions {
            time: sbi::probe_extension(sbi::EID_TIME) != 0,
            spi: sbi::probe_extension(sbi::EID_SPI) != 0,
            rfnc: sbi::probe_extension(sbi::EID_RFNC) != 0,
            hsm: sbi::probe_extension(sbi::EID_HSM) != 0,
            srst: sbi::probe_extension(sbi::EID_SRST) != 0,
            pmu: sbi::probe_extension(sbi::EID_PMU) != 0,
        }))
        && f(Case::GetMVendorId(sbi::get_mvendorid()))
        && f(Case::GetMArchId(sbi::get_marchid()))
        && f(Case::GetMimpId(sbi::get_mimpid()));
    Ok(pass)
}
