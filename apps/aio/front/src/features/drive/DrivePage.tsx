import {useCallback, useMemo} from "react";
import {api} from "../../api/client";
import {JsonBlock, Panel} from "../../components/Panel";
import {useAsyncResource} from "../../hooks/useAsyncResource";

export function DrivePage() {
  const resource = useAsyncResource(async () => {
    const [snapshot, queue, conflicts, trackedRoots] = await Promise.all([
      api.driveSnapshot(),
      api.driveQueue(),
      api.driveConflicts(),
      api.driveTrackedRoots()
    ]);
    return {snapshot, queue, conflicts, trackedRoots};
  });

  const summary = useMemo(() => {
    if (!resource.data) {
      return [];
    }
    const {snapshot, queue, conflicts, trackedRoots} = resource.data;
    return [
      {label: "Roots", value: snapshot.roots.length},
      {label: "Hosted", value: snapshot.hosted.length},
      {label: "Tracked", value: snapshot.tracked.length},
      {label: "Queue", value: queue.length},
      {label: "Conflicts", value: conflicts.length},
      {label: "Tracked Roots", value: trackedRoots.length}
    ];
  }, [resource.data]);

  const handleAction = useCallback(async (actionId: string) => {
    switch (actionId) {
      case "drive.refresh":
        await resource.reload();
        break;
      case "drive.sync":
        await api.driveSync();
        await resource.reload();
        break;
      case "drive.retry-queue":
        await api.driveRetryQueue();
        await resource.reload();
        break;
      case "drive.host-skills":
        await api.driveHost();
        await resource.reload();
        break;
      case "drive.unhost-skills":
        await api.driveUnhost();
        await resource.reload();
        break;
      default:
        break;
    }
  }, [resource]);

  return (
    <div className="pageGrid">
      <div className="metricGrid">
        {summary.map(item => (
          <Panel key={item.label} title={item.label} metric={item.value}>
            <div className="metricText">{item.value}</div>
          </Panel>
        ))}
      </div>
      <Panel title="Snapshot">
        <JsonBlock value={resource.data?.snapshot ?? {loading: resource.loading, error: resource.error}} />
      </Panel>
      <Panel title="Queue">
        <JsonBlock value={resource.data?.queue ?? []} />
      </Panel>
      <Panel title="Conflicts">
        <JsonBlock value={resource.data?.conflicts ?? []} />
      </Panel>
      <Panel title="Tracked Roots">
        <JsonBlock value={resource.data?.trackedRoots ?? []} />
      </Panel>
      <ActionBridge onAction={handleAction} />
    </div>
  );
}

function ActionBridge({onAction}: {onAction: (actionId: string) => Promise<void>}) {
  (window as Window & {__AIO_PAGE_ACTION__?: (actionId: string) => Promise<void>}).__AIO_PAGE_ACTION__ = onAction;
  return null;
}
