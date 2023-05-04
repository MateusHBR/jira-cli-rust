use anyhow::{anyhow, Result};
use std::rc::Rc;

use crate::db::JiraDatabase;
use crate::models::Action;

use super::{page_helpers::get_column_string, Page};

pub struct StoryDetail {
    pub epic_id: u32,
    pub story_id: u32,
    pub db: Rc<JiraDatabase>,
}

impl Page for StoryDetail {
    fn draw_page(&self) -> Result<()> {
        let db_state = self.db.read()?;
        let story = db_state
            .stories
            .get(&self.story_id)
            .ok_or_else(|| anyhow!(format!("Failed to get story with id: {}", self.story_id)))?;

        println!("------------------------------ STORY ------------------------------");
        println!("  id  |     name     |         description         |    status    ");
        let story_id = get_column_string(&self.story_id.to_string(), 5);
        let story_name = get_column_string(&story.name, 12);
        let story_description = get_column_string(&story.description, 27);
        let story_status = get_column_string(&story.status.to_string(), 13);
        println!("{story_id} | {story_name} | {story_description} |{story_status}");

        println!();
        println!();

        println!("[p] previous | [u] update story | [d] delete story");

        Ok(())
    }

    fn handle_input(&self, input: &str) -> Result<Option<Action>> {
        match input {
            "p" => Ok(Some(Action::NavigateToPreviousPage)),
            "u" => Ok(Some(Action::UpdateStoryStatus {
                story_id: self.story_id,
            })),
            "d" => Ok(Some(Action::DeleteStory {
                epic_id: self.epic_id,
                story_id: self.story_id,
            })),
            _ => Ok(None),
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

    fn build_page() -> StoryDetail {
        let database = Box::new(MockDB::new());
        let db = Rc::new(JiraDatabase { database });

        let epic = Epic::new("".to_owned(), "".to_owned());
        let epic_id = db.create_epic(epic).unwrap();

        let story = Story::new("".to_owned(), "".to_owned());
        let story_id = db.create_story(story, epic_id).unwrap();

        StoryDetail {
            db,
            epic_id,
            story_id,
        }
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
        let page = StoryDetail {
            db,
            epic_id: 1,
            story_id: 2,
        };
        assert!(page.draw_page().is_err());
    }

    #[test]
    fn handle_input_should_not_throw_on_invalid_input() {
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
        let story_id = page.story_id;

        let previous_page = "p";
        assert_eq!(
            page.handle_input(previous_page).unwrap(),
            Some(Action::NavigateToPreviousPage)
        );

        let update_epic = "u";
        assert_eq!(
            page.handle_input(update_epic).unwrap(),
            Some(Action::UpdateStoryStatus { story_id }),
        );

        let delete_epic = "d";
        assert_eq!(
            page.handle_input(delete_epic).unwrap(),
            Some(Action::DeleteStory { epic_id, story_id }),
        );
    }
}
