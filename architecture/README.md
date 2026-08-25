# Architecture implementation catalogue

The maintained architecture direction remains under
[`docs/architecture/`](../docs/architecture/README.md). This directory contains
machine-readable catalogues used by build and consistency checks.

- [`components.json`](components.json) lists every logical component, its
  maturity, principal contract families and any current repository path.
- [`../contracts/catalog.json`](../contracts/catalog.json) lists every contract
  family and its detailed specification or schema when one exists.

A logical component receives a source package only when a demonstrated slice
needs executable behaviour. The catalogue therefore does not create one service
or crate per component. A repository path marked `skeleton` identifies a build
boundary only; it is not evidence that the component behaviour is implemented.

Run `pnpm check:architecture` to verify identifiers, references, documentation
links and declared repository paths.
