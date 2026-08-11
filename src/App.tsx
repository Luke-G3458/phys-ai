import { Titlebar } from "./components/layout/Titlebar";
import BottomBar from "./components/layout/BottomBar";
import LeftSidePane from "./components/layout/LeftSidePane";
import RightSidePane from "./components/layout/RightSidePane";
import { SimulationCanvas } from "./components/simulation/SimulationCanvas";
import { SimulationControls } from "./components/simulation/SimulationControls";
import { useSimulation } from "./components/simulation/useSimulation";

function App() {
  const simulation = useSimulation();

  return (
    <div className="flex h-screen flex-col bg-[#e9eeeb]">
      <Titlebar />
      <div className="flex min-h-0 grow">
        <LeftSidePane>
          <SimulationControls controller={simulation} />
        </LeftSidePane>
        <main className="relative min-w-0 grow overflow-hidden text-black">
          <SimulationCanvas snapshot={simulation.snapshot} />
          {simulation.isLoading && (
            <div className="pointer-events-none absolute inset-0 grid place-items-center text-sm text-black/50">
              Starting simulator…
            </div>
          )}
          {simulation.error && (
            <div className="absolute bottom-3 left-1/2 max-w-lg -translate-x-1/2 rounded-md border border-red-300 bg-red-50 px-4 py-2 text-sm text-red-800 shadow-sm">
              {simulation.error}
            </div>
          )}
        </main>
        <RightSidePane />
      </div>
      <BottomBar />
    </div>
  );
}

export default App;
