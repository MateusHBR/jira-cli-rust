use anyhow::{anyhow, Result};
use itertools::Itertools;
use std::rc::Rc;

use super::{page_helpers::get_column_string, Page};
use crate::db::JiraDatabase;
use crate::models::Action;

pub struct EpicDetail {
    pub epic_id: u32,
    pub db: Rc<JiraDatabase>,
}

impl Page for EpicDetail {
    fn draw_page(&self) -> Result<()> {
        let db_state = self.db.read()?;
        let epic = db_state
            .epics
            .get(&self.epic_id)
            .ok_or_else(|| anyhow!(format!("Epic with {} not found", &self.epic_id)))?;

        println!("------------------------------ EPIC ------------------------------");
        println!("  id  |     name     |         description         |    status    ");
        let epic_id = get_column_string(&self.epic_id.to_string(), 5);
        let epic_name = get_column_string(&epic.name, 12);
        let epic_description = get_column_string(&epic.description, 27);
        let epic_status = get_column_string(&epic.status.to_string(), 13);
        println!("{epic_id} | {epic_name} | {epic_description} | {epic_status}");

        println!();

        println!("---------------------------- STORIES ----------------------------");
        println!("     id     |               name               |      status      ");
        db_state.stories.keys().sorted().for_each(|id| {
            let story = &db_state.stories[id];
            let story_id = get_column_string(&id.to_string(), 11);
            let story_name = get_column_string(&story.name, 32);
            let story_status = get_column_string(&story.status.to_string(), 17);
            println!("{story_id} | {story_name} | {story_status}");
        });

        println!();
        println!();

        println!("[p] previous | [u] update epic | [d] delete epic | [c] create story | [:id:] navigate to story");

        Ok(())
    }

    fn handle_input(&self, input: &str) -> Result<Option<Action>> {
        let db = self.db.read()?;
        let stories = db.stories;
        let epic_id = self.epic_id;
        match input {
            "p" => Ok(Some(Action::NavigateToPreviousPage)),
            "u" => Ok(Some(Action::UpdateEpicStatus { epic_id })),
            "d" => Ok(Some(Action::DeleteEpic { epic_id })),
            "c" => Ok(Some(Action::CreateStory { epic_id })),
            input => {
                let Ok(story_id) = input.parse::<u32>() else {
                    return Ok(None);
                };

                if stories.contains_key(&story_id) {
                    return Ok(Some(Action::NavigateToStoryDetail { epic_id, story_id }));
                }

                Ok(None)
            }
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::test_utils::MockDB,
        models::{Epic, Story},
    };

    fn build_page() -> EpicDetail {
        let database = Box::new(MockDB::new());
        let db = Rc::new(JiraDatabase { database });
        let epic = Epic::new("".to_owned(), "".to_owned());
        let epic_id = db.create_epic(epic).unwrap();

        EpicDetail { db, epic_id }
    }

    #[test]
    fn draw_page_should_not_throw_error() {
        let page = build_page();
        assert!(page.draw_page().is_ok());
    }

    #[test]
    fn draw_page_should_throw_error_when_epic_doesnt_exists() {
        let database = Box::new(MockDB::new());
        let db = Rc::new(JiraDatabase { database });
        let page = EpicDetail { db, epic_id: 1 };
        assert!(page.draw_page().is_err());
    }

    #[test]
    fn input_should_not_throw_on_non_existent_story() {
        let page = build_page();
        let empty = "";
        let non_existent_story = "999";
        assert!(page.handle_input(empty).is_ok());
        assert!(page.handle_input(non_existent_story).is_ok());
    }

    #[test]
    fn handle_input_should_not_throw_on_invalid_story_id() {
        let page = build_page();

        let junk_input = "j983f2j";
        let junk_input_with_valid_prefix = "q983f2j";
        let input_with_trailing_white_spaces = "q\n";
        assert!(page.handle_input(junk_input).unwrap().is_none());
        assert!(page
            .handle_input(junk_input_with_valid_prefix)
            .unwrap()
            .is_none());
        assert!(page
            .handle_input(input_with_trailing_white_spaces)
            .unwrap()
            .is_none());
    }

    #[test]
    fn handle_input_should_return_correct_action() {
        let page = build_page();
        let epic_id = page.epic_id;
        let story = Story::new("".to_owned(), "".to_owned());
        let story_id = page.db.create_story(story, epic_id).unwrap();

        let previous_page = "p";
        assert_eq!(
            page.handle_input(previous_page).unwrap(),
            Some(Action::NavigateToPreviousPage)
        );

        let update_epic = "u";
        assert_eq!(
            page.handle_input(update_epic).unwrap(),
            Some(Action::UpdateEpicStatus { epic_id }),
        );

        let delete_epic = "d";
        assert_eq!(
            page.handle_input(delete_epic).unwrap(),
            Some(Action::DeleteEpic { epic_id }),
        );

        let create_story = "c";
        assert_eq!(
            page.handle_input(create_story).unwrap(),
            Some(Action::CreateStory { epic_id }),
        );

        assert_eq!(
            page.handle_input(&story_id.to_string()).unwrap(),
            Some(Action::NavigateToStoryDetail { epic_id, story_id })
        );
    }
}
