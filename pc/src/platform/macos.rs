use super::{PlatformBackend, PlatformKind};
use crate::protocol::HfpSupport;

pub struct MacOSBackend;

impl MacOSBackend {
    pub fn new() -> Self { Self }
}

impl Default for MacOSBackend {
    fn default() -> Self { Self::new() }
}

impl PlatformBackend for MacOSBackend {
    fn kind(&self) -> PlatformKind { PlatformKind::MacOS }
    fn hfp_support(&self) -> HfpSupport { HfpSupport::Unknown }
    fn answer_call(&self) -> anyhow::Result<()> { Ok(()) }
    fn decline_call(&self) -> anyhow::Result<()> { Ok(()) }
    fn end_call(&self) -> anyhow::Result<()> { Ok(()) }
}
