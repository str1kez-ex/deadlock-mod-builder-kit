# Deadlock Mod Build Kit (DMBK)

> **All-in-One Mod Manager, VPK Fusion Engine & Toolkit for Valve's Deadlock**  
> *Developed by str1kez*

![Deadlock Mod Build Kit](static/app-icon.ico)

---

## ✨ Features

- **🎮 Community Mod Browser**: Direct integration with GameBanana API. Browse, filter by character or category, check response times, and install mods with a single click.
- **⚡ VPK Fusion Engine**: Pure Rust Source 2 VPK reader & packer (`vpk.rs`). Merge components from different mods (e.g. models from Mod A + custom sounds from Mod B) into a unified `.vpk`.
- **🛠️ 5-Step Mod Creation Wizard**: Build custom Deadlock mods inspired by Deadlock Mod Builder. Select target heroes, bind custom 3D models, weapons, sounds, and adjust camera calibration.
- **🔍 Collapsible VPK Asset Inspector**: Inspect installed VPK files, browse internal directory trees (`models/`, `materials/`, `sounds/`), view author info and thumbnail covers.
- **📊 Tracker (Meta & Builds)**: In-depth competitive meta tier list (winrates, pickrates, GPM, DPM, hard counters, synergies) and item progression build guides.
- **⚙️ Toolchain Manager**: One-click setup for Source 2 Community SDK (CSDK13) and Source 2 Viewer.
- **🎨 Dark Tactical UI**: Clean, WCAG-conscious dark theme with customizable accent palettes (Default Green, Neon Cyan, Crimson Red, Deep Violet, Vintage Amber).

---

## 🚀 Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://www.rust-lang.org/) & Cargo (stable)
- Windows 10/11 with WebView2

### Development

```bash
# Clone the repository
git clone https://github.com/<your-username>/deadlock-mod-kit.git
cd deadlock-mod-kit

# Install frontend dependencies
npm install

# Run in development mode with hot-reload
npm run tauri dev
```

### Production Build

```bash
# Compile standalone exe and installers (NSIS & MSI)
npm run tauri build
```

The output packages will be located in:
- `src-tauri/target/release/deadlock-mod-kit.exe` (Portable executable)
- `src-tauri/target/release/bundle/nsis/` (Windows installer)
- `src-tauri/target/release/bundle/msi/` (MSI package)

---

## 📜 License

MIT License. Deadlock and Source 2 are trademarks of Valve Corporation. This project is an independent community tool not affiliated with Valve Corporation.
