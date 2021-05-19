#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;

use rocket::{Build, Rocket};
use rocket_contrib::json::JsonValue;

#[get("/")]
fn hello() -> &'static str {
    "'sudoku-api' up and running!"
}

#[get("/api")]
fn get_test() -> JsonValue {
    json!({"hello": "world"})
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build().mount("/", routes![hello, get_test])
}
