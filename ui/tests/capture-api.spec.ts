import { execFileSync } from "node:child_process";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { expect, test } from "@playwright/test";
import * as captureCore from "../../api/_lib/capture-core.ts";

const { handleCapturePost } = captureCore;
type CaptureConfig = captureCore.CaptureConfig;
type FetchFn = captureCore.FetchFn;

const SECRET_KEY = "test-capture-key-xyzzy";
const SECRET_TOKEN = "ghp_test_token_do_not_leak";

function baseCapture(overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    v: 1,
    game_id: "42-deadbeef",
    seed: "42",
    deckA: { "10001110": 40 },
    deckB: { "10002110": 40 },
    first: "a",
    actions: [{ end_turn: { player: "a" } }],
    policy: "h0",
    strong: "h0:nodes=1",
    engine: "test",
    humanSide: "a",
    winner: null,
    final: false,
    created: "2026-10-11T12:00:00.000Z",
    updated: "2026-10-11T12:00:00.000Z",
    source: "browser",
    session_id: "abcdefghij",
    bot_meta: [],
    device: { cores: 4, standalone: false, ua: "test" },
    ...overrides,
  };
}

type Recorded = { url: string; method: string; body?: string };

function mockFetch(script: (url: string, init?: RequestInit) => Recorded | "pass"): FetchFn {
  const calls: Recorded[] = [];
  const fn = (async (url: string | URL | Request, init?: RequestInit) => {
    const u = typeof url === "string" ? url : url.toString();
    const rec = script(u, init);
    if (rec === "pass") {
      return new Response("{}", { status: 404 });
    }
    calls.push(rec);
    return new Response("{}", { status: 404 });
  }) as FetchFn;
  (fn as FetchFn & { calls: Recorded[] }).calls = calls;
  return fn;
}

function cfg(): CaptureConfig {
  return {
    captureKey: SECRET_KEY,
    ghToken: SECRET_TOKEN,
    repo: "melnce/arena",
    branch: "captures",
  };
}

function assertNoSecrets(text: string): void {
  expect(text).not.toContain(SECRET_KEY);
  expect(text).not.toContain(SECRET_TOKEN);
}

async function post(
  body: unknown,
  opts?: { method?: string; key?: string | null; fetchFn?: FetchFn },
): Promise<{ res: Response; text: string }> {
  const fetchFn = opts?.fetchFn ?? mockFetch(() => ({ url: "", method: "GET" }));
  const res = await handleCapturePost(
    {
      method: opts?.method ?? "POST",
      headers: {
        get: (n) => {
          if (n !== "x-capture-key") return null;
          if (opts && "key" in opts) return opts.key;
          return SECRET_KEY;
        },
      },
      text: async () => JSON.stringify(body),
    },
    cfg(),
    fetchFn,
  );
  const text = await res.text();
  assertNoSecrets(text);
  return { res, text };
}

test("capture API: method and auth", async () => {
  const { res: r405 } = await post(baseCapture(), { method: "GET" });
  expect(r405.status).toBe(405);

  const { res: r401a } = await post(baseCapture(), { key: null });
  expect(r401a.status).toBe(401);

  const { res: r401b } = await post(baseCapture(), { key: "wrong-key" });
  expect(r401b.status).toBe(401);

  const fetchFn = mockFetch(() => ({ url: "", method: "GET" }));
  const r503 = await handleCapturePost(
    {
      method: "POST",
      headers: { get: (n) => (n === "x-capture-key" ? SECRET_KEY : null) },
      text: async () => JSON.stringify(baseCapture()),
    },
    { ...cfg(), ghToken: "" },
    fetchFn,
  );
  const t503 = await r503.text();
  assertNoSecrets(t503);
  expect(r503.status).toBe(503);
});

test("capture API: validation errors", async () => {
  const cases: Array<{ body: unknown; reason: string }> = [
    { body: { ...baseCapture(), v: 2 }, reason: "v" },
    { body: { ...baseCapture(), game_id: "bad" }, reason: "game_id" },
    { body: { ...baseCapture(), session_id: "short" }, reason: "session_id" },
    { body: { ...baseCapture(), actions: "nope" }, reason: "actions" },
    { body: { ...baseCapture(), first: "z" }, reason: "first" },
    { body: { ...baseCapture(), final: "yes" }, reason: "final" },
    { body: { ...baseCapture(), winner: "draw" }, reason: "winner" },
    { body: { ...baseCapture(), deckA: [] }, reason: "deckA" },
    { body: { ...baseCapture(), created: "not-a-date" }, reason: "created" },
    { body: { ...baseCapture(), created: "2026-10-11T12:00:00" }, reason: "created lenient" },
  ];
  for (const c of cases) {
    const { res, text } = await post(c.body);
    expect(res.status, text).toBe(400);
  }
});

test("capture API: new file on existing branch", async () => {
  const puts: Recorded[] = [];
  const fetchFn = (async (url: string | URL | Request, init?: RequestInit) => {
    const u = typeof url === "string" ? url : url.toString();
    const method = init?.method ?? "GET";
    if (u.includes("/git/ref/heads/captures")) {
      return new Response(JSON.stringify({ object: { sha: "branchsha" } }), { status: 200 });
    }
    if (u.includes("/contents/captures/") && method === "GET") {
      return new Response("not found", { status: 404 });
    }
    if (u.includes("/contents/captures/") && method === "PUT") {
      const body = String(init?.body ?? "");
      puts.push({ url: u, method, body });
      return new Response(JSON.stringify({ commit: { sha: "commit1" } }), { status: 200 });
    }
    return new Response("unexpected", { status: 500 });
  }) as FetchFn;

  const cap = baseCapture();
  const { res, text } = await post(cap, { fetchFn });
  expect(res.status, text).toBe(200);
  expect(JSON.parse(text)).toMatchObject({ ok: true, path: "captures/2026-10/42-deadbeef-abcdefghij.json" });
  expect(puts).toHaveLength(1);
  const putBody = JSON.parse(puts[0]!.body!);
  expect(putBody.branch).toBe("captures");
  expect(putBody.sha).toBeUndefined();
  expect(putBody.content).toBeTruthy();
  assertNoSecrets(JSON.stringify(putBody));
});

test("capture API: create branch from main then put", async () => {
  const puts: Recorded[] = [];
  const fetchFn = (async (url: string | URL | Request, init?: RequestInit) => {
    const u = typeof url === "string" ? url : url.toString();
    const method = init?.method ?? "GET";
    if (u.endsWith("/git/ref/heads/captures")) return new Response("", { status: 404 });
    if (u.endsWith("/git/ref/heads/main")) {
      return new Response(JSON.stringify({ object: { sha: "mainhead" } }), { status: 200 });
    }
    if (u.endsWith("/git/refs") && method === "POST") {
      return new Response("{}", { status: 201 });
    }
    if (u.includes("/contents/") && method === "GET") return new Response("", { status: 404 });
    if (u.includes("/contents/") && method === "PUT") {
      puts.push({ url: u, method, body: String(init?.body) });
      return new Response(JSON.stringify({ commit: { sha: "c2" } }), { status: 200 });
    }
    return new Response(u, { status: 500 });
  }) as FetchFn;
  const { res } = await post(baseCapture(), { fetchFn });
  expect(res.status).toBe(200);
  expect(puts).toHaveLength(1);
});

test("capture API: keep vs replace rules", async () => {
  const storedFinal = baseCapture({ final: true, actions: [{ a: 1 }, { a: 2 }] });
  const storedPartial = baseCapture({ final: false, actions: [{ a: 1 }, { a: 2 }, { a: 3 }] });
  const path = "captures/2026-10/42-deadbeef-abcdefghij.json";
  const b64 = (obj: Record<string, unknown>) =>
    Buffer.from(`${JSON.stringify(obj, null, 1)}\n`).toString("base64");

  async function run(
    stored: Record<string, unknown>,
    incoming: Record<string, unknown>,
  ): Promise<{ status: number; puts: number; json: Record<string, unknown> }> {
    let puts = 0;
    const fetchFn = (async (url: string | URL | Request, init?: RequestInit) => {
      const u = typeof url === "string" ? url : url.toString();
      const method = init?.method ?? "GET";
      if (u.includes("/git/ref/heads/captures")) {
        return new Response(JSON.stringify({ object: { sha: "b" } }), { status: 200 });
      }
      if (u.includes("/contents/") && method === "GET") {
        return new Response(
          JSON.stringify({ sha: "filesha", content: b64(stored), encoding: "base64" }),
          { status: 200 },
        );
      }
      if (u.includes("/contents/") && method === "PUT") {
        puts += 1;
        return new Response(JSON.stringify({ commit: { sha: "new" } }), { status: 200 });
      }
      return new Response("err", { status: 500 });
    }) as FetchFn;
    const { res, text } = await post(incoming, { fetchFn });
    return { status: res.status, puts, json: JSON.parse(text) as Record<string, unknown> };
  }

  const keepPartial = await run(storedFinal, baseCapture({ final: false, actions: [{ a: 9 }] }));
  expect(keepPartial.status).toBe(200);
  expect(keepPartial.json.kept).toBe(true);
  expect(keepPartial.puts).toBe(0);

  const replaceWithFinal = await run(storedPartial, baseCapture({ final: true, actions: [{ a: 1 }] }));
  expect(replaceWithFinal.puts).toBe(1);

  const replaceLonger = await run(
    baseCapture({ final: false, actions: [{ a: 1 }] }),
    baseCapture({ final: false, actions: [{ a: 1 }, { a: 2 }] }),
  );
  expect(replaceLonger.puts).toBe(1);

  const keepShorter = await run(
    baseCapture({ final: false, actions: [{ a: 1 }, { a: 2 }] }),
    baseCapture({ final: false, actions: [{ a: 1 }] }),
  );
  expect(keepShorter.json.kept).toBe(true);
  expect(keepShorter.puts).toBe(0);

  const keepEqual = await run(
    baseCapture({ final: true, actions: [{ a: 1 }, { a: 2 }] }),
    baseCapture({ final: true, actions: [{ a: 1 }, { a: 2 }] }),
  );
  expect(keepEqual.json.kept).toBe(true);
  expect(keepEqual.puts).toBe(0);
});

test("capture API: 409 retry once", async () => {
  let putCount = 0;
  let getCount = 0;
  const stored = baseCapture({ final: false, actions: [{ a: 1 }] });
  const b64 = Buffer.from(`${JSON.stringify(stored, null, 1)}\n`).toString("base64");
  const fetchFn = (async (url: string | URL | Request, init?: RequestInit) => {
    const u = typeof url === "string" ? url : url.toString();
    const method = init?.method ?? "GET";
    if (u.includes("/git/ref/heads/captures")) {
      return new Response(JSON.stringify({ object: { sha: "b" } }), { status: 200 });
    }
    if (u.includes("/contents/") && method === "GET") {
      getCount += 1;
      return new Response(JSON.stringify({ sha: "filesha", content: b64, encoding: "base64" }), {
        status: 200,
      });
    }
    if (u.includes("/contents/") && method === "PUT") {
      putCount += 1;
      if (putCount === 1) return new Response("conflict", { status: 409 });
      return new Response(JSON.stringify({ commit: { sha: "ok" } }), { status: 200 });
    }
    return new Response("err", { status: 500 });
  }) as FetchFn;
  const { res } = await post(baseCapture({ final: false, actions: [{ a: 1 }, { a: 2 }] }), { fetchFn });
  expect(res.status).toBe(200);
  expect(putCount).toBe(2);
  expect(getCount).toBeGreaterThanOrEqual(2);
});

test("compiled capture.js POST handler loads (Node ESM)", async () => {
  const out = mkdtempSync(path.join(tmpdir(), "capture-compile-"));
  const root = path.resolve(import.meta.dirname, "../..");
  execFileSync(
    "npx",
    [
      "tsc",
      path.join(root, "api/capture.ts"),
      path.join(root, "api/_lib/capture-core.ts"),
      "--module",
      "nodenext",
      "--target",
      "es2022",
      "--types",
      "node",
      "--skipLibCheck",
      "--outDir",
      out,
    ],
    { cwd: path.join(root, "ui"), stdio: "pipe" },
  );
  const mod = await import(pathToFileURL(path.join(out, "capture.js")).href);
  const res = await mod.POST(
    new Request("http://localhost/api/capture", {
      method: "POST",
      headers: { "x-capture-key": "any" },
      body: "{}",
    }),
  );
  expect(res.status).toBe(503);
});
