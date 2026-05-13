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

//
// ffi helpers
//

// implement callback setter code block
macro_rules! impl_callback_setter {
    ($dest:ident, $context:expr, $callback:expr, $error:expr) => {
        translate_failures((), $error, || -> Result<()> {
            let key = $context as TegoKey;
            match get_object_map().get_mut(&key) {
                Some(TegoObject::Context(context)) => {
                    let mut callbacks = context
                        .callbacks
                        .lock()
                        .expect("another thread panicked while holding callback's mutex");
                    callbacks.$dest = $callback;
                }
                Some(_) => bail!("not a tego_context pointer: {:?}", key as *const c_void),
                None => bail!("not a valid pointer: {:?}", key as *const c_void),
            };
            Ok(())
        })
    };
}
pub(crate) use impl_callback_setter;

// implement deleter code block
macro_rules! impl_deleter {
    ($tego_object:pat, $value:expr) => {
        let key = $value as TegoKey;
        let mut object_map = get_object_map();
        if let Some($tego_object) = object_map.get(&key) {
            object_map.remove(&key);
        } else {
            panic!("");
        }
    };
}
pub(crate) use impl_deleter;

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
