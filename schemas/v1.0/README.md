# HuntEval schema 1.0

Schema 1.0 begins the R8 release-candidate contract family. R8-00 introduces the public interface inventory and its derived freeze manifest. R8-01 adds the exact compatibility matrix, and R8-02 adds the migration inventory and content-addressed receipts. An entry in the inventory is not a support or compatibility claim; only entries that pass the fail-closed freeze rules appear in `eligible_interfaces`.

The platform target matrix records native runner, archive, sandbox, support, and validation state separately for every release architecture. Evaluator-private artifacts, pending pre-R8 connectors, preview integrations, and experimental interfaces cannot be stable freeze candidates.
