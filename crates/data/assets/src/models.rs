automod::dir!(pub(crate) "src/models");

pub(crate) fn asset_models() -> toasty::ModelSet {
    toasty::models!(
        asset::AssetRecord,
        asset_edge::AssetEdgeRecord,
        ai_model_provider::AiModelProviderRecord,
        ai_prompt_button::AiPromptButtonRecord
    )
}
