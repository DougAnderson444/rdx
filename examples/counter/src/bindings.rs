pub type Event = component::plugin::types::Event;
#[allow(unused_unsafe, clippy::all)]
/// Import the event handler.
pub fn emit(evt: &Event) {
    unsafe {
        let component::plugin::types::Event { name: name0, value: value0 } = evt;
        let vec1 = name0;
        let ptr1 = vec1.as_ptr().cast::<u8>();
        let len1 = vec1.len();
        let vec2 = value0;
        let ptr2 = vec2.as_ptr().cast::<u8>();
        let len2 = vec2.len();
        #[cfg(target_arch = "wasm32")]
        #[link(wasm_import_module = "$root")]
        extern "C" {
            #[link_name = "emit"]
            fn wit_import(_: *mut u8, _: usize, _: *mut u8, _: usize);
        }
        #[cfg(not(target_arch = "wasm32"))]
        fn wit_import(_: *mut u8, _: usize, _: *mut u8, _: usize) {
            unreachable!()
        }
        wit_import(ptr1.cast_mut(), len1, ptr2.cast_mut(), len2);
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn _export_load_cabi<T: Guest>() -> *mut u8 {
    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
    let result0 = T::load();
    let ptr1 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
    let vec2 = (result0.into_bytes()).into_boxed_slice();
    let ptr2 = vec2.as_ptr().cast::<u8>();
    let len2 = vec2.len();
    ::core::mem::forget(vec2);
    *ptr1.add(4).cast::<usize>() = len2;
    *ptr1.add(0).cast::<*mut u8>() = ptr2.cast_mut();
    ptr1
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn __post_return_load<T: Guest>(arg0: *mut u8) {
    let l0 = *arg0.add(0).cast::<*mut u8>();
    let l1 = *arg0.add(4).cast::<usize>();
    _rt::cabi_dealloc(l0, l1, 1);
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn _export_increment_cabi<T: Guest>() -> i32 {
    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
    let result0 = T::increment();
    _rt::as_i32(result0)
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn _export_decrement_cabi<T: Guest>() -> i32 {
    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
    let result0 = T::decrement();
    _rt::as_i32(result0)
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn _export_current_cabi<T: Guest>() -> i32 {
    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
    let result0 = T::current();
    _rt::as_i32(result0)
}
pub trait Guest {
    /// Returns the RDX script.
    fn load() -> _rt::String;
    /// Increments the counter.
    fn increment() -> i32;
    /// Decrements the counter.
    fn decrement() -> i32;
    /// Returns the current counter value.
    fn current() -> i32;
}
#[doc(hidden)]
macro_rules! __export_world_plugin_world_cabi {
    ($ty:ident with_types_in $($path_to_types:tt)*) => {
        const _ : () = { #[export_name = "load"] unsafe extern "C" fn export_load() -> *
        mut u8 { $($path_to_types)*:: _export_load_cabi::<$ty > () } #[export_name =
        "cabi_post_load"] unsafe extern "C" fn _post_return_load(arg0 : * mut u8,) {
        $($path_to_types)*:: __post_return_load::<$ty > (arg0) } #[export_name =
        "increment"] unsafe extern "C" fn export_increment() -> i32 {
        $($path_to_types)*:: _export_increment_cabi::<$ty > () } #[export_name =
        "decrement"] unsafe extern "C" fn export_decrement() -> i32 {
        $($path_to_types)*:: _export_decrement_cabi::<$ty > () } #[export_name =
        "current"] unsafe extern "C" fn export_current() -> i32 { $($path_to_types)*::
        _export_current_cabi::<$ty > () } };
    };
}
#[doc(hidden)]
pub(crate) use __export_world_plugin_world_cabi;
#[repr(align(4))]
struct _RetArea([::core::mem::MaybeUninit<u8>; 8]);
static mut _RET_AREA: _RetArea = _RetArea([::core::mem::MaybeUninit::uninit(); 8]);
#[allow(dead_code)]
pub mod component {
    #[allow(dead_code)]
    pub mod plugin {
        #[allow(dead_code, clippy::all)]
        pub mod types {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            /// The Event type.
            #[derive(Clone)]
            pub struct Event {
                /// The variable name
                pub name: _rt::String,
                pub value: _rt::String,
            }
            impl ::core::fmt::Debug for Event {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("Event")
                        .field("name", &self.name)
                        .field("value", &self.value)
                        .finish()
                }
            }
        }
    }
}
mod _rt {
    pub use alloc_crate::string::String;
    #[cfg(target_arch = "wasm32")]
    pub fn run_ctors_once() {
        wit_bindgen_rt::run_ctors_once();
    }
    pub unsafe fn cabi_dealloc(ptr: *mut u8, size: usize, align: usize) {
        if size == 0 {
            return;
        }
        let layout = alloc::Layout::from_size_align_unchecked(size, align);
        alloc::dealloc(ptr, layout);
    }
    pub fn as_i32<T: AsI32>(t: T) -> i32 {
        t.as_i32()
    }
    pub trait AsI32 {
        fn as_i32(self) -> i32;
    }
    impl<'a, T: Copy + AsI32> AsI32 for &'a T {
        fn as_i32(self) -> i32 {
            (*self).as_i32()
        }
    }
    impl AsI32 for i32 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for u32 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for i16 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for u16 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for i8 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for u8 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for char {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for usize {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    extern crate alloc as alloc_crate;
    pub use alloc_crate::alloc;
}
/// Generates `#[no_mangle]` functions to export the specified type as the
/// root implementation of all generated traits.
///
/// For more information see the documentation of `wit_bindgen::generate!`.
///
/// ```rust
/// # macro_rules! export{ ($($t:tt)*) => (); }
/// # trait Guest {}
/// struct MyType;
///
/// impl Guest for MyType {
///     // ...
/// }
///
/// export!(MyType);
/// ```
#[allow(unused_macros)]
#[doc(hidden)]
macro_rules! __export_plugin_world_impl {
    ($ty:ident) => {
        self::export!($ty with_types_in self);
    };
    ($ty:ident with_types_in $($path_to_types_root:tt)*) => {
        $($path_to_types_root)*:: __export_world_plugin_world_cabi!($ty with_types_in
        $($path_to_types_root)*);
    };
}
#[doc(inline)]
pub(crate) use __export_plugin_world_impl as export;
#[cfg(target_arch = "wasm32")]
#[link_section = "component-type:wit-bindgen:0.30.0:plugin-world:encoded world"]
#[doc(hidden)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 327] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\xc4\x01\x01A\x02\x01\
A\x0c\x01B\x02\x01r\x02\x04names\x05values\x04\0\x05event\x03\0\0\x03\x01\x16com\
ponent:plugin/types\x05\0\x02\x03\0\0\x05event\x03\0\x05event\x03\0\x01\x01@\x01\
\x03evt\x02\x01\0\x03\0\x04emit\x01\x03\x01@\0\0s\x04\0\x04load\x01\x04\x01@\0\0\
z\x04\0\x09increment\x01\x05\x04\0\x09decrement\x01\x05\x04\0\x07current\x01\x05\
\x04\x01\x1dcomponent:plugin/plugin-world\x04\0\x0b\x12\x01\0\x0cplugin-world\x03\
\0\0\0G\x09producers\x01\x0cprocessed-by\x02\x0dwit-component\x070.215.0\x10wit-\
bindgen-rust\x060.30.0";
#[inline(never)]
#[doc(hidden)]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen_rt::maybe_link_cabi_realloc();
}
