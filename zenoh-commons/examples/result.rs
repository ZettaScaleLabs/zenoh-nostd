#![no_std]

use zenoh_commons::{
    result::{ZError, ZResult},
    zbail, zctx,
};

fn io_error_example() -> ZResult<()> {
    zbail!(ZError::DidNotRead)
}

fn res_main() -> ZResult<()> {
    let io_res = zctx!(io_error_example())?;

    Ok(io_res)
}

fn main() {
    extern crate std;

    if let Err(e) = res_main() {
        std::println!("Error: {}", e);
    }
}
