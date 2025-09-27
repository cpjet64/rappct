//! Internal Win32 API shim trait for testing (skeleton).

pub trait Win32Api {}

pub struct RealWin32;
impl Win32Api for RealWin32 {}
