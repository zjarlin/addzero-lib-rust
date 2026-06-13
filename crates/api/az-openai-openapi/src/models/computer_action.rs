// Generated from OpenAPI spec. Do not edit by hand.
//! `ComputerAction` DTO.

use serde::{Deserialize, Serialize};

use crate::models::{
    ClickParam,
    DoubleClickAction,
    DragParam,
    KeyPressAction,
    MoveParam,
    ScreenshotParam,
    ScrollParam,
    TypeParam,
    WaitParam,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ComputerAction {
    ClickParam(ClickParam),
    DoubleClickAction(DoubleClickAction),
    DragParam(DragParam),
    KeyPressAction(KeyPressAction),
    MoveParam(MoveParam),
    ScreenshotParam(ScreenshotParam),
    ScrollParam(ScrollParam),
    TypeParam(TypeParam),
    WaitParam(WaitParam),
}
