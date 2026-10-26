#!/usr/bin/env bash
# peer_trip6.sh：证差分会红。diffred = after 里把「映射位置换进 units」改成只改槽号为偶数的那一半（一个会让镜像变的故障），
# 编 diff_history_remount，跑种子 1–32、每段 64 步，与 out/diff-before-seeds1-32.lines（改前的同一批种子）逐行比，应当不同。
set -u
root=$(pwd); threads=${THREADS:-31}
cp "$root/model/tests/diff_history_remount.rs" "$root/diffred/crates/singlefs-harness/tests/diff_history_remount.rs"
python3 -I - "$root/diffred/crates/singlefs-core/src/transaction.rs" <<'PYEOF'
import sys
path = sys.argv[1]
text = open(path, encoding="utf-8").read()
old = "            version.units[*position].slot = slot;"
assert text.count(old) == 1
open(path, "w", encoding="utf-8").write(text.replace(old, "            if slot.0 % 2 == 0 {\n                version.units[*position].slot = slot;\n            }"))
PYEOF
find "$root/diffred/crates" -type f -exec touch {} +
( cd "$root/diffred" && bash "$root/scripts/hosts/capped.sh" "$threads" cargo test --offline --release -p singlefs-harness --test diff_history_remount --no-run 2>&1 | tail -2 )
bin=$(ls -t "$root"/diffred/target/release/deps/diff_history_remount-* 2>/dev/null | grep -v '\.d$' | head -1)
( cd "$root/diffred/crates/singlefs-harness" && DIFF_SEED_FIRST=1 DIFF_SEEDS=32 DIFF_STEPS=64 bash "$root/scripts/hosts/capped.sh" "$threads" "$bin" --exact diff_history_remount_history --nocapture > "$root/diffred.txt" 2>&1 )
grep '^seed=' "$root/diffred.txt" > "$root/diffred.lines"
if cmp -s "$root/diffred.lines" "$root/model/out/diff-before-seeds1-32.lines"; then v=identical; else v=different; fi
echo "name=differential_red seeds=1-32 steps=64 lines=$(wc -l < "$root/diffred.lines") lines_before=$(wc -l < "$root/model/out/diff-before-seeds1-32.lines") faulty_vs_before=$v"
diff "$root/model/out/diff-before-seeds1-32.lines" "$root/diffred.lines" | head -4 | cut -c1-260 | sed 's/^/name=differential_red_first_difference /'
echo "name=differential_red differing_lines=$(diff "$root/model/out/diff-before-seeds1-32.lines" "$root/diffred.lines" | grep -c '^>')"
