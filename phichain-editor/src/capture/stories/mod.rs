mod audit;
mod quick_start;

use super::story::Story;

const STORIES: &[(&str, fn() -> Story)] = &[
    ("quick-start", quick_start::quick_start),
    ("audit", audit::audit),
];

pub fn find(name: &str) -> Option<Story> {
    STORIES
        .iter()
        .find(|(story, _)| *story == name)
        .map(|(_, build)| build())
}

pub fn names() -> Vec<&'static str> {
    STORIES.iter().map(|(name, _)| *name).collect()
}
