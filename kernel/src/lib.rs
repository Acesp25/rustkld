#![no_std]
#![feature(alloc_error_handler)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub use paste::paste as __paste;

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

pub extern crate libc;

#[macro_use]
mod macros;
unsafe impl Sync for mod_metadata {}
unsafe impl Sync for moduledata{}
unsafe impl Sync for sysinit {}
unsafe impl Sync for sysinit__bindgen_ty_1 {}

mod bindings;
pub use bindings::imports::*;

pub mod io;
pub use io::KernelDebugWriter;

mod allocator;
pub use allocator::KernelAllocator;
pub extern crate alloc;
#[global_allocator]
static ALLOCATOR: KernelAllocator = KernelAllocator;

mod char_ffi;
pub use char_ffi::{ffi_open, ffi_close, ffi_read, ffi_write};

mod uio_wrap;
pub use uio_wrap::Uio;

mod cdev_wrap;
pub use cdev_wrap::Cdev;

mod flags;
pub use flags::{Ioflag, Oflags};

pub trait Cdevsw {    
    fn quiesce(&mut self) -> Result<(), libc::c_int>;
    fn open(&mut self, dev: Cdev, oflags: Oflags, devtype: libc::c_int, td: *mut thread) -> Result<(), libc::c_int>;
    fn close(&mut self, dev: Cdev, oflags: Oflags, devtype: libc::c_int, td: *mut thread) -> Result<(), libc::c_int>;
    fn write(&mut self, dev: Cdev, uio_ptr: Uio, ioflag: Ioflag) -> Result<libc::c_int, libc::c_int>;
    fn read(&mut self, dev: Cdev, uio_ptr: Uio, ioflag: Ioflag) -> Result<libc::c_int, libc::c_int>;
/*
    fn ioctl(...);
    fn poll(...);
    fn mmap(...);
    fn strategy(...);
    fn kqfilter(...);
    fn purge(...);
    fn mmap_single(...);
*/
}

pub trait Read {
    fn read(&mut self, buf: &mut alloc::vec::Vec<u8>) -> Result<usize, libc::c_int>;   
    fn read_buf(&mut self, buf: &mut alloc::vec::Vec<u8>) -> Result<usize, libc::c_int>;   
}

pub trait Write {
    fn write(&mut self, buf: &mut alloc::vec::Vec<u8>) -> Result<usize, libc::c_int>;
    fn write_all(&mut self, buf: &mut alloc::vec::Vec<u8>) -> Result<usize, libc::c_int>;
}

pub enum ModEventType {
    Load = 0,
    Unload = 1,
    Shutdown = 2,
    Quiesce = 3,
    Unknown = 4,
}
impl From<i32> for ModEventType {
    fn from(i: i32) -> Self {
        match i {
            0 => ModEventType::Load,
            1 => ModEventType::Unload,
            2 => ModEventType::Shutdown,
            3 => ModEventType::Quiesce,
            _ => ModEventType::Unknown,
        }
    }
}
