//! STARTUPINFOEX attribute list RAII for Windows.

#[cfg(windows)]
pub struct AttrList {
    _buf: Vec<u8>,
    ptr: windows::Win32::System::Threading::LPPROC_THREAD_ATTRIBUTE_LIST,
}

#[cfg(windows)]
impl AttrList {
    pub fn new(attribute_count: usize) -> crate::Result<Self> {
        use windows::Win32::System::Threading::{
            InitializeProcThreadAttributeList, LPPROC_THREAD_ATTRIBUTE_LIST,
        };
        unsafe {
            let mut size: usize = 0;
            // First call to get required size
            let _ =
                InitializeProcThreadAttributeList(None, attribute_count as u32, Some(0), &mut size);
            #[cfg(feature = "tracing")]
            tracing::trace!(
                "InitializeProcThreadAttributeList(size query): attr_count={}, required_size={}",
                attribute_count,
                size
            );
            if size == 0 {
                return Err(crate::AcError::Win32(
                    "InitializeProcThreadAttributeList size query failed".into(),
                ));
            }
            let mut buf = vec![0u8; size];
            let ptr = LPPROC_THREAD_ATTRIBUTE_LIST(buf.as_mut_ptr() as _);
            InitializeProcThreadAttributeList(Some(ptr), attribute_count as u32, Some(0), &mut size)
                .map_err(|_| {
                    #[cfg(feature = "tracing")]
                    {
                        use windows::Win32::Foundation::GetLastError;
                        let gle = GetLastError().0;
                        tracing::error!(
                            "InitializeProcThreadAttributeList(init) failed: attr_count={}, size={}, GLE={}",
                            attribute_count,
                            size,
                            gle
                        );
                    }
                    crate::AcError::Win32("InitializeProcThreadAttributeList failed".into())
                })?;
            #[cfg(feature = "tracing")]
            tracing::trace!(
                "InitializeProcThreadAttributeList(init): attr_list_ptr={:p}, size={}",
                ptr.0,
                size
            );
            Ok(Self { _buf: buf, ptr })
        }
    }
    pub fn as_mut_ptr(
        &mut self,
    ) -> windows::Win32::System::Threading::LPPROC_THREAD_ATTRIBUTE_LIST {
        self.ptr
    }
}

#[cfg(windows)]
impl Drop for AttrList {
    fn drop(&mut self) {
        use windows::Win32::System::Threading::DeleteProcThreadAttributeList;
        unsafe {
            let _ = DeleteProcThreadAttributeList(self.ptr);
        }
    }
}

#[cfg(not(windows))]
pub struct AttrList {
    _private: (),
}
#[cfg(not(windows))]
impl AttrList {
    pub fn new(_attribute_count: usize) -> crate::Result<Self> {
        Err(crate::AcError::UnsupportedPlatform)
    }
}
