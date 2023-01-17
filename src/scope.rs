use serde::{Deserialize, Serialize};
use zbus::zvariant::{self, Type};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[zvariant(signature = "s")]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    Normal,
    Temp,
    Disk,
}

impl Default for Scope {
    fn default() -> Self {
        Self::Normal
    }
}

impl From<zvariant::OwnedValue> for Scope {
    fn from(value: zvariant::OwnedValue) -> Self {
        match value.downcast_ref::<zvariant::Str>().unwrap().as_str() {
            "temp" => Self::Temp,
            "disk" => Self::Disk,
            _ => Self::default(),
        }
    }
}
