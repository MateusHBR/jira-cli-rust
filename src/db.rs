use std::fs;

use anyhow::Result;

use crate::models::{DBState, Epic, Status, Story};

trait Database {
    fn read(&self) -> Result<DBState>;
    fn write(&self, db_state: &DBState) -> Result<()>;
}

struct JSONFileDatabase {
    pub file_path: String,
}

impl Database for JSONFileDatabase {
    fn read(&self) -> Result<DBState> {
        let file = fs::File::open(&self.file_path)?;
        let db_state = serde_json::from_reader(file)?;
        Ok(db_state)
    }

    fn write(&self, db_state: &DBState) -> Result<()> {
        fs::write(&self.file_path, &serde_json::to_vec(db_state)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
