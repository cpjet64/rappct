# rappct Examples Guide

This directory contains comprehensive examples demonstrating all features of the rappct crate for Windows AppContainer and LPAC functionality.

## Quick Start

### Prerequisites
- **Windows 10 1703+** (required for LPAC features)
- **Administrator privileges** (recommended for firewall loopback and some ACL operations; basic profile and sandboxed launches do not require elevation)
- **Internet connectivity** (for network capability demonstrations)

### Running Examples

```bash
# Simple introduction (basic functionality)
cargo run --example rappct_demo

# With automatic firewall configuration (recommended)
cargo run --example rappct_demo --features net

# CLI tool for profile and launch management
cargo run --example acrun -- --help

# Complete feature walkthrough (requires Administrator)
cargo run --example comprehensive_demo --features net

# Network troubleshooting and testing (requires Administrator)
cargo run --example network_demo --features net

# Advanced features demonstration (requires Administrator)
cargo run --example advanced_features --features "net,introspection"
```

## Example Overview

### 1. `rappct_demo.rs` - Quick Introduction
**Purpose**: Simple 5-minute introduction to rappct
**Privileges**: Basic features work without Administrator
**What you'll see**:
- AppContainer profile creation/deletion
- Process isolation demonstration
- Network capability granting (may have limitations)

**Best for**: First-time users, quick proof-of-concept

### 2. `acrun.rs` - CLI Tool for Profiles and Launches
**Purpose**: Command-line interface for profile management and process launching
**Privileges**: Basic features work without Administrator
**What you'll see**:
- Profile creation/deletion via `ensure` and `delete` subcommands
- Token introspection with `whoami` (supports `--json` output)
- Process launching with `launch` subcommand (supports `--lpac` flag)
- Practical CLI patterns for integration

**Best for**: Quick testing, scripting, understanding CLI integration patterns

### 3. `network_demo.rs` - Network Troubleshooting
**Purpose**: Focused network capability testing and troubleshooting
**Privileges**: Requires Administrator
**What you'll see**:
- Systematic network capability testing
- Multiple connection methods (nslookup, HTTP, ping)
- LPAC network behavior
- Detailed troubleshooting guidance

**Best for**: Debugging network issues, production deployment preparation

### 4. `advanced_features.rs` - Advanced API Features
**Purpose**: Comprehensive coverage of advanced and lesser-known features
**Privileges**: Requires Administrator
**What you'll see**:
- Profile path resolution (folder_path, named_object_path)
- Direct SID derivation without profile creation
- Custom named capabilities beyond the built-in ones
- Configuration diagnostics and validation
- Advanced launch options (custom env, suspended launch)
- Enhanced I/O with full error handling
- Network container enumeration
- Capability-based ACLs
- LPAC testing environment variables

**Best for**: Advanced users, comprehensive API exploration, production features

### 5. `comprehensive_demo.rs` - Interactive Walkthrough
**Purpose**: Complete feature demonstration with user interaction
**Privileges**: Requires Administrator for full functionality
**What you'll see**:
- Step-by-step interactive demos
- Real-world scenarios (secure web scraper)
- Token introspection
- Process I/O redirection
- Detailed explanations of each feature

**Best for**: Training sessions, comprehensive understanding

## Common Issues and Solutions

### ❌ "The system could not find the environment option that was entered" (Error 203)
**Cause**: When passing custom environment via `LaunchOptions::env`, it **completely replaces** the parent environment. Windows processes require essential system variables (SystemRoot, ComSpec, PATHEXT, TEMP, TMP) to function.
**Solution**: Since `LaunchOptions::inherit_parent_env` defaults to `true`, essential
Windows variables are merged automatically. Just provide your custom variables:
```rust
let opts = LaunchOptions {
    env: Some(vec![
        (OsString::from("MY_VAR"), OsString::from("value")),
    ]),
    ..Default::default()
};
```
If you need complete control over the environment block (no automatic merging), set
`inherit_parent_env: false` and manage system variables yourself.
**See**: `advanced_features.rs` Demo 5 for a complete example

### ❌ "Access is denied" reading console output buffer (Error 0x5)
**Cause**: PowerShell tries to access the console output buffer for formatting, which AppContainers restrict for security.
**Solution**: Redirect PowerShell output to temporary files instead of console:
```rust
// Create temp file path
let temp_dir = env::temp_dir();
let output_file = temp_dir.join(format!("output_{}.txt", std::process::id()));

// Grant ACL access to temp directory for the AppContainer
grant_to_package(
    ResourcePath::Directory(temp_dir.clone()),
    &profile.sid,
    AccessMask(0x001F01FF), // GENERIC_ALL
)?;

// Redirect PowerShell output to file, read back with cmd, and auto-cleanup
let cmdline = format!(
    r#"/C powershell -Command "... | Out-File -FilePath '{}' -Encoding ASCII" && type "{}" && del "{}" 2>nul"#,
    output_file.display(), output_file.display(), output_file.display()
);
```
**See**: `network_demo.rs` and `comprehensive_demo.rs` Demo 4 for complete examples

### ❌ "Process launch failed at CreateProcessW" or "The system cannot find the file specified"
**Cause**: Not running as Administrator OR AppContainer restrictions blocking access
**Solutions**:
1. Run PowerShell/cmd as Administrator
2. If still failing, the executable path may not be accessible from AppContainer
3. Try simpler commands or check if the process requires additional capabilities

### ❌ "Unable to contact IP driver. General failure." or "Access is denied" for DNS
**Cause**: Network capabilities need Windows Firewall loopback exemption OR DNS service restrictions
**Solutions**:
1. **Use the `net` feature** (recommended):
   ```bash
   cargo run --example rappct_demo --features net
   ```
   This automatically handles firewall exemptions
2. **Try HTTP instead of nslookup/ping** (nslookup may be restricted even with network capabilities)
3. **Manual firewall exemption** (if needed):
   ```cmd
   CheckNetIsolation.exe LoopbackExempt -a -n=YourAppContainerName
   ```
4. **Check corporate network policies** (may block AppContainer network access)
5. **Note**: DNS resolution often fails in AppContainer even with InternetClient capability - this is normal Windows behavior

### ❌ "Access is denied" for file operations
**Cause**: ACL permissions not properly set
**Solution**: Ensure you're running as Administrator when setting ACLs

### ❌ LPAC features not working
**Cause**: Windows version < 10 1703
**Solution**: Upgrade to Windows 10 1703 or later

## Understanding the Output

### Process Output Prefixes
- `[ISOLATED]` - Process with no capabilities
- `[NETWORK]` - Process with network capabilities
- `[ACL-TEST]` - File access testing
- `[LPAC]` - Low Privilege AppContainer mode
- `[LIMITS]` - Resource-limited process
- `[PIPE]` - Process with redirected I/O

### Expected Behaviors

| Scenario | Expected Result |
|----------|----------------|
| No capabilities | Process runs but most operations fail |
| Internet Client | HTTP/HTTPS works, but DNS/ping often fail (normal) |
| File ACL granted | Can read/write specific files only |
| LPAC mode | Limited registry/COM access + network |
| Resource limits | Process constrained by memory/CPU limits |
| Notepad launch | May fail if not running as Administrator (normal) |
| Advanced demos | Some may fail due to AppContainer restrictions (expected) |

### Understanding AppContainer Isolation

Our examples now use a **"before/after" approach** to clearly demonstrate AppContainer security:

**✅ BASELINE (Normal Processes):**
- DNS resolution works (nslookup, ping)
- Full file system access
- Can launch any executable
- Full network connectivity
- Complete registry access

**❌ APPCONTAINER (Isolated Processes):**
- DNS resolution fails (even with network capabilities) - **                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        