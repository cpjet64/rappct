# Capability Catalog

This is a living catalog of common AppContainer and LPAC capability names and
their intent. Use these to derive capability SIDs via `SecurityCapabilitiesBuilder`.

- internetClient — Outbound internet access
- internetClientServer — Inbound/outbound internet access (listen + connect)
- privateNetworkClientServer — Local network (enterprise/lan) access
- registryRead (LPAC) — Read-only registry access (limited)
- lpacCom (LPAC) — Minimal COM initialization for LPAC

Starter sets
- Baseline AC: [] (no capabilities)
- Client networking: [internetClient]
- Broad networking: [internetClient, internetClientServer, privateNetworkClientServer]
- LPAC defaults: [registryRead, lpacCom] with `.with_lpac_defaults()`

High-level `UseCase` presets in the API mirror these starter sets:

- `SecureWebScraper` → `internetClient`
- `IsolatedBuildEnvironment` → LPAC defaults (`registryRead`, `lpacCom`) plus base AC profile restrictions
- `NetworkConstrainedTool` → `privateNetworkClientServer`
- `MinimalLpac` → `registryRead`, `lpacCom`
- `FullDesktopApp` → representative broad set used by examples
- `Custom` → manual composition with `SecurityCapabilitiesBuilder`

References
- Windows AppContainer capabilities overview — Microsoft Docs
- Low Privilege AppContainer (LPAC) — Microsoft Docs

Notes
- Capability availability can vary by Windows build/SKU.
- LPAC is stricter than AC; prefer `.with_lpac_defaults()` when enabling LPAC.
