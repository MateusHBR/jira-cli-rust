use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    NavigateToEpicDetail { epic_id: u32 },
    NavigateToStoryDetail { epic_id: u32, story_id: u32 },
    NavigateToPreviousPage,
    CreateEpic,
    UpdateEpicStatus { epic_id: u32 },
    DeleteEpic { epic_id: u32 },
    CreateStory { epic_id: u32 },
    UpdateStoryStatus { story_id: u32 },
    DeleteStory { epic_id: u32, story_id: u32 },
    Exit,
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = match self {
            Action::NavigateToEpicDetail { epic_id: _ } => "NavigateToEpicDetail",
            Action::NavigateToStoryDetail {
                epic_id: _,
                story_id: _,
            } => "NavigateToStoryDetail",
            Action::NavigateToPreviousPage => "NavigateToPreviousPage",
            Action::CreateEpic => "CreateEpic",
            Action::UpdateEpicStatus { epic_id: _ } => "UpdateEpicStatus",
            Action::DeleteEpic { epic_id: _ } => "DeleteEpic",
            Action::CreateStory { epic_id: _ } => "CreateStory",
            Action::UpdateStoryStatus { story_id: _ } => "UpdateStoryStatus",
            Action::DeleteStory {
                epic_id: _,
                story_id: _,
            } => "DeleteStory",
            Action::Exit => "Exit",
        };
        write!(f, "{result}")
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub enum Status {
    Open,
    InProgress,
    Resolved,
    Closed,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = match self {
            Status::Open => "Open",
            Status::InProgress => "In Progress",
            Status::Resolved => "Resolved",
            Status::Closed => "Closed",
        };

        write!(f, "{result}")
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct Epic {
    pub name: String,
    pub description: String,
    pub status: Status,
    pub stories: Vec<u32>,
}

impl Epic {
    pub fn new(name: String, description: String) -> Epic {
        Epic {
            name,
            description,
            status: Status::Open,
            stories: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Eq, Debug)]
pub struct Story {
    pub name: String,
    pub description: String,
    pub status: Status,
}

impl Story {
    pub fn new(name: String, description: String) -> Story {
        Story {
            name,
            description,
            status: Status::Open,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Eq, Debug)]
pub struct DBState {
    pub last_item_id: u32,
    pub epics: HashMap<u32, Epic>,
    pub stories: HashMap<u32, Story>,
}
