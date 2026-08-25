import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";
import Ajv2020 from "ajv/dist/2020.js";
import addFormats from "ajv-formats";

const root = resolve(import.meta.dirname, "..");
const stateDir = mkdtempSync(resolve(tmpdir(), "public-purpose-lab-m1-"));
const fixture = resolve(
  root,
  "contracts/common/examples/c-001-m1-conformance-command.json",
);
const ajv = new Ajv2020({ allErrors: true, strict: true });
addFormats(ajv);

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function readJson(relativePath) {
  return JSON.parse(readFileSync(resolve(root, relativePath), "utf8"));
}

for (const path of [
  "contracts/common/c-002-authority-and-purpose-context.schema.json",
  "contracts/common/c-001-interaction-envelope.schema.json",
  "contracts/common/c-004-evidence-reference.schema.json",
  "contracts/common/c-003-command-outcome-and-failure.schema.json",
]) {
  ajv.addSchema(readJson(path));
}

function runHost(args) {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--quiet",
      "--locked",
      "--bin",
      "ppl-framework-host",
      "--",
      ...args,
    ],
    { cwd: root, encoding: "utf8" },
  );
  assert(
    result.status === 0,
    `framework host failed with code ${result.status}: ${result.stderr.trim()}`,
  );
  return JSON.parse(result.stdout);
}

try {
  const commonArgs = [
    "--state-dir",
    stateDir,
    "--environment-id",
    "env-local-001",
    "--now",
    "2030-08-25T12:01:00Z",
    fixture,
  ];
  const first = runHost(["process", ...commonArgs]);
  const duplicate = runHost(["process", ...commonArgs]);
  const health = runHost([
    "healthcheck",
    "--state-dir",
    stateDir,
    "--environment-id",
    "env-local-001",
  ]);

  const validateOutcome = ajv.getSchema(
    "urn:public-purpose-lab:contract:C-003:1.0.0",
  );
  assert(validateOutcome(first), "accepted runtime outcome violates C-003");
  assert(
    validateOutcome(duplicate),
    "duplicate runtime outcome violates C-003",
  );
  assert(first.status === "accepted", "first delivery was not accepted");
  assert(
    duplicate.status === "duplicate",
    "second delivery was not reconciled",
  );
  assert(
    duplicate.originalOutcomeId === first.outcomeId,
    "duplicate did not identify the original outcome",
  );
  assert(health.interactionState === "ready", "interaction state is not ready");
  assert(health.journalRecords === 2, "unexpected journal record count");

  const journal = readFileSync(
    resolve(stateDir, "interaction-journal.jsonl"),
    "utf8",
  );
  for (const prohibited of [
    "m1-common-interaction",
    "idempotency-m1-conformance-001",
    "auth-context-assurance-001",
    "workload-framework-host",
    "synthetic-actor-reviewer",
  ]) {
    assert(!journal.includes(prohibited), `journal disclosed ${prohibited}`);
  }

  console.log(
    "M1 runtime OK: accepted once, duplicate reconciled after restart, outcomes valid, journal redacted.",
  );
} finally {
  rmSync(stateDir, { recursive: true, force: true });
}
