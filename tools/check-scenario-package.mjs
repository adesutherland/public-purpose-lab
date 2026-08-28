import { createHash } from "node:crypto";
import { lstatSync, readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";
import canonicalize from "canonicalize";
import jsonValidator from "json-dup-key-validator";

const root = resolve(import.meta.dirname, "..");
const packageDirectory = resolve(
  root,
  "scenarios/presentation-control-assurance",
);
const permittedFiles = new Set(["manifest.json", "scenario.json"]);
const maximumDocumentBytes = 65_536;

function fail(reason) {
  throw new Error(`Scenario package refused: ${reason}`);
}

function readStrictJson(path) {
  const data = readFileSync(path);
  if (data.byteLength > maximumDocumentBytes) fail("document-size-exceeded");
  const text = data.toString("utf8");
  if (text.includes("\u0000")) fail("non-i-json-content");
  return { data, value: jsonValidator.parse(text, false) };
}

function refuseProhibitedContent(value, path = "$") {
  if (Array.isArray(value)) {
    value.forEach((item, index) =>
      refuseProhibitedContent(item, `${path}[${index}]`),
    );
    return;
  }
  if (value === null || typeof value !== "object") {
    if (typeof value === "string" && /https?:\/\//iu.test(value))
      fail(`route-or-url-content:${path}`);
    return;
  }
  for (const [key, item] of Object.entries(value)) {
    if (
      new Set([
        "password",
        "secret",
        "token",
        "cookie",
        "credential",
        "privatekey",
        "apikey",
        "route",
        "url",
        "broker",
        "subject",
        "shell",
        "sql",
        "script",
      ]).has(key.replaceAll("-", "").toLowerCase())
    )
      fail(`prohibited-field:${path}.${key}`);
    refuseProhibitedContent(item, `${path}.${key}`);
  }
}

function digest(value) {
  return createHash("sha256").update(canonicalize(value), "utf8").digest("hex");
}

for (const entry of readdirSync(packageDirectory)) {
  if (!permittedFiles.has(entry)) fail(`unlisted-file:${entry}`);
  const metadata = lstatSync(resolve(packageDirectory, entry));
  if (!metadata.isFile() || metadata.isSymbolicLink())
    fail(`unsafe-file:${entry}`);
}

const { value: manifest } = readStrictJson(
  resolve(packageDirectory, "manifest.json"),
);
const { data: scenarioBytes, value: scenario } = readStrictJson(
  resolve(packageDirectory, "scenario.json"),
);

if (manifest.packageId !== scenario.packageId) fail("package-id-conflict");
if (manifest.packageVersion !== scenario.packageVersion)
  fail("package-version-conflict");
if (manifest.scenario.path !== "scenario.json") fail("unsafe-scenario-path");
if (manifest.scenario.sizeBytes !== scenarioBytes.byteLength)
  fail("scenario-size-conflict");
if (manifest.scenario.digest !== digest(scenario))
  fail("scenario-digest-conflict");
refuseProhibitedContent(scenario);

console.log(
  JSON.stringify({
    packageId: manifest.packageId,
    packageVersion: manifest.packageVersion,
    packageDigest: digest(manifest),
    scenarioDigest: manifest.scenario.digest,
  }),
);
