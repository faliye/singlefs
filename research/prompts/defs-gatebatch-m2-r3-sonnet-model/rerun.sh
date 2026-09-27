#!/usr/bin/env bash
# L2 复核用：重建 r2 判 K4 那一刻的 admission.py（对今天的文件依次 patch -R 掉 G2、G1 两份 diff 的 admission.py 部分），
# 再用 r2 攻方腿自己的探针 probe_k4_cost.py（未改一字）分别跑在：①今天的 admission.py ②重建出的 r2 基线 上。
# 复跑：bash rerun.sh <仓根>
set -euo pipefail
ROOT="${1:?用法：rerun.sh <仓根>}"
WORK="$(mktemp -d /tmp/claude-1000/defs-gatebatch-m2-r3-sonnet/reconstruct-XXXXXX)"
echo "== 工作目录：$WORK =="

extract_admission_only() { # extract_admission_only <diff文件> <输出文件>
  python3 - "$1" "$2" <<'PY'
import sys
diff_path, out_path = sys.argv[1], sys.argv[2]
with open(diff_path) as f:
    lines = f.readlines()
out, keep = [], False
for line in lines:
    if line.startswith("--- a/") or line.startswith("+++ b/"):
        keep = "research/scripts/admission.py" in line
    if keep:
        out.append(line)
with open(out_path, "w") as f:
    f.writelines(out)
PY
}

extract_admission_only "$ROOT/research/prompts/defs-gatebatch-m2-r2-fixes.diff" "$WORK/g1-admission-only.diff"
extract_admission_only "$ROOT/research/prompts/defs-gatebatch-m2-g2-changes.diff" "$WORK/g2-admission-only.diff"
echo "G1 admission.py hunk 数：$(grep -c '^@@ ' "$WORK/g1-admission-only.diff")"
echo "G2 admission.py hunk 数：$(grep -c '^@@ ' "$WORK/g2-admission-only.diff")"

mkdir -p "$WORK/step1/research/scripts" "$WORK/step2/research/scripts"
cp "$ROOT/research/scripts/admission.py" "$WORK/today-admission.py"
cp "$WORK/today-admission.py" "$WORK/step1/research/scripts/admission.py"
( cd "$WORK/step1" && patch -R -p1 --fuzz=0 < "$WORK/g2-admission-only.diff" )
cp "$WORK/step1/research/scripts/admission.py" "$WORK/step2/research/scripts/admission.py"
( cd "$WORK/step2" && patch -R -p1 --fuzz=0 < "$WORK/g1-admission-only.diff" )
cp "$WORK/step2/research/scripts/admission.py" "$WORK/baseline-r2-admission.py"

echo "今天 admission.py 行数：$(wc -l < "$WORK/today-admission.py")"
echo "重建出的 r2 基线行数：$(wc -l < "$WORK/baseline-r2-admission.py")（r2 判决原句说 3194 行）"

echo
echo "== ① 用 r2 的 probe_k4_cost.py 原样跑在今天的 admission.py 上 =="
K1_ADMISSION="$WORK/today-admission.py" python3 "$ROOT/research/prompts/defs-gatebatch-m2-r2-opus-model/probe_k4_cost.py" | grep '^MEASURE'

echo
echo "== ② 同一份探针跑在重建出的 r2 基线上（应与 r2 判决的 365/3194/11% 对照） =="
K1_ADMISSION="$WORK/baseline-r2-admission.py" python3 "$ROOT/research/prompts/defs-gatebatch-m2-r2-opus-model/probe_k4_cost.py" | grep '^MEASURE'

echo
echo "== ③ 今天真正出货的 D2 闭包算法（crash_case_judging_digest_text），跑在今天的 admission.py 上 =="
python3 - "$ROOT" <<'PY'
import sys
sys.path.insert(0, sys.argv[1] + "/research/scripts")
import admission
with open(sys.argv[1] + "/research/scripts/admission.py", encoding="utf-8") as f:
    source = f.read()
digest_bytes, wanted_count = admission.crash_case_judging_digest_text(source)
digest_lines = digest_bytes.decode("utf-8", "surrogateescape").count("\n")
total_lines = source.count("\n")
print(f"MEASURE 今天出货的 D2 闭包：{wanted_count} 个定义、{digest_lines} 行（{digest_lines*100//total_lines}%，总 {total_lines} 行）")
PY
