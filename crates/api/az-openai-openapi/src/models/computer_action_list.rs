// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `ComputerActionList` DTO.

use crate::models::{
    ComputerAction,
};

/// Flattened batched actions for `computer_use`. Each action includes an `type` discriminator and
/// action-specific fields.
pub type ComputerActionList = Vec<ComputerAction>;
