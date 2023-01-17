use std::collections::HashMap;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use zbus::{
    zvariant::{ObjectPath, OwnedObjectPath, SerializeDict, Type},
    Result,
};

use crate::{Profile, Scope};

// TODO Use PascalCase
#[derive(SerializeDict, Type, Default)]
#[zvariant(signature = "dict")]
struct DeviceProperties<'a> {
    created: u64,
    modified: u64,
    model: String,
    serial: String,
    vendor: String,
    colorspace: String,
    kind: String,
    device_id: String,
    profiles: Vec<Profile<'a>>,
    mode: Mode,
    format: String,
    scope: Scope,
    owner: u32,
    enabled: bool,
    seat: String,
    embedded: bool,
    metadata: HashMap<String, String>,
    profiling_inhibitors: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Type)]
#[zvariant(signature = "s")]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Scanner,
    Display,
    Camera,
    Printer,
    Webcam,
}

impl From<zbus::zvariant::OwnedValue> for Kind {
    fn from(value: zbus::zvariant::OwnedValue) -> Self {
        match value
            .downcast_ref::<zbus::zvariant::Str>()
            .unwrap()
            .as_str()
        {
            "scanner" => Self::Scanner,
            "Camera" => Self::Camera,
            _ => Self::Display,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[zvariant(signature = "s")]
#[serde(rename_all = "lowercase")]
pub enum Relation {
    Soft,
    Hard,
}

impl Default for Relation {
    fn default() -> Self {
        Self::Hard
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[zvariant(signature = "s")]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Virtual,
    Physical,
    Unknown,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Unknown
    }
}

impl From<zbus::zvariant::OwnedValue> for Mode {
    fn from(value: zbus::zvariant::OwnedValue) -> Self {
        match value
            .downcast_ref::<zbus::zvariant::Str>()
            .unwrap()
            .as_str()
        {
            "virtual" => Self::Virtual,
            "physical" => Self::Physical,
            _ => Self::default(),
        }
    }
}

#[derive(Type, Debug)]
#[zvariant(signature = "o")]
#[doc(alias = "org.freedesktop.ColorManager.Device")]
/// A wrapper of the `org.freedesktop.ColorManager.Device` DBus object.
pub struct Device<'a>(zbus::Proxy<'a>);

impl<'a> Device<'a> {
    pub async fn new<P>(connection: &zbus::Connection, object_path: P) -> Result<Device<'a>>
    where
        P: TryInto<ObjectPath<'a>>,
        P::Error: Into<zbus::Error>,
    {
        let inner = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.ColorManager.Device")?
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
    ) -> Result<Vec<Device<'a>>>
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

    #[doc(alias = "SetProperty")]
    /// Sets a property on the object.
    pub async fn set_property(&self, property_name: &str, property_value: &str) -> Result<()> {
        self.inner()
            .call_method("SetProperty", &(property_name, property_value))
            .await?;

        Ok(())
    }

    #[doc(alias = "AddProfile")]
    /// Adds a profile to the device. The profile must have been previously
    /// created.
    ///
    /// This method also stores the device to profile mapping in a persistent
    /// datadase, so that if the device and profile happen to both exist in the
    /// future, the profiles are auto-added to the device.
    pub async fn add_profile(&self, relation: Relation, profile: &Profile<'_>) -> Result<()> {
        self.inner()
            .call_method("AddProfile", &(relation, profile))
            .await?;

        Ok(())
    }

    #[doc(alias = "RemoveProfile")]
    /// Removes a profile for a device.
    ///
    /// This method also removes the device to profile mapping from a persistent
    /// datadase, so that if the device and profile happen to both exist in the
    /// future, the profiles are no longer auto-added to the device.
    ///
    /// If the profile was automatically added due to metadata in the profile
    /// (e.g. the profile was created for the device) then manually removing the
    /// profile will cause this metadata add to be supressed. This allows the
    /// user to remove old or obsolete profiles from any color control panel
    /// without having to delete them.
    pub async fn remove_profile(&self, profile: &Profile<'_>) -> Result<()> {
        self.inner()
            .call_method("RemoveProfile", &(profile))
            .await?;

        Ok(())
    }

    #[doc(alias = "MakeProfileDefault")]
    /// Sets the default profile for a device.
    pub async fn make_profile_default(&self, profile: &Profile<'_>) -> Result<()> {
        self.inner()
            .call_method("MakeProfileDefault", &(profile))
            .await?;

        Ok(())
    }

    #[doc(alias = "GetProfileForQualifiers")]
    /// Gets a single profile object path for a qualifier.
    ///
    /// The search term can contain `*` and `?` wildcards.
    pub async fn profile_for_qualifiers(&self, qualifiers: &[&str]) -> Result<Profile> {
        let msg = self
            .inner()
            .call_method("GetProfileForQualifiers", &(qualifiers))
            .await?;
        let reply = msg.body::<OwnedObjectPath>()?;

        Profile::new(self.inner().connection(), reply).await
    }

    #[doc(alias = "GetProfileRelation")]
    /// Gets a single profile object path for a qualifier.
    ///
    /// The search term can contain `*` and `?` wildcards.
    pub async fn profile_relation(&self, profile: &Profile<'_>) -> Result<Relation> {
        let msg = self
            .inner()
            .call_method("GetProfileRelation", &(profile))
            .await?;

        msg.body()
    }

    #[doc(alias = "ProfilingInhibit")]
    /// Adds an inhibit on all profiles for this device.
    ///
    /// This means that any calls to GetProfileForQualifier will always match no
    /// profiles.
    ///
    /// This method will be used when creating profiles for devices, where the
    /// session color manager wants to be very sure that no profiles are being
    /// applied wen displaying color samples or printing color swatches.
    ///
    /// If the calling program exits without calling `ProfilingUninhibit` then
    /// the inhibit is automatically removed.
    pub async fn profiling_inhibit(&self) -> Result<()> {
        self.inner().call_method("ProfilingInhibit", &()).await?;

        Ok(())
    }

    #[doc(alias = "ProfilingUninhibit")]
    /// Removes an inhibit on the device.
    ///
    /// This method should be used when profiling has finished and normal device
    /// matching behaviour should resume.
    pub async fn profiling_uninhibit(&self) -> Result<()> {
        self.inner().call_method("ProfilingUninhibit", &()).await?;

        Ok(())
    }

    #[doc(alias = "SetEnabled")]
    /// Sets the device enable state.
    pub async fn set_enabled(&self, enabled: bool) -> Result<()> {
        self.inner().call_method("SetEnabled", &(enabled)).await?;

        Ok(())
    }

    #[doc(alias = "Changed")]
    /// Some value on the interface has changed.
    pub async fn changed(&self) -> Result<()> {
        let mut stream = self.inner().receive_signal("Changed").await?;
        stream
            .next()
            .await
            .ok_or(zbus::Error::Failure("No response".into()))?;

        Ok(())
    }

    #[doc(alias = "Created")]
    /// The date the device was created.
    pub async fn created(&self) -> Result<u64> {
        self.inner().get_property("Created").await
    }

    #[doc(alias = "Modified")]
    /// The date the device was created.
    pub async fn modified(&self) -> Result<u64> {
        self.inner().get_property("Modified").await
    }

    #[doc(alias = "Model")]
    /// The device model string.
    pub async fn model(&self) -> Result<String> {
        self.inner().get_property("Model").await
    }

    #[doc(alias = "Serial")]
    /// The device serial string.
    pub async fn serial(&self) -> Result<String> {
        self.inner().get_property("Serial").await
    }

    #[doc(alias = "Vendor")]
    /// The device vendor string.
    pub async fn vendor(&self) -> Result<String> {
        self.inner().get_property("Vendor").await
    }

    #[doc(alias = "Colorspace")]
    /// The device colorspace string.
    pub async fn colorspace(&self) -> Result<String> {
        self.inner().get_property("Colorspace").await
    }

    #[doc(alias = "Kind")]
    /// The device kind string.
    pub async fn kind(&self) -> Result<Kind> {
        self.inner().get_property("Kind").await
    }

    #[doc(alias = "DeviceId")]
    /// The device id string.
    pub async fn device_id(&self) -> Result<String> {
        self.inner().get_property("DeviceId").await
    }

    #[doc(alias = "Profiles")]
    /// The profile paths associated with this device.
    ///
    /// Profiles are returned even if the device is disabled or is profiling,
    /// and clients should not assume that the first profile in this array
    /// should be applied.
    pub async fn profiles(&self) -> Result<Vec<Profile<'static>>> {
        let reply = self
            .inner()
            .get_property::<Vec<OwnedObjectPath>>("Profiles")
            .await?;

        Profile::from_paths(self.inner().connection(), reply).await
    }

    #[doc(alias = "Mode")]
    /// The mode of the device, e.g. `virtual`, `physical` or `unknown`.
    ///
    /// Virtual devices are not tied to a specific item of hardware and can
    /// represent abstract devices such as "Boots Photo Lab".
    ///
    /// Physical devices correspond to a connected device that cannot be removed
    /// by client software.
    ///
    /// If a virtual 'disk' device gets added by a client then it is promoted to
    /// a 'physical' device. This can happen if a printer is saved and then
    /// restored at next boot before the CUPS daemon is running.
    pub async fn mode(&self) -> Result<Mode> {
        self.inner().get_property::<Mode>("Mode").await
    }

    // TODO Is this an enum?
    #[doc(alias = "Format")]
    /// The qualifier format for the device, e.g.
    /// `ColorModel.OutputMode.OutputResolution`.
    pub async fn format(&self) -> Result<String> {
        self.inner().get_property("Format").await
    }

    #[doc(alias = "Scope")]
    /// The scope of the device.
    pub async fn scope(&self) -> Result<Scope> {
        self.inner().get_property("Scope").await
    }

    #[doc(alias = "Owner")]
    /// The user ID of the account that created the device.
    pub async fn owner(&self) -> Result<u32> {
        self.inner().get_property("Owner").await
    }

    #[doc(alias = "Enabled")]
    /// If the device is enabled.
    ///
    /// Devices are enabled by default until `Device.SetEnabled(False)` is
    /// called. If the enabled state is changed then this is reflected for all
    /// users and persistent across reboots.
    pub async fn enabled(&self) -> Result<bool> {
        self.inner().get_property("Enabled").await
    }

    #[doc(alias = "Seat")]
    /// The seat that the device belongs to, or an empty string for none or
    /// unknown.
    pub async fn seat(&self) -> Result<String> {
        self.inner().get_property("Seat").await
    }

    #[doc(alias = "Embedded")]
    /// If the device is embedded into the hardware itself, for example the
    /// internal webcam or laptop screen.
    pub async fn embedded(&self) -> Result<String> {
        self.inner().get_property("Embedded").await
    }

    #[doc(alias = "Metadata")]
    /// The metadata for the device, which may include optional keys like
    /// `XRANDR_name`.
    pub async fn metadata(&self) -> Result<HashMap<String, String>> {
        self.inner().get_property("Metadata").await
    }

    #[doc(alias = "ProfilingInhibitors")]
    /// The bus names of all the clients that have inhibited the device for
    /// profiling. e.g. `[ ":1.99", ":1.109" ]`.
    pub async fn profiling_inhibitors(&self) -> Result<Vec<String>> {
        self.inner().get_property("ProfilingInhibitors").await
    }
}

impl<'a> Serialize for Device<'a> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ObjectPath::serialize(self.inner().path(), serializer)
    }
}
