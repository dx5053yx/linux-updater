use colored::*;
use std::process::{Command, Stdio};

use crate::detect::PackageManager;

pub fn update_manager(manager: &PackageManager) -> bool {
    let command_string = match manager {
        PackageManager::Apt => "sudo apt update && sudo apt upgrade -y",
        PackageManager::Dnf => "sudo dnf upgrade -y",
        PackageManager::Yum => "sudo yum update -y",
        PackageManager::Pacman => "sudo pacman -Syu --noconfirm",
        PackageManager::Yay => "yay -Syu --noconfirm",
        PackageManager::Paru => "paru -Syu --noconfirm",
        PackageManager::Zypper => "sudo zypper refresh && sudo zypper update -y",
        PackageManager::Apk => "sudo apk update && sudo apk upgrade",
        PackageManager::Xbps => "sudo xbps-install -Su",
        // Use `emaint -a sync` as the modern Gentoo sync mechanism.
        PackageManager::Emerge => "sudo emaint -a sync && sudo emerge -uDN @world",
        PackageManager::Flatpak => "flatpak update -y",
        PackageManager::Snap => "sudo snap refresh",
    };

    println!("{}", format!("Memulai update {}...", manager.name()).cyan());

    let status = if cfg!(target_os = "linux") {
        Command::new("sh")
            .arg("-c")
            .arg(command_string)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Hanya mendukung Linux saat ini",
        ))
    };

    match status {
        Ok(exit) if exit.success() => {
            println!("{}", "✓ Selesai!".green());
            true
        }
        Ok(exit) => {
            let code = exit.code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "dibunuh oleh signal".to_string());
            println!("{}", format!("✗ Gagal! Exit code: {}", code).red());
            false
        }
        Err(err) => {
            println!("{}", format!("✗ Gagal menjalankan perintah: {}", err).red());
            false
        }
    }
}

pub fn update_all(relevant: &[PackageManager], available: &[PackageManager]) {
    let filtered = filter_redundant_managers(relevant, available);
    let mut any_failed = false;
    
    let mut any_skipped = false;
    for manager in filtered {
        if available.contains(&manager) {
            let ok = update_manager(&manager);
            if !ok {
                any_failed = true;
            }
        } else {
            println!("{}", format!("{} di-skip karena tidak terpasang.", manager.name()).yellow());
            any_skipped = true;
        }
    }

    if any_failed {
        println!("{}", "Selesai dengan beberapa kegagalan.".yellow());
    } else if any_skipped {
        println!("{}", "Selesai. Beberapa manager di-skip karena tidak terpasang.".yellow());
    } else {
        println!("{}", "Semua update selesai tanpa error.".green());
    }
}

fn filter_redundant_managers(relevant: &[PackageManager], available: &[PackageManager]) -> Vec<PackageManager> {
    // Two-pass approach: use *available* to detect whether an AUR helper or a
    // DNF/YUM binary is present. This prevents filtering out `pacman` when an
    // AUR helper is listed in `relevant` but isn't actually installed.
    let has_aur_helper = available.iter().any(|m| matches!(m, PackageManager::Yay | PackageManager::Paru));
    let mut has_dnf_or_yum = false;
    let mut has_aur_added = false;

    relevant
        .iter()
        .filter_map(|manager| match manager {
            PackageManager::Pacman if has_aur_helper => None,
            PackageManager::Yay | PackageManager::Paru => {
                if !has_aur_added && available.contains(manager) {
                    has_aur_added = true;
                    Some(manager.clone())
                } else {
                    None
                }
            }
            PackageManager::Dnf | PackageManager::Yum => {
                if !has_dnf_or_yum && available.contains(manager) {
                    has_dnf_or_yum = true;
                    Some(manager.clone())
                } else {
                    None
                }
            }
            _ => Some(manager.clone()),
        })
        .collect()
}
