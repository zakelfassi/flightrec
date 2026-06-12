# Security Policy

## Supported Versions

Only the latest release in the **0.x** series receives security fixes. Older patch versions are not backported.

| Version | Supported |
|---------|-----------|
| 0.x (latest) | Yes |
| 0.x (previous patch) | No |

## Reporting a Vulnerability

**Do not file a public GitHub issue for security vulnerabilities.**

Report via the GitHub private advisory flow:

**https://github.com/zakelfassi/flightrec/security/advisories/new**

There is no email channel for security reports. The advisory form is the only accepted intake path.

### What to include

- A description of the vulnerability and its impact
- Steps to reproduce (a minimal proof-of-concept is ideal)
- flightrec version and OS/architecture
- Whether you have a proposed fix or patch

### Response timeline

- **Acknowledgement**: within 48 hours of receiving the advisory
- **Triage**: within 7 days
- **Fix or resolution**: within **90 days** of the initial report

If a fix is complex and 90 days is not enough, we will communicate the revised timeline through the advisory thread before the window closes.

### Disclosure

We follow **coordinated disclosure**. Please do not publish details of the vulnerability until a fix has been released or the 90-day window has elapsed, whichever comes first. We will credit reporters in the release notes unless anonymity is requested.

## Threat Surface

flightrec is a local-only daemon. Its security surface is narrow but worth understanding:

- **Watched-path access**: flightrec reads every file under the configured watch roots and writes content-addressable blobs into `$FLIGHTREC_HOME` (default: `~/.flightrec/`). The blob store **inherits the filesystem permissions of the process running flightrec**. If flightrec is run as root or with elevated privileges, blobs will be owned by root. Do not run flightrec with more privilege than needed.
- **Config file**: `$FLIGHTREC_HOME/config.toml` is read at startup. Malicious content in this file can cause flightrec to watch unintended paths.
- **LLM provider keys**: if LLM reporting is enabled, the API key configured in `config.toml` is sent over HTTPS to the configured provider. The key is never stored in blobs or diff events.
- **No network listeners**: flightrec does not bind any ports. There is no remote-control surface.
