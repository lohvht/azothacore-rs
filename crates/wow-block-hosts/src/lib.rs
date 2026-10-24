//! `LD_PRELOAD` library that makes Blizzard host lookups fail inside the game client process only.
//!
//! Under Wine, the 7.3.5 client can hang on the loading screen: it keeps an HTTP connection to
//! `launcher.worldofwarcraft.com` open, closes it when the player clicks Enter World, and the
//! instance (8086) socket opened next reuses that handle while the client still holds the old
//! socket's entry. If the lookup fails, that socket never exists. See this crate's `README.md`.
//!
//! The library replaces `getaddrinfo`, `gethostbyname` and `gethostbyname_r`. For a blocked name
//! they report "host not found"; every other name goes to the real libc function.
//!
//! Environment:
//! - `WOW_BLOCK_HOSTS="a.example b.example"` replaces [`DEFAULT_HOSTS`] (space- or comma-separated).
//! - `WOW_BLOCK_HOSTS_DEBUG=1` prints each blocked lookup to stderr.

use std::{
    ffi::{CStr, c_char, c_int, c_void},
    sync::OnceLock,
};

use libc::{addrinfo, hostent, size_t};

/// Hosts blocked when `WOW_BLOCK_HOSTS` is not set: the launcher alert host and the patch CDNs
/// listed in the client's `.build.info`.
pub const DEFAULT_HOSTS: &str = "launcher.worldofwarcraft.com blzddist1-a.akamaihd.net edgecast.blizzard.com level3.blizzard.com";

/// `HOST_NOT_FOUND` from `<netdb.h>`.
const HOST_NOT_FOUND: c_int = 1;

unsafe extern "C" {
    /// glibc's thread-local `h_errno`, which `gethostbyname` reports errors through.
    fn __h_errno_location() -> *mut c_int;
}

type GetAddrInfoFn = unsafe extern "C" fn(*const c_char, *const c_char, *const addrinfo, *mut *mut addrinfo) -> c_int;
type GetHostByNameFn = unsafe extern "C" fn(*const c_char) -> *mut hostent;
type GetHostByNameRFn = unsafe extern "C" fn(*const c_char, *mut hostent, *mut c_char, size_t, *mut *mut hostent, *mut c_int) -> c_int;

/// True if `name` (ignoring ASCII case and one trailing dot) is in the space- or comma-separated `list`.
#[must_use]
pub fn host_in_list(name: &str, list: &str) -> bool {
    let name = name.strip_suffix('.').unwrap_or(name);
    !name.is_empty() && list.split([' ', ',']).any(|entry| entry.eq_ignore_ascii_case(name))
}

/// Checks a C host name against `WOW_BLOCK_HOSTS` (or [`DEFAULT_HOSTS`]).
///
/// # Safety
/// `name` must be null or point to a NUL-terminated string.
unsafe fn is_blocked(name: *const c_char) -> bool {
    if name.is_null() {
        return false;
    }
    // SAFETY: the caller guarantees a NUL-terminated string.
    let Ok(name) = unsafe { CStr::from_ptr(name) }.to_str() else {
        return false;
    };
    let list = std::env::var("WOW_BLOCK_HOSTS");
    let blocked = host_in_list(name, list.as_deref().unwrap_or(DEFAULT_HOSTS));
    if blocked && std::env::var_os("WOW_BLOCK_HOSTS_DEBUG").is_some() {
        eprintln!("wow-block-hosts: blocked lookup of {name}");
    }
    blocked
}

/// Looks up the next definition of `symbol` after this library (the real libc function).
fn next_symbol(symbol: &CStr) -> *mut c_void {
    // SAFETY: `symbol` is NUL-terminated; RTLD_NEXT is valid in a shared library.
    let ptr = unsafe { libc::dlsym(libc::RTLD_NEXT, symbol.as_ptr()) };
    assert!(!ptr.is_null(), "wow-block-hosts: {symbol:?} not found after this library");
    ptr
}

/// Replacement for libc `getaddrinfo`.
///
/// # Safety
/// Same contract as libc `getaddrinfo`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getaddrinfo(node: *const c_char, service: *const c_char, hints: *const addrinfo, res: *mut *mut addrinfo) -> c_int {
    static REAL: OnceLock<GetAddrInfoFn> = OnceLock::new();
    // SAFETY: `node` follows the getaddrinfo contract (null or NUL-terminated).
    if unsafe { is_blocked(node) } {
        return libc::EAI_NONAME;
    }
    // SAFETY: the symbol is libc's getaddrinfo, which has this signature.
    let real = REAL.get_or_init(|| unsafe { std::mem::transmute::<*mut c_void, GetAddrInfoFn>(next_symbol(c"getaddrinfo")) });
    // SAFETY: arguments are passed through unchanged.
    unsafe { real(node, service, hints, res) }
}

/// Replacement for libc `gethostbyname`.
///
/// # Safety
/// Same contract as libc `gethostbyname`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gethostbyname(name: *const c_char) -> *mut hostent {
    static REAL: OnceLock<GetHostByNameFn> = OnceLock::new();
    // SAFETY: `name` follows the gethostbyname contract.
    if unsafe { is_blocked(name) } {
        // SAFETY: glibc always returns a valid thread-local pointer.
        unsafe { *__h_errno_location() = HOST_NOT_FOUND };
        return std::ptr::null_mut();
    }
    // SAFETY: the symbol is libc's gethostbyname, which has this signature.
    let real = REAL.get_or_init(|| unsafe { std::mem::transmute::<*mut c_void, GetHostByNameFn>(next_symbol(c"gethostbyname")) });
    // SAFETY: arguments are passed through unchanged.
    unsafe { real(name) }
}

/// Replacement for libc `gethostbyname_r`.
///
/// # Safety
/// Same contract as libc `gethostbyname_r`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gethostbyname_r(
    name: *const c_char,
    ret: *mut hostent,
    buf: *mut c_char,
    buflen: size_t,
    result: *mut *mut hostent,
    h_errnop: *mut c_int,
) -> c_int {
    static REAL: OnceLock<GetHostByNameRFn> = OnceLock::new();
    // SAFETY: `name` follows the gethostbyname_r contract.
    if unsafe { is_blocked(name) } {
        // SAFETY: `result` is a valid out-pointer and `h_errnop` is null or valid, per the contract.
        unsafe {
            *result = std::ptr::null_mut();
            if !h_errnop.is_null() {
                *h_errnop = HOST_NOT_FOUND;
            }
        }
        return 0;
    }
    // SAFETY: the symbol is libc's gethostbyname_r, which has this signature.
    let real = REAL.get_or_init(|| unsafe { std::mem::transmute::<*mut c_void, GetHostByNameRFn>(next_symbol(c"gethostbyname_r")) });
    // SAFETY: arguments are passed through unchanged.
    unsafe { real(name, ret, buf, buflen, result, h_errnop) }
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_HOSTS, host_in_list};

    #[test]
    fn matches_default_hosts() {
        assert!(host_in_list("launcher.worldofwarcraft.com", DEFAULT_HOSTS));
        assert!(host_in_list("LEVEL3.Blizzard.com.", DEFAULT_HOSTS));
        assert!(!host_in_list("worldofwarcraft.com", DEFAULT_HOSTS));
        assert!(!host_in_list("localhost", DEFAULT_HOSTS));
        assert!(!host_in_list("", DEFAULT_HOSTS));
    }

    #[test]
    fn custom_list_with_commas_and_spaces() {
        let list = "a.example, b.example  c.example";
        assert!(host_in_list("b.example", list));
        assert!(host_in_list("c.example", list));
        assert!(!host_in_list("d.example", list));
    }
}
