# Running rappct Examples

## Run All Examples

```bash
cargo run --example rappct_demo --features net && \
cargo run --example acrun -- --help && \
cargo run --example acrun -- whoami && \
cargo run --example acrun -- whoami --json && \
cargo run --example network_demo --features net && \
cargo run --example advanced_features --all-features
```

## Individual Examples

### 1. Main Demo (rappct_demo)

Shows core features: profiles, sandboxing, capabilities, network isolation

```bash
cargo run --example rappct_demo --features net
```

### 2. CLI Tool (acrun)

Command-line interface for profile management and process launching

**Show help:**

```bash
cargo run --example acrun -- --help
```

**Check current token:**

```bash
cargo run --example acrun -- whoami
```

**Check token (JSON format):**

```bash
cargo run --example acrun -- whoami --json
```

**Create profile:**

```bash
cargo run --example acrun -- ensure test.profile
```

**Launch in container:**

```bash
cargo run --example acrun -- launch test.profile "C:\Windows\System32\notepad.exe"
```

**Delete profile:**

```bash
cargo run --example acrun -- delete test.profile
```

### 3. Network Demo (network_demo)

Demonstrates network capabilities and firewall configuration

```bash
cargo run --example network_demo --features net
```

### 4. Advanced Features (advanced_features)

Shows profile paths, custom capabilities, diagnostics

```bash
cargo run --example advanced_features --all-features
```

### 5. Comprehensive Demo (comprehensive_demo)

Interactive demo covering all major features (9 demos)

```bash
cargo run --example comprehensive_demo --all-features
```

> **Note:** This is interactive - press Enter to advance through demos

## Feature Flags

- `--features net` - Enable network/firewall helpers (required for localhost loopback tests)
- `--all-features` - Enable all features (net + introspection + tracing)

## Prerequisites

- **Windows 10 1703+** for full LPAC support
- **Administrator rights** recommended for firewall operations
- **Internet connectivity** for network tests

## What Each Example Demonstrates

| Example | Key Features |
|---------|--------------|
| **rappct_demo** | Profile creation, sandboxing, capabilities, localhost vs internet, cleanup |
| **acrun** | CLI commands for profile/token management and launching |
| **network_demo** | Firewall loopback exemptions, network capability comparison |
| **advanced_features** | Path resolution, custom capabilities, diagnostics, job limits |
| **comprehensive_demo** | All features in 9 separate demos (interactive) |
