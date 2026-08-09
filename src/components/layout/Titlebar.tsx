import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import WindowsControl from "../ui/WindowsControl.tsx";

/** True when running on macOS — checked once via the user-agent. */
const IS_MAC = navigator.userAgent.toLowerCase().includes("mac");

/**
 * Custom titlebar that replaces the native title bar on all platforms.
 *
 * - macOS: leaves a 72px gap on the left for the native traffic-light buttons
 *   (rendered via Tauri's titleBarStyle "Overlay").
 * - Windows/Linux: renders its own minimize / maximize / close controls on the
 *   right-hand side, since decorations are disabled entirely.
 */
export function Titlebar() {
  const appWindow = getCurrentWindow();
  const [isMaximized, setIsMaximized] = useState(false);

  useEffect(() => {
    appWindow.isMaximized().then(setIsMaximized);

    // Keep maximize icon in sync when the user resizes/restores
    const unlisten = appWindow.onResized(() => {
      appWindow.isMaximized().then(setIsMaximized);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [appWindow]);

  return (
    <header
      data-tauri-drag-region
      className="flex h-8 shrink-0 select-none items-center bg-black/5 border-b border-black/20"
    >
      {/* Spacer for macOS traffic lights */}
      {IS_MAC && <div className="w-[80px] shrink-0" data-tauri-drag-region />}

      {/* Right section — Windows/Linux window controls */}
      {!IS_MAC && (
        <div className="flex h-full items-stretch">
          <WindowsControl appWindow={appWindow} isMaximized={isMaximized} />
        </div>
      )}
    </header>
  );
}
