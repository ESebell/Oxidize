mod auth;
mod dashboard;
mod workout;
mod stats_page;
mod settings;
mod routine_builder;

pub use auth::{Login, Register};
pub use dashboard::Dashboard;
pub use workout::Workout;
pub use stats_page::Stats;
pub use settings::Settings;
pub use routine_builder::RoutineBuilder;
