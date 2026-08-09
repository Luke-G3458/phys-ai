import { useCallback } from "react";
import {
  MAX_RIGHT_PANEL_WIDTH,
  MIN_RIGHT_PANEL_WIDTH,
  useGlobalState,
  useGlobalStateDispatch,
} from "../../lib/state/store";
import ResizableSidePane from "../ui/ResizableSidePane";

export default function RightSidePane() {
  const {
    rightPanel: { isOpen, width },
  } = useGlobalState();
  const dispatch = useGlobalStateDispatch();
  const handleResize = useCallback(
    (nextWidth: number) => dispatch({ kind: "right-panel-resized", width: nextWidth }),
    [dispatch],
  );

  return (
    <ResizableSidePane
      side="right"
      isOpen={isOpen}
      width={width}
      minWidth={MIN_RIGHT_PANEL_WIDTH}
      maxWidth={MAX_RIGHT_PANEL_WIDTH}
      onResize={handleResize}
    >
      <div className="h-full" />
    </ResizableSidePane>
  );
}
