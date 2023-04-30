mod db;
mod models;
mod ui;

fn main() {
    let db = db::JiraDatabase::new("data/db.json".to_owned());
    let epic = models::Epic::new("My dummy epic".to_owned(), "xd".to_owned());
    db.create_epic(epic).unwrap();
}
