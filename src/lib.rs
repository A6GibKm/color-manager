#![deny(rustdoc::broken_intra_doc_links)]
#![doc = include_str!("../README.md")]
#![cfg_attr(feature = "docs", feature(doc_auto_cfg))]

mod color_manager;
pub mod device;
mod profile;
mod scope;
mod sensor;

pub use color_manager::ColorManager;
pub use device::Device;
pub use profile::Profile;
pub use scope::Scope;
pub use sensor::Sensor;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
