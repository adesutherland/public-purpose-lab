# Architecture implementation evidence

This directory records bounded implementation evidence for architecture
milestones. An evidence record states the code and profile exercised, the
result, limitations and remaining acceptance gates. It does not promote a
working baseline into production or compliance assurance.

From Gate A onward, each implementation gate also retains a progress and
show-and-tell report in Markdown, real synthetic-data screenshots from the
evidenced walkthrough, and a visually verified PDF distribution copy. The
report identifies the exact source revision and environment, maps screenshots
to approved flow steps, records functions and rules delivered, explains prior
context and limitations, and states the next gate. Automated results do not
replace the walkthrough.

## Review by exception

A complete gate candidate is built, tested, walked through, documented,
rendered and visually verified once. Its implementation, Markdown evidence and
PDF distribution copy are published together as
`Final — subject to review sign-off by exception`.

- A successful founder review requires no status-only commit, repeated test or
  walkthrough, regenerated report or second publication.
- If review identifies an exception, the affected implementation or evidence
  is corrected, checks and walkthrough evidence are repeated in proportion to
  the change, and a new candidate is published.
- Risk-assess evidence reuse before creating a new build or fingerprint. An
  evidence- or documentation-only correction reuses the validated runtime and
  unaffected evidence, with its actual digest and provenance, unless it can
  change behaviour, presentation, contracts, security or the evidence claim.
  Rebuild and repeat only the assurance work needed for a material risk.
- The latest repository head is canonical. A corrected report supersedes the
  earlier report retained in Git history; a separate invalidation artefact is
  unnecessary unless an exceptional governance or safety issue requires one.

This process changes the mechanics of sign-off, not the approval boundary: a
candidate is not an approved gate until founder review succeeds.

- [M1 common interaction baseline](m1-common-interaction-baseline.md)
- [M2 local-synthetic identity baseline](m2-local-synthetic-identity-baseline.md)
- [M3.2 Google Cloud hosted-lifecycle spike](m3-2-google-cloud-hosted-lifecycle-spike.md)
- [M3.3 runtime walking skeleton](m3-3-runtime-walking-skeleton.md)
- [M3.4 local identity and resilience](m3-4-local-identity-and-resilience.md)
- [M3.4 managed-hosted identity and resilience](m3-4-managed-hosted-identity-and-resilience.md)
- [Gate A deployed component mesh show-and-tell](gate-a-component-mesh-show-and-tell.md)
- [Gate B environment, identity and portal orchestration show-and-tell](gate-b-orchestration-show-and-tell.md)
- [Gate C source validation and staged release progress show-and-tell](gate-c-validation-and-staging-show-and-tell.md)
