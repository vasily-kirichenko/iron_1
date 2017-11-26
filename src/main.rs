extern crate iron;
extern crate time;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate router;

use iron::prelude::*;
use iron::status;
use iron::{BeforeMiddleware, AfterMiddleware, typemap};
use time::precise_time_ns;

struct ResponseTime;

impl typemap::Key for ResponseTime {
    type Value = u64;
}

impl BeforeMiddleware for ResponseTime {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<ResponseTime>(precise_time_ns());
        Ok(())
    }
}

impl AfterMiddleware for ResponseTime {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        let delta = precise_time_ns() - *req.extensions.get::<ResponseTime>().unwrap();
        println!("Request took: {} ms", (delta as f64) / 1_000_000.0);
        Ok(res)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Good {
    message: String
}

fn main() {
    let router =
        router! {
            options: options "/" => options_handler,
            good: get "/good" => good_handler,
            error: get "/error" => error_handler
        };

    fn options_handler(_: &mut Request) -> IronResult<Response> {
        let mut resp = Response::new();
        resp.status = Some(status::Ok);
        resp.headers.set(iron::headers::AccessControlAllowOrigin::Any);
        Ok(resp)
    }

    fn good_handler(_: &mut Request) -> IronResult<Response> {
        let good = Good { message: "oops!".to_string() };
        Ok(match serde_json::to_string(&good) {
            Ok(json) => Response::with((status::Ok, json)),
            Err(_) => Response::with(status::BadRequest)
        })
    }

    fn error_handler(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "plane text error")))
    }

let mut chain = Chain::new(router);
chain.link_before(ResponseTime);
chain.link_after(ResponseTime);
Iron::new(chain).http("localhost:3000").unwrap();
}
