import { createContext, useContext, type Dispatch } from "react";

export const MIN_LEFT_PANEL_WIDTH = 160;
export const MAX_LEFT_PANEL_WIDTH = 420;
export const MIN_RIGHT_PANEL_WIDTH = 160;
export const MAX_RIGHT_PANEL_WIDTH = 420;

export const INITIAL_STATE: GlobalState = {
  leftPanel: {
    width: 260,
    isOpen: true,
  },
  rightPanel: {
    width: 200,
    isOpen: false,
  },
  bottomPanel: {
    height: 200,
    isOpen: false,
  },
};

export type GlobalState = {
  leftPanel: {
    width: number;
    isOpen: boolean;
  };
  rightPanel: {
    width: number;
    isOpen: boolean;
  };
  bottomPanel: {
    height: number;
    isOpen: boolean;
  };
};

export const GlobalStateContext = createContext<GlobalState>(INITIAL_STATE);
export const GlobalStateDispatchContext = createContext<Dispatch<Action>>(() => {});

export function useGlobalState() {
  return useContext(GlobalStateContext);
}

export function useGlobalStateDispatch() {
  return useContext(GlobalStateDispatchContext);
}

export type Action =
  | {
      kind: "open-right-panel-clicked";
    }
  | {
      kind: "toggle-right-panel-clicked";
    }
  | {
      kind: "right-panel-resized";
      width: number;
    }
  | {
      kind: "toggle-left-panel-clicked";
    }
  | {
      kind: "left-panel-resized";
      width: number;
    }
  | { kind: "open-bottom-panel-clicked" };

export function globalStateReducer(state: GlobalState, action: Action): GlobalState {
  switch (action.kind) {
    case "open-right-panel-clicked":
      return { ...state, rightPanel: { ...state.rightPanel, isOpen: true } };
    case "toggle-right-panel-clicked":
      return { ...state, rightPanel: { ...state.rightPanel, isOpen: !state.rightPanel.isOpen } };
    case "right-panel-resized":
      return {
        ...state,
        rightPanel: {
          ...state.rightPanel,
          width: Math.min(MAX_RIGHT_PANEL_WIDTH, Math.max(MIN_RIGHT_PANEL_WIDTH, action.width)),
        },
      };
    case "toggle-left-panel-clicked":
      return { ...state, leftPanel: { ...state.leftPanel, isOpen: !state.leftPanel.isOpen } };
    case "left-panel-resized":
      return {
        ...state,
        leftPanel: {
          ...state.leftPanel,
          width: Math.min(MAX_LEFT_PANEL_WIDTH, Math.max(MIN_LEFT_PANEL_WIDTH, action.width)),
        },
      };
    case "open-bottom-panel-clicked":
      return { ...state, bottomPanel: { ...state.bottomPanel, isOpen: true } };
  }
}
