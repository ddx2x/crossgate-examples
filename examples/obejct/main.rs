use crossgate::{
    object::{metadata, Object},
    service::MongoStoreService,
    store::new_mongo_condition,
};
use std::env;

use crossgate::store::MongoStore;

use once_cell::sync::OnceCell;

static MONGO_STORE: OnceCell<MongoStore> = OnceCell::new();

async fn _create_mongo_store() {
    let database_url = "mongodb://127.0.0.1:27017";
    let store = MongoStore::new(&database_url)
        .await
        .expect("Mongo global must set success");

    MONGO_STORE
        .set(store)
        .expect("Mongo global must set success")
}

#[inline]
pub async fn get_mongo_store() -> &'static MongoStore {
    // Safety: tt is already set when the program is initialized
    if MONGO_STORE.get().is_none() {
        _create_mongo_store().await;
    }
    unsafe { MONGO_STORE.get_unchecked() }
}

#[metadata(uid)]
struct A {
    name: String,
    sex: Option<u8>,
}

fn example() {
    // let a = A {
    //     uid: "Abc".to_string(),
    //     version: 0,
    //     kind: Default::default(),
    // };

    // let aa = serde_json::to_string(&a).unwrap();

    // println!("{}", aa);

    // let s = r#"{ "id": "Abc", "version": 0, "kind": "A" }"#;

    // let aaa: A = serde_json::from_str(s).unwrap();

    // println!("{:?}", aaa);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let store = get_mongo_store().await;
    let abc = MongoStoreService::<A>::new("test", "a", store.clone());

    abc.0
        .save(
            A {
                uid: "Abc".to_string(),
                version: 0,
                kind: Default::default(),
                name: "123".to_string(),
                sex: None,
            },
            new_mongo_condition(),
        )
        .await?;

    abc.0
        .update(
            A {
                uid: "Abc".to_string(),
                version: 0,
                kind: Default::default(),
                name: "321".to_string(),
                sex: Some(1),
            },
            new_mongo_condition()
                .to_owned()
                .with_fields(&["name", "sex"])
                .wheres("_id='Abc'")?
                .to_owned(),
        )
        .await?;

    abc.0
        .apply(
            A {
                uid: "Abc".to_string(),
                version: 0,
                kind: Default::default(),
                name: "111".to_string(),
                sex: Some(0),
            },
            new_mongo_condition()
                .to_owned()
                .with_fields(&["name", "sex"])
                .wheres("_id='Abc'")?
                .to_owned(),
        )
        .await?;

    abc.0
        .remove(
            new_mongo_condition()
                .to_owned()
                .with_fields(&["name", "sex"])
                .wheres("_id='Abc'")?
                .to_owned(),
        )
        .await?;

    Ok(())
}
