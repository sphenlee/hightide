Hightide
--------

Hightide is an extension to the tide web framework.
It provides a higher level interface for building responses.

To use it wrap your endpoint with the `wrap` function. This wrapper allows your endpoints to
return various types that implement the `Responder` trait.

Hightide also includes a Response type that is easier to use than the one provided by
tide. It has shortcut methods for setting the body to a JSON or Form payload, and for adding
typed headers from the `hyperx` crate.

`Responder` is implemented for various types, for example `(StatusCode, impl Responder)` which
allows you to do:

```
use tide::{Request, StatusCode};
use hightide::Responder;

fn example(_: tide::Request<()>) -> impl Responder {
     (StatusCode::Conflict, "Already Exists")
}
```

Which is simpler than the equivalent code from plain `tide`:

```
use tide::{Request, StatusCode};
use hightide::Responder;

fn example(_: Request<()>) -> tide::Result {
    Ok(Response::builder(StatusCode::Conflict)
        .body("Already Exists")
        .build())
}
```

The `Json` wrapper also allows returning a JSON response more directly.

```
use tide::{Request};
use hightide::{Responder, Json};

fn example(_: tide::Request<()>) -> impl Responder {
     Json(MyData{ ... })
}
```

Compared to:

```
use tide::{Request, StatusCode};

fn example(_: Request<()>) -> tide::Result {
    Ok(Response::builder(StatusCode::Ok)
        .body(Body::from_json(&MyData{ ... })?)
        .build())
}
```
