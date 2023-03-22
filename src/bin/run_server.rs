#[macro_use]
extern crate rocket;

use portfolio::{allocate, analyze};
use simple_logger::SimpleLogger;

#[launch]
fn rocket() -> _ {
    SimpleLogger::new().init().unwrap();
    rocket::build().mount("/", routes![allocate, analyze])
}
