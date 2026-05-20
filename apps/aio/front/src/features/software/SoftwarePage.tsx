import {useCallback} from "react";
import {api} from "../../api/client";
import {JsonBlock, Panel} from "../../components/Panel";
import {useAsyncResource} from "../../hooks/useAsyncResource";

export function SoftwarePage() {
  const resource = useAsyncResource(async () => {
    const [catalog, installers] = await Promise.all([api.softwareCatalog(), api.softwareInstallers()]);
    return {catalog, installers};
  });

  const handleAction = useCallback(async (actionId: string) => {
    switch (actionId) {
      case "software-center.refresh":
      case "software-center.scan-installers":
        await resource.reload();
        break;
      case "software-center.organize-installers":
        await api.organizeInstallers();
        await resource.reload();
        break;
      default:
        break;
    }
  }, [resource]);

  return (
    <div className="pageGrid">
      <Panel title="Software Catalog" metric={resource.data?.catalog.items.length ?? 0}>
        <JsonBlock value={resource.data?.catalog ?? {loading: resource.loading, error: resource.error}} />
      </Panel>
      <Panel title="Installer Scan" metric={resource.data?.installers.length ?? 0}>
        <JsonBlock value={resource.data?.installers ?? []} />
      </Panel>
      <ActionBridge onAction={handleAction} />
    </div>
  );
}

function ActionBridge({onAction}: {onAction: (actionId: string) => Promise<void>}) {
  (window as Window & {__AIO_PAGE_ACTION__?: (actionId: string) => Promise<void>}).__AIO_PAGE_ACTION__ = onAction;
  return null;
}
