# HuntEval schema 0.9

Schema 0.9 adds local analytical corpus, deterministic query, extension capability, and one-shot managed-tool adapter request/response contracts. Schemas 0.3 through 0.8 remain immutable. Analytical corpora contain verified public artifacts only; evaluator analytical corpora are never deployment-visible. Extension manifests request capabilities but do not grant them. Unknown fields and versions fail closed.
