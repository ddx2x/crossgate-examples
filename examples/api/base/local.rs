use crossgate::{
    object::{metadata, Object},
    store::MongoDbModel,
};

#[metadata(uid)]
struct Local {
    #[serde(default)]
    pub name: String,

    pub district: String,
}

impl MongoDbModel for Local {}
