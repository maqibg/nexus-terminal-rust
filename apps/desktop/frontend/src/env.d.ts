/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue';
  const component: DefineComponent<{}, {}, any>;
  export default component;
}

declare module 'splitpanes' {
  import type { DefineComponent } from 'vue';
  export const Splitpanes: DefineComponent<any, any, any>;
  export const Pane: DefineComponent<any, any, any>;
}


declare module '@novnc/novnc/lib/rfb' {
  const RFB: any;
  export default RFB;
}


declare module '@novnc/novnc/lib/rfb.js' {
  const RFB: any;
  export default RFB;
}
