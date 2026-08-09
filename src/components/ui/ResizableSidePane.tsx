import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type PointerEvent as ReactPointerEvent,
  type ReactNode,
} from "react";

type Props = {
  side: "left" | "right";
  isOpen: boolean;
  width: number;
  minWidth: number;
  maxWidth: number;
  onResize: (width: number) => void;
  children?: ReactNode;
};

export default function ResizableSidePane({
  side,
  isOpen,
  width,
  minWidth,
  maxWidth,
  onResize,
  children,
}: Props) {
  const [isResizing, setIsResizing] = useState(false);
  const resizeStartRef = useRef({ pointerX: 0, width: 0 });

  const handleResizeStart = useCallback(
    (event: ReactPointerEvent<HTMLDivElement>) => {
      if (!isOpen) {
        return;
      }

      event.preventDefault();
      event.currentTarget.setPointerCapture(event.pointerId);
      resizeStartRef.current = {
        pointerX: event.clientX,
        width,
      };
      setIsResizing(true);
    },
    [isOpen, width],
  );

  useEffect(() => {
    if (!isResizing) {
      return;
    }

    const handleResizeMove = (event: PointerEvent) => {
      const pointerDelta = event.clientX - resizeStartRef.current.pointerX;
      const nextWidth =
        side === "left"
          ? resizeStartRef.current.width + pointerDelta
          : resizeStartRef.current.width - pointerDelta;

      onResize(nextWidth);
    };

    const handleResizeEnd = () => {
      setIsResizing(false);
    };

    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
    window.addEventListener("pointermove", handleResizeMove);
    window.addEventListener("pointerup", handleResizeEnd);

    return () => {
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
      window.removeEventListener("pointermove", handleResizeMove);
      window.removeEventListener("pointerup", handleResizeEnd);
    };
  }, [isResizing, onResize, side]);

  const borderClass =
    side === "left"
      ? isOpen
        ? "border-r border-black/20"
        : "border-r border-transparent"
      : isOpen
        ? "border-l border-black/20"
        : "border-l border-transparent";

  const resizeHandleClass = side === "left" ? "right-0" : "left-0";

  return (
    <aside
      className={`relative h-full shrink-0 overflow-hidden bg-black/5 text-black ${
        isResizing ? "transition-none" : "transition-[width] duration-150 ease-out"
      } ${borderClass}`}
      style={{ width: isOpen ? width : 0 }}
      aria-hidden={!isOpen}
    >
      <div className="h-full" style={{ width: isOpen ? width : minWidth }}>
        {children}
      </div>
      {isOpen && (
        <div
          role="separator"
          aria-orientation="vertical"
          aria-valuemin={minWidth}
          aria-valuemax={maxWidth}
          aria-valuenow={width}
          onPointerDown={handleResizeStart}
          className={`absolute top-0 h-full w-2 cursor-col-resize touch-none ${resizeHandleClass}`}
        />
      )}
    </aside>
  );
}
