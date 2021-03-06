#![doc(html_root_url = "https://docs.rs/rincon_session_async/0.1.0")]
#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
//    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
)]

extern crate futures;
extern crate serde;

extern crate rincon_client;
extern crate rincon_core;

mod async;
pub use self::async::*;
