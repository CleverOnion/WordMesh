pub mod auth;
pub mod sense;
pub mod word;

pub use sense::{SenseService, SenseUpdateInput};
pub use word::{AddWordInput, SearchOptions, SenseInput, WordService};
