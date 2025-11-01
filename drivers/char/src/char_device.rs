use kernel::{cdev, cdevsw, thread};
use kernel::{Cdev, Uio, Cdevsw};
use kernel::{ffi_open, ffi_close, ffi_read, ffi_write};
use kernel::{Read, Write};
use kernel::{make_dev_p, destroy_dev};
use kernel::{D_VERSION, UID_ROOT, GID_WHEEL};
use kernel::{MAKEDEV_WAITOK, MAKEDEV_CHECKNAME};
use kernel::{Ioflag, Oflags};
use kernel::println;

use alloc::boxed::Box;
use alloc::vec::Vec;

use libc::{c_int, EBUSY};
use core::{mem, ptr};

const BUFFERSIZE: usize = 256;

pub struct EchoDevice {
    cdevsw_ptr: *mut cdevsw,
    echo_dev: *mut cdev,
    echo_buf: Vec<u8>,
}

impl EchoDevice {
    pub fn initialize() -> Result<Box<Box<dyn Cdevsw>>, c_int> {
        let echo_buf = Vec::with_capacity(BUFFERSIZE);

        // move cdevsw to the heap using Box
        let boxed_cdevsw = Box::new(Self::cdevsw_init());

        // get raw pointer to give to kernel during cdev call
        let cdevsw_ptr: *mut cdevsw = Box::into_raw(boxed_cdevsw);

        let mut echo_dev: *mut cdev = ptr::null_mut();
        let error = unsafe {
            make_dev_p(MAKEDEV_CHECKNAME | MAKEDEV_WAITOK,
		        &mut echo_dev,
                cdevsw_ptr,
                ptr::null_mut(),
		        UID_ROOT.try_into().unwrap(),
		        GID_WHEEL.try_into().unwrap(),
		        0o666,
		        c"echo".as_ptr(),
            )
        };

        if error != 0 {
            unsafe { let _ = Box::from_raw(cdevsw_ptr); } // reclaim and free cdevsw on failure
            return Err(error);
        }

        let me = Box::new(EchoDevice{
            cdevsw_ptr,
            echo_dev,
            echo_buf,
        });
        let out: Box<Box<dyn Cdevsw>> = Box::new(me); 
        let out_ptr: *mut Box<dyn Cdevsw> = Box::into_raw(out);

        unsafe { 
            (*echo_dev).si_drv1 = out_ptr.cast()
        };
        let out: Box<Box<dyn Cdevsw>> = unsafe { Box::from_raw(out_ptr) };

        Ok(out)
    }

    fn cdevsw_init() -> cdevsw {
        cdevsw {
            d_version: D_VERSION,
            d_name: c"echo".as_ptr(),
            d_open: Some(ffi_open),
            d_close: Some(ffi_close),
            d_read: Some(ffi_read),
            d_write: Some(ffi_write),
            .. unsafe { mem::zeroed() }
        }
    }
    pub fn get_usecount(&self) -> usize {
        unsafe {
            (*self.echo_dev).si_usecount as usize  
        }
    }
}

impl Drop for EchoDevice {
    fn drop(&mut self) {
        unsafe {
            destroy_dev(self.echo_dev);
            let _ = Box::from_raw(self.cdevsw_ptr);
        }
    }
}

impl Cdevsw for EchoDevice {
    fn quiesce(&mut self) -> Result<(), c_int> {
        if self.get_usecount() != 0 {
            return Err(EBUSY);
        }
        Ok(())
    }
    fn open(&mut self, mut dev: Cdev, _oflags: Oflags, _devtype: c_int, _td: *mut thread) -> Result<(), c_int> {
        dev.cdev_ref();

        println!("[char_device.rs] echo device opened");
        Ok(())
    }

    fn close(&mut self, mut dev: Cdev, _oflags: Oflags, _devtype: c_int, _td: *mut thread) -> Result<(), c_int> {
        dev.cdev_rel();

        println!("[char_device.rs] echo device closed");
        Ok(())
    }

    fn write(&mut self, _dev: Cdev, mut safe_uio: Uio, _ioflag: Ioflag) -> Result<c_int, c_int> {
        match safe_uio.read(&mut self.echo_buf) {
            Ok(bytes) => {
                println!("[char_device.rs] {} bytes read into buffer", bytes);
                Ok(0)
            }
            Err(error) => {
                println!("[char_device.rs] {} error was returned", error);
                Err(error)
            }
        }
    }

    fn read(&mut self, _dev: Cdev, mut safe_uio: Uio, _ioflag: Ioflag) -> Result<c_int, c_int> {
        match safe_uio.write(&mut self.echo_buf) {
            Ok(bytes) => {
                println!("[char_device.rs] {} bytes written from buffer", bytes);
                Ok(0)
            }
            Err(error) => {
                println!("[char_device.rs] {} error was returned", error);
                Err(error)
            }
        }
    }
}
