//! Lootsmith GUI (placeholder).
//!
//! Phase 6 will build the `egui` front end here, following the screen order in
//! the plan. For now this only confirms the save container decodes.

fn main() -> std::process::ExitCode {
    let path = std::env::args().nth(1);
    match path {
        Some(p) => match std::fs::read(&p).and_then(|d| {
            lootsmith_core::unpack(&d, lootsmith_core::Platform::PC)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
        }) {
            Ok(save) => {
                println!(
                    "loaded {p}: {} protobuf bytes, crc32 {}",
                    save.proto.len(),
                    if save.crc32_valid() { "ok" } else { "mismatch" }
                );
                std::process::ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("error: {e}");
                std::process::ExitCode::FAILURE
            }
        },
        None => {
            eprintln!("usage: lootsmith-gui <file.sav>  (GUI not implemented yet)");
            std::process::ExitCode::from(2)
        }
    }
}
