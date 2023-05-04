mod db;
mod io_utils;
mod models;
mod navigator;
mod ui;

fn main() {
    let db = db::JiraDatabase::new("data/db.json".to_owned());
    let epic = models::Epic::new("My dummy epic".to_owned(), "xd".to_owned());
    db.create_epic(epic).unwrap();
}
