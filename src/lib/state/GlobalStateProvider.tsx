import { useReducer, type ReactNode } from "react";
import {
  GlobalStateContext,
  GlobalStateDispatchContext,
  INITIAL_STATE,
  globalStateReducer,
} from "./store";

export function GlobalStateProvider({ children }: { children: ReactNode }) {
  const [globalState, dispatch] = useReducer(globalStateReducer, INITIAL_STATE);

  return (
    <GlobalStateContext value={globalState}>
      <GlobalStateDispatchContext value={dispatch}>{children}</GlobalStateDispatchContext>
    </GlobalStateContext>
  );
}
