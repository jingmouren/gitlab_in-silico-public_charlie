#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use portfolio::static_rocket_route_info_for_allocate;

fn main() {
    rocket::ignite().mount("/", routes![allocate]).launch();
}
