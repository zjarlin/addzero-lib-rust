// Generated from openai/openai-openapi openapi.yaml. Do not edit by hand.
//! `MessagePhase` DTO.


/// Labels an `assistant` message as intermediate commentary (`commentary`) or the final answer
/// (`final_answer`). For models like `gpt-5.3-codex` and beyond, when sending follow-up requests,
/// preserve and resend phase on all assistant messages — dropping it can degrade performance. Not used
/// for user messages.
pub type MessagePhase = String;
