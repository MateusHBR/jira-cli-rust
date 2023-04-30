use std::rc::Rc;

use itertools::Itertools;

use crate::db::JiraDatabase;
use crate::models::Action;

use super::{page_helpers::get_column_string, Page};

pub struct HomePage {
    pub db: Rc<JiraDatabase>,
}

impl Page for HomePage {
    fn draw_page(&self) -> anyhow::Result<()> {
        let db_state = self.db.read()?;
        println!("----------------------------- EPICS -----------------------------");
        println!("     id     |               name               |      status      ");
        db_state.epics.keys().sorted().for_each(|epic_id| {
            let epic = &db_state.epics[epic_id];
            let epic_id = get_column_string(&epic_id.to_string(), 11);
            let epic_name = get_column_string(&epic.name, 32);
            let epic_status = get_column_string(&epic.status.to_string(), 17);
            println!("{} | {} | {}", epic_id, epic_name, epic_status);
        });

        println!();
        println!();

        println!("[q] quit | [c] create epic | [:id:] navigate to epic");

        Ok(())
    }

    fn handle_input(&self, input: &str) -> anyhow::Result<Option<Action>> {
        match input {
            "q" => Ok(Some(Action::Exit)),
            "c" => Ok(Some(Action::CreateEpic)),
            input => {
                let db_state = &self.db.read()?;
                let Ok(epic_id) = input.parse::<u32>() else {
                    return Ok(None);
                };

                if db_state.epics.contains_key(&epic_id) {
                    return Ok(Some(Action::NavigateToEpicDetail { epic_id: epic_id }));
                }

                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    use crate::{
        db::{test_utils::MockDB, JiraDatabase},
        models::Epic,
    };

    fn build_page() -> HomePage {
        let db = Rc::new(JiraDatabase {
            database: Box::new(MockDB::new()),
        });

        HomePage { db }
    }

    #[test]
    fn draw_page_should_not_throw_error() {
        let page = build_page();

        assert!(page.draw_page().is_ok());
    }

    #[test]
    fn handle_input_should_not_throw_on_not_existent_epic_id() {
        let page = build_page();

        let input = "";
        assert!(page.handle_input(input).is_ok());

        let input = "1";
        assert!(page.handle_input(input).is_ok());
    }

    #[test]
    fn handle_input_should_not_throw_on_invalid_epic_id() {
        let page = build_page();

        let junk_input = "j983f2j";
        let junk_input_with_valid_prefix = "q983f2j";
        let input_with_trailing_white_spaces = "q\n";
        assert!(page.handle_input(junk_input).is_ok());
        assert!(page.handle_input(junk_input_with_valid_prefix).is_ok());
        assert!(page.handle_input(input_with_trailing_white_spaces).is_ok());
    }

    #[test]
    fn handle_input_should_return_the_correct_actions() {
        let page = build_page();
        let epic = Epic::new("".to_owned(), "".to_owned());
        let epic_id = page.db.create_epic(epic).unwrap();

        let quit_input = "q";
        let crete_epic_input = "c";
        assert_eq!(page.handle_input(quit_input).unwrap(), Some(Action::Exit));
        assert_eq!(
            page.handle_input(&epic_id.to_string()).unwrap(),
            Some(Action::NavigateToEpicDetail { epic_id: epic_id })
        );
        assert_eq!(
            page.handle_input(crete_epic_input).unwrap(),
            Some(Action::CreateEpic)
        );
    }
}
