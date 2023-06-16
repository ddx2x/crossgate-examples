pub mod gps;
pub mod local;
use crossgate::utils::{from_str, Unstructed};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;

use bson::doc;
pub use gps::Gps;
pub use local::Local;

use crossgate::service::MongoStoreService;

use crossgate::store::{
    new_mongo_condition, Event, MongoDbModel, MongoStorageAggregationExtends, MongoStorageExtends,
    MongoStorageOpExtends, MongoStore,
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
        "/base/api".to_owned()
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
        self.local
            .0
            .list(
                new_mongo_condition()
                    .to_owned()
                    .wheres("name ='other'")?
                    .to_owned(),
            )
            .await
    }

    pub async fn list_local_unstructed(&self) -> anyhow::Result<Option<Vec<Unstructed>>> {
        let q = new_mongo_condition()
            .to_owned()
            .with_db("base")
            .with_table("local")
            .with_fields(&["name"])
            .to_owned();
        let res = self
            .mongo_store
            .clone()
            .list_any_type::<Unstructed>(q)
            .await?;

        Ok(Some(res))
    }

    pub async fn get_local(&self, name: &str) -> anyhow::Result<Option<Local>> {
        let mut cond = new_mongo_condition();
        cond.wheres(&format!("name='{}'", name))?;

        Ok(self.local.0.get(cond).await?)
    }

    pub async fn incr_count(&self) -> anyhow::Result<Option<Local>> {
        let q = new_mongo_condition()
            .to_owned()
            .with_db("base")
            .with_table("local")
            .to_owned();

        Ok(self
            .mongo_store
            .clone()
            .incr::<Local>(&[("count", 1)], q)
            .await?)
    }

    pub async fn partial_update_local(&self) -> anyhow::Result<Option<Unstructed>> {
        let q = new_mongo_condition()
            .to_owned()
            .with_db("base")
            .with_table("local")
            .with_fields(&["a"])
            .wheres("_id = 'abc124'")?
            .to_owned();

        Ok(Some(
            self.mongo_store
                .clone()
                .apply_any_type::<Unstructed>(from_str(r#"{"a":{"b":2}}"#)?, q)
                .await?,
        ))
    }

    pub async fn batch_remove_local(&self) -> anyhow::Result<u64> {
        let q = new_mongo_condition()
            .to_owned()
            .with_db("base")
            .with_table("local")
            .wheres("_id ~ ('abc122','abc129')")?
            .to_owned();

        Ok(self.mongo_store.clone().batch_remove(q).await?)
    }

    pub async fn list_gpsinfo(&self) -> anyhow::Result<Vec<GpsInfo>> {
        let mut cond = new_mongo_condition();

        cond.with_db("base")
            .with_table("gps_info")
            .with_fields(&["vname", "points"])
            .wheres(&format!("vname='{}'", "云F***88"))?;

        // select vname,points from base.gps_info where vname="云F***88"
        let r = self
            .mongo_store
            .clone()
            .list_any_type::<GpsInfo>(cond)
            .await?;
        Ok(r)
    }

    pub async fn get_gpsinfo(&self, id: &str) -> anyhow::Result<GpsInfo> {
        let mut cond = new_mongo_condition();

        cond.with_db("base")
            .with_table("gps_info")
            .wheres(&format!("_id='{}'", id))?;

        // select * from base.gps_info where _id = ?;
        let r = self
            .mongo_store
            .clone()
            .get_any_type::<GpsInfo>(cond)
            .await?;
        Ok(r)
    }

    pub async fn create_gpsinfo(&self, g: GpsInfo) -> anyhow::Result<()> {
        let mut cond = new_mongo_condition();
        cond.with_db("base").with_table("gps_info");

        let _ = self.mongo_store.clone().save_any_type(g, cond).await?;

        Ok(())
    }

    pub async fn delete_gpsinfo(&self, vname: &str) -> anyhow::Result<()> {
        let mut cond = new_mongo_condition();
        cond.with_db("base")
            .with_table("gps_info")
            .wheres(&format!("vname = '{}'", vname))?;

        self.mongo_store
            .clone()
            .delete_any_type::<GpsInfo>(cond)
            .await?;
        Ok(())
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

        let r = self.mongo_store.clone().watch_any_type(ctx, q).await?;

        Ok(r)
    }

    pub async fn gps_count(&self) -> anyhow::Result<Vec<GpsCount>> {
        // select count(1) from base.gps_info where vname="云F***88" group by vname order by count desc;
        let r = self
            .mongo_store
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
            .await?;

        Ok(r)
    }
}
