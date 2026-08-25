# Frontend workspace

The frontend workspace contains three independently deployable browser
surfaces and two shared packages:

- `apps/workbench` for governed asset discovery, staging, query and reporting;
- `apps/director` for scenario preparation and demonstration control;
- `apps/presentation` for audience-facing demonstration views;
- `packages/ui` for the accessible visual shell and shared surface catalogue;
  and
- `packages/contracts` for TypeScript consumption of the canonical M1 common
  contract shapes and maturity vocabulary.

The browser applications remain repository skeletons. The Director does not
yet control other surfaces, and none of the visible actions connects to
identity, events, data, retrieval, reporting or operational services. The
contract package is an in-development consumption surface, not a live browser
integration. Those integrations will be added only through reviewed contracts.

Use `pnpm dev:workbench`, `pnpm dev:director` or `pnpm dev:presentation` from
the repository root to run one surface locally.
