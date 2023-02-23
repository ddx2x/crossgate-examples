use async_stream::stream;
use axum::{
    extract::{self, Path},
    response::sse::{Event, Sse},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Extension, Router,
};
use hyper::{Body, StatusCode};
use tokio_context::context::Context;

use futures::{future::BoxFuture, stream::Stream};
use std::{convert::Infallible, net::SocketAddr, time::Duration};

use crossgate::store::Event as OpEvent;

use crate::{
    base::{Base, GpsInfo},
    db_wrapper::get_mongo_store,
};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Pagination {
    page: usize,
    per_page: usize,
    sort: String,
}

#[derive(serde::Serialize)]
pub(crate) struct Empty {}

#[derive(serde::Serialize)]
pub(crate) struct Message<T>(T);

pub(crate) fn respone_ok<T>(msg: T) -> Response<Body>
where
    T: serde::Serialize,
{
    let body = match serde_json::to_vec(&msg) {
        Ok(b) => Body::from(b),
        Err(e) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!("{}", e)))
                .unwrap()
        }
    };

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(body)
        .unwrap()
}

pub(crate) fn respone_failed<T>(status_code: StatusCode, msg: T) -> Response<Body>
where
    T: serde::Serialize,
{
    let body = match serde_json::to_vec(&msg) {
        Ok(b) => Body::from(b),
        Err(e) => Body::from(format!("marshal error: {}", e)),
    };

    Response::builder().status(status_code).body(body).unwrap()
}

async fn hello() -> &'static str {
    "base"
}

async fn list_local(Extension(base): Extension<Base>) -> Response<Body> {
    match base.list().await {
        Ok(locals) => respone_ok(Message(locals)),
        Err(e) => respone_failed(StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)),
    }
}
async fn list_local_name(Extension(base): Extension<Base>) -> Response<Body> {
    match base.list_local_unstructed().await {
        Ok(locals) => respone_ok(Message(locals)),
        Err(e) => respone_failed(StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)),
    }
}

async fn list_gpsinfo(Extension(base): Extension<Base>) -> Response<Body> {
    match base.list_gpsinfo().await {
        Ok(gps_infos) => respone_ok(Message(gps_infos)),
        Err(e) => respone_failed(StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)),
    }
}
async fn get_gpsinfo(Path(id): Path<String>, Extension(base): Extension<Base>) -> Response<Body> {
    match base.get_gpsinfo(&id).await {
        Ok(gps_infos) => respone_ok(Message(gps_infos)),
        Err(e) => respone_failed(StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)),
    }
}

async fn cretae_gpsinfo(
    Extension(base): Extension<Base>,
    extract::Json(gpsinfo): extract::Json<GpsInfo>,
) -> Response<Body> {
    match base.create_gpsinfo(gpsinfo).await {
        Ok(_) => respone_ok(""),
        Err(e) => respone_failed(StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)),
    }
}

async fn delete_gpsinfo(
    Path(id): Path<String>,
    Extension(base): Extension<Base>,
) -> impl IntoResponse {
    match base.delete_gpsinfo(&id).await {
        Ok(_) => respone_ok(""),
        Err(e) => respone_failed(StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)),
    }
}

async fn gps_count(Extension(base): Extension<Base>) -> impl IntoResponse {
    match base.gps_count().await {
        Ok(r) => respone_ok(r),
        Err(e) => respone_failed(StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)),
    }
}

async fn get_local(Extension(base): Extension<Base>) -> impl IntoResponse {
    match base.get_local("other").await {
        Ok(local) => respone_ok(Message(local)),
        Err(e) => respone_failed(StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)),
    }
}

async fn incr_local_count(Extension(base): Extension<Base>) -> impl IntoResponse {
    match base.incr_count().await {
        Ok(local) => respone_ok(Message(local)),
        Err(e) => respone_failed(StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)),
    }
}
async fn batch_remove_local(Extension(base): Extension<Base>) -> impl IntoResponse {
    match base.batch_remove_local().await {
        Ok(local) => respone_ok(Message(local)),
        Err(e) => respone_failed(StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)),
    }
}


async fn watch(
    Extension(base): Extension<Base>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream! {
        let (ctx, h) = Context::new();
        if let Ok(mut r) = base.watch(ctx).await{
              while let Some(item) = r.recv().await {
                if let OpEvent::Error(e) = item {
                    log::error!("error {:?}",e);
                    break;
                }
                yield Ok(Event::default().data(format!("{}", item)));
            }
        }
        h.cancel();
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(10))
            .text("keep-alive"),
    )
}

async fn watch2(
    Extension(base): Extension<Base>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream! {
        let (ctx, h) = Context::new();
        if let Ok(mut r) = base.watch2(ctx).await{
              while let Some(item) = r.recv().await {
                if let OpEvent::Error(e) = item {
                    log::error!("error {:?}",e);
                    break;
                }
                yield Ok(Event::default().data(format!("{}", item)));
            }
        }
        h.cancel();
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(10))
            .text("keep-alive"),
    )
}

pub fn run<'a>(addr: &'a SocketAddr) -> BoxFuture<'a, ()> {
    let block = async move {
        let base = crossgate_rs::micro::make_service(crate::base::Base::create(
            addr,
            get_mongo_store().await,
        ))
        .await;

        let app = Router::new()
            // gpsinf crud
            .route("/base/gps_info", get(list_gpsinfo).post(cretae_gpsinfo))
            .route("/base/gps_info_count", get(gps_count))
            .route(
                "/base/gps_info/:id",
                delete(delete_gpsinfo).get(get_gpsinfo),
            )
            // local base op
            .route("/base/locals", get(list_local))
            .route("/base/local", get(get_local))
            .route("/base/local/op/incr", get(incr_local_count))
            .route("/base/local/op/batch_remove", get(batch_remove_local))
            
            .route("/base", get(hello))
            .route("/base/local_unstruncted", get(list_local_name))
            // watch steam
            .route("/base/watch", get(watch))
            .route("/watch2", get(watch2))
            .layer(Extension(base));

        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    };
    Box::pin(block)
}
