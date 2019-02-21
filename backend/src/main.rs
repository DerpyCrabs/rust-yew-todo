#![feature(proc_macro_hygiene)]
#![feature(decl_macro)]

#[macro_use]
extern crate rust_embed;
#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate maud;

use maud::{html, Markup, DOCTYPE};
use rocket::http::{ContentType, Status};
use rocket::response;
use rocket_contrib::json::Json;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Cursor;
use std::path::PathBuf;

use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct State {
    root: Entry,
    view: usize,
    id: EntryId,
    input: String,
    add_task: usize,
    add_folder: usize,
    editing: Option<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct EntryId {
    latest_id: usize,
}

#[derive(Serialize, Deserialize)]
pub enum Entry {
    Folder(Folder),
    Task(Task),
}

#[derive(Serialize, Deserialize)]
pub struct Folder {
    name: String,
    entries: Vec<Entry>,
    parent: Option<usize>,
    id: usize,
}

#[derive(Serialize, Deserialize)]
pub struct Task {
    name: String,
    done: bool,
    id: usize,
}

#[derive(RustEmbed)]
#[folder = "static/"]
struct Static;

fn main() {
    let path = match std::env::var("TASKS") {
        Ok(tasks) => String::from(tasks),
        Err(_) => String::from("tasks.db"),
    };
    let path_arc = Arc::new(path);
    rocket::ignite()
        .mount("/", routes![index, static_file, get_tasks, update_tasks])
        .manage(path_arc)
        .launch();
}

#[get("/")]
fn index() -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta http-equiv="X-UA-Compatible" content="IE=edge";
                meta content="yes" name="apple-mobile-web-app-capable";
                meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no";
                title { "Oxidized tasks" }

                link rel="stylesheet" href="styles.css" {}
                script src=("jquery-3.3.1.min.js") {}
                script src=("fix_ios.js") {}
            }
            body {}
            script src=("frontend.js") {}
        }
    }
}

#[get("/<path..>")]
fn static_file<'r>(path: PathBuf) -> rocket::response::Result<'r> {
    let filename = path.display().to_string();
    let ext = path
        .as_path()
        .extension()
        .and_then(OsStr::to_str)
        .expect("Could not get file extension");
    let content_type = ContentType::from_extension(ext).expect("Could not get file content type");
    Static::get(&filename.clone()).map_or_else(
        || Err(Status::NotFound),
        |d| {
            response::Response::build()
                .header(content_type)
                .sized_body(Cursor::new(d))
                .ok()
        },
    )
}

#[get("/tasks")]
fn get_tasks(path: rocket::State<Arc<String>>) -> Json<State> {
    let path: &str = path.as_str();
    let reader = File::open(path).expect("Can't open db");
    let state = serde_json::from_reader(reader).expect("Can't deserialize State");
    Json(state)
}

#[post("/tasks", format = "application/json", data = "<tasks>")]
fn update_tasks(
    path: rocket::State<Arc<String>>,
    tasks: Json<State>,
) -> rocket::response::status::Accepted<String> {
    let path: &str = path.as_str();
    let writer = File::create(path).expect("Can't write to db");
    serde_json::to_writer(writer, &tasks.0).expect("Can't serialize tasks");
    rocket::response::status::Accepted(Some(format!("success")))
}
