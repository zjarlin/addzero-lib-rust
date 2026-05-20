import {useCallback} from "react";
import {api} from "../../api/client";
import {JsonBlock, Panel} from "../../components/Panel";
import {useAsyncResource} from "../../hooks/useAsyncResource";

export function AssetsPage() {
  const resource = useAsyncResource(api.assetGraph);

  const handleAction = useCallback(async (actionId: string) => {
    switch (actionId) {
      case "asset-hub.refresh":
        await resource.reload();
        break;
      case "asset-hub.sync":
        await api.syncAssets();
        await resource.reload();
        break;
      default:
        break;
    }
  }, [resource]);

  return (
    <div className="pageGrid">
      <Panel title="Asset Graph" metric={resource.data?.items?.length ?? resource.data?.nodes?.length ?? 0}>
        <JsonBlock value={resource.data ?? {loading: resource.loading, error: resource.error}} />
      </Panel>
      <ActionBridge onAction={handleAction} />
    </div>
  );
}

function ActionBridge({onAction}: {onAction: (actionId: string) => Promise<void>}) {
  (window as Window & {__AIO_PAGE_ACTION__?: (actionId: string) => Promise<void>}).__AIO_PAGE_ACTION__ = onAction;
  return null;
}
