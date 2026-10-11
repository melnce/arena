import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig, type Plugin } from "vite";

const here = path.dirname(fileURLToPath(import.meta.url));
const servePyPath = path.resolve(here, "../py/serve.py");

export function extractServePolicies(source: string): { strong: string; cheat: string } {
  const strongM = source.match(/^DEFAULT_STRONG = "(.*)"$/m);
  const cheatM = source.match(/^DEFAULT_CHEAT = "(.*)"$/m);
  if (!strongM) throw new Error("missing DEFAULT_STRONG in py/serve.py");
  if (!cheatM) throw new Error("missing DEFAULT_CHEAT in py/serve.py");
  const strong = strongM[1];
  const cheat = cheatM[1];
  if (!strong.startsWith("h0:")) {
    throw new Error(`DEFAULT_STRONG must be an h0 spec, got: ${strong}`);
  }
  if (!cheat.startsWith("h0:")) {
    throw new Error(`DEFAULT_CHEAT must be an h0 spec, got: ${cheat}`);
  }
  if (!cheat.split(",").some((part) => part.trim() === "info=all")) {
    throw new Error("DEFAULT_CHEAT must include info=all");
  }
  return { strong, cheat };
}

function readServePolicies(): { strong: string; cheat: string } {
  const source = fs.readFileSync(servePyPath, "utf8");
  return extractServePolicies(source);
}

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

function servePolicyPlugin(): Plugin {
  return {
    name: "serve-policy-define",
    config() {
      const { strong, cheat } = readServePolicies();
      return {
        define: {
          __SERVED_STRONG__: JSON.stringify(strong),
          __SERVED_CHEAT__: JSON.stringify(cheat),
        },
      };
    },
  };
}

const base = process.env.VITE_BASE || "/";

export default defineConfig({
  base,
  plugins: [catalogPlugin(), servePolicyPlugin()],
  assetsInclude: ["**/*.wasm"],
  worker: {
    format: "es",
  },
  server: {
    port: 5173,
    host: "127.0.0.1",
  },
  preview: {
    port: 4173,
    host: "127.0.0.1",
  },
});
