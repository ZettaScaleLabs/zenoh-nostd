use crate::ZResult;

fn entry() -> crate::ZResult<()> {
    Ok(())
}

fn main() {
    match entry() {
        Ok(_) => {}
        Err(e) => {
            zenoh_proto::error!("Error: {:?}", e);
        }
    }
}
