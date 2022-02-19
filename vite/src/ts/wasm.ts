import wasmInit from '../../../archive-wasm/pkg/archive_wasm';
import wasm from '../../../archive-wasm/pkg/archive_wasm_bg.wasm?url';
type Cb = (u: unknown) => void;
let res: Cb, rej: Cb;
// dumb variable because we can't synchronously poll promises
export let isInitStarted = false;
export const initDone = new Promise((res_, rej_) => {
  res = res_;
  rej = rej_;
});
async function init_() {
  let path = wasm;
  // in web worker, URLs are messed up
  if (self.document === undefined) {
    path = path.replace('./assets', '.');
  }
  await wasmInit(new URL(path, `${self.location}`));
}
export default function init() {
  if (!isInitStarted) {
    isInitStarted = true;
    init_().then(res).catch(rej);
  }
  return initDone;
}

export * from '../../../archive-wasm/pkg/archive_wasm';
