#!/usr/bin/env node
/**
 * Deliberate corruptions that must make validate.mjs exit 1.
 * Restores nothing — works on copies in a temp directory.
 */
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const SRC = path.join(ROOT, "cards", "10000", "10001110.json");

function runValidate(cardsDir) {
  const tmpRoot = fs.mkdtempSync(path.join(os.tmpdir(), "arena-proof-"));
  fs.cpSync(path.join(ROOT, "schema"), path.join(tmpRoot, "schema"), { recursive: true });
  fs.cpSync(path.join(ROOT, "tools"), path.join(tmpRoot, "tools"), { recursive: true });
  fs.cpSync(cardsDir, path.join(tmpRoot, "cards"), { recursive: true });
  fs.cpSync(
    path.join(ROOT, "cards", "official"),
    path.join(tmpRoot, "cards", "official"),
    { recursive: true },
  );
  const r = spawnSync("node", ["tools/validate.mjs"], {
    cwd: tmpRoot,
    encoding: "utf8",
    maxBuffer: 20 * 1024 * 1024,
  });
  fs.rmSync(tmpRoot, { recursive: true, force: true });
  return r;
}

function withCopy(mutate, expectNeedle) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "arena-cards-"));
  const dest = path.join(dir, "10001110.json");
  const data = JSON.parse(fs.readFileSync(SRC, "utf8"));
  mutate(data);
  fs.writeFileSync(dest, JSON.stringify(data, null, 2) + "\n");
  const r = runValidate(dir);
  fs.rmSync(dir, { recursive: true, force: true });
  const out = `${r.stdout}\n${r.stderr}`;
  if (r.status === 0) {
    throw new Error(`expected validate to fail (${expectNeedle})\n${out}`);
  }
  if (!out.includes(expectNeedle)) {
    throw new Error(`expected ${JSON.stringify(expectNeedle)} in output:\n${out}`);
  }
  console.log(`ok: ${expectNeedle}`);
}

const proofs = [
  [
    "unknown field",
    (d) => {
      d.nope = 1;
    },
    "must NOT have additional properties",
  ],
  [
    "printed not a substring",
    (d) => {
      d.modes[0].printed = "Enhance (4): NOT A SUBSTRING";
    },
    "printed is not a substring of text",
  ],
  [
    "dangling crest",
    (d) => {
      d.modes[0].effects[0].op = "crest";
      d.modes[0].effects[0].gain = "crest:00000000";
      d.modes[0].effects[0].player = "self";
      delete d.modes[0].effects[0].select;
      delete d.modes[0].effects[0].attack;
      delete d.modes[0].effects[0].defense;
    },
    "dangling crest id crest:00000000",
  ],
  [
    "shared sentence",
    (d) => {
      d.modes[0].effects.push({
        printed: "Give this follower +3/+3.",
        op: "buff",
        select: { pick: "self" },
        attack: 1,
        defense: 1,
      });
    },
    "shares a sentence with another clause root",
  ],
  [
    "A pool pick missing zone",
    (d) => {
      d.modes[0].effects[0].select = {
        pick: "random",
        side: "ally",
        kind: "follower",
      };
    },
    "must have required property 'zone'",
  ],
  [
    "C duplicated mode subtree",
    (d) => {
      d.abilities = [
        {
          on: "fanfare",
          printed: "Give this follower +3/+3.",
          effects: [
            {
              printed: "Give this follower +3/+3.",
              op: "buff",
              select: { pick: "self" },
              attack: 3,
              defense: 3,
            },
          ],
        },
      ];
      d.text = "Give this follower +3/+3.\nEnhance (4): Give this follower +3/+3.";
      d.modes[0].printed = "Enhance (4): Give this follower +3/+3.";
    },
    "mode copies an ability effect subtree",
  ],
  [
    "K faith: gain",
    (d) => {
      d.modes[0].effects[0] = {
        printed: "Give this follower +3/+3.",
        op: "crest",
        gain: "faith:10634120",
        player: "self",
      };
    },
    "must not reference a faith: id",
  ],
];

for (const [name, mutate, needle] of proofs) {
  process.stdout.write(`${name}… `);
  withCopy(mutate, needle);
}
console.log(`ok: ${proofs.length} corruption proofs`);
