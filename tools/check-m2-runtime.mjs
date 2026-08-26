import {
  chmodSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { spawn, spawnSync } from "node:child_process";
import Ajv2020 from "ajv/dist/2020.js";
import addFormats from "ajv-formats";

const root = resolve(import.meta.dirname, "..");
const temporary = mkdtempSync(resolve(tmpdir(), "public-purpose-lab-m2-"));
const firstState = resolve(temporary, "environment-one");
const secondState = resolve(temporary, "environment-two");
const rebuiltState = resolve(temporary, "environment-rebuilt");
const requestPath = resolve(
  root,
  "contracts/identity/examples/iam-01-grant-request.json",
);
const grantPath = resolve(temporary, "grant-one.json");
const expiredGrantPath = resolve(temporary, "grant-expired.json");
const workflowRequestPath = resolve(temporary, "workflow-request.json");
const workflowGrantPath = resolve(temporary, "workflow-grant.json");
const tamperedGrantPath = resolve(temporary, "grant-tampered.json");
const binary = resolve(
  root,
  "target/debug",
  process.platform === "win32"
    ? "ppl-framework-host.exe"
    : "ppl-framework-host",
);
const ajv = new Ajv2020({ allErrors: true, strict: true });
addFormats(ajv);

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function buildHost() {
  const result = spawnSync(
    "cargo",
    ["build", "--quiet", "--locked", "--bin", "ppl-framework-host"],
    { cwd: root, encoding: "utf8" },
  );
  assert(
    !result.error,
    `M2 host build could not start cargo: ${result.error?.message ?? "unknown error"}`,
  );
  assert(
    result.status === 0,
    `M2 host build failed: ${(result.stderr ?? "").trim()}`,
  );
}

function runHost(args, options = {}) {
  const result = spawnSync(binary, args, {
    cwd: root,
    encoding: "utf8",
    env: { ...process.env, ...(options.env ?? {}) },
  });
  const expected = options.expected ?? [0];
  assert(
    expected.includes(result.status),
    `host ${args[0]} exited ${result.status}: ${result.stderr.trim()}`,
  );
  return {
    stdout: result.stdout,
    stderr: result.stderr,
    json: result.stdout.trim() ? JSON.parse(result.stdout) : undefined,
  };
}

function runHostAsync(args) {
  return new Promise((resolvePromise, reject) => {
    const child = spawn(binary, args, { cwd: root, encoding: "utf8" });
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (chunk) => {
      stdout += chunk;
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk;
    });
    child.on("error", reject);
    child.on("close", (status) => {
      if (status !== 0) {
        reject(new Error(`concurrent establish exited ${status}: ${stderr}`));
        return;
      }
      resolvePromise(JSON.parse(stdout));
    });
  });
}

function stateArgs(state, now) {
  return ["--state-dir", state, "--now", now];
}

function addSchema(relativePath) {
  const schema = readJson(resolve(root, relativePath));
  ajv.addSchema(schema);
}

try {
  buildHost();
  addSchema("contracts/identity/i-004-demonstration-sign-in-grant.schema.json");
  addSchema("contracts/identity/i-005-synthetic-session-outcome.schema.json");
  const validateGrant = ajv.getSchema(
    "urn:public-purpose-lab:contract:I-004:1.0.0",
  );
  const validateOutcome = ajv.getSchema(
    "urn:public-purpose-lab:contract:I-005:1.0.0",
  );

  runHost([
    "iam-configure-demo",
    ...stateArgs(firstState, "2030-08-26T10:00:00Z"),
  ]);
  runHost([
    "iam-configure-demo",
    ...stateArgs(secondState, "2030-08-26T10:00:00Z"),
  ]);
  const firstHealth = runHost(["iam-health", "--state-dir", firstState]).json;
  const secondHealth = runHost(["iam-health", "--state-dir", secondState]).json;
  assert(firstHealth.identityState === "ready", "first identity is not ready");
  assert(
    firstHealth.activeTrustProfile === "local-synthetic",
    "local trust profile is not visible",
  );
  assert(
    firstHealth.prominentWarning.includes("LOCAL-SYNTHETIC"),
    "local trust warning is not prominent",
  );
  assert(
    firstHealth.environmentId !== secondHealth.environmentId &&
      firstHealth.trustDomain !== secondHealth.trustDomain &&
      firstHealth.signerFingerprint !== secondHealth.signerFingerprint,
    "independent environments reused identity or trust",
  );

  const hostedHealth = runHost(["iam-health", "--state-dir", firstState], {
    expected: [3],
    env: {
      PPL_ENVIRONMENT_CLASS: "hosted-shared",
      PPL_TRUST_PROFILE: "managed",
    },
  }).json;
  assert(
    hostedHealth.identityState === "not-ready" &&
      hostedHealth.reasonCode === "trust-profile-incompatible",
    "local trust became ready for a hosted profile",
  );

  const workload = runHost([
    "iam-workload",
    ...stateArgs(firstState, "2030-08-26T10:00:05Z"),
    "--workload-id",
    "workload-director",
  ]).json;
  assert(workload.contractId === "I-002", "workload context is not I-002");
  assert(
    workload.contractActions.includes("I-004:request-grant"),
    "workload lacks bounded grant authority",
  );

  const issue = runHost([
    "iam-issue-grant",
    ...stateArgs(firstState, "2030-08-26T10:00:10Z"),
    "--output",
    grantPath,
    requestPath,
  ]);
  assert(!issue.stdout.includes("signature"), "raw grant leaked to stdout");
  const grant = readJson(grantPath);
  assert(validateGrant(grant), "runtime grant violates I-004");
  if (process.platform !== "win32") {
    assert(
      (statSync(grantPath).mode & 0o777) === 0o600,
      "grant file is not owner-only",
    );
  }

  const establishArgs = [
    "iam-establish",
    ...stateArgs(firstState, "2030-08-26T10:00:20Z"),
    grantPath,
  ];
  const [firstSession, duplicateSession] = await Promise.all([
    runHostAsync(establishArgs),
    runHostAsync(establishArgs),
  ]);
  assert(validateOutcome(firstSession), "first outcome violates I-005");
  assert(validateOutcome(duplicateSession), "duplicate outcome violates I-005");
  assert(
    firstSession.status === "established" &&
      duplicateSession.status === "established" &&
      firstSession.sessionReference === duplicateSession.sessionReference,
    "concurrent delivery created or reported different sessions",
  );
  const restarted = runHost(establishArgs).json;
  assert(
    restarted.sessionReference === firstSession.sessionReference,
    "restart did not reconcile the original session",
  );

  const foreign = runHost([
    "iam-establish",
    ...stateArgs(secondState, "2030-08-26T10:00:20Z"),
    grantPath,
  ]).json;
  assert(
    foreign.status === "refused" && foreign.sessionReference === undefined,
    "another environment accepted the grant",
  );

  const tampered = structuredClone(grant);
  tampered.claims.applicationId = "workflow-app";
  writeFileSync(tamperedGrantPath, JSON.stringify(tampered));
  chmodSync(tamperedGrantPath, 0o600);
  const tamperedOutcome = runHost([
    "iam-establish",
    ...stateArgs(firstState, "2030-08-26T10:00:25Z"),
    tamperedGrantPath,
  ]).json;
  assert(
    tamperedOutcome.status === "refused" &&
      tamperedOutcome.reasonCode === "grant-signature-invalid",
    "modified signed claims were not refused",
  );

  runHost([
    "iam-issue-grant",
    ...stateArgs(firstState, "2030-08-26T10:00:30Z"),
    "--output",
    expiredGrantPath,
    requestPath,
  ]);
  const expired = runHost([
    "iam-establish",
    ...stateArgs(firstState, "2030-08-26T10:03:00Z"),
    expiredGrantPath,
  ]).json;
  assert(expired.status === "expired", "expired grant was not expired");

  const workflowRequest = {
    ...readJson(requestPath),
    actorId: "synthetic-coordinator",
    applicationId: "workflow-app",
    audience: "workflow-app-backend",
    surfaceId: "surface-workflow-001",
    roles: ["coordinator"],
  };
  writeFileSync(workflowRequestPath, JSON.stringify(workflowRequest));
  runHost([
    "iam-issue-grant",
    ...stateArgs(firstState, "2030-08-26T10:00:40Z"),
    "--output",
    workflowGrantPath,
    workflowRequestPath,
  ]);
  const workflowSession = runHost([
    "iam-establish",
    ...stateArgs(firstState, "2030-08-26T10:00:50Z"),
    workflowGrantPath,
  ]).json;
  assert(
    workflowSession.status === "established" &&
      workflowSession.actorId !== firstSession.actorId &&
      workflowSession.applicationId !== firstSession.applicationId,
    "scenario did not support distinct actors and applications",
  );

  const terminated = runHost([
    "iam-terminate",
    ...stateArgs(firstState, "2030-08-26T10:01:00Z"),
    "--session-reference",
    firstSession.sessionReference,
    "--reason",
    "scenario-stopped",
  ]).json;
  const terminatedAgain = runHost([
    "iam-terminate",
    ...stateArgs(firstState, "2030-08-26T10:01:10Z"),
    "--session-reference",
    firstSession.sessionReference,
    "--reason",
    "scenario-stopped",
  ]).json;
  assert(
    terminated.status === "terminated" &&
      terminated.outcomeId === terminatedAgain.outcomeId,
    "termination was not monotonic and idempotent",
  );

  const journal = readFileSync(
    resolve(firstState, "iam-01/iam-security-journal.jsonl"),
    "utf8",
  );
  for (const prohibited of [
    grant.signature.value,
    grant.claims.grantId,
    "local-signing-key.bin",
    "authorization",
    "cookie",
  ]) {
    assert(
      !journal.includes(prohibited),
      `security journal disclosed ${prohibited}`,
    );
  }

  runHost([
    "iam-revoke",
    ...stateArgs(firstState, "2030-08-26T10:02:00Z"),
    "--reason",
    "operator-security-response",
  ]);
  const revoked = runHost(["iam-health", "--state-dir", firstState], {
    expected: [3],
  }).json;
  assert(revoked.identityState === "not-ready", "revoked trust remained ready");

  runHost([
    "iam-configure-demo",
    ...stateArgs(rebuiltState, "2030-08-26T10:05:00Z"),
  ]);
  const rebuilt = runHost(["iam-health", "--state-dir", rebuiltState]).json;
  assert(
    rebuilt.trustDomain !== firstHealth.trustDomain,
    "rebuild retained the former trust domain",
  );

  console.log(
    "M2 runtime OK: independent roots, visible trust profile, bounded workload and policy authority, signed grants, cross-environment refusal, at-most-one sessions, termination, revocation and rebuild recovery.",
  );
} finally {
  rmSync(temporary, { recursive: true, force: true });
}
