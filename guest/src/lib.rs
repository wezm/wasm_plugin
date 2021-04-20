#![doc(html_root_url = "https://docs.rs/wasm_plugin_guest/0.1.3")]
#![deny(missing_docs)]

//! A low-ish level tool for easily writing WASM based plugins to be hosted by
//! wasm_plugin_host.
//!
//! The goal of wasm_plugin is to make communicating across the host-plugin
//! boundary as simple and idiomatic as possible while being unopinionated
//! about how you actually use the plugin.
//!
//! This crate currently supports serialization either using bincode or json
//! selected by feature:
//! `serialize_bincode`: Uses serde and bincode. It is selected by default.
//! `serialize_json`: Uses serde and serde_json.
//! `serialize_nanoserde_json': Uses nanoserde.
//!
//! Bincode is likely the best choice if all plugins the system uses will be
//! written in Rust. Json is useful if a mix or languages will be used.
//!
//! Plugins are meant to be run using [wasm_plugin_host](https://crates.io/crates/wasm_plugin_host)

mod serialization;
pub use wasm_plugin_guest_derive::{export_function, import_functions};

#[no_mangle]
static mut MESSAGE_BUFFER: [u8; 1024 * 10] = [0; 1024 * 10];

/// Read a message from the buffer used to communicate with the host. You should
/// never need to call this directly.
pub fn read_message<T: serialization::Deserializable>() -> T {
    let buf = unsafe { &mut MESSAGE_BUFFER };
    T::deserialize(buf)
}

/// Write a message to the buffer used to communicate with the host. You should
/// never need to call this directly.
pub fn write_message<U>(message: &U) -> i32
where
    U: serialization::Serializable,
{
    let buf = unsafe { &mut MESSAGE_BUFFER };
    let message: Vec<u8> = message.serialize();
    let len = message.len();
    buf[0..len].copy_from_slice(&message);
    len as i32
}

#[cfg(feature = "inject_getrandom")]
mod getrandom_shim {
    use getrandom::register_custom_getrandom;

    use getrandom::Error;

    extern "C" {
        fn __getrandom(ptr: i32, len: i32);
    }

    #[allow(clippy::unnecessary_wraps)]
    fn external_getrandom(buf: &mut [u8]) -> Result<(), Error> {
        let len = buf.len();
        let ptr = buf.as_ptr();
        unsafe {
            __getrandom(ptr as i32, len as i32);
        }
        Ok(())
    }
    register_custom_getrandom!(external_getrandom);
}
