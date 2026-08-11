import { useCallback, type ReactNode } from "react";
import {
  MAX_LEFT_PANEL_WIDTH,
  MIN_LEFT_PANEL_WIDTH,
  useGlobalState,
  useGlobalStateDispatch,
} from "../../lib/state/store";
import ResizableSidePane from "../ui/ResizableSidePane";

export default function LeftSidePane({ children }: { children?: ReactNode }) {
  const {
    leftPanel: { isOpen, width },
  } = useGlobalState();
  const dispatch = useGlobalStateDispatch();
  const handleResize = useCallback(
    (nextWidth: number) => dispatch({ kind: "left-panel-resized", width: nextWidth }),
    [dispatch],
  );

  return (
    <ResizableSidePane
      side="left"
      isOpen={isOpen}
      width={width}
      minWidth={MIN_LEFT_PANEL_WIDTH}
      maxWidth={MAX_LEFT_PANEL_WIDTH}
      onResize={handleResize}
    >
      {children}
    </ResizableSidePane>
  );
}
