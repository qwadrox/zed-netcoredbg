# NetCoreDbg Debug Adapter Extension for Zed

A Zed extension that provides debugging support for .NET applications using [netcoredbg](https://github.com/Samsung/netcoredbg).

## Prerequisites

- [Rust](https://rustup.rs/) with the `wasm32-wasip1` target:
  ```
  rustup target add wasm32-wasip1
  ```
- [Zed](https://zed.dev/) with debug adapter support
- **Windows only**: Visual Studio Build Tools or Visual Studio with the "Desktop development with C++" workload (needed to compile proc-macros during the WASM build)

## Installation

This extension is not published to the Zed marketplace. Install it as a dev extension:

1. Clone this repository
2. Open Zed
3. Open the command palette (`Ctrl+Shift+P` / `Cmd+Shift+P`) and run **"zed: extensions"**
4. Click **"Install Dev Extension"**
5. Select the cloned directory

Zed will compile the extension to WASM and load it. When you make changes to the source, use **"zed: rebuild extensions"** from the command palette to recompile.

### Windows: Fixing the `link.exe` conflict

On Windows, Git ships with `usr/bin/link.exe` (a Unix symlink utility) that often appears on `PATH` before the MSVC `link.exe` linker. Even though this extension compiles to WASM, Cargo still needs the MSVC linker to build proc-macros (like `serde_derive`) for the host target. If Git's `link.exe` is found first, the build fails with cryptic linker errors.

To fix this, create a `.cargo/config.toml` that explicitly points to the MSVC linker:

1. Copy the template:
   ```
   copy .cargo\config.toml.example .cargo\config.toml
   ```
2. Open a **Developer Command Prompt for VS 2022** and run:
   ```
   where link.exe
   echo %LIB%
   ```
3. Edit `.cargo/config.toml` — replace `<VERSION>` with your MSVC version (e.g. `14.43.34808`) and `<SDK_VERSION>` with your Windows SDK version (e.g. `10.0.26100.0`)

## Configuration

### Debug tasks (`.zed/debug.json`)

Create a `.zed/debug.json` in your project to define debug configurations. You can also use the Debugger UI to add configurations.

**Launch a .NET application:**

```json
[
  {
    "label": "Debug .NET App",
    "adapter": "netcoredbg",
    "request": "launch",
    "program": "${ZED_WORKTREE_ROOT}/bin/Debug/net8.0/MyApp.dll",
    "cwd": "${ZED_WORKTREE_ROOT}",
    "args": [],
    "env": {
      "ASPNETCORE_ENVIRONMENT": "Development"
    },
    "stopAtEntry": false,
    "justMyCode": true,
    "build": {
      "command": "dotnet",
      "args": ["build"]
    }
  }
]
```

| Field | Required | Description |
|-------|----------|-------------|
| `program` | Yes | Path to the `.dll` or `.exe` to debug |
| `cwd` | No | Working directory (defaults to worktree root) |
| `args` | No | Command line arguments passed to the program |
| `env` | No | Environment variables for the launched process |
| `stopAtEntry` | No | Break at the entry point (`false` by default) |
| `justMyCode` | No | Skip framework/library code when stepping (`true` by default) |
| `enableStepFiltering` | No | Step over properties and operators (`true` by default) |
| `build` | No | Build command Zed runs before launching the debugger |

**Attach to a running process:**

```json
[
  {
    "label": "Attach to .NET Process",
    "adapter": "netcoredbg",
    "request": "attach",
    "processId": 12345
  }
]
```

### Custom netcoredbg binary (`.zed/settings.json`)

The extension automatically downloads the netcoredbg binary from GitHub. To use a specific binary instead, add to your project's `.zed/settings.json`:

```json
{
  "dap": {
    "netcoredbg": {
      "binary": "/path/to/netcoredbg"
    }
  }
}
```

On Windows, use forward slashes or escaped backslashes:
```json
{
  "dap": {
    "netcoredbg": {
      "binary": "C:/tools/netcoredbg/netcoredbg.exe"
    }
  }
}
```

## Troubleshooting

### No debug tasks appear

- Ensure `.zed/debug.json` is valid JSON (no trailing commas, no comments)
- The `"adapter"` field must be exactly `"netcoredbg"`
- Restart Zed after installing the extension

### Build fails on Windows with linker errors

This is likely the `link.exe` conflict described in [Windows: Fixing the link.exe conflict](#windows-fixing-the-linkexe-conflict) above.

### "Failed to parse WebAssembly module" in Zed logs

This means the WASM binary is a plain module instead of a Component Model binary. When using "Install Dev Extension", Zed handles the conversion automatically. If you're building manually outside of Zed, you need to convert the output using [wasm-tools](https://github.com/bytecodealliance/wasm-tools) and a [WASI Preview1 adapter](https://github.com/bytecodealliance/wasmtime/releases) (`wasi_snapshot_preview1.reactor.wasm`):

```bash
cargo build --target wasm32-wasip1 --release
wasm-tools component new target/wasm32-wasip1/release/netcoredbg.wasm \
  -o extension.wasm \
  --adapt wasi_snapshot_preview1.reactor.wasm
```

### Extension not visible after installing

Check Zed's log (`View > Toggle Log`) for errors. The extension requires the `[lib]` section in `extension.toml` to load — this is already included in the repository.

## Why netcoredbg?

While Microsoft provides official debugging libraries for .NET Core (`Microsoft.VisualStudio.clrdbg`), these come with [restrictive licensing terms](https://github.com/dotnet/core/issues/505) that limit their use to specific IDEs like Visual Studio Code.

**netcoredbg** solves this by providing an open-source alternative developed by Samsung:

- **Open Source** with a permissive license
- **Cross-Platform**: Windows, macOS, and Linux
- **IDE Agnostic**: Works with any editor supporting the Debug Adapter Protocol (DAP)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License.
