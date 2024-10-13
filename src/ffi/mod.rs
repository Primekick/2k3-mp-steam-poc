pub mod maniacs;

use std::sync::Mutex;
use lazy_static::lazy_static;
use winsafe::prelude::*;
use winsafe::HINSTANCE;

#[macro_export]
macro_rules! rust_string {
    ($c_ptr:expr) => {
        {
            let c_str = unsafe { ::std::ffi::CStr::from_ptr($c_ptr) };
            c_str.to_str().unwrap().to_string()
        }
    };
}

#[macro_export]

macro_rules! ffi_call {
    ($address:expr, $t:ty) => {
        std::mem::transmute::<*const (), $t>($address as _)
    };
}

extern "system" {
    fn AllocConsole() -> bool;
}

pub fn open_console() {
    unsafe { AllocConsole(); }
}

lazy_static! {
    static ref MODULE_BASE: Mutex<HINSTANCE> = Mutex::new(HINSTANCE::GetModuleHandle(None).expect("Couldn't get module handle"));
}

pub fn get_module_base() -> *const usize {
    MODULE_BASE.lock().unwrap().ptr() as *const usize
}