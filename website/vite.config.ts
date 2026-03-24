import path from "path";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig(({ mode }) => ({
  base: "/",
  plugins: [react(), tailwindcss()],
  css: {
    transformer: "lightningcss",
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
      "@images": path.resolve(__dirname, "../docs/images"),
    },
  },
  server: {
    port: 5173,
    fs: {
      allow: [".."],
    },
  },
}));
