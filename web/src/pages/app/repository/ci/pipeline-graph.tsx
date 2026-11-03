import { useEffect, useMemo } from "react";
import dagre from "@dagrejs/dagre";
import {
  Background,
  Controls,
  Handle,
  MiniMap,
  Position,
  ReactFlow,
  useNodesState,
  type Connection,
  type Node,
  type NodeProps,
} from "@xyflow/react";
import { Box, CheckCircle2, Circle, Clock3, Loader2, XCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { needsOf, stagesOf, type CIConfig, type CIJob } from "./config-model";
import "@xyflow/react/dist/style.css";
import "./flow.scss";

type JobNode = Node<{ name: string; job: CIJob; status?: string; t: (key: string) => string }, "job">;
const StatusIcon = ({ status }: { status?: string }) => {
  if (status === "succeeded") return <CheckCircle2 className="size-4" style={{ color: "hsl(var(--success))" }} />;
  if (status === "failed") return <XCircle className="size-4 text-destructive" />;
  if (status === "running") return <Loader2 className="size-4 animate-spin text-primary motion-reduce:animate-none" />;
  if (status === "pending" || status === "blocked") return <Clock3 className="size-4 text-muted-foreground" />;
  return <Circle className="size-4 text-muted-foreground" />;
};
const TaskNode = ({ data, selected }: NodeProps<JobNode>) => (
  <div className={`ci-task-node ${selected ? "is-selected" : ""}`}>
    <Handle type="target" position={Position.Left} />
    <div className="flex items-center gap-2 font-semibold">
      <Box className="size-4 text-muted-foreground" />
      <span className="truncate">{data.name}</span>
      {data.status && <StatusIcon status={data.status} />}
    </div>
    <div className="truncate text-xs text-muted-foreground">{data.job.image || data.t("Default runner image")}</div>
    <pre className="mt-3 truncate border-t border-border pt-2 text-xs">
      {(typeof data.job.script === "string" ? data.job.script : data.job.script.join("\n")).slice(0, 180)}
    </pre>
    {data.status && <p className="mt-2 text-xs">{data.t(data.status)}</p>}
    <Handle type="source" position={Position.Right} />
  </div>
);
const StageHeader = ({ data }: NodeProps<Node<{ label: string }, "stage">>) => (
  <div className="border-b border-border bg-muted/50 px-3 py-3 text-sm font-medium">{data.label}</div>
);
const nodeTypes = { job: TaskNode, stage: StageHeader };
const layout = (config: CIConfig, t: (key: string) => string, statuses?: Record<string, string>): Node[] => {
  const stages = stagesOf(config);
  const graph = new dagre.graphlib.Graph().setDefaultEdgeLabel(() => ({}));
  graph.setGraph({ rankdir: "LR", nodesep: 70, ranksep: 80 });
  Object.keys(config.jobs).forEach((name) => graph.setNode(name, { width: 180, height: 135 }));
  Object.keys(config.jobs).forEach((name) => needsOf(config, name).forEach((need) => graph.setEdge(need, name)));
  dagre.layout(graph);
  const groups = stages.map((stage) =>
    Object.entries(config.jobs)
      .filter(([, job]) => (job.stage ?? "test") === stage)
      .sort(([a], [b]) => graph.node(a).y - graph.node(b).y),
  );
  const height = Math.max(400, ...groups.map((items) => items.length * 185 + 100));
  return stages.flatMap((stage, index): Node[] => [
    {
      id: `stage:${index}`,
      type: "stage",
      position: { x: index * 215, y: 0 },
      data: { label: stage },
      style: { width: 200, height },
      selectable: false,
      draggable: false,
      connectable: false,
    },
    ...groups[index].map(([name, job], row): JobNode => ({
      id: `job:${name}`,
      type: "job",
      parentId: `stage:${index}`,
      extent: "parent",
      position: { x: 10, y: 75 + row * 185 },
      data: { name, job: { ...job, image: job.image || config.image }, status: statuses?.[name], t },
    })),
  ]);
};
interface Props {
  config: CIConfig;
  selected?: string;
  statuses?: Record<string, string>;
  readonly?: boolean;
  onSelect: (name: string) => void;
  onConnect?: (connection: Connection) => void;
  isValidConnection?: (connection: Connection) => boolean;
  t: (key: string) => string;
}
export const PipelineGraph = ({
  config,
  selected,
  statuses,
  readonly = false,
  onSelect,
  onConnect,
  isValidConnection,
  t,
}: Props) => {
  const initialNodes = useMemo(() => layout(config, t, statuses), [config, t, statuses]);
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  useEffect(() => {
    setNodes((current) =>
      initialNodes.map((node) => {
        const previous = current.find((item) => item.id === node.id && item.parentId === node.parentId);
        return previous && node.type === "job" ? { ...node, position: previous.position } : node;
      }),
    );
  }, [initialNodes, setNodes]);
  const edges = useMemo(
    () =>
      Object.keys(config.jobs).flatMap((name) =>
        needsOf(config, name).map((need) => ({
          id: JSON.stringify([need, name]),
          source: `job:${need}`,
          target: `job:${name}`,
          type: "smoothstep",
          style: { strokeDasharray: config.jobs[name].needs == null ? "5 4" : undefined },
          deletable: false,
        })),
      ),
    [config],
  );
  return (
    <div
      className="ci-flow relative h-[600px] min-h-[400px] min-w-0 flex-1"
      aria-label={t(readonly ? "Pipeline execution graph" : "Pipeline editor canvas")}
    >
      <ReactFlow
        nodes={nodes.map((node) => ({ ...node, selected: node.id === `job:${selected}` }))}
        edges={edges}
        nodeTypes={nodeTypes}
        onNodesChange={onNodesChange}
        onNodeClick={(_, node) => {
          if (node.type === "job") onSelect(String(node.data.name));
        }}
        nodesDraggable={!readonly}
        nodesConnectable={!readonly}
        edgesReconnectable={false}
        deleteKeyCode={null}
        onConnect={(connection) =>
          onConnect?.({ ...connection, source: connection.source.slice(4), target: connection.target.slice(4) })
        }
        isValidConnection={
          isValidConnection
            ? (connection) =>
                isValidConnection({
                  source: connection.source.slice(4),
                  target: connection.target.slice(4),
                  sourceHandle: "sourceHandle" in connection ? (connection.sourceHandle ?? null) : null,
                  targetHandle: "targetHandle" in connection ? (connection.targetHandle ?? null) : null,
                })
            : undefined
        }
        fitView
        minZoom={0.25}
        maxZoom={2}
      >
        <Background gap={20} size={1} />
        <Controls showInteractive={false} />
        <MiniMap pannable zoomable />
      </ReactFlow>
      <Button className="absolute bottom-3 left-40" size="sm" variant="outline" onClick={() => setNodes(initialNodes)}>
        {t("Auto layout")}
      </Button>
    </div>
  );
};
