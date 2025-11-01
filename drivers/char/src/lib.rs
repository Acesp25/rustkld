#![no_std]
#![no_main]

mod module_events;
mod char_device;

use kernel::declare_module;
use kernel::{module, ModEventType, moduledata_t};
use kernel::{sysinit_elem_order_SI_ORDER_ANY, sysinit_sub_id_SI_SUB_DRIVERS};

use core::ptr::null_mut;
use libc::{c_int, c_void, EOPNOTSUPP};
use module_events::Events;

extern crate alloc;

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
pub static char_mod: moduledata_t = moduledata_t {
    name: c"CharacterDevice".as_ptr(),
    evhand: Some(module_event),
    priv_: null_mut(),
};

declare_module!(
    char_dev,
    char_mod,
    sysinit_sub_id_SI_SUB_DRIVERS,
    sysinit_elem_order_SI_ORDER_ANY
);
