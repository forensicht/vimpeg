<div align="center">
    <br/>
    <img src="./data/icons/Vimpeg.svg" width="150" />
    <h1>Vimpeg</h1>
    <h3>Extract video frames into a single image in a simple way.</h3>
</div>

![Screenshot](./data/screenshots/vimpeg.png#gh-light-mode-only)
![Screenshot](./data/screenshots/dark.png#gh-dark-mode-only)

# Build

## Dependencies to build

Cargo:
- gtk4
- libadwaita
- ffmpeg

### Arch Linux:
```bash
sudo pacman -S libadwaita gtk4 ffmpeg
```
### Fedora:
```
sudo dnf -y install libadwaita-devel gtk4-devel ffmpeg-devel
```
### Windows:
Install [MSYS2](https://www.msys2.org/)

In MSYS2, open the MINGW64 terminal and execute the commands below to install the dependencies:
```
pacman -S mingw-w64-x86_64-toolchain mingw-w64-x86_64-gcc mingw-w64-x86_64-clang mingw-w64-x86_64-pkgconf
pacman -S mingw-w64-x86_64-gtk4 mingw-w64-x86_64-gettext mingw-w64-x86_64-libxml2 mingw-w64-x86_64-librsvg
pacman -S mingw-w64-x86_64-libadwaita
pacman -S mingw-w64-x86_64-ffmpeg
```
Add the paths below in the PATH environment variable:
* C:\msys64\mingw64\bin
* C:\msys64\mingw64\lib
* C:\msys64\mingw64\include

Install the gnu toolchain for Rust:
```
rustup toolchain install stable-x86_64-pc-windows-gnu
rustup default stable-gnu
```

Copyright 2024 © Tiago Martins

Vimpeg is released under the terms of the [Mozilla Public License v2](https://github.com/forensicht/vimpeg/blob/main/LICENSE)