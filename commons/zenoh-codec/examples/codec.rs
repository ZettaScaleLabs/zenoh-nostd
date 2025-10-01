#![no_std]

use zenoh_result::ZResult;

fn res_main() -> ZResult<()> {
    Ok(())
}

fn main() {
    extern crate std;

    std::process::exit(match res_main() {
        Ok(_) => 0,
        Err(e) => {
            std::eprintln!("Error: {}", e);
            1
        }
    });
}
