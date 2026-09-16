//! Public game logic for Feast Frenzy.
//!
//! The binary owns only the Macroquad window and delegates simulation,
//! persistence, data, and UI coordination to this library so integration tests
//! exercise the same code path as the shipped game.

pub mod app;
pub mod assets;
pub mod audio;
pub mod commands;
pub mod data;
pub mod engine;
pub mod gameplay;
pub mod lifecycle;
pub mod persistence;
pub mod player;
pub mod simulation;
pub mod state;
pub mod ui;
