import type { Window } from "@tauri-apps/api/window";

export default function WindowsControl({
  appWindow,
  isMaximized,
}: {
  appWindow: Window;
  isMaximized: boolean;
}) {
  return (
    <div className="flex flex-row items-center justify-center">
      <button
        onClick={() => {
          appWindow.minimize();
        }}
        className="active:bg-black/50 hover:bg-black/5 scale-90 h-full w-9.5 p-2 flex items-center justify-center"
      >
        <div className="w-3 h-0.5 bg-black" />
      </button>
      <button
        onClick={() => {
          appWindow.toggleMaximize();
        }}
        className="active:bg-black/50 hover:bg-black/5 group h-full w-9.5 p-2 flex items-center justify-center"
      >
        {isMaximized ? (
          <div className="w-4 h-4 relative flex items-center justify-center">
            <div className="absolute top-0 right-0 w-3 h-3 scale-85 border-black border-2 rounded-xs" />
            <div className="absolute w-3 h-3 z-10 bg-white scale-85 border-black border-2 rounded-xs" />
          </div>
        ) : (
          <div className="w-3 h-3 border-black border-2 scale-90 rounded-xs" />
        )}
      </button>
      <button
        onClick={() => {
          appWindow.close();
        }}
        className="hover:bg-red-700/60 h-full w-9.5 p-2 flex items-center justify-center"
      >
        <div className="relative flex h-3.5 w-3.5 justify-center items-center">
          <div className="absolute w-full h-0.5 bg-black scale-90 rotate-45 rounded-xs" />
          <div className="absolute w-full h-0.5 bg-black scale-90 -rotate-45 rounded-xs" />
        </div>
      </button>
    </div>
  );
}
