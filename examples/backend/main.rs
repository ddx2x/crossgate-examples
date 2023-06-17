use crossgate::store::MongoStore;
use crossgate_rs::{
    micro::{Executor, Register},
    plugin::Plugin,
};
use db_wrapper::get_mongo_store;

mod db_wrapper;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
};
use tokio::{self, sync::Mutex};

#[derive(Clone, Copy)]
struct Task<'a> {
    store: &'a MongoStore,
}

impl<'a> Executor for Task<'a> {
    fn group(&self) -> String {
        "test".to_string()
    }

    fn start<'b>(
        &self,
        ctx: tokio_context::context::Context,
        register: &'b Register,
    ) -> futures::future::BoxFuture<'b, anyhow::Result<()>> {
        let mut ctx = ctx;
        Box::pin(async move {
            log::info!("start task");
            tokio::select! {
                _ = async move {
                    loop {

                        if let Ok((id,ids))=register.get_backend_service("test").await{
                            log::info!("id is {} ,ids is {:?}",id,ids);
                        }

                       //sleep 1 secs
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
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

    let store = get_mongo_store().await;

    let task = Task { store };

    tokio::select! {
        _= crossgate_rs::micro::backend_service_run(task, crossgate_rs::plugin::PluginType::Mongodb)=>{},
        // _= crossgate_rs::micro::backend_service_run(task, crossgate_rs::plugin::PluginType::Mongodb)=>{},

    }
}
