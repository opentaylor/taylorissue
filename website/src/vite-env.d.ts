/// <reference types="vite/client" />

declare module "@images/*.png" {
  const src: string;
  export default src;
}

declare module "@images/*.jpg" {
  const src: string;
  export default src;
}

declare module "@images/*.svg" {
  const src: string;
  export default src;
}
