use clap::{Parser, Subcommand};
use rappct::derive_sid_from_name;
use rappct::*;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "acrun", version)]
/// Developer CLI for rappct (skeleton). Commands will fail until Windows impls are added.
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Ensure {
        name: String,
        #[arg(long, default_value = "rappct")]
        display: String,
    },
    Delete {
        name: String,
    },
    Whoami {
        #[arg(long)]
        json: bool,
    },
    Launch {
        name: String,
        exe: PathBuf,
        #[arg(long)]
        lpac: bool,
    },
}

fn main() -> rappct::Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Ensure { name, display } => {
            let p = AppContainerProfile::ensure(&name, &display, Some("rappct"))?;
            println!(
                "Created/Opened profile: {} SID={}",
                p.name,
                p.sid.as_string()
            );
        }
        Cmd::Delete { name } => {
            let derived_sid = derive_sid_from_name(&name)?;
            let p = AppContainerProfile {
                name: name.clone(),
                sid: derived_sid,
            };
            p.delete()?;
            println!("Deleted profile: {}", name);
        }
        Cmd::Whoami { json } => match token::query_current_process_token() {
            Ok(info) => {
                if json {
                    let package_sid = info.package_sid.as_ref().map(|s| s.as_string());
                    let serialized = serde_json::to_string_pretty(&serde_json::json!({
                        "is_appcontainer": info.is_appcontainer,
                        "is_lpac": info.is_lpac,
                        "package_sid": package_sid,
                        "capabilities": info.capability_sids,
                    }))
                    .map_err(|e| {
                        AcError::Win32(format!("Failed to serialize token info: {}", e))
                    })?;
                    println!("{}", serialized);
                } else {
                    println!("is_appcontainer={}", info.is_appcontainer);
                    println!("is_lpac={}", info.is_lpac);
                    match info.package_sid.as_ref() {
                        Some(sid) => println!("package_sid={}", sid.as_string()),
                        None => println!("package_sid=<none>"),
                    };
                    if info.capability_sids.is_empty() {
                        println!("capabilities=[]");
                    } else {
                        println!("capabilities:");
                        for sid in &info.capability_sids {
                            println!("  {}", sid);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("whoami failed: {e}");
                return Err(e);
            }
        },
        Cmd::Launch { name, exe, lpac } => {
            let p = AppContainerProfile::ensure(&name, &name, None)?;
            let mut builder = SecurityCapabilitiesBuilder::new(&p.sid)
                .with_known(&[KnownCapability::InternetClient]);
            if lpac {
                supports_lpac()?;
                builder = builder.with_lpac_defaults();
            }
            let caps = builder.build()?;
            let child = launch_in_container(
                &caps,
                &LaunchOptions {
                    exe,
                    ..Default::default()
                },
            )?;
            println!("PID: {}", child.pid);
        }
    }
    Ok(())
}
