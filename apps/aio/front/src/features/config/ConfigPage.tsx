import {useCallback} from "react";
import {api} from "../../api/client";
import {JsonBlock, Panel} from "../../components/Panel";
import {useAsyncResource} from "../../hooks/useAsyncResource";

export function ConfigPage() {
  const resource = useAsyncResource(api.configLocalStatus);

  const handleAction = useCallback(async (actionId: string) => {
    switch (actionId) {
      case "config-center.refresh":
        await resource.reload();
        break;
      case "config-center.import-env-providers":
        await api.importEnvProviders();
        await resource.reload();
        break;
      case "config-center.test-openai":
        await api.testProvider("open_ai");
        await resource.reload();
        break;
      case "config-center.test-anthropic":
        await api.testProvider("anthropic");
        await resource.reload();
        break;
      case "config-center.test-gemini":
        await api.testProvider("gemini");
        await resource.reload();
        break;
      default:
        break;
    }
  }, [resource]);

  return (
    <div className="pageGrid">
      <Panel title="Local Status">
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
