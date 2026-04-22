use std::path::Path;

use self_update::update::ReleaseAsset;

#[derive(Debug, PartialEq, Clone)]
pub enum InstallationType {
    Portable,
    Appimage,
    Normal,
}

#[cfg(target_os = "windows")]
fn is_os(name: &String) -> bool {
    name.contains("Windows")
}

#[cfg(target_os = "linux")]
fn is_os(name: &String) -> bool {
    name.contains("Linux")
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn is_os(name: &String) -> bool {
    true
}

#[cfg(target_arch = "x86_64")]
fn is_arch(name: &String) -> bool {
    !name.contains("aarch64") && !name.contains("arm64")
}

#[cfg(target_arch = "aarch64")]
fn is_arch(name: &String) -> bool {
    name.contains("aarch64")
}

#[cfg(target_arch = "arm")]
fn is_arch(name: &String) -> bool {
    name.contains("arm64")
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "arm")))]
fn is_arch(name: &String) -> bool {
    true
}

impl InstallationType {
    pub fn new(root: &Path) -> InstallationType {
        if root.starts_with("/tmp/mount_") {
            return InstallationType::Appimage;
        }
        let portable_file = root.join("portable.txt");
        let portable_folder = root.join("UserData");
        if (portable_file.exists() && portable_file.is_file())
            || (portable_folder.exists() && portable_folder.is_dir())
        {
            return InstallationType::Portable;
        }
        InstallationType::Normal
    }

    pub fn match_asset(&self, asset_name: &String) -> bool {
        match self {
            InstallationType::Portable => {
                asset_name.contains("Portable") && is_os(asset_name) && is_arch(asset_name)
            }
            InstallationType::Appimage => asset_name.ends_with(".AppImage"),
            InstallationType::Normal => {
                !asset_name.contains("Portable")
                    && !asset_name.contains(".AppImage")
                    && is_os(asset_name)
                    && is_arch(asset_name)
            }
        }
    }

    pub fn find_best_asset<'a>(
        &self,
        assets: &'a [ReleaseAsset],
        compiler: &String,
    ) -> Option<&'a ReleaseAsset> {
        assets
            .iter()
            .find(|x| self.match_asset(&x.name) && x.name.contains(compiler))
    }
}
