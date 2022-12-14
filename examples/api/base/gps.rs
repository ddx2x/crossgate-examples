use crossgate::{
    object::{decorate, Object},
    store::MongoDbModel,
};

#[decorate]
struct Gps {
    #[serde(default)]
    gps: Vec<Vec<f64>>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct GpsCount {
    #[serde(rename(serialize = "uid"))]
    _id: String,
    count: i64,
}
impl MongoDbModel for GpsCount {}
