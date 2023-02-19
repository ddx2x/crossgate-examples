use crossgate::object::{decorate, Object};

#[decorate]
struct Local {
    #[serde(default)]
    pub name: String,

    pub district: String,
}
