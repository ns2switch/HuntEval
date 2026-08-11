# R8 independent reproducibility review

**Status:** awaiting independent reviewer

**Reviewed revision:** not assigned

**Reviewer identity and organization:** not assigned

## Required clean-room procedure

An independent reviewer must use a clean ephemeral `x86_64-unknown-linux-gnu` environment, empty declared caches, Rust 1.93.1, Python 3.11 or newer, the documented Bubblewrap capability, no production credentials, and the exact protected source revision.

The reviewer must execute `r8-release.sh` into a new absolute directory, compare package inventories and normalized evidence across two builds, install the archive, run the CLI and worker checks, validate retained migration fixtures, execute the official benchmark pack from published instructions, verify reports, checksums, SBOM, provenance, and signatures offline, and record every exact or policy-allowed difference.

## Result

No independent clean-room result exists yet. The implementation author cannot self-certify this review. A failed reproduction rejects the candidate and requires a new immutable identity after source or documentation correction.
