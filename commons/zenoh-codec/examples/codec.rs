#![no_std]

use zenoh_result::ZResult;

fn res_main() -> ZResult<()> {
    let data = [0u8; 128];
    let zbuf_1 = zenoh_buffer::ZBuf(&data);
    let zbuf_2 = zbuf_1.clone();

    // Assert ptrs are the same
    assert_eq!(zbuf_1.as_bytes().as_ptr(), zbuf_2.as_bytes().as_ptr());
    assert_eq!(zbuf_1.len(), zbuf_2.len());
    assert_eq!(zbuf_1, zbuf_2);

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
