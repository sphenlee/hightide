//! Hightide is an extension to the tide web framework.
//! It provides a higher level interface for building responses
//!
//! To use it wrap your endpoint with the `wrap` function. This wrapper allows your endpoints to
//! return various types that implement the `Responder` trait.
//!
//! Hightide also includes a Response type that is easier to use than the one provided by
//! tide. It has shortcut methods for setting the body to a JSON or Form payload, and for adding
//! typed headers from the `hyperx` crate.

use async_trait::async_trait;
use futures::Future;
use hyperx::header::Header;
use std::fmt::Display;
use tide::convert::Serialize;
use tide::http::headers::{ToHeaderValues, HeaderName, HeaderValue};
use tide::{Body, Request, StatusCode};

/// This trait is implemented for all the common types you can return from an endpoint
///
/// It's also implemented for `tide::Response` and `hightide::Response` for compatibility.
/// There is an implementation for `tide::Result<R> where R: Responder` which allows fallible
/// functions to be used as endpoints
///
/// ```
/// use hightide::{Responder, Json};
/// use tide::{StatusCode, Request};
///
/// fn example_1(_: Request<()>) -> impl Responder {
///     // return status code
///     StatusCode::NotFound
/// }
///
/// fn example_2(_: Request<()>) -> impl Responder {
///     // return strings (&str or String)
///     "Hello World"
/// }
///
/// fn example_3(_: Request<()>) -> impl Responder {
///     // return status code with data
///     (StatusCode::NotFound, "Not found!")
/// }
///
/// fn example_4(_: Request<()>) -> impl Responder {
///     // return JSON data - for any type implementing `serde::Serialize`
///     Json(MyData{ id: 0, key: "foo"})
/// }
///
/// fn example_5(_: Request<()>) -> tide::Result<impl Responder> {
///     // fallible functions too
///     // (also works the return type as `impl Responder` as long as Rust can infer the function returns `tide::Result`)
///     Ok((StatusCode::Conflict, "Already Exists"))
/// }
/// ```
pub trait Responder {
    fn into_response(self) -> tide::Result<tide::Response>;
}

/// Wraps the endpoint to bypass the orphan rules - pretty much ignore this one
pub struct High<F>(F);

/// Wrap an endpoint to allow it to return the Responder types
pub fn wrap<F>(f: F) -> High<F> {
    High(f)
}

// implement endpoint for fallible functions ( Request -> Into<Result<Response>>)
#[async_trait]
impl<State, F, Fut, Res> tide::Endpoint<State> for High<F>
where
    State: Clone + Send + Sync + 'static,
    F: Send + Sync + 'static + Fn(Request<State>) -> Fut,
    Fut: Future<Output = Res> + Send + 'static,
    Res: Responder + 'static,
{
    async fn call(&self, req: Request<State>) -> tide::Result<tide::Response> {
        let fut = (self.0)(req);
        let res = fut.await;
        res.into_response()
    }
}

/// A wrapper over `tide::Response` with better ergonomics
///
/// ```
/// use hightide::{Responder, Response};
/// use tide::Request;
/// fn example(_: Request<()>) -> impl Responder {
///     Response::ok().json(MyData{...})
/// }
/// ```
pub struct Response {
    inner: tide::Response,
}

impl Response {
    /// Create an empty response with status code OK (200)
    pub fn ok() -> Self {
        Self {
            inner: tide::Response::from(StatusCode::Ok),
        }
    }

    /// Create an empty response with the given status code
    pub fn status(s: StatusCode) -> Self {
        Self {
            inner: tide::Response::from(s),
        }
    }

    /// Set the body of the response
    pub fn body(mut self, body: impl Into<Body>) -> Self {
        self.inner.set_body(body);
        self
    }

    /// Set the body of the response to a JSON payload
    pub fn json(mut self, body: impl Serialize) -> tide::Result<Self> {
        self.inner.set_body(Body::from_json(&body)?);
        Ok(self)
    }

    /// Set the body of the response to form data
    pub fn form(mut self, body: impl Serialize) -> tide::Result<Self> {
        self.inner.set_body(Body::from_form(&body)?);
        Ok(self)
    }

    /// Set a header (from the `hyperx` typed headers)
    pub fn header<H: Header + Display>(mut self, h: H) -> Self {
        self.inner.insert_header(
            H::header_name(),
            h.to_string()
                .parse::<HeaderValue>()
                .expect("invalid header"),
        );
        self
    }

    /// Set a raw header (from the `http_types` crate)
    pub fn raw_header(mut self, name: impl Into<HeaderName>, key: impl ToHeaderValues) -> Self {
        self.inner.insert_header(name, key);
        self
    }

    /// Consume this response and return the inner `tide::Response`
    pub fn into_inner(self) -> tide::Response {
        self.inner
    }
}

impl Responder for StatusCode {
    fn into_response(self) -> tide::Result<tide::Response> {
        Ok(tide::Response::from(self))
    }
}

impl Responder for String {
    fn into_response(self) -> tide::Result<tide::Response> {
        Ok(tide::Response::from(self))
    }
}

impl Responder for &str {
    fn into_response(self) -> tide::Result<tide::Response> {
        Ok(tide::Response::from(self))
    }
}

impl Responder for &[u8] {
    fn into_response(self) -> tide::Result<tide::Response> {
        Ok(tide::Response::from(Body::from(self)))
    }
}

impl<R> Responder for (StatusCode, R)
where
    R: Responder,
{
    fn into_response(self) -> tide::Result<tide::Response> {
        let mut resp = self.1.into_response()?;
        resp.set_status(self.0);
        Ok(resp)
    }
}

/// A Wrapper to return a JSON payload. This can be wrapped over any `serde::Serialize` type.
/// ```
/// use tide::Request;
/// use hightide::{Responder, Json};
/// fn returns_json(_: Request<()>) -> impl Responder {
///     Json(vec!["an", "array"])
/// }
/// ```
pub struct Json<T: Serialize>(pub T);

impl<T: Serialize> Responder for Json<T> {
    fn into_response(self) -> tide::Result<tide::Response> {
        Response::ok().json(self.0).map(|r| r.into_inner())
    }
}

/// A Wrapper to return Form data. This can be wrapped over any `serde::Serialize` type.
pub struct Form<T: Serialize>(pub T);

impl<T: Serialize> Responder for Form<T> {
    fn into_response(self) -> tide::Result<tide::Response> {
        Response::ok().form(self.0).map(|r| r.into_inner())
    }
}

impl Responder for Response {
    fn into_response(self) -> tide::Result<tide::Response> {
        Ok(self.into_inner())
    }
}

impl Responder for tide::Response {
    fn into_response(self) -> tide::Result<tide::Response> {
        Ok(self)
    }
}

impl<R> Responder for tide::Result<R>
where
    R: Responder,
{
    fn into_response(self) -> tide::Result<tide::Response> {
        self.and_then(|r| r.into_response())
    }
}
