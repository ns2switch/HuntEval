# R8 Wave A implementation evidence

## Status

R8-00, R8-01, and R8-02 are implemented locally. They are not complete release milestones until the exact protected revision passes the required GitHub checks. This document does not alter the pending state of v0.7.1 or v0.7.2 and does not claim R8 closure.

## R8-00 inventory and freeze

The versioned interface inventory covers artifacts, the official benchmark pack, CLI, commercial and framework connectors, extensions, knowledge, metrics, declared native package platforms, protocol, reports, schemas, scoring profiles, SDK, and topology. The deterministic freeze manifest is bound to inventory SHA-256 `002eb890ba8a28b1baef4a6b773c48a49f576582209e841881b12b23b3f4abf8`; macOS and Windows entries remain excluded previews.

Stable and retained entries require a public projection, satisfied precondition, documented authority and trust boundary, parser and bound documentation, a canonical fixture, and an exact verification gate. Pending connector entries remain excluded.

## R8-01 compatibility

`examples/contracts/v1.0/compatibility-matrix.json` is the machine-readable source of truth. It is validated against the exact freeze manifest and projected deterministically into `R8_COMPATIBILITY.md`. Supported and retained rules cannot reference excluded interfaces. Preview and unavailable rules require explicit reason codes and limitations. Missing combinations return a typed unavailable result rather than inferred compatibility.

Compatibility proves contract agreement only. It grants no runtime, tool, network, signing, publication, sandbox, or benchmark-quality authority.

## R8-02 migration and rejection

`examples/contracts/v1.0/migration-inventory.json` declares the retained protocol reader and existing benchmark-manifest and scoring-profile in-memory adapters. Future major schemas are rejected explicitly. Undeclared and ambiguous edges fail closed. Migration receipts bind the exact source bytes, target bytes, edge, and implementation without rewriting source artifacts.

## Focused verification

```text
cargo test -p hunteval-release
  10 passed; 0 failed
```

The suite covers canonical schemas and fixtures, deterministic normalization, fixture hashes, ambiguous combinations, excluded-interface support claims, unknown combinations, declared adaptations, future-major rejection, changed target bytes, unknown fields, unsupported versions, unsafe paths, and pending/private/unverified freeze candidates.

## Remaining evidence

- protected `R8 compatibility` workflow evidence on the exact revision;
- branch-protection inclusion after the check exists remotely;
- all later R8 waves;
- resolution or explicit governance revision of the pre-R8 closure dependency before R8 closure.
