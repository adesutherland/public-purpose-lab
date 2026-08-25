# Frontend workspace

The frontend workspace contains three independently deployable browser
surfaces and one shared UI package:

- `apps/workbench` for governed asset discovery, staging, query and reporting;
- `apps/director` for scenario preparation and demonstration control;
- `apps/presentation` for audience-facing demonstration views; and
- `packages/ui` for the accessible visual shell and shared surface catalogue.

These applications are repository skeletons. The Director does not yet control
the other surfaces, and none of the visible actions connects to identity,
events, data, retrieval, reporting or operational services. Those integrations
will be added only through reviewed contracts.

Use `pnpm dev:workbench`, `pnpm dev:director` or `pnpm dev:presentation` from
the repository root to run one surface locally.
