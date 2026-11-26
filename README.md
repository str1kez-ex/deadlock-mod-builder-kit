# Deadlock Mod Build Kit (DMBK) v1.5

> **All-in-One Mod Manager, VPK Fusion Engine & Toolkit for Valve's Deadlock**  
> *Developed by str1kez*

![Deadlock Mod Build Kit](static/app-icon.ico)

---

## ✨ Features

- **🏠 Home Dashboard & Vanilla Isolation**: Quick-launch modded or clean Vanilla Deadlock (`launch_game_vanilla` physically isolates VPK addons while playing Vanilla and safely restores them via `restore_vanilla_addons`), displays Steam Persona/Account ID, Game Build ID, free disk space, and GameBanana ping.
- **🎮 Community Mod Browser & Multi-VPK Installer**: Direct GameBanana integration (`Mod`, `Tool`, `Sound`, `Wip` with exact `_sProfileUrl` links and Deadlock `_aGame._idRow == 20948` verification), `94vw` expanded detail view with zoomable gallery, HTML5 Audio Preview player, optional Global Mod category support (`UI`, `Sounds`, `Maps`, `Crosshairs`, `Weapons`), and multi-VPK checkbox installation queue (`Downloads` tab with speed & ETA tracking).
- **🧑‍🎤 Updated Deadlock Hero Roster (45 Heroes)**: Full hero mapping (`Abrams` through `Yamato`, including *Old Gods, New Blood* and *City Never Sleeps* characters) with matched `static/heroes/` portraits across Loadouts, Browser, Meta Tier List, and Mod Wizard.
- **⚡ VPK Fusion Engine & Mod Creation Wizard**: Pure Rust Source 2 VPK v2 reader/packer (`vpk.rs`) with camera attachment diagnostics on `.vmdl_c` models, real Source 2 `cfg/dmbk_camera.cfg` + `autoexec.cfg` camera fix injection, and a 5-step Mod Creation Wizard.
- **📊 Live Meta Tier List & Player Profile**: Full 45-hero reactive meta tier list with role filters and search, plus Steam Account ID detection (`get_steam_account`) with direct links to Statlocker and Tracklock.
- **📜 Persistent Logging, Localization & Scaling**: Automatic session logs in `%LOCALAPPDATA%\DeadlockModKit\logs\`, Deadlock `console.log` viewer, RU/EN UI switcher, and 80%–140% UI scaling.
- **🔒 Hardened File Handling**: Strict Zip-Slip prevention (Zip/7z/tar), path traversal validation on addon operations (`[a-zA-Z0-9_\-.]` + canonical directory checks), and safe URL/folder opening via `open`.

---

## 🚀 Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://www.rust-lang.org/) & Cargo (stable, Windows MSVC or `x86_64-pc-windows-gnu` with MinGW)
- Windows 10/11 with WebView2 Runtime

### Development

```bash
# Clone the repository
git clone https://github.com/str1kez-ex/deadlock-mod-builder-kit.git
cd deadlock-mod-kit

# Install frontend dependencies
npm install

# Run in development mode with hot-reload
npm run tauri dev
```

### Production Build

```bash
# Build Windows installers (NSIS Setup EXE & MSI)
npm run tauri build
```

The installer packages will be located in:
- `src-tauri/target/release/bundle/nsis/` (Windows NSIS installer `.exe`)
- `src-tauri/target/release/bundle/msi/` (Windows MSI package `.msi`)

---

## 📜 License

MIT License. Deadlock and Source 2 are trademarks of Valve Corporation. This project is an independent community tool not affiliated with Valve Corporation.
