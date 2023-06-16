use crossgate_rs::{
    micro::{Executor, Register},
    plugin::Plugin,
};

use std::sync::Arc;
use tokio::{self, sync::Mutex};

struct Task {}

impl Executor for Task {
    fn group(&self) -> String {
        "test".to_string()
    }

    fn start<'a>(
        &self,
        ctx: tokio_context::context::Context,
        register: &'a Register,
    ) -> futures::future::BoxFuture<'a, anyhow::Result<()>> {
        let mut ctx = ctx;
        Box::pin(async move {
            log::info!("start task");
            tokio::select! {
                _ = async move {
                    loop {

                        log::info!("start task lock");

                        if let Ok((id,ids))=register.get_backend_service("test").await{

                            log::info!("id is {} ,ids is {:?}",id,ids);
                        }

                       //sleep 1 secs
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }


                } =>{},

                _ = ctx.done() =>{ return Ok(())}
            }

            Ok(())
        })
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    crossgate_rs::micro::backend_service_run(Task {}, crossgate_rs::plugin::PluginType::Mongodb)
        .await
}
