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
                pub unsafe fn _export_register_cabi<T: Guest>() -> *mut u8 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let result0 = T::register();
                    let ptr1 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
                    let vec3 = result0;
                    let len3 = vec3.len();
                    let layout3 = _rt::alloc::Layout::from_size_align_unchecked(
                        vec3.len() * 8,
                        4,
                    );
                    let result3 = if layout3.size() != 0 {
                        let ptr = _rt::alloc::alloc(layout3).cast::<u8>();
                        if ptr.is_null() {
                            _rt::alloc::handle_alloc_error(layout3);
                        }
                        ptr
                    } else {
                        ::core::ptr::null_mut()
                    };
                    for (i, e) in vec3.into_iter().enumerate() {
                        let base = result3.add(i * 8);
                        {
                            let vec2 = (e.into_bytes()).into_boxed_slice();
                            let ptr2 = vec2.as_ptr().cast::<u8>();
                            let len2 = vec2.len();
                            ::core::mem::forget(vec2);
                            *base.add(4).cast::<usize>() = len2;
                            *base.add(0).cast::<*mut u8>() = ptr2.cast_mut();
                        }
                    }
                    *ptr1.add(4).cast::<usize>() = len3;
                    *ptr1.add(0).cast::<*mut u8>() = result3;
                    ptr1
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn __post_return_register<T: Guest>(arg0: *mut u8) {
                    let l0 = *arg0.add(0).cast::<*mut u8>();
                    let l1 = *arg0.add(4).cast::<usize>();
                    let base4 = l0;
                    let len4 = l1;
                    for i in 0..len4 {
                        let base = base4.add(i * 8);
                        {
                            let l2 = *base.add(0).cast::<*mut u8>();
                            let l3 = *base.add(4).cast::<usize>();
                            _rt::cabi_dealloc(l2, l3, 1);
                        }
                    }
                    _rt::cabi_dealloc(base4, len4 * 8, 4);
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_add_todo_cabi<T: Guest>(
                    arg0: *mut u8,
                    arg1: usize,
                ) {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let len0 = arg1;
                    let bytes0 = _rt::Vec::from_raw_parts(arg0.cast(), len0, len0);
                    T::add_todo(_rt::string_lift(bytes0));
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_todos_cabi<T: Guest>() -> *mut u8 {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let result0 = T::todos();
                    let ptr1 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
                    let vec3 = result0;
                    let len3 = vec3.len();
                    let layout3 = _rt::alloc::Layout::from_size_align_unchecked(
                        vec3.len() * 8,
                        4,
                    );
                    let result3 = if layout3.size() != 0 {
                        let ptr = _rt::alloc::alloc(layout3).cast::<u8>();
                        if ptr.is_null() {
                            _rt::alloc::handle_alloc_error(layout3);
                        }
                        ptr
                    } else {
                        ::core::ptr::null_mut()
                    };
                    for (i, e) in vec3.into_iter().enumerate() {
                        let base = result3.add(i * 8);
                        {
                            let vec2 = (e.into_bytes()).into_boxed_slice();
                            let ptr2 = vec2.as_ptr().cast::<u8>();
                            let len2 = vec2.len();
                            ::core::mem::forget(vec2);
                            *base.add(4).cast::<usize>() = len2;
                            *base.add(0).cast::<*mut u8>() = ptr2.cast_mut();
                        }
                    }
                    *ptr1.add(4).cast::<usize>() = len3;
                    *ptr1.add(0).cast::<*mut u8>() = result3;
                    ptr1
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn __post_return_todos<T: Guest>(arg0: *mut u8) {
                    let l0 = *arg0.add(0).cast::<*mut u8>();
                    let l1 = *arg0.add(4).cast::<usize>();
                    let base4 = l0;
                    let len4 = l1;
                    for i in 0..len4 {
                        let base = base4.add(i * 8);
                        {
                            let l2 = *base.add(0).cast::<*mut u8>();
                            let l3 = *base.add(4).cast::<usize>();
                            _rt::cabi_dealloc(l2, l3, 1);
                        }
                    }
                    _rt::cabi_dealloc(base4, len4 * 8, 4);
                }
                pub trait Guest {
                    /// Returns the RDX script.
                    fn load() -> _rt::String;
                    /// Register the todos() function with RDX
                    fn register() -> _rt::Vec<_rt::String>;
                    /// Increments the counter.
                    fn add_todo(todo: _rt::String);
                    /// Returns the current todos
                    fn todos() -> _rt::Vec<_rt::String>;
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
                        "component:plugin/run#register"] unsafe extern "C" fn
                        export_register() -> * mut u8 { $($path_to_types)*::
                        _export_register_cabi::<$ty > () } #[export_name =
                        "cabi_post_component:plugin/run#register"] unsafe extern "C" fn
                        _post_return_register(arg0 : * mut u8,) { $($path_to_types)*::
                        __post_return_register::<$ty > (arg0) } #[export_name =
                        "component:plugin/run#add-todo"] unsafe extern "C" fn
                        export_add_todo(arg0 : * mut u8, arg1 : usize,) {
                        $($path_to_types)*:: _export_add_todo_cabi::<$ty > (arg0, arg1) }
                        #[export_name = "component:plugin/run#todos"] unsafe extern "C"
                        fn export_todos() -> * mut u8 { $($path_to_types)*::
                        _export_todos_cabi::<$ty > () } #[export_name =
                        "cabi_post_component:plugin/run#todos"] unsafe extern "C" fn
                        _post_return_todos(arg0 : * mut u8,) { $($path_to_types)*::
                        __post_return_todos::<$ty > (arg0) } };
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
    pub use alloc_crate::alloc;
    pub use alloc_crate::vec::Vec;
    pub unsafe fn string_lift(bytes: Vec<u8>) -> String {
        if cfg!(debug_assertions) {
            String::from_utf8(bytes).unwrap()
        } else {
            String::from_utf8_unchecked(bytes)
        }
    }
    extern crate alloc as alloc_crate;
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
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 410] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\x97\x02\x01A\x02\x01\
A\x08\x01B\x02\x01r\x02\x04names\x05values\x04\0\x05event\x03\0\0\x03\0\x16compo\
nent:plugin/types\x05\0\x02\x03\0\0\x05event\x03\0\x05event\x03\0\x01\x01B\x04\x02\
\x03\x02\x01\x01\x04\0\x05event\x03\0\0\x01@\x01\x03evt\x01\x01\0\x04\0\x04emit\x01\
\x02\x03\0\x15component:plugin/host\x05\x03\x01B\x08\x01@\0\0s\x04\0\x04load\x01\
\0\x01ps\x01@\0\0\x01\x04\0\x08register\x01\x02\x01@\x01\x04todos\x01\0\x04\0\x08\
add-todo\x01\x03\x04\0\x05todos\x01\x02\x04\0\x14component:plugin/run\x05\x04\x04\
\0\x1dcomponent:plugin/plugin-world\x04\0\x0b\x12\x01\0\x0cplugin-world\x03\0\0\0\
G\x09producers\x01\x0cprocessed-by\x02\x0dwit-component\x070.220.0\x10wit-bindge\
n-rust\x060.35.0";
#[inline(never)]
#[doc(hidden)]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen_rt::maybe_link_cabi_realloc();
}
