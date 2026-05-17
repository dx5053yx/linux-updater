# updet 🔄

**Universal Linux update tool** — satu binary, semua distro.

`updet` mendeteksi otomatis package manager yang ada di sistemmu dan menyajikan menu interaktif untuk update. Tidak perlu ingat perintah berbeda di tiap distro.

```
╔══════════════════════════════════════════╗
║         updet — universal update         ║
║   Distro: arch                           ║
║   Kernel: 6.9.3-arch1-1                 ║
╚══════════════════════════════════════════╝

  Pilih opsi update:

  [1] Update pacman
  [2] Update yay
  [3] Update paru (tidak tersedia)
  [4] Update flatpak
  [5] Update snap (tidak tersedia)
  [6] Update semua yang tersedia
  [0] Keluar
```

---

## Distro yang didukung

| Distro / Turunan | Package Manager |
|---|---|
| Arch Linux, Manjaro, EndeavourOS | `pacman`, `yay`, `paru` |
| Debian, Ubuntu, Mint, Pop!\_OS, Kali, Raspbian | `apt` |
| Fedora, RHEL, CentOS | `dnf` / `yum` |
| CentOS Stream, Rocky Linux, AlmaLinux | `dnf` |
| openSUSE Leap / Tumbleweed | `zypper` |
| Alpine Linux | `apk` |
| Void Linux | `xbps-install` |
| Gentoo | `emerge` (via `emaint`) |
| Fedora Silverblue, Kinoite, CoreOS, Bazzite, dll *(immutable)* | `flatpak`, `snap` |
| Semua distro *(jika terinstall)* | `flatpak`, `snap` |

Distro tidak dikenal? `updet` otomatis fallback dan scan binary yang tersedia di `$PATH`.

---

## Install

Pastikan [Rust & Cargo](https://rustup.rs/) sudah terinstall.

```bash
git clone https://github.com/username/updet.git
cd updet
cargo build --release
sudo cp target/release/updet /usr/local/bin/
```

Setelah install, `updet` bisa dipanggil dari terminal manapun — bash, zsh, fish, kitty, Alacritty, tmux, dll.

### Verifikasi

```bash
updet --version
```

---

## Penggunaan

### Mode interaktif (default)

```bash
updet
```

Menampilkan menu TUI. Pilih angka sesuai yang ingin diupdate.

### Mode langsung (non-interaktif)

```bash
updet --all        # Update semua yang tersedia
updet --pkg        # Update package manager utama (pacman / apt / dnf / dll)
updet --aur        # Update AUR helper (yay / paru) — khusus Arch
updet --flatpak    # Update Flatpak
updet --snap       # Update Snap
updet --help       # Tampilkan bantuan
updet --version    # Tampilkan versi
```

---

## Fitur

- **Auto-detect** — baca `/etc/os-release` + scan `$PATH` untuk temukan manager yang benar-benar terinstall
- **Immutable system aware** — deteksi otomatis sistem berbasis `rpm-ostree` (Silverblue, Bazzite, Aurora, dll) dan hanya tawarkan Flatpak/Snap
- **Smart deduplication** — kalau `yay` ada, `pacman` di-skip di update-all karena sudah ter-cover. Kalau ada `dnf` dan `yum`, cukup `dnf` yang jalan
- **Real-time output** — output perintah langsung streaming ke terminal, tidak nunggu selesai
- **Graceful error** — kalau satu manager gagal, sisanya tetap jalan
- **Zero runtime dependency** — single binary, tidak butuh Python/Node/Ruby/runtime apapun

---

## Build dari source

```bash
# Release build
cargo build --release

# Binary ada di:
./target/release/updet
```

**Dependency:** [`colored`](https://crates.io/crates/colored) untuk output berwarna.

---

## Struktur proyek

```
updet/
├── src/
│   ├── main.rs      — entry point, menu TUI, CLI flag handler
│   ├── detect.rs    — deteksi distro, package manager, immutable system
│   └── updater.rs   — eksekusi update, filter redundant manager
└── Cargo.toml
```

---

## Kontribusi

Pull request dan issue sangat disambut! Beberapa area yang bisa dikontribusikan:

- Tambah dukungan distro baru di `detect.rs`
- Unit test untuk `detect_relevant_managers` dan `filter_redundant_managers`
- Perbaikan UX / pesan

---

