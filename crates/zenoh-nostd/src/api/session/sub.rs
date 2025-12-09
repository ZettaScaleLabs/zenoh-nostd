use crate::api::{Sample, ZCallbacks, ZConfig};
use zenoh_proto::{fields::*, keyexpr, msgs::*};

impl<Config> super::Session<'_, Config>
where
    Config: ZConfig,
{
    pub async fn declare_subscriber(
        &self,
        ke: &'static keyexpr,
        callback: impl Into<<Config::SubscriberCallbacks as ZCallbacks<*const Sample, ()>>::Callback>,
    ) -> crate::ZResult<()> {
        let id = self.resources.next().await;
        let mut subscribers = self.resources.subscribers.lock().await;
        subscribers.insert(id, ke, callback)?;

        let msg = Declare {
            body: DeclareBody::DeclareSubscriber(DeclareSubscriber {
                id: id,
                wire_expr: WireExpr::from(ke),
            }),
            ..Default::default()
        };

        self.driver.send(msg).await
    }
}
