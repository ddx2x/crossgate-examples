use crossgate::store::MongoStore;
use crossgate_rs::{
    micro::{Executor, Register},
    plugin::Plugin,
};
use db_wrapper::get_mongo_store;

mod db_wrapper;
use std::{
    cell::{Cell, RefCell},
    hash::{Hash, Hasher},
    rc::Rc,
    sync::Arc,
};
use tokio::{self, sync::Mutex};

fn random_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let id: u32 = rng.gen();
    id.to_string()
}

fn hash_by_partition(id: &str, partition: usize) -> usize {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    id.hash(&mut hasher);
    let hash = hasher.finish();
    (hash % partition as u64) as usize
}

#[derive(Clone, Copy)]
struct Task<'a> {
    store: &'a MongoStore,
}

impl<'a> Executor<'a> for Task<'a> {
    fn group(&self) -> String {
        "test".to_string()
    }

    fn start<'b>(
        &'a mut self,
        ctx: tokio_context::context::Context,
        register: &'b Register,
    ) -> futures::future::BoxFuture<'b, anyhow::Result<()>>
    where
        'a: 'b,
    {
        let mut ctx = ctx;
        Box::pin(async move {
            log::info!("start task");

            tokio::select! {
                _ = async move {
                    loop {
                        let random_id = random_id();
                        if let Ok((id,ids))=register.get_backend_service("test").await{
                            if let Some(index) =  ids.iter().position(|r| r == &id){
                                if hash_by_partition(&random_id, ids.len()) == index {
                                    log::info!("handle random_id {} id is {} ,ids is {:?}",random_id,id,ids);
                                }
                            }
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

    let mut task = Task { store };

    tokio::select! {
        _= crossgate_rs::micro::backend_service_run(&mut task, crossgate_rs::plugin::PluginType::Mongodb)=>{},
        // _= crossgate_rs::micro::backend_service_run(task, crossgate_rs::plugin::PluginType::Mongodb)=>{},

    }
}
