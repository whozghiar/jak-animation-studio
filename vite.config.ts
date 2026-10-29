import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import Icons from "unplugin-icons/vite";
import { fileURLToPath, URL } from "url";

export default defineConfig({
  server: {
    // the launcher's dev server takes 3000, Jak Level Editor's 3001
    port: 3002,
    strictPort: true,
  },
  plugins: [svelte(), Icons({ compiler: "svelte" }), tailwindcss()],
  resolve: {
    alias: {
      $lib: fileURLToPath(new URL("./src/lib", import.meta.url)),
    },
  },
});
