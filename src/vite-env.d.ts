/// <reference types="vite/client" />

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;

  export const __APP_VERSION__: string
  export const __BUILD_NUMBER__: string
}
