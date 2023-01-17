use std::collections::HashMap;

use futures_util::StreamExt;
use serde::Serialize;
use zbus::{
    zvariant::{ObjectPath, Type},
    Result,
};

use crate::Scope;

#[derive(Type, Debug)]
#[zvariant(signature = "o")]
#[doc(alias = "org.freedesktop.ColorManager.Profile")]
/// A wrapper of the `org.freedesktop.ColorManager.Profile` DBus object.
pub struct Profile<'a>(zbus::Proxy<'a>);

impl<'a> Profile<'a> {
    pub async fn new<P>(connection: &zbus::Connection, object_path: P) -> Result<Profile<'a>>
    where
        P: TryInto<ObjectPath<'a>>,
        P::Error: Into<zbus::Error>,
    {
        let inner = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.ColorManager.Profile")?
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
    ) -> Result<Vec<Profile<'a>>>
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

    #[doc(alias = "InstallSystemWide")]
    /// Copies the profile system-wide so it can be used by all users on the
    /// system or when no users are logged-in.
    pub async fn install_system_wide(&self) -> Result<()> {
        self.inner().call_method("InstallSystemWide", &()).await?;

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

    #[doc(alias = "ProfileId")]
    /// The identification hash of the profile.
    pub async fn profile_id(&self) -> Result<String> {
        self.inner().get_property("ProfileId").await
    }

    #[doc(alias = "Title")]
    /// The printable title for the profile.
    pub async fn title(&self) -> Result<String> {
        self.inner().get_property("Title").await
    }

    #[doc(alias = "Metadata")]
    /// The metadata for the profile, which may include optional keys like
    /// `EDID_md5` and `EDID_manufacturer` that are set by several CMS
    /// frameworks.
    pub async fn metadata(&self) -> Result<HashMap<String, String>> {
        self.inner().get_property("Metadata").await
    }

    #[doc(alias = "Qualifier")]
    /// The qualifier for the profile.
    ///
    /// A qualifier is used as a way to select a profile for a device. This
    /// might be something free text like `High quality studio` or something
    /// more programmable like `RGB.Plain.300dpi`.
    pub async fn qualifier(&self) -> Result<String> {
        self.inner().get_property("Qualifier").await
    }

    #[doc(alias = "Format")]
    /// The qualifier format for the profile.
    pub async fn format(&self) -> Result<String> {
        self.inner().get_property("Format").await
    }

    // TODO Use enum.
    #[doc(alias = "Kind")]
    /// The profile kind, e.g. `colorspace-conversion`, `abstract` or
    /// `display-device`.
    pub async fn kind(&self) -> Result<String> {
        self.inner().get_property("Kind").await
    }

    #[doc(alias = "Colorspace")]
    /// The profile colorspace, e.g. `rgb`.
    pub async fn colorspace(&self) -> Result<String> {
        self.inner().get_property("Colorspace").await
    }

    #[doc(alias = "HasVcgt")]
    /// If the profile has a VCGT entry.
    pub async fn has_vcgt(&self) -> Result<bool> {
        self.inner().get_property("HasVcgt").await
    }

    #[doc(alias = "IsSystemWide")]
    /// If the profile is installed system wide and available for all users.
    pub async fn is_system_wide(&self) -> Result<bool> {
        self.inner().get_property("IsSystemWide").await
    }

    // TODO Use Path or something.
    #[doc(alias = "Filename")]
    /// The profile filename, if one exists.
    pub async fn filename(&self) -> Result<String> {
        self.inner().get_property("Filename").await
    }

    #[doc(alias = "Created")]
    /// The date and time the profile was created in UNIX time.
    ///
    /// NOTE: this is NOT the time the meta-profile was created, or added to
    /// colord, nor the disk timestamp for the profile filename. This is the
    /// encoded date and time inside the ICC filename.
    pub async fn created(&self) -> Result<u64> {
        self.inner().get_property("Created").await
    }

    #[doc(alias = "Scope")]
    /// The scope of the device, e.g. `normal`, `temp` or `disk`.
    pub async fn scope(&self) -> Result<Scope> {
        self.inner().get_property("Scope").await
    }

    #[doc(alias = "Owner")]
    /// The user ID of the account that created the profile.
    pub async fn owner(&self) -> Result<u32> {
        self.inner().get_property("Owner").await
    }

    #[doc(alias = "Warnings")]
    /// Any warnings for the profile.
    ///
    /// e.g. 'description-missing' or 'vcgt-non-monotonic'.
    pub async fn warnings(&self) -> Result<Vec<String>> {
        self.inner().get_property("Warnings").await
    }
}

impl<'a> Serialize for Profile<'a> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ObjectPath::serialize(self.inner().path(), serializer)
    }
}
