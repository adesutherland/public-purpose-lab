# ADR-0014: Bind presenters through Google OIDC and backend sessions

Status: Accepted
Date: 2026-08-27

## Context

M3 requires an external human presenter identity distinct from the Scenario
Director workload, synthetic actors, operators and business authority. The
initial founder group already has Google identities, and Google OpenID Connect
provides a readily available standards-based authentication source.

Authentication by Google does not establish a Public Purpose Lab role,
permission, relationship, consent, professional status or authority to perform
a scenario or business action. The frontend must not use a Google token as a
shared service credential or pass it through component events.

Local and hosted use need the same identity and authorisation semantics while
using different registered redirect clients. Automated assurance must not
depend on a real person's account.

## Decision

Use Google OpenID Connect as the first external-human authentication binding
for authorised presenters, followed by an ordinary backend-managed Lab
session and explicit local role mapping.

The binding is:

- a backend-mediated OpenID Connect Authorization Code flow, with PKCE where
  supported, exact registered redirects, state and nonce validation;
- validation of issuer, audience, signature, issue/expiry time and nonce using
  Google's discovery and key material;
- stable external identity keyed by the verified issuer and `sub` claim, not by
  email address or display name;
- minimal `openid`, `email` and `profile` scopes for the first profile, with no
  access to other Google APIs and no offline access/refresh token;
- an environment-specific mapping from that authenticated identity to an
  enabled Lab presenter role and purposes;
- `AUT-01` and receiving-component enforcement for every protected action; and
- a new backend application session after successful login, using a secure,
  HttpOnly, appropriately SameSite cookie, session rotation and CSRF/origin
  controls.

Google tokens and authorisation codes remain within the authentication backend
and are discarded when no longer required. They do not enter the browser
application state, component event bus, scenario package, cue, evidence record
or analytics. The browser sees the resulting application session and
privacy-minimised presenter context only.

Local interactive use opens the system browser and uses a separately registered
loopback/localhost client appropriate to the application type. Hosted use uses
a separate web client and exact HTTPS callback. A local client secret is not
treated as confidential merely because it is packaged with desktop software.
Embedded browser login is not used.

Automated tests use an explicit in-process or test-environment identity adapter
that produces the same `I-001` semantic context. That adapter is synthetic,
unavailable in hosted/shared profiles and cannot make external-authentication
readiness pass.

Initial role administration is deliberately small: a protected allow-list of
founder-approved issuer/subject identities and environment roles. Email may be
shown for human confirmation but changes to email do not silently create a new
or transferred authority mapping. Unknown authenticated Google users are
denied without identity enumeration.

## Alternatives considered

- **Use Google login as authorisation:** rejected because provider identity
  does not express Lab, scenario or business authority.
- **Share one founder account or provider token:** rejected because it removes
  attribution and creates an unsafe shared credential.
- **Username/password accounts owned by the Lab:** rejected initially because
  password lifecycle, reset and breach responsibilities add no scenario value.
- **Passkeys or another enterprise identity provider first:** both remain
  viable later, but Google OIDC is the smallest available external binding for
  the initial authorised group.
- **Local bypass for every developer:** rejected as the acceptance path. A
  visibly synthetic test adapter remains available only for automated or
  isolated local tests.

## Consequences

- Presenter identity becomes attributable without making the Lab a password
  provider.
- Google availability and OAuth client configuration become dependencies for
  interactive presenter login.
- Separate local and hosted clients, redirect configuration and consent-screen
  governance are required.
- Role mapping, removal, session revocation and evidence remain Lab
  responsibilities.
- A user may authenticate successfully and still be denied every presenter or
  business action; the UI must explain this safely.
- Replacing Google with another OIDC provider remains possible behind the
  `I-001` adapter and does not change scenario contracts.

## Validation and review

Evidence must include:

- authorised and authenticated-but-unauthorised users;
- wrong issuer, audience, redirect, state, nonce, expired token and replay;
- changed email with stable subject and changed subject with the same email;
- session fixation, logout, role removal and environment mismatch;
- CSRF/origin refusal for protected Director actions;
- separate presenter, operator, synthetic-human and workload identities in one
  scenario;
- test-adapter rejection in hosted/shared profiles; and
- scanning of events, logs, browser state and evidence for provider tokens,
  codes and unnecessary claims.

Review whether an allow-list remains adequate when the authorised group grows
or another organisation participates. That review may select group/relationship
administration or an external policy source; it must not infer authority from
an email domain alone.

## Reference material

- [Google OpenID Connect](https://developers.google.com/identity/openid-connect/openid-connect)
