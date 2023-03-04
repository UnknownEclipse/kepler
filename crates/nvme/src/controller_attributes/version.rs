use core::fmt::Debug;

use bitfrob::u32_get_value;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Version(u32);

impl Debug for Version {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut dbg = f.debug_tuple("Version");
        match self.spec_version() {
            Ok(v) => {
                dbg.field(&v);
            }
            Err(raw) => {
                dbg.field(&raw);
            }
        }
        dbg.finish()
    }
}

impl Version {
    pub fn major(&self) -> u16 {
        u32_get_value(16, 31, self.0) as u16
    }

    pub fn minor(&self) -> u8 {
        u32_get_value(15, 8, self.0) as u8
    }

    pub fn tertiary(&self) -> u8 {
        u32_get_value(0, 7, self.0) as u8
    }

    pub fn spec_version(&self) -> Result<SpecVersion, (u16, u8, u8)> {
        match (self.major(), self.minor(), self.tertiary()) {
            (1, 0, _) => Ok(SpecVersion::Version1),
            (1, 1, _) => Ok(SpecVersion::Version1_1),
            (1, 2, 1) => Ok(SpecVersion::Version1_2_1),
            (1, 2, _) => Ok(SpecVersion::Version1_2),
            (1, 3, 0) => Ok(SpecVersion::Version1_3),
            (1, 4, 0) => Ok(SpecVersion::Version1_4),
            (2, 0, 0) => Ok(SpecVersion::Version2),
            v => Err(v),
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum SpecVersion {
    Version1,
    Version1_1,
    Version1_2,
    Version1_2_1,
    Version1_3,
    Version1_4,
    Version2,
}
