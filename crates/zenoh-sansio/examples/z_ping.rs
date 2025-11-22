use zenoh_proto::ZResult;

fn entry() -> ZResult<()> {
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
