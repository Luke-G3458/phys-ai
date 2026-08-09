import cx from "clsx";
import { PanelLeft, PanelRight } from "lucide-react";
import { useGlobalState, useGlobalStateDispatch } from "../../lib/state/store";

const BottomBar = () => {
  const {
    leftPanel: { isOpen: isLeftPanelOpen },
    rightPanel: { isOpen: isRightPanelOpen },
  } = useGlobalState();
  const dispatch = useGlobalStateDispatch();

  return (
    <div className="h-8 bg-black/5 border-t border-black/20 flex items-center justify-between px-1">
      <button
        onClick={() => {
          dispatch({ kind: "toggle-left-panel-clicked" });
        }}
        className="hover:bg-black/10 w-6 h-6 flex justify-center items-center rounded"
      >
        <PanelLeft className={cx("w-4", isLeftPanelOpen ? "text-black/70" : "text-black/50")} />
      </button>
      <button
        onClick={() => {
          dispatch({ kind: "toggle-right-panel-clicked" });
        }}
        className="hover:bg-black/10 w-6 h-6 flex justify-center items-center rounded"
      >
        <PanelRight className={cx("w-4", isRightPanelOpen ? "text-black/70" : "text-black/50")} />
      </button>
    </div>
  );
};

export default BottomBar;
