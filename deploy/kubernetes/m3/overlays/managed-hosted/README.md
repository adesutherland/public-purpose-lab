# M3.4 managed-hosted Kubernetes contract

Status: In-development, synthetic-only deployment contract

This overlay binds the portable M3 workloads to the managed identity profile.
It is intentionally incomplete without a protected, account-specific overlay.
It must not be applied directly or treated as a deployable hosted service.

The account-specific layer must supply, without committing their contents:

- `ppl-m3-managed-environment`, with `environmentId`, `projectId`,
  `kmsIssuerKeyVersion`, the two exact HTTPS origins and their exact Google OIDC
  callback URIs;
- `ppl-m3-managed-identity`, with the retained environment
  `trust-bundle.json` and protected `identity-configuration.json`;
- separate `ppl-m3-google-oidc-director` and
  `ppl-m3-google-oidc-presentation` Secrets, each containing `clientId`,
  `client-secret` and a protected `role-mapping.json`;
- environment-specific NATS TLS and NKey Secrets referenced by the base;
- an immutable image digest and source-revision evidence; and
- HTTPS ingress, DNS, certificate, network policy, expiry and teardown bindings
  owned by the protected infrastructure repository.

Only the `m3-identity-broker` service account receives a projected Kubernetes
identity token. The protected infrastructure layer grants that exact principal
only the Cloud KMS signing permission for the pinned issuer key version. The
Director and Presentation service accounts receive no Google API authority.
No exported Google service-account key is supported.

Render the portable contract for structural validation:

```sh
kubectl kustomize deploy/kubernetes/m3/overlays/managed-hosted
```

A successful render proves only that the portable Kubernetes objects compose.
Managed readiness, OIDC, KMS signing, immutable-image use, ingress streaming,
automatic expiry, cost and conclusive teardown require protected hosted
evidence.
