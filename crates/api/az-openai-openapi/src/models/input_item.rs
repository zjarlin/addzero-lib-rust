// Generated from OpenAPI spec. Do not edit by hand.
//! `InputItem` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    EasyInputMessage,
    Item,
    ItemReferenceParam,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InputItem {
    EasyInputMessage(EasyInputMessage),
    Item(Item),
    ItemReferenceParam(ItemReferenceParam),
}
