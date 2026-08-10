# Use case: query verified evidence and validate an extension

An evaluator wants to find prior runs that mention a compromised access key without exposing historical evaluator artifacts to the deployment under test. The same operator also wants to preflight a local managed-tool adapter before allowing it into a benchmark definition.

## 1. Build and query an evaluator-only corpus

The corpus manifest binds one authorization scope, safe relative source paths, exact SHA-256 digests, public source kinds, and successful verification. HuntEval reopens each source without following symlinks, confines it to the selected root, verifies its digest, rejects private-field families, and derives a deterministic index.

```bash
target/debug/hunteval knowledge validate \
  examples/contracts/v0.9/analytical-corpus-manifest.json

target/debug/hunteval knowledge build \
  examples/contracts/v0.9/analytical-corpus-manifest.json \
  --root .

target/debug/hunteval knowledge query \
  examples/contracts/v0.9/analytical-corpus-manifest.json \
  examples/contracts/v0.9/analytical-query.json \
  --root . \
  --audit /tmp/hunteval-retrieval-audit.jsonl

target/debug/hunteval knowledge verify \
  examples/contracts/v0.9/analytical-corpus-manifest.json \
  --root . \
  --audit /tmp/hunteval-retrieval-audit.jsonl
```

Add `--format html` to render the same result as escaped, script-free static HTML; JSON remains authoritative. The result identifies the exact source, artifact hash, normalized field, and bounded excerpt. Each query appends a UTC event that binds the query, result, index, scope, measured local latency, explicit unavailable cost, sequence, and previous event hash. Offline verification rejects a broken journal. It does not invent a metric or causal conclusion. A deployment-visible corpus is a different scope and accepts authored public documents only; it cannot query evaluator history.

## 2. Resolve adapter capabilities

An extension manifest requests capabilities and bounded resources. It cannot authorize itself. The runner intersects the request with a versioned policy and returns either an exact eligible resolution or stable rejection reasons.

```bash
target/debug/hunteval extension validate \
  examples/contracts/v0.9/extension-manifest.json \
  --policy examples/contracts/v0.9/extension-capability-policy.json
```

Network remains denied, executable bytes are content-addressed, and scored tools remain runner-mediated. `extension conformance` binds a selected local executable to the manifest and policy. Deployment adapters that declare protocol 0.3 run through the production supervised protocol flow. Managed-tool adapters run as a bounded one-request/one-response schema 0.9 process. Both record a transcript hash and reject timeout, crash, malformed output, identity drift, or policy failure. Passing applies only to those exact bytes, declared controls, and public fixtures; it is not a general security certification.

## 3. Use the Python SDK offline

```python
from pathlib import Path
from hunteval_sdk import read_public_artifact

artifact = read_public_artifact(
    Path("."),
    "examples/r7/public-run.json",
    "561157cc864a2ad2582b00395face376cc5a25f75f234328f061dfa37b52d0e6",
    "run",
)
print(artifact.schema_version)
```

The SDK supplies strict public contract models, local digest-verifying readers, and a bounded deployment-side JSONL peer. It has no evaluation, scoring, orchestration, provider, ground-truth, or direct scored-tool authority.
