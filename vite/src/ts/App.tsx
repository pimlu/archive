import { useCallback, useEffect, useState } from "preact/hooks";
import wasmInit, { WasmClient, startClient, connect, useConnection } from "./wasm";

let {requestIdleCallback} = window;
requestIdleCallback ??= (cb: IdleRequestCallback) => (cb as any)();

requestIdleCallback(() => wasmInit());

export default function App() {
  const [client, setClient] = useState<WasmClient | null>(null);
  useEffect(() => {
    (async () => {
      await wasmInit();
      let pkg = await startClient();
      setClient(pkg.client);
      try {
        pkg.run();
      } catch(e: any) {
        let shouldSuppress = e?.message?.includes("Using exceptions for control flow");
        if (!shouldSuppress) {
          throw e;
        }
      }
    })();
  }, []);
  const onClickPlay = useCallback(() => {
    if (!client) return;
    (async () => {
      try {
        let connection = await connect('http://localhost:3030');
        await useConnection(client, connection);
      } catch(e) {
        console.error(e);
      }
    })();
  }, [client]);
 return <div>
   <button type="button" class="play-btn" disabled={!client} onClick={onClickPlay}>Play</button>
   </div>;
}
