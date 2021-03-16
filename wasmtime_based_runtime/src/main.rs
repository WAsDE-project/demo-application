mod global;
mod host;
mod leb128;

use std::fs;
use wasmtime::*;

fn main() -> Result<(), Trap> {
    host::init_logging();

    let args: Vec<String> = std::env::args().collect();
    let mut metafile_path = "./metafile.json";
    if args.len() >= 2 {
        if args[1].trim() == "--exit" {
            return Ok(());
        }
        metafile_path = &args[1];
    }

    let main_module_id = "main";

    let linker = host::create_linker(main_module_id)?;
    let store = linker.store().clone();

    let metadata = fs::read_to_string(metafile_path).map_err(|e| Trap::new(e.to_string()))?;

    let mut attributes = match host::get_platform_string() {
        "x86_64" => {
            info!("x86_64 detected. Setting attribute");
            vec!["x86_64".to_string()]
        }
        "aarch64" => {
            info!("aarch64 detected. Setting attribute");
            vec!["aarch64".to_string()]
        }
        _ => {
            vec![]
        }
    };
    #[cfg(feature = "enable_wasi")]
    {
        info!("WASI detected. Setting attribute");
        attributes.push("WASI".to_string());
    }

    host::initialize(
        &metadata,
        main_module_id,
        "./cache",
        attributes,
        linker,
        store,
    )
    .expect("initialization failed");
    host::run_main();
    Ok(())
}
