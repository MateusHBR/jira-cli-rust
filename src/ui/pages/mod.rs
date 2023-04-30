use crate::models::Action;
use anyhow::Result;

mod epic_detail_page;
mod home_page;
mod page_helpers;
mod story_detail_page;

pub trait Page {
    fn draw_page(&self) -> Result<()>;
    fn handle_input(&self, input: &str) -> Result<Option<Action>>;
}
