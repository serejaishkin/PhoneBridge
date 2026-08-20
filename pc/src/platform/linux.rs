use super::{PlatformBackend, PlatformKind};
use crate::protocol::HfpSupport;

pub struct LinuxBackend;

impl LinuxBackend {
    pub fn new() -> Self { Self }
}

impl Default for LinuxBackend {
    fn default() -> Self { Self::new() }
}

impl PlatformBackend for LinuxBackend {
    fn kind(&self) -> PlatformKind { PlatformKind::Linux }
    fn hfp_support(&self) -> HfpSupport { HfpSupport::Unknown }
    fn answer_call(&self) -> anyhow::Result<()> { Ok(()) }
    fn decline_call(&self) -> anyhow::Result<()> { Ok(()) }
    fn end_call(&self) -> anyhow::Result<()> { Ok(()) }
}
