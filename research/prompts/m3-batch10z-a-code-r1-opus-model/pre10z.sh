#!/usr/bin/env bash
# 对照臂：同一份模型跑在第 10z 批之前的显存账上（把改动全文里 gpu_memory_ledger.rs 那一段反着打回去）。
# 起设备那一步换成 10z 之前的 check_registered_before_starting_the_device（没有重新登记）；别的一字不改。断言照 10z 的数写，在这一臂上红是预期的，只看 ATTACK 行。
# 用法（仓根）：bash scripts/hosts/run-with-memory-cap.sh 12G bash scripts/hosts/capped.sh 15 \
#   bash research/prompts/m3-batch10z-a-code-r1-opus-model/pre10z.sh <空目录> <编译目录>
set -euo pipefail
copy="${1:?空目录}"
target="${2:?编译目录}"
repository="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
model="$repository/research/prompts/m3-batch10z-a-code-r1-opus-model"
rsync -a --exclude target --exclude .git "$HOME/.local/share/singlefs/work/m3-batch10z-a-code-r1/tree/" "$copy/"
sed -n '1278,2496p' "$repository/research/prompts/_m3-batch10z-a-code-r1-diff.md" > "$copy/ledger-10z.patch"
(cd "$copy" && git apply -R "$copy/ledger-10z.patch")
python3 - "$model/ledger_attack_10z.rs" "$copy/crates/singlefs-harness/tests/ledger_attack_on_10y.rs" <<'PY'
import sys
t = open(sys.argv[1]).read()
for old, new in [
    ("        self.ledger\n            .prepare_the_driver_share_before_starting_the_device(0)?;",
     "        self.ledger.check_registered_before_starting_the_device(0);"),
    ("let prepared = bench.ledger.prepare_the_driver_share_before_starting_the_device(0);",
     "let prepared: Result<(), ()> = { bench.ledger.check_registered_before_starting_the_device(0); Ok(()) };"),
]:
    assert t.count(old) == 1, old
    t = t.replace(old, new)
open(sys.argv[2], "w").write(t)
PY
cd "$copy"
CARGO_TARGET_DIR="$target" nice -n 19 cargo test --offline -p singlefs-harness --test ledger_attack_on_10y -- --nocapture --test-threads 1 || true
