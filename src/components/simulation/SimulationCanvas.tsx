import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type PointerEvent as ReactPointerEvent,
} from "react";
import type { SimulationSnapshot, Vec2 } from "../../lib/types";

type ViewOffset = { x: number; y: number };

export function SimulationCanvas({ snapshot }: { snapshot: SimulationSnapshot | null }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const dragRef = useRef<{ pointer: Vec2; offset: ViewOffset } | null>(null);
  const [size, setSize] = useState({ width: 1, height: 1 });
  const [offset, setOffset] = useState<ViewOffset>({ x: 0, y: 0 });
  const [isDragging, setIsDragging] = useState(false);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) {
      return;
    }
    const observer = new ResizeObserver(([entry]) => {
      setSize({ width: entry.contentRect.width, height: entry.contentRect.height });
    });
    observer.observe(canvas);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || !snapshot || size.width <= 1 || size.height <= 1) {
      return;
    }
    drawSimulation(canvas, snapshot, size, offset);
  }, [offset, size, snapshot]);

  const handlePointerDown = useCallback(
    (event: ReactPointerEvent<HTMLCanvasElement>) => {
      event.currentTarget.setPointerCapture(event.pointerId);
      dragRef.current = {
        pointer: { x: event.clientX, y: event.clientY },
        offset,
      };
      setIsDragging(true);
    },
    [offset],
  );

  const handlePointerMove = useCallback((event: ReactPointerEvent<HTMLCanvasElement>) => {
    const drag = dragRef.current;
    if (!drag) {
      return;
    }
    setOffset({
      x: drag.offset.x + event.clientX - drag.pointer.x,
      y: drag.offset.y + event.clientY - drag.pointer.y,
    });
  }, []);

  const handlePointerUp = useCallback(() => {
    dragRef.current = null;
    setIsDragging(false);
  }, []);

  return (
    <canvas
      ref={canvasRef}
      className={`h-full w-full touch-none ${isDragging ? "cursor-grabbing" : "cursor-grab"}`}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={handlePointerUp}
      onPointerCancel={handlePointerUp}
      onDoubleClick={() => setOffset({ x: 0, y: 0 })}
      aria-label="Pannable world simulation"
    />
  );
}

function drawSimulation(
  canvas: HTMLCanvasElement,
  snapshot: SimulationSnapshot,
  size: { width: number; height: number },
  offset: ViewOffset,
) {
  const pixelRatio = window.devicePixelRatio || 1;
  canvas.width = Math.round(size.width * pixelRatio);
  canvas.height = Math.round(size.height * pixelRatio);
  const context = canvas.getContext("2d");
  if (!context) {
    return;
  }
  context.setTransform(pixelRatio, 0, 0, pixelRatio, 0, 0);
  context.fillStyle = "#e9eeeb";
  context.fillRect(0, 0, size.width, size.height);

  const bounds = snapshot.world.bounds;
  const scale = Math.max(
    1,
    Math.min((size.width - 80) / bounds.width, (size.height - 80) / bounds.height),
  );
  const origin = {
    x: (size.width - bounds.width * scale) * 0.5 + offset.x,
    y: (size.height + bounds.height * scale) * 0.5 + offset.y,
  };
  const point = (world: Vec2): Vec2 => ({
    x: origin.x + world.x * scale,
    y: origin.y - world.y * scale,
  });

  context.save();
  context.beginPath();
  context.rect(
    origin.x,
    origin.y - bounds.height * scale,
    bounds.width * scale,
    bounds.height * scale,
  );
  context.clip();
  context.fillStyle = "#fbfcfb";
  context.fillRect(
    origin.x,
    origin.y - bounds.height * scale,
    bounds.width * scale,
    bounds.height * scale,
  );

  drawGrid(context, origin, bounds, scale);

  const robotOrigin = point(snapshot.robot.pose);
  context.strokeStyle = "rgba(5, 150, 105, 0.22)";
  context.fillStyle = "rgba(5, 150, 105, 0.7)";
  context.lineWidth = 1;
  snapshot.robot.lidar.forEach((distance, index) => {
    const angle =
      snapshot.robot.pose.orientation + (index * Math.PI * 2) / snapshot.robot.lidar.length;
    const endpoint = point({
      x: snapshot.robot.pose.x + Math.cos(angle) * distance,
      y: snapshot.robot.pose.y + Math.sin(angle) * distance,
    });
    context.beginPath();
    context.moveTo(robotOrigin.x, robotOrigin.y);
    context.lineTo(endpoint.x, endpoint.y);
    context.stroke();
    if (distance < snapshot.robot.lidarMaximumRange - 0.001) {
      context.beginPath();
      context.arc(endpoint.x, endpoint.y, 1.75, 0, Math.PI * 2);
      context.fill();
    }
  });

  context.strokeStyle = "#27322c";
  context.lineCap = "round";
  for (const wall of snapshot.world.walls) {
    const start = point(wall.start);
    const end = point(wall.end);
    context.lineWidth = Math.max(1.5, wall.thickness * scale);
    context.beginPath();
    context.moveTo(start.x, start.y);
    context.lineTo(end.x, end.y);
    context.stroke();
  }

  drawRobot(context, snapshot, robotOrigin, scale);
  context.restore();

  context.strokeStyle = "#536159";
  context.lineWidth = 2;
  context.strokeRect(
    origin.x,
    origin.y - bounds.height * scale,
    bounds.width * scale,
    bounds.height * scale,
  );
}

function drawGrid(
  context: CanvasRenderingContext2D,
  origin: Vec2,
  bounds: { width: number; height: number },
  scale: number,
) {
  context.strokeStyle = "rgba(39, 50, 44, 0.07)";
  context.lineWidth = 1;
  for (let x = 1; x < bounds.width; x += 1) {
    context.beginPath();
    context.moveTo(origin.x + x * scale, origin.y);
    context.lineTo(origin.x + x * scale, origin.y - bounds.height * scale);
    context.stroke();
  }
  for (let y = 1; y < bounds.height; y += 1) {
    context.beginPath();
    context.moveTo(origin.x, origin.y - y * scale);
    context.lineTo(origin.x + bounds.width * scale, origin.y - y * scale);
    context.stroke();
  }
}

function drawRobot(
  context: CanvasRenderingContext2D,
  snapshot: SimulationSnapshot,
  origin: Vec2,
  scale: number,
) {
  context.save();
  context.translate(origin.x, origin.y);
  context.rotate(-snapshot.robot.pose.orientation);
  const length = snapshot.robot.length * scale;
  const width = snapshot.robot.width * scale;
  context.fillStyle = snapshot.robot.collided ? "#dc2626" : "#047857";
  context.strokeStyle = "#052e24";
  context.lineWidth = 1.5;
  context.beginPath();
  context.roundRect(-length * 0.5, -width * 0.5, length, width, Math.min(4, width * 0.15));
  context.fill();
  context.stroke();
  context.strokeStyle = "#d1fae5";
  context.lineWidth = 2;
  context.beginPath();
  context.moveTo(0, 0);
  context.lineTo(length * 0.45, 0);
  context.stroke();
  context.restore();
}
