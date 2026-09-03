import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import Ajv2020 from "ajv/dist/2020.js";
import addFormats from "ajv-formats";

const root = resolve(import.meta.dirname, "..");
const catalog = readJson("contracts/catalog.json");
const contractSets = [
  {
    name: "common",
    ids: ["C-001", "C-002", "C-003", "C-004", "C-005", "C-006"],
    fixtures: "contracts/common/fixtures.json",
    compatibility: "contracts/common/compatibility.json",
  },
  {
    name: "identity",
    ids: ["I-001", "I-002", "I-003", "I-004", "I-005"],
    fixtures: "contracts/identity/fixtures.json",
    compatibility: "contracts/identity/compatibility.json",
  },
  {
    name: "authorisation",
    ids: ["AZ-001"],
    fixtures: "contracts/authorisation/fixtures.json",
    compatibility: "contracts/authorisation/compatibility.json",
  },
  {
    name: "demonstration",
    ids: ["D-001", "D-002", "D-003", "D-004"],
    fixtures: "contracts/demonstration/fixtures.json",
    compatibility: "contracts/demonstration/compatibility.json",
  },
  {
    name: "presentation",
    ids: ["P-001", "P-002", "P-003", "P-004"],
    fixtures: "contracts/presentation/fixtures.json",
    compatibility: "contracts/presentation/compatibility.json",
  },
  {
    name: "source",
    ids: ["A-001", "A-002"],
    fixtures: "contracts/source/fixtures.json",
    compatibility: "contracts/source/compatibility.json",
    statuses: { "A-001": "implemented", "A-002": "working-draft" },
    versions: { "A-001": "0.1.0", "A-002": "0.1.0" },
  },
  {
    name: "knowledge",
    ids: ["K-001"],
    fixtures: "contracts/knowledge/fixtures.json",
    compatibility: "contracts/knowledge/compatibility.json",
    statuses: { "K-001": "implemented" },
    versions: { "K-001": "0.1.0" },
  },
];
const ajv = new Ajv2020({ allErrors: true, strict: true });

addFormats(ajv);

function readJson(relativePath) {
  return JSON.parse(readFileSync(resolve(root, relativePath), "utf8"));
}

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function formatErrors(errors) {
  return (errors ?? [])
    .map((error) => `${error.instancePath || "/"} ${error.message}`)
    .join("; ");
}

const contractsById = new Map();
for (const set of contractSets) {
  const contracts = catalog.contracts.filter((contract) =>
    set.ids.includes(contract.id),
  );
  assert(
    contracts.length === set.ids.length,
    `Expected ${set.ids.length} ${set.name} contracts, found ${contracts.length}`,
  );
  for (const contract of contracts) {
    const expectedStatus =
      set.statuses?.[contract.id] ?? set.status ?? "agreed";
    const expectedVersion =
      set.versions?.[contract.id] ?? set.version ?? "1.0.0";
    assert(
      contract.status === expectedStatus,
      `${contract.id} must be ${expectedStatus}`,
    );
    assert(
      contract.documentation !== null,
      `${contract.id} has no documentation`,
    );
    assert(contract.schema !== null, `${contract.id} has no schema`);
    assert(
      existsSync(resolve(root, contract.documentation)),
      `${contract.id} documentation is missing: ${contract.documentation}`,
    );
    const schema = readJson(contract.schema);
    assert(
      schema.$id ===
        `urn:public-purpose-lab:contract:${contract.id}:${expectedVersion}`,
      `${contract.id} has unexpected schema identifier ${schema.$id}`,
    );
    assert(
      ajv.validateSchema(schema),
      `${contract.id} schema is invalid: ${formatErrors(ajv.errors)}`,
    );
    ajv.addSchema(schema);
    contractsById.set(contract.id, contract);
  }
}

let fixtureCount = 0;
for (const set of contractSets) {
  const fixtureManifest = readJson(set.fixtures);
  for (const fixture of fixtureManifest.fixtures) {
    const validate = ajv.getSchema(fixture.schemaId);
    assert(
      validate !== undefined,
      `Unknown fixture schema ${fixture.schemaId}`,
    );
    const actual = validate(readJson(fixture.file));
    assert(
      actual === fixture.valid,
      `${fixture.file} expected valid=${fixture.valid}, got valid=${actual}: ${formatErrors(validate.errors)}`,
    );
    fixtureCount += 1;
  }
}

const compatibilitySchema = ajv.getSchema(
  "urn:public-purpose-lab:contract:C-006:1.0.0",
);
assert(compatibilitySchema !== undefined, "C-006 schema was not registered");
const describedIds = new Set();
for (const set of contractSets) {
  const compatibilityManifest = readJson(set.compatibility);
  assert(
    compatibilityManifest.descriptors.length === set.ids.length,
    `Every ${set.name} contract must have one compatibility descriptor`,
  );
  for (const path of compatibilityManifest.descriptors) {
    const descriptor = readJson(path);
    assert(
      compatibilitySchema(descriptor),
      `${path} is invalid: ${formatErrors(compatibilitySchema.errors)}`,
    );
    assert(
      set.ids.includes(descriptor.describedContractId),
      `${path} describes an unexpected contract ${descriptor.describedContractId}`,
    );
    assert(
      !describedIds.has(descriptor.describedContractId),
      `${descriptor.describedContractId} has duplicate compatibility descriptors`,
    );
    describedIds.add(descriptor.describedContractId);

    const catalogEntry = contractsById.get(descriptor.describedContractId);
    const schema = readJson(catalogEntry.schema);
    assert(
      descriptor.schemaId === schema.$id,
      `${path} does not identify the catalogued schema`,
    );
    assert(
      descriptor.status === catalogEntry.status,
      `${path} status differs from the contract catalogue`,
    );
    for (const example of descriptor.examples) {
      assert(
        existsSync(resolve(root, example)),
        `${path} example is missing: ${example}`,
      );
    }
  }
}

console.log(
  `Contracts OK: ${contractsById.size} schemas, ${fixtureCount} fixtures, ${describedIds.size} compatibility descriptors.`,
);
