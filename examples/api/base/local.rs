use crossgate::{
    object::{decorate, Object},
    store::MongoDbModel,
};

#[decorate]
struct Local {
    #[serde(default)]
    pub name: String,

    pub district: String,
}

impl MongoDbModel for Local {}
