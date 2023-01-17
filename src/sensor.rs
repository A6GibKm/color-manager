use std::collections::HashMap;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use zbus::{
    zvariant::{ObjectPath, Type, Value},
    Result,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[zvariant(signature = "s")]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Ambient,
    Printer,
    Unknown,
}

impl From<zbus::zvariant::OwnedValue> for Mode {
    fn from(value: zbus::zvariant::OwnedValue) -> Self {
        match value
            .downcast_ref::<zbus::zvariant::Str>()
            .unwrap()
            .as_str()
        {
            "ambient" => Self::Ambient,
            "printer" => Self::Printer,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Type)]
#[zvariant(signature = "s")]
#[serde(rename_all = "lowercase")]
pub enum Capability {
    Crt,
    Ambient,
    Lcd,
    Led,
    Projector,
}

#[derive(Type, Debug)]
#[zvariant(signature = "o")]
#[doc(alias = "org.freedesktop.ColorManager.Sensor")]
/// A wrapper of the `org.freedesktop.ColorManager.Sensor` DBus object.
pub struct Sensor<'a>(zbus::Proxy<'a>);

impl<'a> Sensor<'a> {
    pub async fn new<P>(connection: &zbus::Connection, object_path: P) -> Result<Sensor<'a>>
    where
        P: TryInto<ObjectPath<'a>>,
        P::Error: Into<zbus::Error>,
    {
        let inner = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.ColorManager.Sensor")?
            .path(object_path)?
            .destination("org.freedesktop.ColorManager")?
            .cache_properties(zbus::CacheProperties::No)
            .build()
            .await?;
        Ok(Self(inner))
    }

    pub(crate) async fn from_paths<P>(
        connection: &zbus::Connection,
        paths: Vec<P>,
    ) -> Result<Vec<Sensor<'a>>>
    where
        P: TryInto<ObjectPath<'a>>,
        P::Error: Into<zbus::Error>,
    {
        let mut items = Vec::with_capacity(paths.capacity());
        for path in paths.into_iter() {
            items.push(Self::new(connection, path).await?);
        }
        Ok(items)
    }

    pub fn inner(&self) -> &zbus::Proxy {
        &self.0
    }

    #[doc(alias = "Lock")]
    /// Locks the sensor for use by an application.
    ///
    /// If the current holder of the lock quits without calling Unlock then it
    /// is automatically removed.
    pub async fn lock(&self) -> Result<()> {
        self.inner().call_method("Lock", &()).await?;

        Ok(())
    }

    #[doc(alias = "Unlock")]
    /// Unlocks the sensor for use by other applications.
    pub async fn unlock(&self) -> Result<()> {
        self.inner().call_method("Unlock", &()).await?;

        Ok(())
    }

    #[doc(alias = "GetSample")]
    /// Gets a color sample using the sensor.
    pub async fn sample(&self, capability: Capability) -> Result<(f64, f64, f64)> {
        let msg = self.inner().call_method("GetSample", &(capability)).await?;

        msg.body()
    }

    #[doc(alias = "GetSpectrum")]
    /// Gets a color spectrum using the sensor.
    pub async fn spectrum(&self, capability: Capability) -> Result<(f64, f64, Vec<f64>)> {
        let msg = self
            .inner()
            .call_method("GetSpectrum", &(capability))
            .await?;

        msg.body()
    }

    #[doc(alias = "SetOptions")]
    /// Sets one or multiple options on the sensor.
    pub async fn set_options<V: Into<Value<'a>>>(&self, values: HashMap<&str, V>) -> Result<()> {
        let map = values
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect::<HashMap<&str, Value<'a>>>();
        self.inner().call_method("SetOptions", &(map)).await?;

        Ok(())
    }

    #[doc(alias = "ButtonPressed")]
    /// A button on the sensor has been pressed.
    pub async fn button_pressed(&self) -> Result<()> {
        let mut stream = self.inner().receive_signal("ButtonPressed").await?;
        stream
            .next()
            .await
            .ok_or(zbus::Error::Failure("No response".into()))?;

        Ok(())
    }

    #[doc(alias = "SensorId")]
    /// The sensor id string.
    pub async fn sensor_id(&self) -> Result<String> {
        self.inner().get_property("SensorId").await
    }

    // TODO Use enum?.
    #[doc(alias = "Kind")]
    /// The kind of the sensor, e.g. `colormunki`
    pub async fn kind(&self) -> Result<String> {
        self.inner().get_property("Kind").await
    }

    #[doc(alias = "State")]
    /// The state of the sensor, e.g. `starting`, `idle` or `measuring`.
    pub async fn state(&self) -> Result<String> {
        self.inner().get_property("State").await
    }

    #[doc(alias = "Mode")]
    /// The operating mode of the sensor, e.g. ambient, printer or unknown.
    ///
    /// On some devices, a sensor has to be set to a specific position before a
    /// reading can be taken. This property should be set to the current device
    /// mode.
    pub async fn mode(&self) -> Result<Mode> {
        self.inner().get_property::<Mode>("Mode").await
    }

    #[doc(alias = "Serial")]
    /// The sensor serial number, e.g. `012345678a`.
    pub async fn serial(&self) -> Result<String> {
        self.inner().get_property("Serial").await
    }

    #[doc(alias = "Model")]
    /// The sensor model, e.g. `ColorMunki`.
    pub async fn model(&self) -> Result<String> {
        self.inner().get_property("Model").await
    }

    #[doc(alias = "Vendor")]
    /// The sensor vendor, e.g. `XRite`.
    pub async fn vendor(&self) -> Result<String> {
        self.inner().get_property("Vendor").await
    }

    #[doc(alias = "Native")]
    /// If the sensor is supported with a native driver, which does not require
    /// additional tools such as argyllcms.
    pub async fn native(&self) -> Result<bool> {
        self.inner().get_property("Native").await
    }

    #[doc(alias = "Locked")]
    /// If the sensor is locked for use by colord.
    pub async fn locked(&self) -> Result<bool> {
        self.inner().get_property("Locked").await
    }

    #[doc(alias = "Capabilities")]
    /// The capabilities of the sensor, e.g `['display', 'printer', 'projector',
    /// 'spot']`.
    pub async fn capabilities(&self) -> Result<Vec<String>> {
        self.inner().get_property("Capabilities").await
    }

    #[doc(alias = "Metadata")]
    /// The metadata for the sensor, which may include optional keys like
    /// `AttachImage`.
    pub async fn metadata(&self) -> Result<HashMap<String, String>> {
        self.inner().get_property("Metadata").await
    }
}

impl<'a> Serialize for Sensor<'a> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ObjectPath::serialize(self.inner().path(), serializer)
    }
}
