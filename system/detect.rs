use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PackageManager {
    Apt,
    Dnf,
    Yum,
    Pacman,
    Yay,
    Paru,
    Zypper,
    Apk,
    Xbps,
    Emerge,
    Flatpak,
    Snap,
}

impl PackageManager {
    pub fn name(&self) -> &'static str {
        match self {
            PackageManager::Apt => "apt",
            PackageManager::Dnf => "dnf",
            PackageManager::Yum => "yum",
            PackageManager::Pacman => "pacman",
            PackageManager::Yay => "yay",
            PackageManager::Paru => "paru",
            PackageManager::Zypper => "zypper",
            PackageManager::Xbps => "xbps-install",
            PackageManager::Emerge => "emerge",
            PackageManager::Apk => "apk",
            PackageManager::Flatpak => "flatpak",
            PackageManager::Snap => "snap",
        }
    }
}

#[derive(Debug)]
pub struct SystemInfo {
    pub distro_id: String,
    pub distro_like: Vec<String>,
    pub available_managers: Vec<PackageManager>,
    pub relevant_managers: Vec<PackageManager>,
    pub is_immutable: bool,
}

pub fn detect_system() -> Result<SystemInfo, String> {
    let os_release = if Path::new("/etc/os-release").exists() {
        fs::read_to_string("/etc/os-release").map_err(|e| format!("Gagal membaca /etc/os-release: {}", e))?
    } else {
        String::new()
    };

    let mut distro_id = String::new();
    let mut distro_like = Vec::new();

    for line in os_release.lines() {
        if let Some(value) = line.strip_prefix("ID=") {
            distro_id = value.trim_matches('"').to_lowercase();
        }
        if let Some(value) = line.strip_prefix("ID_LIKE=") {
            distro_like = value
                .trim_matches('"')
                .split_whitespace()
                .map(|s| s.to_lowercase())
                .collect();
        }
    }

    let mut relevant = detect_relevant_managers(&distro_id, &distro_like);
    let available = detect_available_managers();

    // Detect rpm-ostree / immutable systems. Prefer a binary check which is
    // more reliable than enumerating every possible distro ID.
    let is_immutable = distro_id == "silverblue"
        || distro_id == "kinoite"
        || distro_id == "fedora-coreos"
        || binary_exists("rpm-ostree");

    if is_immutable {
        // On immutable systems we should not attempt dnf/yum/pacman upgrades;
        // limit relevant managers to app layers like Flatpak and Snap.
        relevant = vec![PackageManager::Flatpak, PackageManager::Snap];
    }

    if available.is_empty() {
        return Err("Tidak terdeteksi package manager yang didukung. Pastikan salah satu package manager terpasang.".to_string());
    }

    Ok(SystemInfo {
        distro_id,
        distro_like,
        available_managers: available,
        relevant_managers: relevant,
        is_immutable,
    })
}

fn detect_relevant_managers(distro_id: &str, distro_like: &[String]) -> Vec<PackageManager> {
    let mut managers = Vec::new();

    // detect_relevant_managers focuses on mapping distro IDs to common
    // package managers; immutability is handled in `detect_system`.

    let distro_names: Vec<&str> = std::iter::once(distro_id)
        .chain(distro_like.iter().map(|s| s.as_str()))
        .collect();

    for distro in distro_names {
        match distro {
            "debian" | "ubuntu" | "linuxmint" | "pop" | "pop_os" | "popos" | "kali" | "raspbian" => {
                if !managers.contains(&PackageManager::Apt) {
                    managers.push(PackageManager::Apt);
                }
            }
            "fedora" | "rhel" | "centos" => {
                if !managers.contains(&PackageManager::Dnf) {
                    managers.push(PackageManager::Dnf);
                }
                if !managers.contains(&PackageManager::Yum) {
                    managers.push(PackageManager::Yum);
                }
            }
            "centos-stream" | "rocky" | "almalinux" => {
                if !managers.contains(&PackageManager::Dnf) {
                    managers.push(PackageManager::Dnf);
                }
            }
            "arch" | "manjaro" | "endeavouros" | "endeavor" | "endeavoros" => {
                if !managers.contains(&PackageManager::Pacman) {
                    managers.push(PackageManager::Pacman);
                }
                if !managers.contains(&PackageManager::Yay) {
                    managers.push(PackageManager::Yay);
                }
                if !managers.contains(&PackageManager::Paru) {
                    managers.push(PackageManager::Paru);
                }
            }
            "opensuse" | "suse" | "opensuse-leap" | "opensuse-tumbleweed" => {
                if !managers.contains(&PackageManager::Zypper) {
                    managers.push(PackageManager::Zypper);
                }
            }
            "alpine" => {
                if !managers.contains(&PackageManager::Apk) {
                    managers.push(PackageManager::Apk);
                }
            }
            "void" => {
                if !managers.contains(&PackageManager::Xbps) {
                    managers.push(PackageManager::Xbps);
                }
            }
            "gentoo" => {
                if !managers.contains(&PackageManager::Emerge) {
                    managers.push(PackageManager::Emerge);
                }
            }
            _ => {}
        }
    }

    if managers.is_empty() {
        managers = vec![
            PackageManager::Apt,
            PackageManager::Dnf,
            PackageManager::Pacman,
            PackageManager::Flatpak,
            PackageManager::Snap,
        ];
    }

    if !managers.contains(&PackageManager::Flatpak) {
        managers.push(PackageManager::Flatpak);
    }
    if !managers.contains(&PackageManager::Snap) {
        managers.push(PackageManager::Snap);
    }

    managers
}

fn detect_available_managers() -> Vec<PackageManager> {
    let mut managers = Vec::new();

    for manager in &[
        PackageManager::Pacman,
        PackageManager::Yay,
        PackageManager::Paru,
        PackageManager::Apt,
        PackageManager::Dnf,
        PackageManager::Yum,
        PackageManager::Xbps,
        PackageManager::Emerge,
        PackageManager::Zypper,
        PackageManager::Apk,
        PackageManager::Flatpak,
        PackageManager::Snap,
    ] {
        if *manager == PackageManager::Snap {
            if snap_available() {
                managers.push(manager.clone());
            }
        } else if binary_exists(manager.name()) {
            managers.push(manager.clone());
        }
    }

    managers
}

fn snap_available() -> bool {
    if !binary_exists("snap") {
        return false;
    }

    // Snap membutuhkan systemd. Sistem non-systemd (OpenRC, runit, dll) tidak didukung.
    if !binary_exists("systemctl") {
        return false;
    }

    Command::new("systemctl")
        .arg("is-active")
        .arg("--quiet")
        .arg("snapd")
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn binary_exists(name: &str) -> bool {
    std::env::var_os("PATH")
        .and_then(|paths| {
            std::env::split_paths(&paths)
                .find(|dir| {
                    let path = dir.join(name);
                    path.is_file() && is_executable(&path)
                })
                .map(|_| ())
        })
        .is_some()
}

#[cfg(unix)]
fn is_executable(path: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|meta| meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &std::path::Path) -> bool {
    path.is_file()
}
