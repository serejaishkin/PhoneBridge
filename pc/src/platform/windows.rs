use super::{PlatformBackend, PlatformKind};
use crate::protocol::HfpSupport;

pub struct WindowsBackend;

impl WindowsBackend {
    pub fn new() -> Self { Self }
}

impl Default for WindowsBackend {
    fn default() -> Self { Self::new() }
}

impl PlatformBackend for WindowsBackend {
    fn kind(&self) -> PlatformKind { PlatformKind::Windows }
    fn hfp_support(&self) -> HfpSupport { HfpSupport::Unknown }
    fn answer_call(&self) -> anyhow::Result<()> { Ok(()) }
    fn decline_call(&self) -> anyhow::Result<()> { Ok(()) }
    fn end_call(&self) -> anyhow::Result<()> { Ok(()) }
}
