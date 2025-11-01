use crate::{cdev, uio, thread}; 
use libc::{c_int, EFAULT};
extern crate alloc;
use alloc::boxed::Box;
use crate::Cdevsw;
use crate::uio_wrap::Uio;
use crate::cdev_wrap::Cdev;
use crate::flags::{Ioflag, Oflags};

unsafe fn get_Cdevsw<'a>(dev: *mut cdev) -> Result<&'a mut dyn Cdevsw, c_int> {
    if dev.is_null() {
        println!("[char_ffi.rs] cdev is null");
        return Err(EFAULT);
    }
    let out_ptr = unsafe { (*dev).si_drv1 as *mut Box<dyn Cdevsw> };
    if out_ptr.is_null() {
        println!("[char_ffi.rs] (*cdev).si_drv1 is null"); 
        return Err(EFAULT);
    }
    unsafe {
        Ok(&mut **out_ptr)
    }
}

/// # Safety
///
/// This function extracts a dyn Cdevsw trait object and executes its open() method
pub unsafe extern "C" fn ffi_open(
    dev: *mut cdev,
    c_oflags: c_int,
    devtype: c_int,
    td: *mut thread,
) -> c_int {
    let charDev = unsafe {
        match get_Cdevsw(dev) {
            Ok(obj) => obj,
            Err(error) => return error, 
        }
    };

    let cdevr = unsafe {&mut *dev};
    let safe_dev = Cdev::new(cdevr);

    let oflags = Oflags::convert(c_oflags);

    match charDev.open(safe_dev, oflags, devtype, td) {
        Ok(()) => 0,
        Err(error) => error,
    }
}

/// # Safety
///
/// This function extracts a dyn Cdevsw trait object and executes its close() method
pub unsafe extern "C" fn ffi_close(
    dev: *mut cdev,
    c_oflags: c_int,
    devtype: c_int,
    td: *mut thread,
) -> c_int {
    let charDev = unsafe {
        match get_Cdevsw(dev) {
            Ok(obj) => obj,
            Err(error) => return error, 
        }
    };

    let cdevr = unsafe {&mut *dev};
    let safe_dev = Cdev::new(cdevr);
    
    let oflags = Oflags::convert(c_oflags);

    match charDev.close(safe_dev, oflags, devtype, td) {
        Ok(()) => 0,
        Err(error) => error,
    }
}

/// # Safety
///
/// This function extracts a dyn Cdevsw trait object and executes its read() method
pub unsafe extern "C" fn ffi_read(
    dev: *mut cdev,
    uio_ptr: *mut uio,
    c_ioflag: c_int
) -> c_int {
    let charDev = unsafe {
        match get_Cdevsw(dev) {
            Ok(obj) => obj,
            Err(error) => return error, 
        }
    };

    if uio_ptr.is_null() {
        println!("[char_ffi.rs] uio_ptr is NULL");
        return EFAULT;
    }

    let cdevr = unsafe {&mut *dev};
    let safe_dev = Cdev::new(cdevr);

    let uior = unsafe {&mut *uio_ptr};
    let safe_uio = Uio::new(uior);

    let ioflag = Ioflag::convert(c_ioflag);

    match charDev.read(safe_dev, safe_uio, ioflag) {
        Ok(n) => n,
        Err(error) => error,
    }
}

/// # Safety
///
/// This function extracts a dyn Cdevsw trait object and executes its write() method
pub unsafe extern "C" fn ffi_write(
    dev: *mut cdev,
    uio_ptr: *mut uio,
    c_ioflag: c_int,
) -> c_int {
    let charDev = unsafe {
        match get_Cdevsw(dev) {
            Ok(obj) => obj,
            Err(error) => return error, 
        }
    };

    if uio_ptr.is_null() {
        println!("[char_ffi.rs] uio_ptr is NULL");
        return EFAULT;
    }

    let cdevr = unsafe {&mut *dev};
    let safe_dev = Cdev::new(cdevr);

    let uior = unsafe {&mut *uio_ptr};
    let safe_uio = Uio::new(uior);

    let ioflag = Ioflag::convert(c_ioflag);

    match charDev.write(safe_dev, safe_uio, ioflag) {
        Ok(n) => n,
        Err(error) => error,
    }
}
