import { defineConfig } from "vite";

// Configuración recomendada por Tauri: puerto fijo y sin limpiar la consola.
export default defineConfig({
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  build: {
    target: "es2022",
    outDir: "dist",
  },
});
