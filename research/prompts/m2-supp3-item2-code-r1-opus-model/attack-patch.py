#!/usr/bin/env python3
"""Attack-leg instrumentation for a repo copy (m2-supp3-item2-code-r1, Opus attack leg).
Adds runtime switches to the harness copy only; no semantic change when the env vars are unset.
  ATTACK_NO_HARNESS=1   drop the item-1 harness judgements (allocation generation, cold-start failure)
  ATTACK_NO_CHECKER=1   ignore checker violations (histories are then stopped only by model disagreement or panic)
  ATTACK_PROBE=1        print one line per step: step, kind, outcome, model bound of the session's current root, impl record count
Each anchor must match exactly once."""
import sys, pathlib
root = pathlib.Path(sys.argv[1])
def rep(path, old, new):
    p = root / path
    s = p.read_text()
    n = s.count(old)
    if n != 1:
        sys.exit(f"{path}: anchor count {n}: {old[:60]!r}")
    p.write_text(s.replace(old, new))
H = "crates/singlefs-harness/src/history.rs"
rep(H, "            } = apply_operation(pool, operation, step_index);\n",
       "            } = apply_operation(pool, operation, step_index);\n"
       "            let harness_judgement = if std::env::var_os(\"ATTACK_NO_HARNESS\").is_some() { None } else { harness_judgement };\n"
       "            if std::env::var_os(\"ATTACK_PROBE\").is_some() {\n"
       "                let impl_count = pool.session.as_ref().map(|s| s.allocator.records().len());\n"
       "                println!(\"ATTACK_PROBE seed={} step={} kind={:?} outcome={:?} model_bound={:?} impl_records={:?} model_disagreement={:?}\",\n"
       "                    history.seed.0, step_index, operation.kind(), outcome, pool.model.attack_current_bound(), impl_count,\n"
       "                    model_verdict.as_ref().and_then(|v| v.disagreement.as_ref()).map(|d| d.aspect));\n"
       "            }\n")
rep(H, "                violations = violations_on(&image, &mut tally);\n",
       "                violations = violations_on(&image, &mut tally);\n"
       "                if std::env::var_os(\"ATTACK_NO_CHECKER\").is_some() { violations.clear(); }\n")
rep(H, "        let violations_after_the_starting_point = violations_on(&image, &mut tally);\n",
       "        let mut violations_after_the_starting_point = violations_on(&image, &mut tally);\n"
       "        if std::env::var_os(\"ATTACK_NO_CHECKER\").is_some() { violations_after_the_starting_point.clear(); }\n")
M = "crates/singlefs-harness/src/model.rs"
rep(M, "    #[must_use]\n    pub fn has_writable_session(&self) -> bool {\n",
       "    /// attack-leg probe: allocation-record upper bound of the session's current root.\n"
       "    pub fn attack_current_bound(&self) -> Option<u64> {\n"
       "        self.session.as_ref().map(|s| s.current.allocation_records_upper_bound)\n"
       "    }\n\n"
       "    #[must_use]\n    pub fn has_writable_session(&self) -> bool {\n")
print("patched", root)
