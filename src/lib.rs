//! # WMI-rs
//!
//! [WMI] is a management API for Windows-based operating systems.
//! This crate provides a high level Rust API focused around data retrieval (vs. making changes to
//! the system and watching for event which are also supported by WMI).
//!
//! This crate also uses `serde` to transform pointers to WMI class objects into plain Rust structs.
//!
//! All data is copied to Owning data structures, so the final structs are not tied in any way to
//! the original WMI object (refer to MSDN's [Creating a WMI Application Using C++] to learn more about how data is handled by WMI).
//!
//! Before using WMI, a connection must be created.
//!
//! ```rust
//! use wmi::{COMLibrary, WMIConnection};
//! let com_con = COMLibrary::new().unwrap();
//! let wmi_con = WMIConnection::new(com_con.into()).unwrap();
//! ```
//!
//! There are multiple ways to get data from the OS using this crate.
//!
//! # Operating on untyped Variants
//!
//! WMI data model is based on COM's [`VARIANT`] Type, which is a struct capable of holding
//! many types of data.
//!
//! This crate provides the analogous [`Variant`][Variant] enum.
//!
//! Using this enum, we can execute a simple WMI query and inspect the results.
//!
//! ```edition2018
//! # use wmi::*;
//! # let wmi_con = WMIConnection::new(COMLibrary::new().unwrap().into()).unwrap();
//! use std::collections::HashMap;
//! use wmi::Variant;
//! let results: Vec<HashMap<String, Variant>> = wmi_con.raw_query("SELECT * FROM Win32_OperatingSystem").unwrap();
//!
//! for os in results {
//!     println!("{:#?}", os);
//! }
//! ```
//!
//! # Using strongly typed data structures
//!
//! Using `serde`, it is possible to return a struct representing the the data.
//!
//! ```edition2018
//! # use wmi::*;
//! # let wmi_con = WMIConnection::new(COMLibrary::new().unwrap().into()).unwrap();
//! use serde::Deserialize;
//! use wmi::WMIDateTime;
//!
//! #[derive(Deserialize, Debug)]
//! #[serde(rename = "Win32_OperatingSystem")]
//! #[serde(rename_all = "PascalCase")]
//! struct OperatingSystem {
//!     caption: String,
//!     debug: bool,
//!     last_boot_up_time: WMIDateTime,
//! }
//!
//! let results: Vec<OperatingSystem> = wmi_con.query().unwrap();
//!
//! for os in results {
//!     println!("{:#?}", os);
//! }
//! ```
//!
//! Because the name of the struct given to `serde` matches the [WMI class] name, the SQL query
//! is inferred.
//!
//! [WMI]: https://docs.microsoft.com/en-us/windows/desktop/wmisdk/about-wmi
//! [Creating a WMI Application Using C++]: https://docs.microsoft.com/en-us/windows/desktop/wmisdk/creating-a-wmi-application-using-c-
//! [`VARIANT`]: https://docs.microsoft.com/en-us/windows/desktop/api/oaidl/ns-oaidl-tagvariant
//! [WMI class]: https://docs.microsoft.com/en-us/windows/desktop/cimwin32prov/win32-operatingsystem
//!
//! # Internals
//!
//! [`WMIConnection`](WMIConnection) is used to create and execute a WMI query, returning
//! [`IWbemClassWrapper`](query::IWbemClassWrapper) which is a wrapper for a WMI object pointer.
//!
//! Then, [`from_wbem_class_obj`](de::wbem_class_de::from_wbem_class_obj) is used to create a Rust struct with the equivalent data.
//!
//! Deserializing data from WMI and into Rust is done via `serde` and is implemented in the [`de`][de] module.
//! More info can be found in `serde`'s documentation about [writing a data format].
//! The deserializer will either use the field names defined on the output struct,
//! or retrieve all field names from WMI if the ouput is a `HashMap`.
//!
//! [writing a data format]: https://serde.rs/data-format.html
//!
//! There are two main data structures (other than pointers to object) which convert native data to Rust data structures:
//! [`Variant`](Variant) and [`SafeArrayAccessor`](safearray::SafeArrayAccessor).
//!
//! Most native objects has an equivalent wrapper struct which implements `Drop` for that data.
//!
//!
pub mod connection;
pub mod datetime;
pub mod de;
pub mod error;
pub mod query;
pub mod result_enumerator;
pub mod safearray;
pub mod utils;
pub mod variant;

#[cfg(any(test, feature = "test"))]
pub mod tests;

pub use connection::{COMLibrary, WMIConnection};
pub use datetime::WMIDateTime;
pub use variant::Variant;
