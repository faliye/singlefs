#!/usr/bin/env bash
# 增补 3 第 3 件代码轮第二轮 云端攻方（Opus）K1／K3 的复跑脚本。
# 全部在仓的副本上跑，不动工作区。用法：bash reproduce.sh <副本目录>
# 副本这么来：rsync -a --exclude target --exclude .git <仓> <副本目录>/
set -euo pipefail
copy="${1:?用法：reproduce.sh <副本目录>}"
here="$(cd "$(dirname "$0")" && pwd)"

cp "$here/probe1-fast-tier-subset-coverage.rs" "$copy/crates/singlefs-harness/tests/zz_opus_r2_probe.rs"
cp "$here/probe2-deeper-enumeration.rs" "$copy/crates/singlefs-harness/tests/zz_opus_r2_probe2.rs"
cd "$copy"

echo "── 1. 干净构建：快档同参的段内子集覆盖与记录核对器计数（报告第二节 2.1、第三节 3.1）"
nice -n 19 cargo test -q -p singlefs-harness --test zz_opus_r2_probe -- --nocapture

echo "── 2. 干净构建：整个崩溃注入测试二进制（报告第三节 3.1 的四段报告）"
nice -n 19 cargo test -q -p singlefs-harness --test second_transaction_supplement_three_crash_injection -- --nocapture

echo "── 3. 变异 A：计数照增、判据丢掉（报告第三节 3.2）"
python3 - <<'PY'
p = "crates/singlefs-harness/src/crash_injection.rs"
s = open(p).read()
old = """        if violations.is_empty() && disagreement.is_none() && record_check == RecordCheck::default()
        {
            continue;
        }"""
new = """        if violations.is_empty() && disagreement.is_none()
        {
            continue;
        }"""
assert s.count(old) == 1
open(p + ".orig", "w").write(s)
open(p, "w").write(s.replace(old, new))
PY
nice -n 19 cargo test -q -p singlefs-harness --test second_transaction_supplement_three_crash_injection || true
nice -n 19 cargo test -q -p singlefs-harness --lib crash_injection || true
cp crates/singlefs-harness/src/crash_injection.rs.orig crates/singlefs-harness/src/crash_injection.rs
rm crates/singlefs-harness/src/crash_injection.rs.orig
touch crates/singlefs-harness/src/crash_injection.rs  # 还原之后必须更新 mtime，否则 cargo 不重编

echo "── 4. 变异 67（crates/mutations.tsv 第 67 行原样）：记录核对器判得出红（报告第三节 3.4）"
python3 - <<'PY'
rows = [l.rstrip("\n").split("\t") for l in open("crates/mutations.tsv") if not l.startswith("#")]
r = [x for x in rows if x[0].startswith("步 3：零单元发布在记录与根之间少一道屏障")][0]
f, old, new = r[1], r[2].replace("\\n", "\n"), r[3].replace("\\n", "\n")
s = open(f).read()
assert s.count(old) == 1
open(f + ".orig", "w").write(s)
open(f, "w").write(s.replace(old, new))
PY
nice -n 19 cargo test -q -p singlefs-harness --test second_transaction_supplement_three_crash_injection -- --nocapture || true
cp crates/singlefs-core/src/transaction.rs.orig crates/singlefs-core/src/transaction.rs
rm crates/singlefs-core/src/transaction.rs.orig
touch crates/singlefs-core/src/transaction.rs  # 同上：mv 会保住旧 mtime，cargo 就不重编了

echo "── 5. 变异 C：带单元的发布少掉「单元写 → 屏障 → journal 记录」那一道（报告第三节 3.6）"
python3 - <<'PY'
p = "crates/singlefs-core/src/transaction.rs"
s = open(p).read()
old = """        }
        writer.perform(CommitStep::Barrier)?;
        writer.perform(CommitStep::WriteJournalRecordToEveryDevice {"""
new = """        }
        writer.perform(CommitStep::WriteJournalRecordToEveryDevice {"""
assert s.count(old) == 1
open(p + ".orig", "w").write(s)
open(p, "w").write(s.replace(old, new))
PY
nice -n 19 cargo test -q -p singlefs-harness --test second_transaction_supplement_three_crash_injection -- --nocapture || true
nice -n 19 cargo test -q -p singlefs-harness --test zz_opus_r2_probe2 -- --nocapture || true
cp crates/singlefs-core/src/transaction.rs.orig crates/singlefs-core/src/transaction.rs
rm crates/singlefs-core/src/transaction.rs.orig
touch crates/singlefs-core/src/transaction.rs  # 同上：mv 会保住旧 mtime，cargo 就不重编了
echo "── 跑完，副本已还原到干净状态（两处都已写回并 touch 过）"
