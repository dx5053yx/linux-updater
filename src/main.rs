#[path = "../system/detect.rs"]
mod detect;

#[path = "../system/updater.rs"]
mod updater;

use colored::*;
use detect::{detect_system, PackageManager, SystemInfo};
use std::io::{self, Write};
use unicode_width::UnicodeWidthStr;


fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => {
                print_help();
                return;
            }
            "--version" | "-V" => {
                print_version();
                return;
            }
            _ => {}
        }
    }

    let system_info = match detect_system() {
        Ok(info) => info,
        Err(err) => {
            eprintln!("{}", err.red());
            std::process::exit(1);
        }
    };
    if args.len() > 2 {
        eprintln!("{}", "Peringatan: hanya satu argumen yang didukung. Argumen lain diabaikan.".yellow());
    }
 
    if args.len() > 1 {
        match args[1].as_str() {
            "--all" => {
                updater::update_all(&system_info.relevant_managers, &system_info.available_managers);
                return;
            }
            "--pkg" => {
                update_pkg(&system_info);
                return;
            }
            "--aur" => {
                update_aur(&system_info);
                return;
            }
            "--flatpak" => {
                update_flatpak(&system_info);
                return;
            }
            "--snap" => {
                update_snap(&system_info);
                return;
            }
            _ => {
                eprintln!("{}", format!("Argumen tidak dikenal: {}", args[1]).red());
                print_help();
                std::process::exit(1);
            }
        }
    }

    loop {
        print_menu(&system_info);
        match read_choice() {
            Some(0) => {
                println!("{}", "Keluar. Sampai jumpa!".cyan());
                break;
            }
            Some(choice) => {
                handle_choice(choice, &system_info);
                println!();
                pause();
            }
            None => {
                println!("{}", "Pilihan tidak valid. Silakan coba lagi.".yellow());
            }
        }
    }
}


fn pad_to(value: &str, width: usize) -> String {
    let current = UnicodeWidthStr::width(value);
    if current >= width {
        value.to_string()
    } else {
        format!("{}{}", value, " ".repeat(width - current))
    }
}

fn print_menu(system_info: &SystemInfo) {
    clear_screen();
    let width = 40; // lebar isi dalam box (antara ║ dan ║)
    let title = "updet — universal update";
    let title_pad = (width - title.chars().count()) / 2;
    
    println!("{}", format!("╔{}╗", "═".repeat(width)).cyan());
    println!("{}", format!("║{}{}{}║",
        " ".repeat(title_pad),
        title,
        " ".repeat(width - title_pad - title.chars().count())
    ).cyan());
    println!("{}", format!("║{}║", " ".repeat(width)).cyan());
    println!("{}", format!("║  Distro : {}║",
        pad_to(&fit_display(&system_info.distro_id, width - 12), width - 11)
    ).cyan());
    println!("{}", format!("║  Kernel : {}║",
        pad_to(&fit_display(&kernel_version(), width - 12), width - 11)
    ).cyan());
    println!("{}", format!("╚{}╝", "═".repeat(width)).cyan());
    if !system_info.distro_like.is_empty() {
        println!(
            "{}",
            format!(
                "║   Turunan: {} ║",
                pad_to(&fit_display(&system_info.distro_like.join(", "), 18), 18)
            )
            .cyan(),
        );
    }
    println!();
    println!("  Pilih opsi update:");

    let managers = &system_info.relevant_managers;
    let mut index = 1;
    for manager in managers {
        let label = if system_info.available_managers.contains(manager) {
            manager.name().to_string()
        } else {
            format!("{} (tidak tersedia)", manager.name())
        };
        println!("  [{}] Update {}", index, label);
        index += 1;
    }

    println!("  [{}] Update semua yang tersedia", index);
    println!("  [0] Keluar");
    println!();
    print!("  Masukkan pilihan: ");
    io::stdout().flush().unwrap();
}

fn read_choice() -> Option<u32> {
    let mut input = String::new();
    match std::io::stdin().read_line(&mut input) {
        Ok(0) => std::process::exit(0),
        Ok(_) => input.trim().parse().ok(),
        Err(_) => None,
    }
}

fn handle_choice(choice: u32, system_info: &SystemInfo) {
    if choice == 0 { return; }
    let managers = &system_info.relevant_managers;
    let max = managers.len() as u32 + 1;
    if choice <= managers.len() as u32 {
        let manager = &managers[(choice - 1) as usize];
        if system_info.available_managers.contains(manager) {
            updater::update_manager(manager);
        } else {
            println!(
                "{}",
                format!("{} tidak tersedia pada sistem Anda.", manager.name()).yellow()
            );
        }
    } else if choice == max {
        updater::update_all(&system_info.relevant_managers, &system_info.available_managers);
    } else {
        println!("{}", "Pilihan tidak valid.".yellow());
    }
}

fn kernel_version() -> String {
    std::process::Command::new("uname")
        .arg("-r")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|line| line.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn pause() {
    println!("Tekan Enter untuk kembali ke menu...");
    let mut buffer = String::new();
    let _ = io::stdin().read_line(&mut buffer);
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
}

fn fit_display(value: &str, width: usize) -> String {
    let char_count = value.chars().count();
    if char_count <= width {
        value.to_string()
    } else {
        let truncated: String = value.chars().take(width.saturating_sub(1)).collect();
        format!("{}~", truncated)
    }
}

fn print_help() {
    println!("updet — Universal Linux Update Tool");
    println!();
    println!("Usage: updet [OPTION]");
    println!("  --help, -h       Tampilkan bantuan");
    println!("  --version, -V    Tampilkan versi");
    println!("  --all            Update semua yang tersedia");
    println!("  --pkg            Update package manager utama yang terdeteksi");
    println!("  --aur            Update AUR helper yang terdeteksi");
    println!("  --flatpak        Update Flatpak jika tersedia");
    println!("  --snap           Update Snap jika tersedia");
}

fn print_version() {
    println!("updet {}", env!("CARGO_PKG_VERSION"));
}

fn update_pkg(system_info: &SystemInfo) {
    if system_info.is_immutable {
        println!("{}", "Sistem ini immutable (rpm-ostree). Gunakan Flatpak/Snap untuk aplikasi.".yellow());
        return;
    }
    let primary = [
        PackageManager::Pacman,
        PackageManager::Apt,
        PackageManager::Dnf,
        PackageManager::Zypper,
        PackageManager::Apk,
        PackageManager::Xbps,
        PackageManager::Emerge,
    ];
    for manager in primary {
        if system_info.available_managers.contains(&manager) {
            updater::update_manager(&manager);
            return;
        }
    }
    println!("{}", "Tidak ada package manager utama yang terdeteksi.".yellow());
}

fn update_aur(system_info: &SystemInfo) {
    let aur_helpers = [PackageManager::Yay, PackageManager::Paru];
    for helper in aur_helpers {
        if system_info.available_managers.contains(&helper) {
            updater::update_manager(&helper);
            return;
        }
    }
    println!("{}", "Tidak ada AUR helper yang terdeteksi.".yellow());
}

fn update_flatpak(system_info: &SystemInfo) {
    if system_info.available_managers.contains(&PackageManager::Flatpak) {
        updater::update_manager(&PackageManager::Flatpak);
    } else {
        println!("{}", "Flatpak tidak tersedia pada sistem Anda.".yellow());
    }
}

fn update_snap(system_info: &SystemInfo) {
    if system_info.available_managers.contains(&PackageManager::Snap) {
        updater::update_manager(&PackageManager::Snap);
    } else {
        println!("{}", "Snap tidak tersedia pada sistem Anda.".yellow());
    }
}
