//! Network isolation helpers. Feature: `net`.

use crate::sid::AppContainerSid;
use crate::{AcError, Result};

#[cfg(all(windows, feature = "net"))]
use std::cell::RefCell;
#[cfg(all(windows, feature = "net"))]
use std::collections::HashSet;

#[cfg(all(windows, feature = "net"))]
mod windows_impl;

/// Lists all registered AppContainer profiles and their display names from the firewall config.
pub fn list_appcontainers() -> Result<Vec<(AppContainerSid, String)>> {
    #[cfg(all(windows, feature = "net"))]
    {
        windows_impl::list_appcontainers()
    }
    #[cfg(all(windows, not(feature = "net")))]
    {
        Err(AcError::Unimplemented("net feature not enabled"))
    }
    #[cfg(not(windows))]
    {
        Err(AcError::UnsupportedPlatform)
    }
}

/// Safety latch: force explicit acknowledgement before applying loopback exemptions.
/// Marker type used to acknowledge loopback firewall exemptions in development builds.
#[derive(Debug, Clone)]
pub struct LoopbackAdd(pub AppContainerSid);

#[cfg(all(windows, feature = "net"))]
thread_local! {
    static CONFIRMED_LOOPBACK_SIDS: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
}

/// Applies a loopback firewall exemption for the given AppContainer SID.
/// Callers must acknowledge the operation with `LoopbackAdd::confirm_debug_only` first.
///
/// # Example
/// ```no_run
/// use rappct::{net, AppContainerProfile};
///
/// # fn main() -> rappct::Result<()> {
/// let profile = AppContainerProfile::ensure(
///     "rappct.example",
///     "Example",
///     Some("loopback demo"),
/// )?;
/// net::remove_loopback_exemption(&profile.sid).ok();
/// net::add_loopback_exemption(net::LoopbackAdd(profile.sid.clone()).confirm_debug_only())?;
/// profile.delete()?;
/// # Ok(())
/// # }
/// ```
pub fn add_loopback_exemption(req: LoopbackAdd) -> Result<()> {
    let _ = &req;
    #[cfg(all(windows, feature = "net"))]
    {
        // Safety latch: require explicit confirm prior to call
        let is_confirmed = CONFIRMED_LOOPBACK_SIDS
            .with(|confirmed| confirmed.borrow_mut().remove(req.0.as_string()));
        if !is_confirmed {
            return Err(AcError::AccessDenied {
                context: "loopback exemption requires confirm_debug_only()".into(),
                source: Box::new(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "confirmation missing",
                )),
            });
        }
        windows_impl::set_loopback(true, &req.0)
    }
    #[cfg(all(windows, not(feature = "net")))]
    {
        Err(AcError::Unimplemented("net feature not enabled"))
    }
    #[cfg(not(windows))]
    {
        Err(AcError::UnsupportedPlatform)
    }
}

/// Removes any loopback exemption previously granted to the provided AppContainer SID.
pub fn remove_loopback_exemption(sid: &AppContainerSid) -> Result<()> {
    let _ = sid;
    #[cfg(all(windows, feature = "net"))]
    {
        windows_impl::set_loopback(false, sid)
    }
    #[cfg(all(windows, not(feature = "net")))]
    {
        Err(AcError::Unimplemented("net feature not enabled"))
    }
    #[cfg(not(windows))]
    {
        Err(AcError::UnsupportedPlatform)
    }
}

/// RAII guard that applies a loopback exemption on construction and
/// always removes it on drop. Intended for debug/testing only.
///
/// Note: callers must be explicit and opt in to the operation via
/// `LoopbackAdd::confirm_debug_only()`.
#[must_use]
#[derive(Debug)]
pub struct LoopbackExemptionGuard {
    sid: AppContainerSid,
    active: bool,
}

impl LoopbackExemptionGuard {
    /// Attempts to add a loopback exemption without implicit confirmation.
    ///
    /// This preserves the original constructor shape, but it no longer
    /// auto-confirms the debug-only operation. Use [`Self::new_confirmed`] with
    /// `LoopbackAdd(...).confirm_debug_only()` for the RAII flow.
    pub fn new(sid: &AppContainerSid) -> Result<Self> {
        Self::new_confirmed(LoopbackAdd(sid.clone()))
    }

    /// Adds a loopback exemption from an explicitly confirmed request.
    ///
    /// Typical usage:
    ///
    /// ```no_run
    /// # #[cfg(feature = "net")]
    /// # {
    /// # use rappct::{AppContainerProfile, net::{LoopbackAdd, LoopbackExemptionGuard}};
    /// # let profile = AppContainerProfile::ensure("rappct.guard", "guard", None).unwrap();
    /// let _guard = LoopbackExemptionGuard::new_confirmed(
    ///     LoopbackAdd(profile.sid.clone()).confirm_debug_only(),
    /// ).unwrap();
    /// # }
    /// ```
    pub fn new_confirmed(req: LoopbackAdd) -> Result<Self> {
        let sid = req.0.clone();
        super::net::add_loopback_exemption(req)?;
        Ok(Self { sid, active: true })
    }

    /// Removes the exemption immediately and disables the drop cleanup path.
    pub fn close(mut self) -> Result<()> {
        if self.active {
            self.active = false;
            return remove_loopback_exemption(&self.sid);
        }
        Ok(())
    }

    /// Disable removal on drop (opt-out). Primarily useful for testing.
    pub fn disable(mut self) -> Self {
        self.active = false;
        self
    }
}

impl Drop for LoopbackExemptionGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = remove_loopback_exemption(&self.sid);
        }
    }
}

impl LoopbackAdd {
    /// Confirms that the caller is intentionally requesting a loopback exemption.
    /// Without this acknowledgement `add_loopback_exemption` returns `AccessDenied`.
    ///
    /// Typical usage pairs the guard with `add_loopback_exemption`:
    ///
    /// ```no_run
    /// use rappct::{net, AppContainerProfile};
    ///
    /// # fn main() -> rappct::Result<()> {
    /// let profile = AppContainerProfile::ensure(
    ///     "rappct.confirm",
    ///     "Confirm",
    ///     Some("loopback confirm"),
    /// )?;
    /// net::add_loopback_exemption(net::LoopbackAdd(profile.sid.clone()).confirm_debug_only())?;
    /// profile.delete()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn confirm_debug_only(self) -> Self {
        #[cfg(all(windows, feature = "net"))]
        {
            CONFIRMED_LOOPBACK_SIDS.with(|confirmed| {
                confirmed.borrow_mut().insert(self.0.as_string().to_owned());
            });
        }
        self
    }
}
