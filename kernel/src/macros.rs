// Create a null-terminated constant string at compile time
#[macro_export]
macro_rules! cstr {
    ($arg:expr) => {
        concat!($arg, '\x00')
    };
}

#[macro_export]
macro_rules! print {
	// Static (zero-allocation) implementation that uses compile-time `concat!()` only
	($fmt:expr) => ({
		let msg = $crate::cstr!($fmt);
		let ptr = msg.as_ptr() as *const $crate::libc::c_char;
        unsafe {
	        $crate::uprintf(ptr);
        };
	});

	// Dynamic implementation that processes format arguments
	($fmt:expr, $($arg:tt)*) => ({
		use ::core::fmt::Write;
		use $crate::io::KernelDebugWriter;
		let mut writer = KernelDebugWriter {};
        writer.write_fmt(format_args!($fmt, $($arg)*)).unwrap();
	});
}

// Print kernel debug messages with a trailing newline
#[macro_export]
macro_rules! println {
	($fmt:expr)              => ($crate::print!(concat!($fmt, "\n")));
	($fmt:expr, $($arg:tt)+) => ($crate::print!(concat!($fmt, "\n"), $($arg)*));
}


// Kernel Macros
#[macro_export]
macro_rules! roundup {
    ($x:expr, $y:expr) => (
        (($x) + (($y) - 1)) / ($y) * ($y)
    )
}

#[macro_export]
macro_rules! module_kernel_maxver {
    () => (
        $crate::roundup!($crate::__FreeBSD_version as i32, 100000) - 1
    )
}

#[macro_export]
macro_rules! data_set {
    ($set:ident, $sym:ident, $type:path, $section:tt) => {
        ::core::arch::global_asm!(concat!(".globl __start_set_", stringify!($set)));
        ::core::arch::global_asm!(concat!(".globl  __stop_set_", stringify!($set)));
        $crate::__paste! {
            #[used]
            #[unsafe(link_section = $section)]
            #[allow(non_camel_case_types, non_upper_case_globals)]
            pub static [<__set_ $set _sym_ $sym>]: &$type = &($sym);
        }
    }
}

#[macro_export]
macro_rules! sysinit {
    ($uniquifier:tt, $subsystem:path, $order:path, $func:path, $ident:ident) => (
        $crate::__paste! {
            #[allow(non_camel_case_types, non_upper_case_globals)]
            pub static [<$uniquifier _sys_init>]: $crate::sysinit =
                $crate::sysinit {
                    next: $crate::sysinit__bindgen_ty_1 { stqe_next: core::ptr::null_mut() },
                    subsystem: $subsystem,
                    order: $order,
                    func: Some($func),
                    udata: &$ident as *const _ as *const $crate::libc::c_void,
                };
            $crate::data_set!(sysinit_set, [<$uniquifier _sys_init>], $crate::sysinit,
                      "set_sysinit_set");
        }
    )
}

#[macro_export]
macro_rules! module_metadata {
    ($uniquifier:tt, $type:expr, $data:ident, $cval:tt) => (
        $crate::__paste! {
            #[allow(non_camel_case_types, non_upper_case_globals)]
            pub static [<_mod_metadata $uniquifier>]: $crate::mod_metadata =
                $crate::mod_metadata {
                    md_version: $crate::MDT_STRUCT_VERSION as i32,
                    md_type: $type,
                    md_cval: concat!(stringify!($cval), "\0") as *const _ as *const i8,
                    md_data: &$data as *const _ as *const $crate::libc::c_void,
                };
            $crate::data_set!(modmetadata_set, [<_mod_metadata $uniquifier>],
                      $crate::mod_metadata,
                      "set_modmetadata_set");
        }
    );
}

#[macro_export]
macro_rules! module_depend {
    ($module:tt, $mdepend:tt, $vmin:expr, $vpref:expr, $vmax:expr) => (
        $crate::__paste! {
            #[unsafe(link_section = ".data")]
            #[allow(non_camel_case_types, non_upper_case_globals)]
            pub static [<_ $module _depend_on_ $mdepend>]: $crate::mod_depend =
                $crate::mod_depend {
                    md_ver_minimum: $vmin,
                    md_ver_preferred: $vpref,
                    md_ver_maximum: $vmax,
                };
            $crate::module_metadata!([<_md_ $module _on_ $mdepend>],
                             $crate::MDT_DEPEND as i32,
                             [<_ $module _depend_on_ $mdepend>], $mdepend);
        }
    );
}

#[macro_export]
macro_rules! declare_module {
    ($name:tt, $data:ident, $sub:path, $order:path) => (
        $crate::declare_module!($name, $data, $sub,
                $order,
                $order, 
                $crate::module_kernel_maxver!());
    );
    ($name:tt, $data:ident, $sub:path, $minver:expr, $order:path, $maxver:expr) => (
        $crate::__paste! {
            $crate::module_depend!($name, kernel, 
                           $crate::__FreeBSD_version as i32, 
                           $crate::__FreeBSD_version as i32, 
                           $maxver
            );
            $crate::module_metadata!([<_md_ $name>], $crate::MDT_MODULE as i32, $data, $name);
            $crate::sysinit!([<$name _module>], $sub, $order,
                     $crate::module_register_init, $data);
        }
    )
}
