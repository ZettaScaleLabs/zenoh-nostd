use crate::api::ZConfig;

impl<Config> super::Session<'static, Config>
where
    Config: ZConfig,
{
    pub async fn run(&self) -> crate::ZResult<()> {
        self.driver.run(self.resources).await
        // TODO! implement a `session.close` method that should undeclare all resources
    }
}
