use std::fs;

use anyhow::{anyhow, Context, Result};

use crate::models::{DBState, Epic, Status, Story};

pub struct JiraDatabase {
    database: Box<dyn Database>,
}

impl JiraDatabase {
    pub fn new(file_path: String) -> Self {
        Self {
            database: Box::new(JSONFileDatabase { file_path }),
        }
    }

    pub fn read(&self) -> Result<DBState> {
        self.database.read()
    }

    pub fn create_epic(&self, epic: Epic) -> Result<u32> {
        let mut data = self
            .database
            .read()
            .with_context(|| format!("Failed to read database on create_epic"))?;

        let new_epic_id = data.last_item_id + 1;
        data.last_item_id = new_epic_id;
        data.epics.insert(new_epic_id, epic);
        self.database
            .write(&data)
            .with_context(|| format!("Failed insert epic to database"))?;
        Ok(new_epic_id)
    }

    pub fn create_story(&self, story: Story, epic_id: u32) -> Result<u32> {
        let mut data = self
            .database
            .read()
            .with_context(|| format!("Failed to read database on create_story"))?;

        let Some(epic) = data.epics.get_mut(&epic_id) else {
            return Err(anyhow!("Failed to get epic with id: {}", epic_id));
        };

        let new_story_id = data.last_item_id + 1;
        data.last_item_id = new_story_id;
        epic.stories.push(new_story_id);
        data.stories.insert(new_story_id, story);

        self.database
            .write(&data)
            .with_context(|| format!("Failed insert create story on database"))?;

        Ok(new_story_id)
    }

    pub fn delete_epic(&self, epic_id: u32) -> Result<()> {
        let mut data = self
            .database
            .read()
            .with_context(|| format!("Failed to read database on delete_epic"))?;

        let Some(epic) = data.epics.get(&epic_id) else {
            return Err(anyhow!(format!("Fail to delete epic - inesistent - {epic_id}")));
        };

        for story_id in &epic.stories {
            data.stories.remove(&story_id);
        }
        data.epics.remove(&epic_id);

        self.database
            .write(&data)
            .with_context(|| format!("Failed write deleted epic data"))
    }

    pub fn update_epic_status(&self, epic_id: u32, status: Status) -> Result<()> {
        let mut data = self
            .database
            .read()
            .with_context(|| format!("Failed to read database on update_epic_status"))?;

        let Some(epic) = data.epics.get_mut(&epic_id) else {
            return Err(anyhow!("Epic with {epic_id} not found"))
        };
        epic.status = status;

        self.database
            .write(&data)
            .with_context(|| format!("Failed to update epic status on {epic_id}"))
    }

    pub fn update_story_status(&self, story_id: u32, status: Status) -> Result<()> {
        let mut data = self
            .database
            .read()
            .with_context(|| format!("Failed to read database on update_story_status"))?;

        let Some(story) = data.stories.get_mut(&story_id) else {
            return Err(anyhow!("Story with {story_id} not found"));
        };
        story.status = status;

        self.database
            .write(&data)
            .with_context(|| format!("Failed to update story status on {story_id}"))
    }
}

trait Database {
    fn read(&self) -> Result<DBState>;
    fn write(&self, db_state: &DBState) -> Result<()>;
}

struct JSONFileDatabase {
    pub file_path: String,
}

impl Database for JSONFileDatabase {
    fn read(&self) -> Result<DBState> {
        let file =
            fs::File::open(&self.file_path).with_context(|| format!("Error while opening file"))?;

        let db_state = serde_json::from_reader(file)?;
        Ok(db_state)
    }

    fn write(&self, db_state: &DBState) -> Result<()> {
        fs::write(&self.file_path, &serde_json::to_vec(db_state)?)?;
        Ok(())
    }
}

pub mod test_utils {
    use super::*;
    use std::{cell::RefCell, collections::HashMap};

    pub struct MockDB {
        last_written_state: RefCell<DBState>,
    }

    impl MockDB {
        pub fn new() -> Self {
            Self {
                last_written_state: RefCell::new(DBState {
                    last_item_id: 0,
                    epics: HashMap::new(),
                    stories: HashMap::new(),
                }),
            }
        }
    }

    impl Database for MockDB {
        fn read(&self) -> Result<DBState> {
            let state = self.last_written_state.borrow().clone();
            Ok(state)
        }

        fn write(&self, db_state: &DBState) -> Result<()> {
            let latest_state = &self.last_written_state;
            *latest_state.borrow_mut() = db_state.clone();
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_utils::MockDB;
    use super::*;

    #[test]
    fn create_epic_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("Epic 1".to_owned(), "Epic 1 description".to_owned());
        let result = db.create_epic(epic.clone());

        assert!(result.is_ok());

        let id = result.unwrap();
        let db_state = db.read().unwrap();

        let expected_id = 1;

        assert_eq!(id, expected_id);
        assert_eq!(db_state.last_item_id, expected_id);
        assert_eq!(db_state.epics.get(&id), Some(&epic));
    }

    #[test]
    fn create_story_should_error_if_invalid_epic_id() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let non_existent_epic_id = 999;
        let story = Story::new("".to_owned(), "".to_owned());
        let result = db.create_story(story, non_existent_epic_id);
        assert!(result.is_err());
    }

    #[test]
    fn create_story_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("Epic_1".to_owned(), "Custom_epic".to_owned());
        let story = Story::new("story_1".to_owned(), "description_1".to_owned());

        let create_epic_result = db.create_epic(epic.clone());
        assert!(create_epic_result.is_ok());

        let created_epic_id = create_epic_result.unwrap();
        let created_story_id = db.create_story(story.clone(), created_epic_id);

        let expected_epic_id = 1;
        let expected_story_id = 2;
        let db_state = db.read().unwrap();

        assert!(created_story_id.is_ok());
        assert_eq!(db_state.last_item_id, expected_story_id);
        assert_eq!(db_state.stories.get(&expected_story_id), Some(&story));
        assert!(db_state
            .epics
            .get(&expected_epic_id)
            .unwrap()
            .stories
            .contains(&expected_story_id));
    }

    #[test]
    fn delete_epic_should_error_if_invalid_epic_id() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };

        let inesistent_epic_id = 999;
        let result = db.delete_epic(inesistent_epic_id);

        assert!(result.is_err());
    }

    #[test]
    fn delete_epic_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };

        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let created_epic_id = db.create_epic(epic.clone()).unwrap();
        let created_story_id = db.create_story(story, created_epic_id).unwrap();
        let created_epic_db_state = db.read().unwrap();
        assert_eq!(created_epic_db_state.last_item_id, 2);

        let result = db.delete_epic(created_epic_id);
        let db_state = db.read().unwrap();
        let expected_last_item_id = 2;

        assert_eq!(db_state.last_item_id, expected_last_item_id);
        assert!(db_state.epics.get(&created_epic_id).is_none());
        assert!(db_state.stories.get(&created_story_id).is_none());
        assert!(result.is_ok());
    }

    #[test]
    fn update_epic_status_should_error_if_invalid_epic_id() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };

        let inesistent_epic_id = 999;
        let result = db.update_epic_status(inesistent_epic_id, Status::InProgress);
        assert!(result.is_err());
    }

    #[test]
    fn update_epic_status_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };

        let epic = Epic::new("".to_owned(), "".to_owned());
        let epic_id = db.create_epic(epic.clone()).unwrap();
        assert_ne!(&epic.status, &Status::Resolved);

        let result = db.update_epic_status(epic_id, Status::Resolved);
        assert!(result.is_ok());

        let db_state = db.read().unwrap();
        assert_eq!(
            db_state.epics.get(&epic_id).unwrap().status,
            Status::Resolved
        );
    }

    #[test]
    fn update_story_status_should_error_if_invalid_story_id() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let inesistent_story_status = 999;

        let result = db.update_story_status(inesistent_story_status, Status::InProgress);
        assert!(result.is_err());
    }

    #[test]
    fn update_story_should_work() {
        let db = JiraDatabase {
            database: Box::new(MockDB::new()),
        };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let epic_id = db.create_epic(epic.clone()).unwrap();
        let story_id = db.create_story(story.clone(), epic_id).unwrap();
        assert_ne!(story.status, Status::Resolved);

        let update_story_status_result = db.update_story_status(story_id, Status::Resolved);
        let db_state = db.read().unwrap();
        assert!(update_story_status_result.is_ok());
        assert_eq!(
            db_state.stories.get(&story_id).unwrap().status,
            Status::Resolved
        );
    }

    mod database {
        use std::collections::HashMap;
        use std::io::Write;

        use super::*;

        #[test]
        fn read_db_should_fail_with_invalid_path() {
            let db = JSONFileDatabase {
                file_path: "invalid_path".to_string(),
            };
            assert_eq!(db.read().is_err(), true);
        }

        #[test]
        fn read_db_should_fail_with_invalid_json() {
            let file_contents = r#"{ "last_item_id": 0 epics: {} stories {} }"#;
            let mut tmpfile = tempfile::NamedTempFile::new().unwrap();
            write!(tmpfile, "{}", file_contents).unwrap();
            let db = JSONFileDatabase {
                file_path: tmpfile
                    .path()
                    .to_str()
                    .expect("Failed to convert tmpfile path to str")
                    .to_string(),
            };

            let result = db.read();
            assert_eq!(result.is_err(), true);
        }

        #[test]
        fn read_db_should_parse_json_file() {
            let file_contents = r#"{ "last_item_id": 0, "epics": {}, "stories": {} }"#;
            let mut tmpfile = tempfile::NamedTempFile::new().unwrap();
            write!(tmpfile, "{}", file_contents).unwrap();
            let db = JSONFileDatabase {
                file_path: tmpfile
                    .path()
                    .to_str()
                    .expect("Failed to convert tmpfile path to str")
                    .to_string(),
            };

            let result = db.read();
            if let Err(e) = &result {
                println!("Error: {}", e);
            }
            assert!(result.is_ok());
        }

        #[test]
        fn write_db_should_word() {
            let file_contents = r#"{ "last_item_id": 0, "epics": {}, "stories": {} }"#;
            let mut tmpfile = tempfile::NamedTempFile::new().unwrap();
            write!(tmpfile, "{}", file_contents).unwrap();
            let db = JSONFileDatabase {
                file_path: tmpfile
                    .path()
                    .to_str()
                    .expect("Failed to convert tmpfile path to str")
                    .to_string(),
            };

            let story = Story::new("Story 1".to_owned(), "Description 1".to_owned());
            let epic = Epic {
                name: "Epic 1".to_owned(),
                description: "Description 1".to_owned(),
                status: Status::Open,
                stories: vec![2],
            };

            let db_state = DBState {
                last_item_id: 1,
                epics: HashMap::from_iter([(1, epic)]),
                stories: HashMap::from_iter([(2, story)]),
            };

            let write_result = db.write(&db_state);
            assert!(write_result.is_ok());

            let read_result = db.read().unwrap();
            assert_eq!(read_result, db_state);
        }
    }
}
