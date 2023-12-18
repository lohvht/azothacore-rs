use std::{
    ffi::{c_void, CString},
    io,
    mem,
    path::Path,
    ptr,
};

use casclib_sys::*;
use flagset::{flags, FlagSet};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use thiserror::Error;
use tracing::{error, info};

pub struct CascStorageHandle {
    h: HANDLE,
}

flags! {
    pub enum CascLocale: u32 {
        All = CASC_LOCALE_ALL,
        AllWow = CASC_LOCALE_ALL_WOW,
        None = CASC_LOCALE_NONE,
        Unknown1 = CASC_LOCALE_UNKNOWN1,
        Enus = CASC_LOCALE_ENUS,
        Kokr = CASC_LOCALE_KOKR,
        Reserved = CASC_LOCALE_RESERVED,
        Frfr = CASC_LOCALE_FRFR,
        Dede = CASC_LOCALE_DEDE,
        Zhcn = CASC_LOCALE_ZHCN,
        Eses = CASC_LOCALE_ESES,
        Zhtw = CASC_LOCALE_ZHTW,
        Engb = CASC_LOCALE_ENGB,
        Encn = CASC_LOCALE_ENCN,
        Entw = CASC_LOCALE_ENTW,
        Esmx = CASC_LOCALE_ESMX,
        Ruru = CASC_LOCALE_RURU,
        Ptbr = CASC_LOCALE_PTBR,
        Itit = CASC_LOCALE_ITIT,
        Ptpt = CASC_LOCALE_PTPT,
    }
}

#[derive(Error, Debug, Clone, Copy, FromPrimitive)]
#[repr(u32)]
pub enum CascError {
    #[error("FILE_NOT_FOUND")]
    FileOrPathNotFound = ERROR_FILE_NOT_FOUND,
    #[error("ACCESS_DENIED")]
    AccessDenied = ERROR_ACCESS_DENIED,
    #[error("INVALID_HANDLE")]
    InvalidHandle = ERROR_INVALID_HANDLE,
    #[error("NOT_ENOUGH_MEMORY")]
    NotEnoughMemory = ERROR_NOT_ENOUGH_MEMORY,
    #[error("NOT_SUPPORTED")]
    NotSupported = ERROR_NOT_SUPPORTED,
    #[error("INVALID_PARAMETER")]
    InvalidParameter = ERROR_INVALID_PARAMETER,
    #[error("DISK_FULL")]
    DiskFull = ERROR_DISK_FULL,
    #[error("ALREADY_EXISTS")]
    AlreadyExists = ERROR_ALREADY_EXISTS,
    #[error("INSUFFICIENT_BUFFER")]
    InsufficientBuffer = ERROR_INSUFFICIENT_BUFFER,
    #[error("BAD_FORMAT")]
    BadFormat = ERROR_BAD_FORMAT,
    #[error("NO_MORE_FILES")]
    NoMoreFiles = ERROR_NO_MORE_FILES,
    #[error("HANDLE_EOF")]
    HandleEof = ERROR_HANDLE_EOF,
    #[error("CAN_NOT_COMPLETE")]
    CanNotComplete = ERROR_CAN_NOT_COMPLETE,
    #[error("FILE_CORRUPT")]
    FileCorrupt = ERROR_FILE_CORRUPT,
    #[error("FILE_ENCRYPTED")]
    FileEncrypted = ERROR_FILE_ENCRYPTED,
    #[error("FILE_TOO_LARGE")]
    FileTooLarge = ERROR_FILE_TOO_LARGE,
    #[error("ARITHMETIC_OVERFLOW")]
    ArithmeticOverflow = ERROR_ARITHMETIC_OVERFLOW,
    #[error("NETWORK_NOT_AVAILABLE")]
    NetworkNotAvailable = ERROR_NETWORK_NOT_AVAILABLE,
}

impl TryFrom<u32> for CascError {
    type Error = CascHandlerError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        FromPrimitive::from_u32(value).ok_or(CascHandlerError::CascLibParseError {
            typ: "_casc_error".into(),
            got: value,
        })
    }
}

#[derive(Error, Debug, Clone)]
pub enum CascHandlerError {
    #[error("error when calling casclib: {0}")]
    CascLibError(#[from] CascError),
    #[error("invalid code for {typ}: got {got}")]
    CascLibParseError { typ: String, got: u32 },
}

#[derive(Debug, Clone, FromPrimitive)]
#[repr(u32)]
pub enum CascStorageInfoClass {
    /// Returns the number of local files in the storage. Note that files
    /// can exist under different names, so the total number of files in the archive
    /// can be higher than the value returned by this info class
    LocalFileCount = _CASC_STORAGE_INFO_CLASS_CascStorageLocalFileCount,
    /// Returns the total file count, including the offline files
    TotalFileCount = _CASC_STORAGE_INFO_CLASS_CascStorageTotalFileCount,
    /// Returns the features flag    
    Features = _CASC_STORAGE_INFO_CLASS_CascStorageFeatures,
    /// Not supported
    InstalledLocales = _CASC_STORAGE_INFO_CLASS_CascStorageInstalledLocales,
    /// Gives CASC_STORAGE_PRODUCT
    Product = _CASC_STORAGE_INFO_CLASS_CascStorageProduct,
    /// Gives CASC_STORAGE_TAGS structure
    Tags = _CASC_STORAGE_INFO_CLASS_CascStorageTags,
    /// Gives Path:Product into a LPTSTR bufferPathProduct = _CASC_STORAGE_INFO_CLASS_CascStoragePathProduct,
    InfoClassMax = _CASC_STORAGE_INFO_CLASS_CascStorageInfoClassMax,
}

impl TryFrom<u32> for CascStorageInfoClass {
    type Error = CascHandlerError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        FromPrimitive::from_u32(value).ok_or(CascHandlerError::CascLibParseError {
            typ: "_casc_storage_info_class".into(),
            got: value,
        })
    }
}

#[derive(Debug)]
pub struct CascStorageProduct {
    pub code_name:    String,
    pub build_number: u32,
}

impl CascStorageHandle {
    pub fn build<P: AsRef<Path>>(path: P, local_mask: FlagSet<CascLocale>) -> Result<CascStorageHandle, CascHandlerError> {
        let p = CString::new(path.as_ref().to_string_lossy().to_string()).unwrap();

        let mut handle: *mut c_void = ptr::null_mut();
        let res = unsafe {
            let sz_data_path = p.as_ptr();
            CascOpenStorage(sz_data_path, local_mask.bits(), ptr::addr_of_mut!(handle))
        };

        if !res {
            let last_error = CascError::try_from(unsafe { GetCascError() })?;
            // error!("Error opening casc storage {}: {}", path.as_ref().to_string_lossy().to_string(), last_error);
            unsafe {
                CascCloseStorage(handle);
                SetCascError(last_error as u32);
            };
            return Err(CascHandlerError::CascLibError(last_error));
        };
        info!("Opened casc storage '{}'", path.as_ref().to_string_lossy().to_string());
        Ok(CascStorageHandle { h: handle })
    }

    pub fn get_product_info(&self) -> Result<CascStorageProduct, CascHandlerError> {
        let storage_class = CascStorageInfoClass::Product as _;
        let mut product_info = _CASC_STORAGE_PRODUCT {
            BuildNumber: 0,
            szCodeName:  [0; 28],
        };
        let product_info_ptr = &mut product_info as *mut _ as *mut c_void;

        let mut info_data_size_needed: usize = 0;
        let info_data_size_needed_ptr = &mut info_data_size_needed as *mut _;
        if !(unsafe {
            CascGetStorageInfo(
                self.h,
                storage_class,
                product_info_ptr,
                mem::size_of::<_CASC_STORAGE_PRODUCT>(),
                info_data_size_needed_ptr,
            )
        }) {
            let last_error = CascError::try_from(unsafe { GetCascError() })?;
            return Err(CascHandlerError::CascLibError(last_error));
        }

        Ok(CascStorageProduct {
            code_name:    product_info
                .szCodeName
                .iter()
                .filter_map(|e| {
                    if *e == 0 {
                        return None;
                    }
                    Some(*e as u8 as char)
                })
                .collect(),
            build_number: product_info.BuildNumber,
        })
    }

    pub fn get_installed_locales_mask(&self) -> Result<FlagSet<CascLocale>, CascHandlerError> {
        let storage_class = CascStorageInfoClass::InstalledLocales as _;
        let mut value: u32 = 0;
        let value_ptr = &mut value as *mut _ as *mut c_void;

        let mut info_data_size_needed: usize = 0;
        let info_data_size_needed_ptr = &mut info_data_size_needed as *mut _;
        if !(unsafe { CascGetStorageInfo(self.h, storage_class, value_ptr, mem::size_of::<u32>(), info_data_size_needed_ptr) }) {
            let last_error = CascError::try_from(unsafe { GetCascError() })?;
            return Err(CascHandlerError::CascLibError(last_error));
        }
        FlagSet::<CascLocale>::new(value).map_err(|e| CascHandlerError::CascLibParseError {
            got: value,
            typ: e.to_string(),
        })
    }

    pub fn open_file<P: AsRef<Path>>(&self, file_name: P, locale_mask: FlagSet<CascLocale>) -> Result<CascFileHandle, CascHandlerError> {
        let p = CString::new(file_name.as_ref().to_string_lossy().to_string()).unwrap();

        let mut handle: *mut c_void = ptr::null_mut();
        let res = unsafe {
            let sz_data_path = p.as_ptr() as *mut c_void;
            CascOpenFile(self.h, sz_data_path, locale_mask.bits(), 0, ptr::addr_of_mut!(handle))
        };

        if !res {
            let last_error = CascError::try_from(unsafe { GetCascError() })?;
            // error!(
            //     "Failed to open file {} in CASC storage: {}",
            //     file_name.as_ref().to_string_lossy().to_string(),
            //     last_error
            // );
            unsafe {
                CascCloseFile(handle);
                SetCascError(last_error as u32);
            };
            return Err(CascHandlerError::CascLibError(last_error));
        };
        Ok(CascFileHandle {
            h:           handle,
            current_pos: 0,
        })
    }
}

/// Handles freeing of CascStorageHandle
impl Drop for CascStorageHandle {
    fn drop(&mut self) {
        if !self.h.is_null() {
            unsafe { CascCloseStorage(self.h) };
        }
    }
}

pub struct CascFileHandle {
    h:           HANDLE,
    current_pos: u64,
}

impl CascFileHandle {
    pub fn get_file_size(&self) -> Result<u64, CascHandlerError> {
        let mut value: u64 = 0;
        let value_ptr = &mut value as *mut _;
        let res = unsafe { CascGetFileSize64(self.h, value_ptr) };
        if !res {
            let last_error = CascError::try_from(unsafe { GetCascError() })?;
            // error!("Failed to get filesize: err is {}", last_error);
            return Err(CascHandlerError::CascLibError(last_error));
        };
        Ok(value)
    }

    pub fn is_empty(&self) -> bool {
        let file_size = match self.get_file_size() {
            Err(_e) => return true,
            Ok(s) => s,
        };
        self.current_pos >= file_size
    }
}

/// Handles freeing of CascFileHandle
impl Drop for CascFileHandle {
    fn drop(&mut self) {
        if !self.h.is_null() {
            unsafe { CascCloseFile(self.h) };
        }
    }
}

impl io::Read for CascFileHandle {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len_to_read = buf.len();
        let mut len_read = 0;

        let buf_ptr = buf as *mut _ as *mut c_void;

        let res = unsafe { CascReadFile(self.h, buf_ptr, len_to_read as u32, &mut len_read) };
        if !res {
            let last_error = CascError::try_from(unsafe { GetCascError() })
                .map_or_else(|e| io::Error::new(io::ErrorKind::Other, e), |e| io::Error::new(io::ErrorKind::Other, e));
            return Err(last_error);
        }
        self.current_pos += u64::from(len_read);
        Ok(len_read as usize)
    }
}

impl io::Seek for CascFileHandle {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        let (method, to_move) = match pos {
            io::SeekFrom::Start(c) => (FILE_BEGIN, c as i64),
            io::SeekFrom::Current(c) => (FILE_CURRENT, c),
            io::SeekFrom::End(c) => (FILE_END, c),
        };

        let res = unsafe { CascSetFilePointer64(self.h, to_move, &mut self.current_pos, method) };
        if !res {
            let last_error = CascError::try_from(unsafe { GetCascError() })
                .map_or_else(|e| io::Error::new(io::ErrorKind::Other, e), |e| io::Error::new(io::ErrorKind::Other, e));
            return Err(last_error);
        }
        Ok(self.current_pos)
    }
}
