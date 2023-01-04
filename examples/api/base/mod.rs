pub mod gps;
pub mod local;
use crossgate::utils::Unstructed;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use bson::doc;
pub use gps::Gps;
pub use local::Local;

use crossgate::service::MongoStoreService;
use crossgate::store::{
    new_mongo_condition, Event, MongoDbModel, MongoStorageAggregationExtends, MongoStorageExtends,
    MongoStore,
};

use tokio::sync::mpsc::Receiver;
use tokio_context::context::Context;

use self::gps::GpsCount;

#[derive(Debug, Clone)]
pub struct Base {
    addr: SocketAddr,
    local: MongoStoreService<Local>,
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
            local: MongoStoreService::<Local>::new("base", "local", store.clone()),
            addr: addr.clone(),
            mongo_store: store.clone(),
        };
        base
    }

    pub async fn list(&self) -> anyhow::Result<Vec<Local>> {
        self.local.0.list(new_mongo_condition()).await
    }

    pub async fn get_local(&self, name: &str) -> anyhow::Result<Local> {
        let mut cond = new_mongo_condition();
        cond.wheres(&format!("name='{}'", name))?;
        self.local.0.get(cond).await
    }

    pub async fn list_gpsinfo(&self) -> anyhow::Result<Vec<GpsInfo>> {
        let mut cond = new_mongo_condition();

        cond.with_db("base")
            .with_table("gps_info")
            .with_fields(&["vname", "points"])
            .wheres(&format!("vname='{}'", "云F***88"))?;

        // select vname,points from base.gps_info where vname="云F***88"
        self.mongo_store
            .clone()
            .list_any_type::<GpsInfo>(cond)
            .await
    }

    pub async fn get_gpsinfo(&self, id: &str) -> anyhow::Result<GpsInfo> {
        let mut cond = new_mongo_condition();

        cond.with_db("base")
            .with_table("gps_info")
            .wheres(&format!("_id='{}'", id))?;

        // select * from base.gps_info where _id = ?;
        self.mongo_store.clone().get_any_type::<GpsInfo>(cond).await
    }

    pub async fn create_gpsinfo(&self, g: GpsInfo) -> anyhow::Result<()> {
        let mut cond = new_mongo_condition();
        cond.with_db("base").with_table("gps_info");

        self.mongo_store.clone().save_any_type(g, cond).await
    }

    pub async fn delete_gpsinfo(&self, vname: &str) -> anyhow::Result<()> {
        let mut cond = new_mongo_condition();
        cond.with_db("base")
            .with_table("gps_info")
            .wheres(&format!("vname = '{}'", vname))?;

        self.mongo_store
            .clone()
            .delete_any_type::<GpsInfo>(cond)
            .await
    }

    pub async fn update_local(&self, local: Local) -> anyhow::Result<()> {
        let mut cond = new_mongo_condition();
        cond.wheres(&format!("name='{}'", local.name))?;
        self.local.0.apply(local, cond).await?;
        Ok(())
    }

    pub async fn watch(&self, ctx: Context) -> anyhow::Result<Receiver<Event<Local>>> {
        let mut cond = new_mongo_condition();
        cond.wheres("version>=1")?;

        self.local.0.watch(ctx, cond).await
    }

    pub async fn watch2(&self, ctx: Context) -> anyhow::Result<Receiver<Event<Unstructed>>> {
        let q = new_mongo_condition()
            .to_owned()
            .with_db("abc321_trade")
            .with_table("order")
            .wheres("state=1")?
            .to_owned();

        self.mongo_store.clone().watch_any_type(ctx, q).await
    }

    pub async fn gps_count(&self) -> anyhow::Result<Vec<GpsCount>> {
        // select count(1) from base.gps_info where vname="云F***88" group by vname order by count desc;
        self.mongo_store
            .clone()
            .aggregate::<GpsCount>(
                "base".to_string(),
                "gps_info".to_string(),
                vec![
                    doc! {"$match":{"vname":"云F***88"}},
                    doc! {"$group": { "_id": "$vname", "count": { "$sum": 1 } }},
                    doc! {"$sort": {"count":-1}},
                    doc! {"$limit": 10},
                    doc! {"$skip": 0},
                ],
            )
            .await
    }
}
