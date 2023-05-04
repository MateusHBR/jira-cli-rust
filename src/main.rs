use std::rc::Rc;

mod db;
mod io_utils;
mod models;
mod navigator;
mod ui;

fn main() {
    let db = Rc::new(db::JiraDatabase::new("data/db.json".to_owned()));
    let mut navigator = navigator::Navigator::new(db);

    loop {
        clearscreen::clear().unwrap();
        let current_page = navigator.get_current_page();
        let Some(page) = current_page else {
            break;
        };

        if let Err(e) = page.draw_page() {
            println!("Error rendering page: {}\nPress any key to continue...", e);
            io_utils::wait_for_key_press();
            break;
        }

        let input = io_utils::get_user_input();
        match page.handle_input(&input.trim()) {
            Ok(result) => {
                if let Some(action) = result {
                    navigator.handle_action(action).ok();
                }
            }
            Err(err) => {
                println!(
                    "Error handling page input: {}\nPress any key to continue...",
                    err
                );
                io_utils::wait_for_key_press();
                break;
            }
        }
    }
}
