export const AI_WORKSPACE_SEED_EVENT = "aio:ai-workspace-seed";

export interface AiWorkspaceSeedDetail {
    path?: string;
    material?: string;
    prompt?: string;
    open?: boolean;
    navigate?: boolean;
    appendMaterial?: boolean;
    resetMessages?: boolean;
}

export function emitAiWorkspaceSeed(detail: AiWorkspaceSeedDetail) {
    if (typeof window === "undefined") {
        return;
    }
    window.dispatchEvent(
        new CustomEvent<AiWorkspaceSeedDetail>(AI_WORKSPACE_SEED_EVENT, {
            detail,
        }),
    );
}
