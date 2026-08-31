# M3.4 managed-hosted identity and resilience evidence

Status: Complete at in-development, synthetic-only maturity

Evidence date: 28 August 2026

## Claim and boundary

This record supplements the
[M3.4 local evidence](m3-4-local-identity-and-resilience.md) with a bounded
Google Cloud activation. It demonstrates the accepted managed-hosted binding:
Google-backed external application sessions, environment-bound Cloud KMS
trust, exact GKE workload identity, backend-only synthetic sign-in, two surface
actors, HTTPS presentation events, restart/reconnect and conclusive teardown.

It does not demonstrate production readiness, high availability, legal or
regulatory compliance, clinical safety, non-synthetic-data authority or
business completion. The result qualifies M3.4 only at its stated
in-development, synthetic-only maturity.

## Reviewed runtime

- Public source: `6a0162c859ae8d445563e23f7b125f8c260344aa`
- Runtime image:
  `m3-runtime@sha256:a5cc99cbed59ca1a2ad521d6d85f2f35d90309464923c807be3ae5435086dd58`
- Activation: `m3-4-20260828-01` in `europe-west2`
- HTTPS surfaces: separate Director and Presentation preview names, removed
  after the run
- Composition: one Identity Broker, Director and Presentation Gateway, one
  file-backed NATS JetStream server and component-owned persistent state

All three application Pods resolved the exact immutable digest. External
liveness, readiness and contract endpoints returned `200` over one valid
public certificate covering the two temporary names.

## Trust and identity results

The environment used a dedicated managed trust domain and epoch. Its
non-exportable, software-protected Cloud KMS Ed25519 root and issuer created a
public certificate chain with protected backup-and-restore evidence. The root
was disabled after bootstrap and remained disabled; the runtime issuer remained
enabled.

Only the `public-purpose-lab/m3-identity-broker` Kubernetes principal could
sign with the exact issuer key and read the issuer metadata needed to validate
its state and public key. Director and Presentation mounted no Kubernetes
service-account token. The broker checked the issuer version, algorithm,
protection level, checksum, public key and expected trust fingerprint before
becoming Ready.

Separate Google OAuth clients authenticated the Director and Presentation
origins. Versioned role maps matched Google's exact issuer and subject plus the
required application audience and role; email text was not used as the
authority key.

The Chrome evidence path established separate Google-backed application
sessions and proved:

- `synthetic-audience-user` on `audience-display`;
- `synthetic-reviewer` on `reviewer-workbench` through an independent
  application session;
- semantic `assurance-welcome` delivery and presentation-progress outcomes on
  both surfaces without a business-completion claim;
- existing-session refusal after a role-map version change;
- new-login refusal when the mapped identity was disabled; and
- normal readiness after the exact reviewed map was restored, without reviving
  the invalidated session.

Cloud KMS Data Access audit recorded the broker's public-key read and managed
signature with the expected permissions. Disclosure-minimised application logs
contained no detected provider token, client secret, cookie, CSRF value, NKey
seed or protected subject identifier.

## Restart, stop and successor results

NATS and all three application workloads were deliberately restarted. They
returned Ready on the same image digest. The Director restored its opaque
external session and running Demonstration Session at revision 4. A surface
session invalidated by the role-map change did not silently return.

Stop produced terminal revision 5. Reset created a clean successor, and a
fresh Google-backed workbench session successfully established the same named
synthetic reviewer in that successor. This demonstrated that stop released the
prior synthetic binding instead of sharing it across Demonstration Sessions.
Every successor used for assurance was stopped before infrastructure teardown.

## Expiry, cost and teardown

The cloud lifecycle armed a one-time expiry for `2026-08-29T00:18:48Z` before
creating infrastructure. Normal `off` completed at `2026-08-28T21:55:32Z`,
well before the deadline, and removed the pending task.

The maximum cluster-resource interval was approximately 1 hour 34 minutes.
Using current list prices for the GKE cluster fee, Autopilot requested
resources, forwarding rule, Cloud NAT and small supporting usage, the
conservative gross ceiling for this activation and its image builds is USD
3.00 before credits. Provider billing actual, credits and net billed amount
remain pending because billing data lags; none is recorded as zero.

Independent inventory found no activation cluster, network, subnetwork,
router, NAT, forwarding rule, disk, reserved address or pending expiry task.
Cloudflare and Google resolvers returned NXDOMAIN for both temporary names.
The retained foundation reported no drift; environment trust and recovery
evidence remain by design.

This run proves expiry arming and reconciliation by normal `off`. Execution of
the scheduled expiry callback remains an M3.5 evidence case.

## Result

The local and managed-hosted records together close M3.4 at the accepted
development maturity. Following the founder-approved programme re-baseline on
31 August 2026, M3.5 remains a bounded automatic-expiry and cost-evidence
closure activity that may run in parallel. The M4 Source-to-Report Value Slice
is the primary business-facing delivery milestone; neither this technical
assurance result nor the later business demonstration constitutes production,
compliance or legal authority.
