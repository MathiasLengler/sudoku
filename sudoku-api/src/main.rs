#[macro_use]
extern crate rocket;

use rocket::{Build, Rocket, serde::json::{Value,json}};

#[get("/")]
fn hello() -> &'static str {
    "'sudoku-api' up and running!"
}

#[get("/api")]
fn get_test() -> Value {
    json!({"hello": "world"})
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build().mount("/", routes![hello, get_test])
}
