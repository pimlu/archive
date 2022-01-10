import React from "react";

import wasmInit, {start_loop} from "./wasm";

wasmInit().then(() => start_loop());

export default function App() {
  return <div></div>
}