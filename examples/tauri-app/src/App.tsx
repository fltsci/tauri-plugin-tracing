import { useState } from "react";
import "./App.css";
import { useLogger } from "./logger";
import { useEffect } from "react";
import { info } from "tauri-plugin-tracing";

function App() {
  useLogger();

  const [count, setCount] = useState(0);

  useEffect(() => {
    console.log("Forwarded from console.log", count);
    info("Sent directly via plugin.info", count);
  }, [count]);

  return (
    <>
      <div>
        <h3>Count: {count}</h3>
        <button type="button" onClick={() => setCount(count + 1)}>
          Increment
        </button>
      </div>
    </>
  );
}

export default App;
