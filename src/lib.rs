// OnceLock is the only dependency on std; consider conquer_once or similar crates to support no_std.
// It has to be exported since the macros using it expand in user code.
// Could also disable the need for this in single thread code to allow no_std.
// TODO: In no_std client code, the catch block is unused; consider conditionally accepting it.
pub use std::sync::OnceLock;

#[macro_export]
macro_rules! on_load {
    ($ctor_body:block) => {
        #[no_mangle]
        #[link_section = ".init_array"]
        pub static ld_preload_init: extern "C" fn() = self::ld_preload_on_load;
        extern "C" fn ld_preload_on_load() { $ctor_body }
    };
}

#[macro_export]
macro_rules! extern_c_overrides {
    (unsafe fn $c_api:ident/$real_api:ident($($param_name:ident : $param_type:ty),*) -> $return_type:ty $override_body:block catch $catch_body:block $($more_tokens:tt)*) => {
        pub unsafe fn $real_api($($param_name: $param_type),*) -> $return_type {
            #[link(name = "dl")]
            extern "C" {
                #[allow(dead_code)]
                pub fn dlsym(handle: *const libc::c_void, symbol: *const libc::c_char) -> *const libc::c_void;
            }

            #[allow(non_camel_case_types)]
            type $c_api = fn ($($param_name: $param_type),*) -> $return_type;

            #[allow(non_upper_case_globals)]
            static _dl_resolver: $crate::OnceLock<$c_api> = $crate::OnceLock::new();

            #[allow(unused)]
            let $c_api = _dl_resolver.get_or_init(|| {
                let sym = dlsym(-1isize as *const libc::c_void, concat!(stringify!($c_api), "\0").as_ptr() as *const libc::c_char);
                if sym.is_null() {
                    panic!("dlsym: Cannot get address for {}", stringify!($c_api));
                }
                return core::mem::transmute(sym);
            });
            $c_api($($param_name),*)
        }

        #[no_mangle]
        pub unsafe extern "C" fn $c_api($($param_name: $param_type),*) -> $return_type {
            let $c_api = $real_api;
            extern_c_overrides_body!($override_body, $catch_body)
        }

        extern_c_overrides! { $($more_tokens)* }
    };
    () => {};
}

#[cfg(not(feature = "std"))]
#[macro_export]
macro_rules! extern_c_overrides_body {
    ($override_body:block, $catch_body:block) => { $override_body };
}

#[cfg(feature = "std")]
#[macro_export]
macro_rules! extern_c_overrides_body {
    ($override_body:block, $catch_body:block) => {
        std::panic::catch_unwind(|| $override_body).ok().unwrap_or_else(|| $catch_body)
    };
}

