# AWS IAM episode 001

This deterministic synthetic episode contains ten CloudTrail-like control-plane events. The public fixture includes ordinary automation, a benign emergency-administration event, and suspicious identity activity without labels. The private root contains the evaluator-only expected events, entities, attack path, and techniques.

Regenerate the Parquet telemetry from the versioned source with:

```bash
cargo run -p hunteval-fixture-tool -- generate datasets/aws/aws-iam-001
```

The deployment boundary must expose only `public/`. The package index, source definition, and private root remain trusted runner inputs.
