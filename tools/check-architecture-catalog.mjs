import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const componentsPath = resolve(root, "architecture/components.json");
const contractsPath = resolve(root, "contracts/catalog.json");
const logicalOverviewPath = resolve(
  root,
  "docs/architecture/logical/README.md",
);

const components = JSON.parse(readFileSync(componentsPath, "utf8")).components;
const contracts = JSON.parse(readFileSync(contractsPath, "utf8")).contracts;
const logicalOverview = readFileSync(logicalOverviewPath, "utf8");

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function assertUnique(items, label) {
  const ids = items.map((item) => item.id);
  const duplicates = ids.filter((id, index) => ids.indexOf(id) !== index);
  assert(
    duplicates.length === 0,
    `${label} contain duplicate identifiers: ${duplicates.join(", ")}`,
  );
}

assert(
  components.length === 20,
  `Expected 20 logical components, found ${components.length}`,
);
assert(
  contracts.length === 39,
  `Expected 39 contract families, found ${contracts.length}`,
);
assertUnique(components, "Components");
assertUnique(contracts, "Contracts");

const contractIds = new Set(contracts.map((contract) => contract.id));
const allowedMaturities = new Set([
  "planned",
  "skeleton",
  "in-development",
  "demonstrated",
]);
const allowedContractStatuses = new Set([
  "planned",
  "working-draft",
  "agreed",
  "implemented",
]);

for (const component of components) {
  assert(
    /^[A-Z]{2,3}-\d{2}$/.test(component.id),
    `Invalid component identifier: ${component.id}`,
  );
  assert(
    logicalOverview.includes(`\`${component.id}\``),
    `${component.id} is missing from the logical overview`,
  );
  assert(
    allowedMaturities.has(component.maturity),
    `${component.id} has invalid maturity ${component.maturity}`,
  );
  for (const contractId of component.contracts) {
    assert(
      contractIds.has(contractId),
      `${component.id} references unknown contract ${contractId}`,
    );
  }
  if (component.repositoryPath !== null) {
    assert(
      existsSync(resolve(root, component.repositoryPath)),
      `${component.id} repository path is missing: ${component.repositoryPath}`,
    );
  }
}

for (const contract of contracts) {
  assert(
    /^[A-Z]{1,2}-\d{3}$/.test(contract.id),
    `Invalid contract identifier: ${contract.id}`,
  );
  assert(
    logicalOverview.includes(`\`${contract.id}\``),
    `${contract.id} is missing from the logical overview`,
  );
  assert(
    allowedContractStatuses.has(contract.status),
    `${contract.id} has invalid status ${contract.status}`,
  );
  for (const path of [contract.documentation, contract.schema]) {
    if (path !== null) {
      assert(
        existsSync(resolve(root, path)),
        `${contract.id} declares a missing file: ${path}`,
      );
    }
  }
}

console.log(
  `Architecture catalogue OK: ${components.length} components, ${contracts.length} contract families.`,
);
