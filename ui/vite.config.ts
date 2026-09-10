import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig, type Plugin } from "vite";

const here = path.dirname(fileURLToPath(import.meta.url));

function catalogPlugin(): Plugin {
  const run = () => {
    execFileSync("node", [path.resolve(here, "../tools/gen-ui-catalog.mjs")], {
      stdio: "inherit",
    });
  };
  return {
    name: "gen-ui-catalog",
    buildStart() {
      run();
    },
  };
}

const base = process.env.VITE_BASE || "/";

export default defineConfig({
  base,
  plugins: [catalogPlugin()],
  assetsInclude: ["**/*.wasm"],
  server: {
    port: 5173,
    host: "127.0.0.1",
  },
  preview: {
    port: 4173,
    host: "127.0.0.1",
  },
});
