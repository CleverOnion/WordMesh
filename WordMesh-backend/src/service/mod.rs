pub mod assoc;
pub mod auth;
pub mod sense;
pub mod word;

pub use assoc::AssocService;
pub use sense::{SenseService, SenseUpdateInput};
pub use word::{AddWordInput, SearchOptions, SenseInput, WordService};
