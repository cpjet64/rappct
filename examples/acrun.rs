use clap::{Parser, Subcommand};
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
            let p = AppContainerProfile {
                name: name.clone(),
                sid: sid::AppContainerSid::from_sddl("S-1-0-0"),
            };
            p.delete()?;
            println!("Deleted profile: {}", name);
        }
        Cmd::Whoami { json } => match token::query_current_process_token() {
            Ok(info) => {
                if json {
                    let package_sid = info.package_sid.as_ref().map(|s| s.as_string());
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "is_appcontainer": info.is_appcontainer,
                            "is_lpac": info.is_lpac,
                            "package_sid": package_sid,
                            "capabilities": info.capability_sids,
                        }))
                        .unwrap()
                    );
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
            let caps = SecurityCapabilitiesBuilder::new(&p.sid)
                .with_known(&[KnownCapability::InternetClient])?
                .lpac(lpac)
                .build()?;
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
