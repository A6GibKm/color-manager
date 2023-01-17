use std::{
    collections::HashMap,
    os::unix::io::{AsFd, AsRawFd},
};

use futures_util::StreamExt;
use zbus::{zvariant::OwnedObjectPath, Result};

use crate::{Device, Profile, Sensor};

/// A wrapper of the `org.freedesktop.ColorManager` DBus interface.
#[derive(Debug)]
pub struct ColorManager<'a>(zbus::Proxy<'a>);

impl<'a> ColorManager<'a> {
    /// Creates a new instance of ColorManager
    pub async fn new() -> Result<ColorManager<'a>> {
        let connection = zbus::Connection::system().await?;
        Self::from_connection(&connection).await
    }

    /// Creates a new instance of ColorManager using a given connection, the
    /// connection has to be a system connection.
    pub async fn from_connection(connection: &zbus::Connection) -> Result<ColorManager<'a>> {
        let proxy = zbus::ProxyBuilder::new_bare(connection)
            .interface("org.freedesktop.ColorManager")?
            .path("/org/freedesktop/ColorManager")?
            .destination("org.freedesktop.ColorManager")?
            .cache_properties(zbus::CacheProperties::No)
            .build()
            .await?;
        Ok(Self(proxy))
    }

    /// Get a reference to the underlying Proxy.
    pub fn inner(&self) -> &zbus::Proxy<'_> {
        &self.0
    }

    #[doc(alias = "GetDevices")]
    /// Gets a list of all the devices which have assigned color profiles.
    pub async fn devices(&self) -> Result<Vec<Device<'static>>> {
        let msg = self.inner().call_method("GetDevices", &()).await?;
        let reply = msg.body::<Vec<OwnedObjectPath>>()?;

        Device::from_paths(self.inner().connection(), reply).await
    }

    #[doc(alias = "GetDevicesByKind")]
    /// Gets a list of all the devices which have assigned color profiles.
    pub async fn devices_by_kind(&self, kind: &str) -> Result<Vec<Device>> {
        let msg = self
            .inner()
            .call_method("GetDevicesByKind", &(kind))
            .await?;
        let reply = msg.body::<Vec<OwnedObjectPath>>()?;

        Device::from_paths(self.inner().connection(), reply).await
    }

    #[doc(alias = "FindDeviceById")]
    /// Gets a device path for the device ID. This method is required as device
    /// ID's may have to be mangled to conform with the DBus path specification.
    /// For instance, a device ID of "cups$34:dev' would have a object path of
    /// "/org/freedesktop/ColorManager/cups_34_dev".
    pub async fn find_device_by_id(&self, device_id: &str) -> Result<Device> {
        let msg = self
            .inner()
            .call_method("FindDeviceById", &(device_id))
            .await?;
        let reply = msg.body::<OwnedObjectPath>()?;

        Device::new(self.inner().connection(), reply).await
    }

    #[doc(alias = "FindSensorById")]
    /// Gets a sensor path for the sensor ID.
    pub async fn find_sensor_by_id(&self, device_id: &str) -> Result<Sensor> {
        let msg = self
            .inner()
            .call_method("FindSensorById", &(device_id))
            .await?;
        let reply = msg.body::<OwnedObjectPath>()?;

        Sensor::new(self.inner().connection(), reply).await
    }

    #[doc(alias = "FindDeviceByProperty")]
    /// Gets a device path for the device with the specified property.
    pub async fn find_device_by_property(&self, key: &str, value: &str) -> Result<Device> {
        let msg = self
            .inner()
            .call_method("FindDeviceByProperty", &(key, value))
            .await?;
        let reply = msg.body::<OwnedObjectPath>()?;

        Device::new(self.inner().connection(), reply).await
    }

    #[doc(alias = "FindProfileById")]
    /// Gets a profile path for the profile ID.
    pub async fn find_profile_by_id(&self, profile_id: &str) -> Result<Profile> {
        let msg = self
            .inner()
            .call_method("FindDeviceByProperty", &(profile_id))
            .await?;
        let reply = msg.body::<OwnedObjectPath>()?;

        Profile::new(self.inner().connection(), reply).await
    }

    #[doc(alias = "FindProfileByProperty")]
    /// Gets a profile path for the profile with the specified property.
    pub async fn find_profile_by_property(&self, key: &str, value: &str) -> Result<Profile> {
        let msg = self
            .inner()
            .call_method("FindProfileByProperty", &(key, value))
            .await?;
        let reply = msg.body::<OwnedObjectPath>()?;

        Profile::new(self.inner().connection(), reply).await
    }

    #[doc(alias = "FindProfileByFilename")]
    /// Gets a profile path for the profile filename, either a fully-qualified
    /// filename with path or just the basename of the profile.
    pub async fn find_profile_by_filename(&self, file_name: &str) -> Result<Profile> {
        let msg = self
            .inner()
            .call_method("FindProfileByFilename", &(file_name))
            .await?;
        let reply = msg.body::<OwnedObjectPath>()?;

        Profile::new(self.inner().connection(), reply).await
    }

    #[doc(alias = "GetStandardSpace")]
    /// Gets a profile path for a defined profile space. The defined space is
    /// set from the profile metadata, specifically in the `STANDARD_space`
    /// entry.
    ///
    /// NOTE: only system wide profiles are able to define themselves as
    /// standard spaces.
    pub async fn standard_space(&self, standard_space: &str) -> Result<Profile> {
        let msg = self
            .inner()
            .call_method("GetStandardSpace", &(standard_space))
            .await?;
        let reply = msg.body::<OwnedObjectPath>()?;

        Profile::new(self.inner().connection(), reply).await
    }

    #[doc(alias = "GetSensors")]
    /// Gets a list of all the sensors recognised by the system.
    pub async fn sensors(&self) -> Result<Vec<Sensor>> {
        let msg = self.inner().call_method("GetSensors", &()).await?;
        let reply = msg.body::<Vec<OwnedObjectPath>>()?;

        Sensor::from_paths(self.inner().connection(), reply).await
    }

    #[doc(alias = "GetProfilesByKind")]
    /// Gets a list of all the profiles of a specified type.
    pub async fn profiles_by_kind(&self, kind: &str) -> Result<Vec<Profile>> {
        let msg = self
            .inner()
            .call_method("GetProfilesByKind", &(kind))
            .await?;
        let reply = msg.body::<Vec<OwnedObjectPath>>()?;

        Profile::from_paths(self.inner().connection(), reply).await
    }

    #[doc(alias = "CreateProfileWithFd")]
    /// Creates a profile.
    ///
    /// If the profile has been added to a device in the past, and that device
    /// exists already, then the new profile will be automatically added to the
    /// device. To prevent this from happening, remove the assignment by doing
    /// RemoveProfiledoc:tt> on the relevant device object.
    ///
    /// An optional file descriptor can be sent out of band for the ICC profile
    /// file.
    ///
    /// Using a file descriptor in addition to the filename allows the daemon to
    /// parse the ICC profile without re-opening it, which allows the daemon to
    /// read files inside the users home directory in a SELinux environment.
    pub async fn create_profile_with_fd<F: AsFd>(
        &self,
        profile_id: &str,
        scope: &str,
        handle: F,
        properties: HashMap<&str, &str>,
    ) -> Result<Profile> {
        let raw_fd = handle.as_fd().as_raw_fd();
        let msg = self
            .inner()
            .call_method(
                "CreateProfileWithFd",
                &(profile_id, scope, raw_fd, properties),
            )
            .await?;
        let reply = msg.body::<OwnedObjectPath>()?;
        msg.take_fds();

        Profile::new(self.inner().connection(), reply).await
    }

    #[doc(alias = "CreateProfile")]
    /// Creates a profile without using a file descriptor. It is recomended you
    /// use CreateProfileWithFd() as the daemon may not be running as root and
    /// have read access to profiles in the users home directory.
    pub async fn create_profile(
        &self,
        scope: &str,
        properties: HashMap<&str, &str>,
    ) -> Result<Profile> {
        let msg = self
            .inner()
            .call_method("CreateProfile", &(scope, properties))
            .await?;
        let reply = msg.body::<OwnedObjectPath>()?;

        Profile::new(self.inner().connection(), reply).await
    }
    #[doc(alias = "CreateDevice")]
    ///  Creates a device.
    ///
    /// If the device has profiles added to it in the past, and that profiles
    /// exists already, then the new device will be automatically have profiles
    /// added to the device. To prevent this from happening, remove the
    /// assignment by doing RemoveProfile on the relevant device object.
    pub async fn create_device(
        &self,
        scope: &str,
        properties: HashMap<&str, &str>,
    ) -> Result<Device> {
        let msg = self
            .inner()
            .call_method("CreateDevice", &(scope, properties))
            .await?;
        let reply = msg.body::<OwnedObjectPath>()?;

        Device::new(self.inner().connection(), reply).await
    }

    #[doc(alias = "DeleteDevice")]
    /// Deletes a device.
    pub async fn delete_device(&self, device: Device<'_>) -> Result<()> {
        self.inner().call_method("DeleteDevice", &(device)).await?;

        Ok(())
    }

    #[doc(alias = "DeleteProfile")]
    /// Deletes a profile.
    pub async fn delete_profile(&self, profile: Profile<'_>) -> Result<()> {
        self.inner()
            .call_method("DeleteProfile", &(profile))
            .await?;

        Ok(())
    }

    #[doc(alias = "Changed")]
    /// Some value on the interface or the number of devices or profiles has
    /// changed.
    pub async fn changed(&self) -> Result<()> {
        let mut stream = self.inner().receive_signal("Changed").await?;
        stream
            .next()
            .await
            .ok_or(zbus::Error::Failure("No response".into()))?;

        Ok(())
    }

    #[doc(alias = "DeviceAdded")]
    /// A device has been added.
    pub async fn device_added(&self) -> Result<Device> {
        let mut stream = self.inner().receive_signal("DeviceAdded").await?;
        let message = stream
            .next()
            .await
            .ok_or(zbus::Error::Failure("No response".into()))?;
        let content = message.body::<OwnedObjectPath>()?;

        Device::new(self.inner().connection(), content).await
    }

    #[doc(alias = "DeviceChanged")]
    /// A device has changed.
    pub async fn device_changed(&self) -> Result<Device> {
        let mut stream = self.inner().receive_signal("DeviceChanged").await?;
        let message = stream
            .next()
            .await
            .ok_or(zbus::Error::Failure("No response".into()))?;
        let content = message.body::<OwnedObjectPath>()?;

        Device::new(self.inner().connection(), content).await
    }

    #[doc(alias = "ProfileAdded")]
    /// A profile has been added.
    pub async fn profile_added(&self) -> Result<Profile> {
        let mut stream = self.inner().receive_signal("ProfileAdded").await?;
        let message = stream
            .next()
            .await
            .ok_or(zbus::Error::Failure("No response".into()))?;
        let content = message.body::<OwnedObjectPath>()?;

        Profile::new(self.inner().connection(), content).await
    }

    #[doc(alias = "ProfileRemoved")]
    /// A profile has been removed.
    pub async fn profile_removed(&self) -> Result<Profile> {
        let mut stream = self.inner().receive_signal("ProfileRemoved").await?;
        let message = stream
            .next()
            .await
            .ok_or(zbus::Error::Failure("No response".into()))?;
        let content = message.body::<OwnedObjectPath>()?;

        Profile::new(self.inner().connection(), content).await
    }

    #[doc(alias = "SensorAdded")]
    /// A sensor has been added.
    pub async fn sensor_added(&self) -> Result<Sensor> {
        let mut stream = self.inner().receive_signal("SensorAdded").await?;
        let message = stream
            .next()
            .await
            .ok_or(zbus::Error::Failure("No response".into()))?;
        let content = message.body::<OwnedObjectPath>()?;

        Sensor::new(self.inner().connection(), content).await
    }

    #[doc(alias = "SensorRemoved")]
    /// A sensor has been removed.
    pub async fn sensor_removed(&self) -> Result<Sensor> {
        let mut stream = self.inner().receive_signal("SensorRemoved").await?;
        let message = stream
            .next()
            .await
            .ok_or(zbus::Error::Failure("No response".into()))?;
        let content = message.body::<OwnedObjectPath>()?;

        Sensor::new(self.inner().connection(), content).await
    }

    #[doc(alias = "ProfileChanged")]
    /// A profile has been changed.
    pub async fn profile_changed(&self) -> Result<Profile> {
        let mut stream = self.inner().receive_signal("ProfileChanged").await?;
        let message = stream
            .next()
            .await
            .ok_or(zbus::Error::Failure("No response".into()))?;
        let content = message.body::<OwnedObjectPath>()?;

        Profile::new(self.inner().connection(), content).await
    }

    #[doc(alias = "DaemonVersion")]
    /// The daemon version.
    pub async fn daemon_version(&self) -> Result<String> {
        self.inner().get_property("DaemonVersion").await
    }

    #[doc(alias = "SystemVendor")]
    /// The system vendor.
    pub async fn system_vendor(&self) -> Result<String> {
        self.inner().get_property("SystemVendor").await
    }

    #[doc(alias = "SystemModel")]
    /// The system vendor.
    pub async fn system_model(&self) -> Result<String> {
        self.inner().get_property("SystemModel").await
    }
}
