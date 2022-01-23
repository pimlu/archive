import { useCallback } from "preact/hooks";
import wasmInit, { start_loop, connect } from "./wasm";

wasmInit().then(start_loop).then(run => run()).catch(e => {
  if (!e?.message?.includes("Using exceptions for control flow")) {
    throw e;
  }
});
export default function App() {
  const onClickPlay = useCallback(() => {
    connect('http://localhost:3030').then(console.log).catch(console.error);
  }, [])
 return <div>
   <button type="button" class="play-btn" onClick={onClickPlay}>Play</button>
   </div>;
}
