# Security policy

## Supported versions

HuntEval is currently pre-release. Security fixes are applied to the latest revision of the `main` branch until a versioned support policy is published.

## Reporting a vulnerability

Do not open a public issue for suspected vulnerabilities involving ground-truth disclosure, sandbox escape, SQL policy bypass, secret exposure, provenance forgery, or dependency compromise. Use the repository host's private security-reporting channel. Include a concise impact description, affected revision, reproduction steps, and any proposed mitigation. Do not include real credentials or sensitive production telemetry.

The maintainers will acknowledge a complete report, assess severity, coordinate a fix, and publish remediation information when disclosure is safe. This file intentionally does not promise response deadlines before project governance is established.

## Scope

The evaluated deployment, its output, retrieved documents, generated SQL, contributed datasets, and archive contents are untrusted. HuntEval must fail closed when it cannot enforce the configured ground-truth, filesystem, network, process, or tool-execution boundary.
