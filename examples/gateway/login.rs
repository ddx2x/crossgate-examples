use futures::future::BoxFuture;
use hyper::{Body, Request, Response};

mod User {
    use crossgate::object::{metadata, Object};
    #[metadata(uid)]
    struct User {}
}

mod Role {
    use crossgate::object::{metadata, Object};

    #[metadata(uid)]
    struct Role {}
}

pub fn handle<'a>(
    r: &'a mut Request<Body>,
    w: &'a mut Response<Body>,
) -> BoxFuture<'a, crossgate_rs::micro::IntercepterType> {
    Box::pin(async move { crossgate_rs::micro::IntercepterType::Redirect })
}
