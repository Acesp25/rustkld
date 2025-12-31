# RustKLD Drivers
## Build process
Each driver contains two Makefile type files, a regular Makefile and a Rust Makefile.toml. The Rust Makefile.toml is coordinator of building the kld object. When calling cargo build-kmod, it executes four commands one-by-one.
```rust
[tasks.build-kmod]
description = "Full pipeline: build rust -> extract -> link .ko"
dependencies = [
  "make-obj-dir",
  "build-rust",
  "extract-objs",
  "link-kmod",
]
```

1. First we create the folder that we will store our objects in
```rust
[tasks.make-obj-dir]
description = "Creating the objects folder"
command = "mkdir"
args = ["-p", "${OBJECT_DIR}"]
```

2. Next we compile our Rust code with our target file stored in kernel/
```rust
[tasks.build-rust]
description = "Compile the Rust staticlib for the kernel"
command = "cargo"
args = [
  "build",
  "--target", "../../kernel/${TARGET}.json",
  "--release"
]
```

3. Then we extract .o files from our generated Rust .a files
```rust
[tasks.extract-objs]
description = "Extract .o files from the Rust .a"
command = "ar"
args = [
  "-xv", "../${TARGET}/release/lib${MODULE_NAME}.a",
]
cwd = "${OBJECT_DIR}"
```

4. Finally, we call upon our BSD Makefile to produce our .ko file
```rust
[tasks.link-kmod]
description = "Invoke the FreeBSD kmod makefile to produce hello.ko"
command = "make"
args = [
  "-C", ".",
  "OBJECTDIR=${OBJECT_DIR}",
  "${MODULE_NAME}.ko"
]
```

Our BSD Makefile looks as follows:
```Makefile
OBJECTDIR?=target/objects

KMOD=hello
OBJS=$(OBJECTDIR)/*.o
SRCS=../../kernel/dummy.c

.include <bsd.kmod.mk>
```
It uses our generated objects folder and a blank .c file (dummy.c) to satisfy the make requirements and build our kernel module.

## General
Both driver's lib.rs file is more or less the same. We have the unsafe function call to handle any incoming module's events and a public initialization of a moduledata_t struct used in our .c file. For example, this is how our echo driver's lib.rs looks:
```rust
/// # Safety
///
/// This function is in charge of dealing with any incomming module event
#[unsafe(no_mangle)]
pub unsafe extern "C" fn module_event(
    _mod: *mut module,
    event: i32,
    _arg: *mut c_void,
) -> c_int {
    let error: c_int = match ModEventType::from(event) {
        ModEventType::Load => {
            Events::load()
        },
        ModEventType::Unload => {
            Events::unload()
        },
        ModEventType::Quiesce => {
            Events::quiesce()
        },
        ModEventType::Shutdown => {
            Events::shutdown()
        },
        _ => {
            EOPNOTSUPP
        }
    };
    
    error
}

#[unsafe(no_mangle)]
pub static mut char_mod: moduledata_t = moduledata_t {
    name: c"CharacterDevice".as_ptr(),
    evhand: Some(module_event),
    priv_: core::ptr::null_mut(),
};

declare_module!(
    char_dev,
    char_mod,
    sysinit_sub_id_SI_SUB_DRIVERS,
    sysinit_elem_order_SI_ORDER_ANY
);
```
How the events are being handled is where everything is different. The Hello World! driver just calls simple println! statements. However, the character driver is a bit more convoluted.

## Echo Driver Design
### Creation
Whenever Event::load() is called, we attempt to call initialize() from our EchoDevice struct in our char_device.rs file. This function initializes our required structures (cdevsw, cdev) for the EchoDevice and calls make_dev_p :
```rust
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
                core::ptr::null_mut(),
		        UID_ROOT.try_into().unwrap(),
		        GID_WHEEL.try_into().unwrap(),
		        0o600,
		        c"echo".as_ptr(),
            )
        };
...
```

It then stores a double boxed dyn Cdevsw trait in (\*cdev).si_drv1. Finally, it returns a result of said double boxed dyn Cdevsw to be defined in our module_events.rs file.
```rust
...
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
...
```

### Implementation
The EchoDevice's uses are created when implementing the Cdevsw trait from our kernel/ folder. These functions are called from kernel/src/char_ffi.rs. The arguments for these functions are our safe Rust versions converted in the same char_ffi.rs file.
```rust
impl Cdevsw for EchoDevice {
    fn quiesce(&mut self) -> Result<(), c_int> {
        if self.get_usecount() != 0 {
            return Err(EBUSY);
        }
        Ok(())
    }
    fn open(&mut self, mut dev: Cdev, _oflags: Oflags, _devtype: c_int, _td: *mut thread) -> Result<(), c_int> {
        dev.cdev_ref();

        println!("[char_device.rs] character device opened");
        Ok(())
    }
...

...
    fn read(&mut self, _dev: Cdev, mut safe_uio: Uio, _ioflag: Ioflag) -> Result<c_int, c_int> {
        match safe_uio.write(&mut self.echo_buf) {
            Ok(bytes) => {
                println!("[char_device.rs] {} bytes written into buffer", bytes);
                Ok(0)
            }
            Err(error) => {
                println!("[char_device.rs] {} error was returned", error);
                Err(error)
            }
        }
    }
}
```
