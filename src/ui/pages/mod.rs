use crate::models::Action;
use anyhow::Result;
use std::any::Any;

mod epic_detail_page;
mod home_page;
mod page_helpers;
mod story_detail_page;

pub use self::{epic_detail_page::EpicDetail, home_page::HomePage, story_detail_page::StoryDetail};

pub trait Page {
    fn draw_page(&self) -> Result<()>;
    fn handle_input(&self, input: &str) -> Result<Option<Action>>;
    fn as_any(&self) -> &dyn Any;
}
