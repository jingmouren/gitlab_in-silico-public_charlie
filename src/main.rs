#[macro_use]
extern crate rocket;

use portfolio::allocate;

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![allocate])
}
