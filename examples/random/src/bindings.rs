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
        #[allow(dead_code, clippy::all)]
        pub mod host {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            pub type Event = super::super::super::component::plugin::types::Event;
            #[allow(unused_unsafe, clippy::all)]
            /// emit an event.
            pub fn emit(evt: &Event) {
                unsafe {
                    let super::super::super::component::plugin::types::Event {
                        name: name0,
                        value: value0,
                    } = evt;
                    let vec1 = name0;
                    let ptr1 = vec1.as_ptr().cast::<u8>();
                    let len1 = vec1.len();
                    let vec2 = value0;
                    let ptr2 = vec2.as_ptr().cast::<u8>();
                    let len2 = vec2.len();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "component:plugin/host")]
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
            #[allow(unused_unsafe, clippy::all)]
            /// get a random byte
            pub fn random_byte() -> u8 {
                unsafe {
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "component:plugin/host")]
                    extern "C" {
                        #[link_name = "random-byte"]
                        fn wit_import() -> i32;
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import() -> i32 {
                        unreachable!()
                    }
                    let ret = wit_import();
                    ret as u8
                }
            }
        }
    }
}
#[allow(dead_code)]
pub mod exports {
    #[allow(dead_code)]
    pub mod component {
        #[allow(dead_code)]
        pub mod plugin {
            #[allow(dead_code, clippy::all)]
            pub mod run {
                #[used]
                #[doc(hidden)]
                static __FORCE_SECTION_REF: fn() = super::super::super::super::__link_custom_section_describing_imports;
                use super::super::super::super::_rt;
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
                pub unsafe fn _export_random_cabi<T: Guest>() -> *mut u8 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let result0 = T::random();
                    let ptr1 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
                    let vec2 = (result0).into_boxed_slice();
                    let ptr2 = vec2.as_ptr().cast::<u8>();
                    let len2 = vec2.len();
                    ::core::mem::forget(vec2);
                    *ptr1.add(4).cast::<usize>() = len2;
                    *ptr1.add(0).cast::<*mut u8>() = ptr2.cast_mut();
                    ptr1
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn __post_return_random<T: Guest>(arg0: *mut u8) {
                    let l0 = *arg0.add(0).cast::<*mut u8>();
                    let l1 = *arg0.add(4).cast::<usize>();
                    let base2 = l0;
                    let len2 = l1;
                    _rt::cabi_dealloc(base2, len2 * 1, 1);
                }
                pub trait Guest {
                    /// loads just the XML like markdown
                    fn load() -> _rt::String;
                    /// the random number
                    fn random() -> _rt::Vec<u8>;
                }
                #[doc(hidden)]
                macro_rules! __export_component_plugin_run_cabi {
                    ($ty:ident with_types_in $($path_to_types:tt)*) => {
                        const _ : () = { #[export_name = "component:plugin/run#load"]
                        unsafe extern "C" fn export_load() -> * mut u8 {
                        $($path_to_types)*:: _export_load_cabi::<$ty > () } #[export_name
                        = "cabi_post_component:plugin/run#load"] unsafe extern "C" fn
                        _post_return_load(arg0 : * mut u8,) { $($path_to_types)*::
                        __post_return_load::<$ty > (arg0) } #[export_name =
                        "component:plugin/run#random"] unsafe extern "C" fn
                        export_random() -> * mut u8 { $($path_to_types)*::
                        _export_random_cabi::<$ty > () } #[export_name =
                        "cabi_post_component:plugin/run#random"] unsafe extern "C" fn
                        _post_return_random(arg0 : * mut u8,) { $($path_to_types)*::
                        __post_return_random::<$ty > (arg0) } };
                    };
                }
                #[doc(hidden)]
                pub(crate) use __export_component_plugin_run_cabi;
                #[repr(align(4))]
                struct _RetArea([::core::mem::MaybeUninit<u8>; 8]);
                static mut _RET_AREA: _RetArea = _RetArea(
                    [::core::mem::MaybeUninit::uninit(); 8],
                );
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
    pub use alloc_crate::vec::Vec;
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
        $($path_to_types_root)*::
        exports::component::plugin::run::__export_component_plugin_run_cabi!($ty
        with_types_in $($path_to_types_root)*:: exports::component::plugin::run);
    };
}
#[doc(inline)]
pub(crate) use __export_plugin_world_impl as export;
#[cfg(target_arch = "wasm32")]
#[link_section = "component-type:wit-bindgen:0.35.0:component:plugin:plugin-world:encoded world"]
#[doc(hidden)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 395] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\x88\x02\x01A\x02\x01\
A\x08\x01B\x02\x01r\x02\x04names\x05values\x04\0\x05event\x03\0\0\x03\0\x16compo\
nent:plugin/types\x05\0\x02\x03\0\0\x05event\x03\0\x05event\x03\0\x01\x01B\x06\x02\
\x03\x02\x01\x01\x04\0\x05event\x03\0\0\x01@\x01\x03evt\x01\x01\0\x04\0\x04emit\x01\
\x02\x01@\0\0}\x04\0\x0brandom-byte\x01\x03\x03\0\x15component:plugin/host\x05\x03\
\x01B\x05\x01@\0\0s\x04\0\x04load\x01\0\x01p}\x01@\0\0\x01\x04\0\x06random\x01\x02\
\x04\0\x14component:plugin/run\x05\x04\x04\0\x1dcomponent:plugin/plugin-world\x04\
\0\x0b\x12\x01\0\x0cplugin-world\x03\0\0\0G\x09producers\x01\x0cprocessed-by\x02\
\x0dwit-component\x070.220.0\x10wit-bindgen-rust\x060.35.0";
#[inline(never)]
#[doc(hidden)]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen_rt::maybe_link_cabi_realloc();
}
