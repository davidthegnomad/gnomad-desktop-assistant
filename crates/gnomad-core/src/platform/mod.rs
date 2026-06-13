mod traits;

#[cfg(target_os = "linux")]
pub mod diagnostics;
#[cfg(target_os = "linux")]
pub mod linux_context;

pub use traits::{GnomadCoreContext, PlatformContext};

#[cfg(target_os = "linux")]
pub use diagnostics::{run_linux_diagnostics, CheckStatus, DoctorCheck, DoctorReport};
#[cfg(target_os = "linux")]
pub use linux_context::{format_context_block, gather_desktop_context, DesktopContext};
