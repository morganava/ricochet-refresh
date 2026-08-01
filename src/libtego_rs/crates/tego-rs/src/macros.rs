//
// Argument validation macros
//

//
macro_rules! bail_if {
    ($cond:expr) => {
        if $cond {
            anyhow::bail!(stringify!([<$cond>] must not be true));
}
    };
}
pub(crate) use bail_if;

// ensure pointer is not null
macro_rules! bail_if_null {
    ($ptr:ident) => {
        if $ptr.is_null() {
            anyhow::bail!(stringify!([<$ptr>] must not be null));
        }
    };
}
pub(crate) use bail_if_null;

// ensure pointer is null
macro_rules! bail_if_not_null {
    ($ptr:ident) => {
        if !$ptr.is_null() {
            anyhow::bail!(stringify!([<$ptr>] must be null));
        }
    };
}
pub(crate) use bail_if_not_null;

// ensure values are not equal
macro_rules! bail_if_equal {
    ($left:expr, $right:expr) => {
        if $left == $right {
            anyhow::bail!(stringify!([<$left>] must not be equal [<$right>]));
        }
    };
}
pub(crate) use bail_if_equal;

// ensure values are equal
macro_rules! bail_if_not_equal {
    ($left:expr, $right:expr) => {
        if $left != $right {
            anyhow::bail!(stringify!([<$left>] must equal [<$right>]));
        }
    };
}
pub(crate) use bail_if_not_equal;

// alwys bail
#[allow(unused_macros)]
macro_rules! bail_not_implemented {
    () => {
        anyhow::bail!("not implemented");
    };
}
#[allow(unused_imports)]
pub(crate) use bail_not_implemented;

//
// ffi helpers
//

// implement handle tags
macro_rules! impl_handle_tags {
    ($($name:ident),* $(,)?) => {
        impl_handle_tags!(@inner 1usize, $($name),*);
    };
    (@inner $counter:expr, $name:ident, $($rest:ident),* $(,)?) => {
        pub(crate) const $name: usize = $counter;
        impl_handle_tags!(@inner ($counter + 1), $($rest),*);
    };
    (@inner $counter:expr, $name:ident) => {
        pub(crate) const $name: usize = $counter;
    };
}
pub(crate) use impl_handle_tags;

// implement object storage
macro_rules! impl_object_map {
    ($ffi_type:ty, $obj_type:ty) => {
        paste::paste! {
            impl_object_map!(
                $ffi_type,
                $obj_type,
                TEGO_TAG_BITS,
                [<$ffi_type:upper _TAG>],
                [<$ffi_type:upper _MAP>],
                [<$ffi_type _map>],
                [<$ffi_type:camel Handle>]
            );
        }
    };
    (
        $ffi_type:ty,
        $obj_type:ty,
        $tag_bits:expr,
        $tag:expr,
        $static_name:ident,
        $getter_fn:ident,
        $handle_type:ident
    ) => {
        static $static_name: std::sync::Mutex<
            ffi::object_map::ObjectMap<$obj_type, $ffi_type, $tag_bits, $tag>,
        > = std::sync::Mutex::new(ffi::object_map::ObjectMap::new());

        pub(crate) fn $getter_fn<'a>() -> std::sync::MutexGuard<
            'a,
            ffi::object_map::ObjectMap<$obj_type, $ffi_type, $tag_bits, $tag>,
        > {
            $static_name
                .lock()
                .expect("another thread panicked while holding object map's mutex")
        }
        #[allow(unused)]
        pub(crate) type $handle_type = ffi::handle::Handle<$ffi_type, $tag_bits, $tag>;
    };
}
pub(crate) use impl_object_map;

// implement object deletion code block
macro_rules! impl_object_deleter {
    ($ffi_type:ident, $value:expr) => {
        paste::paste! {
            let handle = Handle::try_from($value)
                .map_err(|err| panic!("{err}"))
                .unwrap();
            [<$ffi_type _map>]()
                .remove(&handle)
                .map_err(|err| panic!("{err}"))
                .unwrap();
        }
    };
}
pub(crate) use impl_object_deleter;

// implement callback setter code block
macro_rules! impl_callback_setter {
    ($dest:ident, $context:expr, $callback:expr, $error:expr) => {
        translate_failures((), $error, || -> Result<()> {
            let context = Handle::try_from($context)?;
            let mut context_map = tego_context_map();
            let mut callbacks = context_map
                .get_mut(&context)?
                .callbacks
                .lock()
                .expect("another thread panicked while holding callback's mutex");
            callbacks.$dest = $callback;
            Ok(())
        })
    };
}
pub(crate) use impl_callback_setter;

// convert unsafe pointer+length to a str&
macro_rules! raw_to_str {
    ($ptr:expr, $len:expr) => {{
        let bytes = unsafe { std::slice::from_raw_parts($ptr as *const u8, $len) };
        std::str::from_utf8(bytes)
    }};
}
pub(crate) use raw_to_str;

//
// logging macros
//

macro_rules! log_error {
    ($($arg:tt)*) => {{
        #[cfg(feature = "logging")]
        crate::logger::Logger::log(crate::logger::LogLevel::Error, format!($($arg)*))
    }};
}
pub(crate) use log_error;

macro_rules! log_info {
    ($($arg:tt)*) => {{
        #[cfg(feature = "logging")]
        crate::logger::Logger::log(crate::logger::LogLevel::Info, format!($($arg)*))
    }};
}
pub(crate) use log_info;

#[cfg(feature = "logging")]
macro_rules! func {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let func = type_name_of(f);
        func.strip_suffix("::f").unwrap().to_string()
    }};
}
#[cfg(feature = "logging")]
pub(crate) use func;

macro_rules! log_trace {
    () => {{
        #[cfg(feature = "logging")]
        crate::logger::Logger::log(crate::logger::LogLevel::Trace, format!("{} in {}:{}", crate::macros::func!(), std::file!(), std::line!()))
    }};
    ($($arg:tt)*) => {{
        #[cfg(feature = "logging")]
        crate::logger::Logger::log(crate::logger::LogLevel::Trace, format!("{} in {}:{} {}", crate::macros::func!(), std::file!(), std::line!(), format!($($arg)*)))
    }};
}
pub(crate) use log_trace;

macro_rules! log_packet {
    ($($arg:tt)*) => {{
        #[cfg(feature = "logging")]
        crate::logger::Logger::log(crate::logger::LogLevel::Packet, format!($($arg)*))
    }};
}
pub(crate) use log_packet;

macro_rules! log_flush {
    ($($arg:tt)*) => {{
        #[cfg(feature = "logging")]
        crate::logger::Logger::flush()
    }};
}
pub(crate) use log_flush;
