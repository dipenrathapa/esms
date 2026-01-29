// Re-export modules for testing
pub mod api;
pub mod background;
pub mod business;
pub mod config;
pub mod error;
pub mod models;
pub mod retry;
pub mod sensor;
pub mod state;
pub mod storage;

// Include test modules when testing
#[cfg(test)]
mod business_tests;