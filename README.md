# Dragon-Miku

Dragon-Miku is a free software built in Tauri + Nuxt 3 that has several useful functions for Dragon Nest development.

## Prerequisites  
Make sure you have installed:  
- [Bun](https://bun.sh/docs/installation)  
- [Rust](https://www.rust-lang.org/tools/install) (for Tauri)  
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites/)  

---

## Installation  

1. **Clone the repository** 
   ``` 
   git clone https://github.com/DevilProMT/Dragon-Miku.git  
   cd Dragon-Miku
   ```

2. **Install dependencies**  
   ```
   bun install
   ```

---

## Running the Project  

- **Start Nuxt 3 in development mode**  
  ```
  bun run dev
  ```

- **Run Tauri for the desktop app**
  ```
  bun run tauri dev
  ```

---

## Building the Application  

- **Build Nuxt 3**  
  ```
  bun run build
  ```

- **Build the Tauri application**  
  ```
  bun run tauri build
  ```

---

## Features  
- DN Table Converter with double type support (v6)
- Act Converter from v6 to v5

---

## Upcoming Features  
- PAK Extractor (with & without encryption)
- REPAK
- SKN Converter from v11 to v10

---

## Troubleshooting  
- If you encounter errors running Tauri, make sure Rust is installed:  
  ```
  rustup update
  ```
  or [Visit our discord](https://discord.gg/Gh7SXHzkje) if you have any questions/issues
---

## Special Thanks  
A huge thanks to [ShrlGnwn](https://github.com/ShrlGnwn) for creating the front-end design 🎨✨  