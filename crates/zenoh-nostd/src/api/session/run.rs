use crate::{Session, driver::ZDriver};

impl<T: ZDriver> Session<'_, T> {
    pub async fn run(&self) -> crate::ZResult<()> {
        self.driver.run().await
    }
}
