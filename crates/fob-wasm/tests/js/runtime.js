import { __fobRuntime } from '../../js/runtime.js';

export function bootstrap(fsRoot) {
  if (fsRoot) {
    __fobRuntime.fs.setRoot(fsRoot);
  }
  globalThis.__fobRuntime = __fobRuntime;
}
