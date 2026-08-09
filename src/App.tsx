import { Titlebar } from "./components/layout/Titlebar";
import BottomBar from "./components/layout/BottomBar";
import LeftSidePane from "./components/layout/LeftSidePane";
import RightSidePane from "./components/layout/RightSidePane";

function App() {
  return (
    <div className="flex h-screen flex-col">
      <Titlebar />
      <div className="flex min-h-0 grow">
        <LeftSidePane />
        <main className="relative flex grow items-center justify-center bg-white p-2 text-black">
          main content
        </main>
        <RightSidePane />
      </div>
      <BottomBar />
    </div>
  );
}

export default App;
