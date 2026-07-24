<h1 align="center">rappct</h1>

<p align="center">
  <a href="https://opensource.org/licenses/MIT">
    <img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="License: MIT" />
  </a>
</p>

<p align="center">
  🇩🇪 <a href="#deutsch">Deutsch</a> · 🇬🇧 <a href="#english">English</a>
</p>

---

## <a id="deutsch"></a>🇩🇪 Deutsch

`rappct` ist eine Software für die Arbeit mit **Windows AppContainer- und LPAC-Prozessgrenzen**. Sie ermöglicht das Erstellen, Verwalten und Ausführen von hochgradig isolierten Prozessen (in einer sogenannten „Sandbox“) unter Windows.

Normalerweise können Windows-Programme auf viele deiner Dateien, Ordner und Netzwerke zugreifen. Dieses Projekt ermöglicht es, Programme in eine extrem sichere, isolierte Umgebung zu stecken. Windows nutzt hierfür die Technologien AppContainer und LPAC (Less Privileged AppContainer / weniger privilegierter AppContainer). Ein so isoliertes Programm läuft in seiner eigenen kleinen Welt und darf standardmäßig auf nichts anderes auf deinem System zugreifen.

Dieses Repository baut direkt auf dieser Technologie auf und fügt eine fertige Sicherheits-Voreinstellung hinzu: `UseCase::McpServerSandbox`. Diese Erweiterung ist speziell darauf ausgelegt, MCP-Server (Model Context Protocol) absolut sicher unter Windows auszuführen. Der Server wird so isoliert, dass er weder auf dein Netzwerk zugreifen noch unbefugte Systemdateien lesen kann – selbst wenn die KI im Hintergrund unvorhergesehene Befehle ausführen sollte.

---

### Schnellstart

Um das Projekt zu bauen und alle Tests auszuführen:

```powershell
# 1. Repository klonen
git clone https://github.com/Veyilla/rappct.git
cd rappct

# 2. Projekt kompilieren
cargo build

# 3. Test-Suite ausführen
cargo test --all-targets
```

---

### Code-Beispiel: Einen MCP-Server sicher isolieren

Hier ist ein einfaches Beispiel, wie ein Prozess mit der neuen `McpServerSandbox` abgesichert und gestartet werden kann:

```rust
use rappct::profile::AppContainerProfile;
use rappct::launch::LaunchBuilder;
use rappct::capability::UseCase;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Erstelle ein Sicherheits-Profil für deinen Server
    let profile = AppContainerProfile::ensure("MeinMcpServerProfile")?;

    // 2. Bereite den sicheren Start (Sandbox) vor
    let launcher = LaunchBuilder::new("C:\\Pfad\\zu\\mcp_server.exe")
        .with_profile(&profile)
        // Verwende das neue Preset für maximale Isolation (kein Netz, minimaler Systemzugriff)
        .with_use_case(UseCase::McpServerSandbox)
        .build()?;

    // 3. Starte den Prozess sicher isoliert
    let mut child = launcher.spawn()?;
    
    // Warte auf das Ende des Prozesses
    let exit_code = child.wait()?;
    println!("Server beendet mit Code: {:?}", exit_code);

    Ok(())
}
```

---

### Lokaler Release-Prozess (kein GitHub-Action-Publish)

Dieses Repository verwendet einen rein lokalen Release-Ablauf. Der Veröffentlichungsinhalt wird durch eine Manifest-Allowlist in `include` gesteuert:

- `LICENSE`
- `README.md`
- `Cargo.toml`
- `src/**`
- `examples/**`
- `tests/**`

Die Release-Kette läuft wie folgt ab:

- `just release-version-check` prüft, ob die Crate-Version höher ist als die auf crates.io veröffentlichte Version.
- `just release-gate` führt Formatierungs-, Lint- und Testprüfungen sowie Paketliste und Trockenlauf-Checks auf einem **sauberen Arbeitsverzeichnis** aus.
- `just release-gate-log` führt den vollständigen Gate-Prozess mit einem zeitgestempelten Protokoll in `output/release-gate` durch.
- `just release` führt den vollständigen Gate-Prozess aus und fragt anschließend zur ausdrücklichen Bestätigung auf.

`cargo publish` darf nicht direkt außerhalb dieses Ablaufs ausgeführt werden.

---

### Navigation & Dokumentation

Die Projektdokumentation befindet sich in [`docs/`](./docs/).

- Einstiegspunkt: [`docs/index.md`](./docs/index.md)
- Werkzeuge & Erstellung: [`docs/TOOLING.md`](./docs/TOOLING.md)
- Modul-Dokumentation: [`docs/modules/index.md`](./docs/modules/index.md)

---

## <a id="english"></a>🇬🇧 English

`rappct` is software for working with **Windows AppContainer and LPAC process boundaries**. It allows you to create, manage, and spawn highly secured (sandboxed) processes on Windows.

Normally, Windows programs can access many of your files, folders, and network resources. This project makes it possible to run programs in an extremely secure, isolated environment (a "sandbox"). Windows uses the technologies AppContainer and LPAC (Less Privileged AppContainer) for this. A program isolated this way runs in its own little world and, by default, cannot access anything else on your system.

This repository builds directly on top of this technology and adds a ready-to-use security preset: `UseCase::McpServerSandbox`. This extension is specifically designed to run MCP servers (Model Context Protocol) securely on Windows. It isolates the server so that it can neither access your network nor read unauthorized system files – even if the AI runs unexpected commands in the background.

---

### Quickstart

To build the project and run all tests:

```powershell
# 1. Clone the repository
git clone https://github.com/Veyilla/rappct.git
cd rappct

# 2. Build the project
cargo build

# 3. Run the test suite
cargo test --all-targets
```

---

### Code Example: Securing an MCP Server

Here is a simple example of how to configure and spawn a process using the new `McpServerSandbox` preset:

```rust
use rappct::profile::AppContainerProfile;
use rappct::launch::LaunchBuilder;
use rappct::capability::UseCase;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create a security profile for your server
    let profile = AppContainerProfile::ensure("MyMcpServerProfile")?;

    // 2. Prepare the secure launch (sandbox)
    let launcher = LaunchBuilder::new("C:\\Path\\to\\mcp_server.exe")
        .with_profile(&profile)
        // Use the new preset for maximum isolation (no network, minimal system access)
        .with_use_case(UseCase::McpServerSandbox)
        .build()?;

    // 3. Spawn the sandboxed process
    let mut child = launcher.spawn()?;
    
    // Wait for the process to exit
    let exit_code = child.wait()?;
    println!("Server exited with code: {:?}", exit_code);

    Ok(())
}
```

---

### Local Release Process (no GitHub Action publish)

This repository uses a local-only release flow. Publish payload is controlled by a manifest `include` allow-list:

- `LICENSE`
- `README.md`
- `Cargo.toml`
- `src/**`
- `examples/**`
- `tests/**`

The release chain is:

- `just release-version-check` verifies crate version is greater than the published crate on crates.io.
- `just release-gate` runs formatting/lints/tests, packaging list, and dry-run checks on a **clean working tree**.
- `just release-gate-log` runs the full gate with a timestamped transcript in `output/release-gate`.
- `just release` runs the full gate and then prompts for explicit publish confirmation.

Do not run `cargo publish` directly outside this flow.

---

### Navigation & Documentation

Project documentation is in [`docs/`](./docs/).

- Start here: [`docs/index.md`](./docs/index.md)
- Tooling & Regeneration: [`docs/TOOLING.md`](./docs/TOOLING.md)
- Module documentation: [`docs/modules/index.md`](./docs/modules/index.md)

---

### Fork & Credits

<details>
<summary><b>🇩🇪 Deutsch (Fork-Details anzeigen)</b></summary>

Dies ist ein Fork des Original-Repositories [cpjet64/rappct](https://github.com/cpjet64/rappct), gepflegt von [@Veyilla](https://github.com/Veyilla).
Dieses Repository wurde um `UseCase::McpServerSandbox` erweitert – ein LPAC-Sicherheits-Preset für die sichere Ausführung von MCP-Servern unter Windows.
</details>

<details>
<summary><b>🇬🇧 English (Show Fork Details)</b></summary>

This is a fork of [cpjet64/rappct](https://github.com/cpjet64/rappct), maintained by [@Veyilla](https://github.com/Veyilla).
Extended with `UseCase::McpServerSandbox` — an LPAC sandbox preset for MCP server processes.
</details>