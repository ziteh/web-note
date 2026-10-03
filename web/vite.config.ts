import { readFileSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { brotliCompressSync, constants } from "node:zlib";
import { defineConfig, type Plugin } from "vite";
import solid from "vite-plugin-solid";
import solidSvg from "vite-plugin-solid-svg";

// Replaces each script and stylesheet with a brotli copy named `<file>.br`;
// the server answers with `Content-Encoding: br`.
// Runs after the files are written, because Vite finalizes chunk contents during bundle generation.
function brotli(): Plugin {
  return {
    name: "brotli-assets",
    apply: "build",
    writeBundle(options, bundle) {
      for (const name of Object.keys(bundle)) {
        if (!/\.(js|css)$/.test(name)) continue;
        const file = path.join(options.dir ?? "", name);
        const compressed = brotliCompressSync(readFileSync(file), {
          params: {
            [constants.BROTLI_PARAM_QUALITY]: constants.BROTLI_MAX_QUALITY,
          },
        });
        writeFileSync(`${file}.br`, compressed);
        rmSync(file);
      }
    },
  };
}

export default defineConfig({
  plugins: [solid(), solidSvg(), brotli()],
  resolve: {
    alias: {
      "~": path.resolve(import.meta.dirname, "./src"),
    },
  },
  server: {
    proxy: {
      "/api": process.env.API_URL ?? "http://127.0.0.1:5393",
    },
  },
});
