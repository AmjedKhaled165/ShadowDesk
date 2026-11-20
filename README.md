# ShadowDesk

Ultra-lightweight Windows desktop privacy utility for real-time window cloaking during screen sharing and recording sessions (Zoom, Microsoft Teams, Discord, Google Meet, OBS).

Developed by Amjed Khaled.

## Features

- **Exclude From Capture**: Target applications remain fully interactive on your primary display while being excluded from desktop capture APIs (`WDA_EXCLUDEFROMCAPTURE`). Remote participants see desktop wallpaper instead of private application windows.
- **Stealth Mode**: ShadowDesk automatically masks its own controller process from capture buffers and previews.
- **Taskbar & Alt+Tab Cloaking**: Strips target windows from the Windows shell taskbar and Alt+Tab cycling switcher via extended window styles (`WS_EX_TOOLWINDOW`).
- **Global Hotkey Interceptors**:
  - `F8` (or `Ctrl + Shift + H`): Instantly toggle visibility and minimization states for protected windows.
  - `F7`: Toggle the ShadowDesk control interface.
- **Real-time Pipeline Preview**: Desktop mirror validating capture feed exclusion before going live on meetings.

## Technical Architecture

- **Language**: Rust
- **Platform**: Windows 10 / 11 (64-bit)
- **Subsystem**: Win32 / Desktop Window Manager (DWM)
- **Key APIs**:
  - `SetWindowDisplayAffinity`
  - `RegisterHotKey` / Global Low-Level Hooks
  - `GetWindowLongPtr` / `SetWindowLongPtr`

## Quick Start

### Running Pre-built Binary
Run `ShadowDesk.exe` directly or pass startup arguments via PowerShell / Command Prompt:
```powershell
ShadowDesk.exe --help
```

### Hotkey Reference

| Shortcut | Action |
| --- | --- |
| `F8` or `Ctrl + Shift + H` | Toggle Show / Minimize protected windows |
| `F7` | Toggle ShadowDesk control window |

### Building from Source

Requirements:
- Rust toolchain (1.75+)
- Visual Studio C++ Build Tools (MSVC)

```powershell
git clone https://github.com/AmjedKhaled165/ShadowDesk.git
cd ShadowDesk
cargo build --release
```
The compiled binary will be located at `target\release\ShadowDesk.exe`.

## License
MIT License.
