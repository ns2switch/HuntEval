#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

readonly temporary="$(mktemp -d)"
trap 'rm -rf -- "$temporary"' EXIT
printf 'bounded R8 artifact\n' >"$temporary/artifact"
ssh-keygen -q -t ed25519 -N '' -C hunteval-r8-test -f "$temporary/key"
readonly public_key="$(cat "$temporary/key.pub")"
readonly fingerprint="$(ssh-keygen -lf "$temporary/key.pub" -E sha256 | awk '{print $2}')"
python3 - "$temporary/policy.json" "$public_key" "$fingerprint" <<'PY'
import json, pathlib, sys
pathlib.Path(sys.argv[1]).write_text(json.dumps({
    "schema_version": "1.0",
    "signer_identity": "hunteval-r8-rehearsal",
    "namespace": "hunteval-release",
    "public_key": sys.argv[2],
    "key_fingerprint": sys.argv[3],
    "repository": "ns2switch/HuntEval",
    "ref": "refs/heads/main",
    "workflow": "r8-local-rehearsal",
    "valid_from_epoch": 0,
    "valid_until_epoch": 4102444800,
    "revoked_fingerprints": [],
}, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
python3 scripts/r8_sign.py sign \
    --artifact "$temporary/artifact" --key "$temporary/key" \
    --policy "$temporary/policy.json" --output "$temporary/signature"
python3 scripts/r8_sign.py verify \
    --artifact "$temporary/artifact" --signature-root "$temporary/signature" \
    --repository ns2switch/HuntEval --ref refs/heads/main \
    --workflow r8-local-rehearsal --at-epoch 1

cp "$temporary/artifact" "$temporary/original"
printf 'tampered\n' >>"$temporary/artifact"
if python3 scripts/r8_sign.py verify \
    --artifact "$temporary/artifact" --signature-root "$temporary/signature" \
    --repository ns2switch/HuntEval --ref refs/heads/main \
    --workflow r8-local-rehearsal --at-epoch 1 >/dev/null 2>&1; then
    echo "error: tampered signed artifact was accepted" >&2
    exit 1
fi
mv "$temporary/original" "$temporary/artifact"
if python3 scripts/r8_sign.py verify \
    --artifact "$temporary/artifact" --signature-root "$temporary/signature" \
    --repository wrong/repository --ref refs/heads/main \
    --workflow r8-local-rehearsal --at-epoch 1 >/dev/null 2>&1; then
    echo "error: substituted signature identity was accepted" >&2
    exit 1
fi

echo "R8 offline signature fixtures pass"
