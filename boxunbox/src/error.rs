use thiserror::Error as ThisError;

use crate::{
    package::error::{ConfigRead, ConfigWrite},
    plan::PlannedLink,
};

#[derive(Debug, ThisError)]
pub enum PlanningError {
    #[error("failed to parse package config")]
    ConfigParse(#[from] ConfigRead),
    #[warn(deprecated_in_future)]
    #[error("failed to save TOML config")]
    ConfigWrite(#[from] ConfigWrite),
    #[error("failed to walk package tree")]
    Walkdir(#[from] walkdir::Error),
}

#[derive(Debug, ThisError)]
pub enum UnboxError {
    #[error("cannot adopt symlink {0:?}")]
    AdoptSymlink(PlannedLink),
    #[error("failed to parse package config")]
    ConfigParse(#[from] ConfigRead),
    #[warn(deprecated_in_future)]
    #[error("failed to save TOML config")]
    ConfigWrite(#[from] ConfigWrite),
    #[error("dry run, not executing")]
    DryRun,
    #[error("failed to unbox {pl:?}")]
    Io {
        pl: PlannedLink,
        source: std::io::Error,
    },
    #[error("failed to plan unboxing")]
    Planning(#[from] PlanningError),
    #[error("target already exists for {0:?}")]
    TargetAlreadyExists(PlannedLink),
}
