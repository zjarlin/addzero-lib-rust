import {useCallback} from "react";
import {api} from "../../api/client";
import {JsonBlock, Panel} from "../../components/Panel";
import {useAsyncResource} from "../../hooks/useAsyncResource";
import type {GatewayRunRequest, GatewayRunResult} from "../../types";

interface GatewayState {
  plan: GatewayRunRequest;
  result?: GatewayRunResult;
}

export function GatewayPage() {
  const resource = useAsyncResource<GatewayState>(async () => {
    const plan = await api.gatewayExample();
    return {plan, result: undefined};
  });

  const handleAction = useCallback(async (actionId: string) => {
    switch (actionId) {
      case "edge-gateway.refresh":
        await resource.reload();
        break;
      case "edge-gateway.load-example":
        resource.setData({plan: await api.gatewayExample(), result: resource.data?.result});
        break;
      case "edge-gateway.run-example":
        {
          const plan = resource.data?.plan ?? (await api.gatewayExample());
          const result = await api.gatewayRun(plan);
          resource.setData({plan, result});
        }
        break;
      default:
        break;
    }
  }, [resource]);

  return (
    <div className="pageGrid">
      <Panel title="Gateway Plan" metric={resource.data?.plan.steps.length ?? 0}>
        <JsonBlock value={resource.data?.plan ?? {loading: resource.loading, error: resource.error}} />
      </Panel>
      <Panel title="Last Result" metric={resource.data?.result?.status ?? "idle"}>
        <JsonBlock value={resource.data?.result ?? {status: "idle"}} />
      </Panel>
      <ActionBridge onAction={handleAction} />
    </div>
  );
}

function ActionBridge({onAction}: {onAction: (actionId: string) => Promise<void>}) {
  (window as Window & {__AIO_PAGE_ACTION__?: (actionId: string) => Promise<void>}).__AIO_PAGE_ACTION__ = onAction;
  return null;
}
