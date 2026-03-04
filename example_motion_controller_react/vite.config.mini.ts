import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [react()],
  build: {
    outDir: "dist-mini",
    rollupOptions: {
      input: {
        index: "index-mini.html",
      },
    },
  },
});
