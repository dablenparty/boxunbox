use thiserror::Error as ThisError;

use crate::package::error::{ConfigRead, ConfigWrite};

#[derive(Debug, ThisError)]
pub enum PlanningError {
    #[error("failed to parse package config: {0}")]
    ConfigParse(#[from] ConfigRead),
    #[warn(deprecated_in_future)]
    #[error("failed to save TOML config: {0}")]
    ConfigWrite(#[from] ConfigWrite),
    #[error("failed to walk package tree: {0}")]
    Walkdir(#[from] walkdir::Error),
}

#[derive(Debug, ThisError)]
pub enum UnboxError {
    #[error("failed to parse package config: {0}")]
    ConfigParse(#[from] ConfigRead),
    #[warn(deprecated_in_future)]
    #[error("failed to save TOML config: {0}")]
    ConfigWrite(#[from] ConfigWrite),
    #[error("failed to plan unboxing: {0}")]
    Planning(#[from] PlanningError),
}
