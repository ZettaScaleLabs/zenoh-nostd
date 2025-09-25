#![no_std]

use getrandom::register_custom_getrandom;

const MY_CUSTOM_ERROR_CODE: u32 = getrandom::Error::CUSTOM_START + 42;
pub fn always_fail(_buf: &mut [u8]) -> Result<(), getrandom::Error> {
    let code = core::num::NonZeroU32::new(MY_CUSTOM_ERROR_CODE).unwrap();
    Err(getrandom::Error::from(code))
}

register_custom_getrandom!(always_fail);

pub mod log;
pub mod tcp;
