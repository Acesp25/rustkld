#![no_std]
#![no_main]

mod hello;

use kernel::declare_module;
use kernel::{module, moduledata_t, ModEventType};
use kernel::{sysinit_elem_order_SI_ORDER_ANY, sysinit_sub_id_SI_SUB_DRIVERS};

use core::ptr::null_mut;
use libc::{c_int, c_void, EOPNOTSUPP};
use hello::HelloWorld;

/// # Safety
///
/// This function is in charge of dealing with any incomming module event
#[unsafe(no_mangle)]
pub unsafe extern "C" fn module_event(
    _mod: *mut module,
    event: i32,
    _arg: *mut c_void,
) -> c_int {
    let mut error = 0;
    match ModEventType::from(event) {
        ModEventType::Load => {
            HelloWorld::load();
        },
        ModEventType::Unload => {
            HelloWorld::unload();
        },
        _ => {
            error = EOPNOTSUPP;
        }
    }
    error
}

#[unsafe(no_mangle)]
pub static hello_mod: moduledata_t = moduledata_t {
    name: c"hello".as_ptr(),
    evhand: Some(module_event),
    priv_: null_mut(),
};

declare_module!(
    hello,
    hello_mod,
    sysinit_sub_id_SI_SUB_DRIVERS,
    sysinit_elem_order_SI_ORDER_ANY
);
