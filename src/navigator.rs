use crate::{
    db::JiraDatabase,
    models::Action,
    ui::{EpicDetail, HomePage, Page, Prompts, StoryDetail},
};
use anyhow::{anyhow, Context, Result};
use std::rc::Rc;

pub struct Navigator {
    pages: Vec<Box<dyn Page>>,
    prompts: Prompts,
    db: Rc<JiraDatabase>,
}

impl Navigator {
    pub fn new(db: Rc<JiraDatabase>) -> Self {
        Self {
            pages: vec![Box::new(HomePage { db: Rc::clone(&db) })],
            prompts: Prompts::new(),
            db,
        }
    }

    pub fn get_current_page(&self) -> Option<&Box<dyn Page>> {
        self.pages.last()
    }

    pub fn handle_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::NavigateToEpicDetail { epic_id } => {
                let epic_details_page = Box::new(EpicDetail {
                    db: Rc::clone(&self.db),
                    epic_id,
                });

                self.pages.push(epic_details_page);
            }
            Action::NavigateToStoryDetail { epic_id, story_id } => {
                let story_details_page = Box::new(StoryDetail {
                    db: Rc::clone(&self.db),
                    epic_id,
                    story_id,
                });

                self.pages.push(story_details_page);
            }
            Action::NavigateToPreviousPage => {
                if !self.pages.is_empty() {
                    self.pages.pop();
                }
            }
            Action::CreateEpic => {
                let epic = (self.prompts.create_epic)();
                self.db
                    .create_epic(epic)
                    .with_context(|| anyhow!("Failed to create epic"))?;
            }
            Action::UpdateEpicStatus { epic_id } => {
                let epic_status = (self.prompts.update_status)();
                if let Some(status) = epic_status {
                    self.db
                        .update_epic_status(epic_id, status)
                        .with_context(|| anyhow!("Failed to update epic status"))?;
                }
            }
            Action::DeleteEpic { epic_id } => {
                let should_delete_epic = (self.prompts.delete_epic)();

                if should_delete_epic {
                    self.db
                        .delete_epic(epic_id)
                        .with_context(|| anyhow!("Failed to delete epic"))?;

                    if !self.pages.is_empty() {
                        self.pages.pop();
                    }
                }
            }
            Action::CreateStory { epic_id } => {
                let story = (self.prompts.create_story)();
                self.db
                    .create_story(story, epic_id)
                    .with_context(|| anyhow!("Failed to create story"))?;
            }
            Action::UpdateStoryStatus { story_id } => {
                let status = (self.prompts.update_status)();
                if let Some(status) = status {
                    self.db
                        .update_story_status(story_id, status)
                        .with_context(|| anyhow!("Failed to update story status!"))?;
                }
            }
            Action::DeleteStory { epic_id, story_id } => {
                let ok = (self.prompts.delete_story)();
                if ok {
                    self.db
                        .delete_story(epic_id, story_id)
                        .with_context(|| anyhow!("Failed to delete story"))?;

                    if !self.pages.is_empty() {
                        self.pages.pop();
                    }
                }
            }
            Action::Exit => self.pages.clear(),
        }
        Ok(())
    }

    fn get_page_count(&self) -> usize {
        self.pages.len()
    }

    fn set_prompts(&mut self, prompts: Prompts) {
        self.prompts = prompts
    }

    fn add_page(&mut self, page: Box<dyn Page>) {
        self.pages.push(page);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::test_utils::MockDB,
        models::{Epic, Status, Story},
    };

    #[test]
    fn should_start_on_home_page() {
        let db = Rc::new(JiraDatabase {
            database: Box::new(MockDB::new()),
        });
        let nav = Navigator::new(db);

        assert_eq!(nav.get_page_count(), 1);

        let current_page = nav.get_current_page().unwrap();
        let home_page = current_page.as_any().downcast_ref::<HomePage>();

        assert_eq!(home_page.is_some(), true);
    }

    #[test]
    fn handle_action_should_navigate_pages() {
        let db = Rc::new(JiraDatabase {
            database: Box::new(MockDB::new()),
        });
        let mut nav = Navigator::new(db);

        nav.handle_action(Action::NavigateToEpicDetail { epic_id: 1 })
            .unwrap();
        let current_page = nav.get_current_page().unwrap();
        assert_eq!(nav.get_page_count(), 2);
        assert!(current_page.as_any().downcast_ref::<EpicDetail>().is_some());

        nav.handle_action(Action::NavigateToStoryDetail {
            epic_id: 1,
            story_id: 2,
        })
        .unwrap();
        let current_page = nav.get_current_page().unwrap();
        assert_eq!(nav.get_page_count(), 3);
        assert!(current_page
            .as_any()
            .downcast_ref::<StoryDetail>()
            .is_some());

        nav.handle_action(Action::NavigateToPreviousPage).unwrap();
        let current_page = nav.get_current_page().unwrap();
        assert_eq!(nav.get_page_count(), 2);
        assert!(current_page.as_any().downcast_ref::<EpicDetail>().is_some());

        nav.handle_action(Action::NavigateToPreviousPage).unwrap();
        let current_page = nav.get_current_page().unwrap();
        assert_eq!(nav.get_page_count(), 1);
        assert!(current_page.as_any().downcast_ref::<HomePage>().is_some());

        nav.handle_action(Action::NavigateToPreviousPage).unwrap();
        assert_eq!(nav.get_page_count(), 0);
        assert!(nav.get_current_page().is_none());

        nav.handle_action(Action::NavigateToPreviousPage).unwrap();
        assert_eq!(nav.get_page_count(), 0);
        assert!(nav.get_current_page().is_none());
    }

    #[test]
    fn handle_exit_action_should_clear_pages() {
        let db = Rc::new(JiraDatabase {
            database: Box::new(MockDB::new()),
        });
        let mut nav = Navigator::new(db);

        nav.handle_action(Action::NavigateToEpicDetail { epic_id: 1 })
            .unwrap();
        assert_eq!(nav.get_page_count(), 2);

        nav.handle_action(Action::Exit).unwrap();
        assert_eq!(nav.get_page_count(), 0);
    }

    #[test]
    fn handle_action_should_handle_create_epic() {
        let db = Rc::new(JiraDatabase {
            database: Box::new(MockDB::new()),
        });
        let mut nav = Navigator::new(Rc::clone(&db));

        let mut prompts = Prompts::new();
        prompts.create_epic = Box::new(|| Epic::new("name".to_owned(), "description".to_owned()));

        let db_state = db.read().unwrap();
        assert_eq!(db_state.epics.len(), 0);

        nav.set_prompts(prompts);
        nav.handle_action(Action::CreateEpic).unwrap();

        let db_state = db.read().unwrap();
        assert_eq!(db_state.epics.len(), 1);

        let (_, epic) = db_state.epics.iter().next().unwrap();
        assert_eq!(epic.name, "name".to_owned());
        assert_eq!(epic.description, "description".to_owned());
    }

    #[test]
    fn handle_action_should_update_epic() {
        let db = Rc::new(JiraDatabase {
            database: Box::new(MockDB::new()),
        });
        let epic = Epic::new("name".to_owned(), "description".to_owned());
        let epic_id = db.create_epic(epic).unwrap();
        let db_state = db.read().unwrap();
        assert_eq!(db_state.epics.len(), 1);
        assert_ne!(
            db_state.epics.iter().next().unwrap().1.status,
            Status::InProgress,
        );

        let mut prompts = Prompts::new();
        prompts.update_status = Box::new(|| Some(Status::InProgress));
        let mut nav = Navigator::new(Rc::clone(&db));
        nav.set_prompts(prompts);

        nav.handle_action(Action::UpdateEpicStatus { epic_id })
            .unwrap();
        let db_state = db.read().unwrap();
        assert_eq!(db_state.epics.len(), 1);
        assert_eq!(
            db_state.epics.iter().next().unwrap().1.status,
            Status::InProgress,
        );
    }

    #[test]
    fn handle_action_should_delete_epic() {
        let db = Rc::new(JiraDatabase {
            database: Box::new(MockDB::new()),
        });
        let epic = Epic::new("name".to_owned(), "description".to_owned());
        let epic_id = db.create_epic(epic).unwrap();

        let mut prompts = Prompts::new();
        prompts.delete_epic = Box::new(|| true);
        let mut nav = Navigator::new(Rc::clone(&db));
        nav.add_page(Box::new(EpicDetail {
            db: Rc::clone(&db),
            epic_id,
        }));
        nav.set_prompts(prompts);

        let db_state = db.read().unwrap();
        assert_eq!(db_state.epics.len(), 1);
        let current_page = nav.get_current_page().unwrap();
        assert_eq!(
            current_page.as_any().downcast_ref::<EpicDetail>().is_some(),
            true
        );
        assert_eq!(nav.pages.len(), 2);

        nav.handle_action(Action::DeleteEpic { epic_id }).unwrap();
        let current_page = nav.get_current_page().unwrap();
        assert_eq!(nav.pages.len(), 1);
        assert_eq!(
            current_page.as_any().downcast_ref::<EpicDetail>().is_some(),
            false
        );
        let db_state = db.read().unwrap();
        assert!(db_state.epics.is_empty());
    }

    #[test]
    fn handle_action_should_create_story() {
        let db = Rc::new(JiraDatabase {
            database: Box::new(MockDB::new()),
        });
        let epic_id = db
            .create_epic(Epic::new("".to_owned(), "".to_owned()))
            .unwrap();

        let create_story_prompt =
            Box::new(|| Story::new("name".to_owned(), "description".to_owned()));
        let mut prompts = Prompts::new();
        prompts.create_story = create_story_prompt;
        let mut nav = Navigator::new(Rc::clone(&db));
        nav.set_prompts(prompts);

        let db_state = db.read().unwrap();
        assert!(db_state.stories.is_empty());

        nav.handle_action(Action::CreateStory { epic_id }).unwrap();

        let db_state = db.read().unwrap();
        assert_eq!(db_state.stories.is_empty(), false);
        let (story_id, story) = db_state.stories.iter().next().unwrap();
        assert_eq!(story.name, "name");
        assert_eq!(story.description, "description");

        let epic = db_state.epics.get(&epic_id).unwrap();
        assert!(epic.stories.contains(story_id));
    }

    #[test]
    fn handle_action_should_update_story() {
        let db = Rc::new(JiraDatabase {
            database: Box::new(MockDB::new()),
        });
        let epic_id = db
            .create_epic(Epic::new("".to_owned(), "".to_owned()))
            .unwrap();
        let story_id = db
            .create_story(Story::new("".to_owned(), "".to_owned()), epic_id)
            .unwrap();
        let db_state = db.read().unwrap();
        assert_ne!(
            db_state.stories.get(&story_id).unwrap().status,
            Status::InProgress
        );

        let mut prompts = Prompts::new();
        prompts.update_status = Box::new(|| Some(Status::InProgress));
        let mut nav = Navigator::new(Rc::clone(&db));
        nav.set_prompts(prompts);

        nav.handle_action(Action::UpdateStoryStatus { story_id })
            .unwrap();

        let db_state = db.read().unwrap();
        assert_eq!(
            db_state.stories.get(&story_id).unwrap().status,
            Status::InProgress
        );
    }

    #[test]
    fn handle_action_should_delete_story() {
        let db = Rc::new(JiraDatabase {
            database: Box::new(MockDB::new()),
        });
        let epic_id = db
            .create_epic(Epic::new("".to_owned(), "".to_owned()))
            .unwrap();
        let story_id = db
            .create_story(Story::new("".to_owned(), "".to_owned()), epic_id)
            .unwrap();
        let db_state = db.read().unwrap();
        assert_eq!(db_state.stories.is_empty(), false);
        assert!(db_state
            .epics
            .get(&epic_id)
            .unwrap()
            .stories
            .contains(&story_id));

        let mut prompts = Prompts::new();
        prompts.delete_story = Box::new(|| true);
        let mut nav = Navigator::new(Rc::clone(&db));
        nav.set_prompts(prompts);
        nav.add_page(Box::new(StoryDetail {
            db: Rc::clone(&db),
            epic_id,
            story_id,
        }));
        let current_page = nav.get_current_page().unwrap();
        assert_eq!(
            current_page
                .as_any()
                .downcast_ref::<StoryDetail>()
                .is_some(),
            true
        );

        nav.handle_action(Action::DeleteStory { epic_id, story_id })
            .unwrap();

        let db_state = db.read().unwrap();
        assert!(db_state.stories.is_empty());
        assert_eq!(
            db_state
                .epics
                .get(&epic_id)
                .unwrap()
                .stories
                .contains(&story_id),
            false
        );
        let current_page = nav.get_current_page().unwrap();
        assert_eq!(
            current_page
                .as_any()
                .downcast_ref::<StoryDetail>()
                .is_some(),
            false
        );
    }
}
