pub mod gps;
pub mod local;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use bson::doc;
use crossgate::store::{new_mongo_condition, Condition, Event, MongoDbModel, MongoFilter};
pub use gps::Gps;
pub use local::Local;

use crossgate::service::Service as CrossgateService;
use crossgate::store::MongoStore;
use tokio::sync::mpsc::Receiver;
use tokio_context::context::Context;

use crossgate::store::MongoStorageExtends;

#[derive(Debug, Clone)]
pub struct Base {
    addr: SocketAddr,

    loc: CrossgateService<Local, MongoFilter, MongoStore>,
    gps: CrossgateService<Gps, MongoFilter, MongoStore>,

    mongo_store: MongoStore,
}

impl crossgate_rs::micro::Service for Base {
    fn name(&self) -> String {
        "base".to_owned()
    }
    fn addr(&self) -> SocketAddr {
        self.addr
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct GpsInfo {
    #[serde(default)]
    vname: String,
    #[serde(default)]
    points: Vec<Vec<f64>>,
}

impl MongoDbModel for GpsInfo {}

impl Base {
    pub fn create(addr: &SocketAddr, store: &MongoStore) -> Self {
        let base = Self {
            loc: CrossgateService::<Local, MongoFilter, MongoStore>::new(
                "base".to_string(),
                "local".to_string(),
                store.clone(),
            ),
            gps: CrossgateService::<gps::Gps, MongoFilter, MongoStore>::new(
                "base".to_string(),
                "gps_latest".to_string(),
                store.clone(),
            ),
            addr: addr.clone(),
            mongo_store: store.clone(),
        };
        base
    }

    pub async fn list(&self) -> Vec<Local> {
        let mut cond = Condition::new(MongoFilter(doc! {}));
        // if let Err(e) = cond.wheres("status=1") {
        //     return vec![];
        // };
        if let Ok(rs) = self.loc.list(cond).await {
            return rs;
        }
        vec![]
    }

    pub async fn get(&self, name: &str) -> Local {
        let mut cond = new_mongo_condition();

        cond.wheres(&format!("name='{}'", name)).unwrap();
        self.loc.get(cond).await.unwrap()
    }

    pub async fn list_gpsinfo(&self) -> anyhow::Result<Vec<GpsInfo>> {
        let mut cond = new_mongo_condition();
        
        cond.with_db("base")
            .with_table("gps_info")
            .wheres(&format!("vname='{}'", "云F***88"))?;

        self.mongo_store.list_any_type::<GpsInfo>(cond).await
    }

    pub async fn watch(&self, ctx: Context) -> Receiver<Event<Local>> {
        let mut cond = new_mongo_condition();
        cond.wheres("version>=1").unwrap();

        log::info!("{:?}", cond);
        self.loc.watch(ctx, cond).await.unwrap()
    }
}
