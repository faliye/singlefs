#!/usr/bin/env bash
# ask-local.sh 的自检：判红那条分支会不会红，以及红了之后证据留没留下。
#
# 为什么要有它：判红分支此前进不去（要靠本地模型恰好吐出损坏输出才走得到），
# 于是「作废轮的原样输出被留下来」这件事没有任何东西验得了 ——
# 而 2026-09-07 实测就丢了一份。fs-design.md 硬要求 2：每条分支必须能被测试强制进入。
set -uo pipefail
cd "$(dirname "$0")/../.."
D="$(mktemp -d)"; trap 'rm -rf "${D:?}"' EXIT
fail=0
say() { printf '  %s %s\n' "$1" "$2"; }

# ① 损坏正文 ⇒ 必须退 5，且必须留下 -output-void1.md
python3 - "$D/corrupt.txt" <<PYEOF
import sys
sys.argv[1]
open(sys.argv[1],"w").write('The allocation table is written once per checkpoint and read again on mount. Each unit carries a header that names the tree, the object and the offset. The scan path walks the device in fixed steps and claims every unit whose magic matches and whose header checksum verifies. A unit that fails either test is skipped and reported to the caller as a bad block. The rebuild path runs only when the index trees are gone, and it accepts a unit only after the tag over the ciphertext verifies. Replicas of one extent hold the same ciphertext, so the payload checksum of two replicas is equal whenever the two units carry the same data. The resetting resetting of the batch is fine. The allocation table is written once per checkpoint and read again on mount. Each unit carries a header that names the tree, the object and the offset. The scan path walks the device in fixed steps and claims every unit whose magic matches and whose header checksum verifies. A unit that fails either test is skipped and reported to the caller as a bad block. The rebuild path runs only when the index trees are gone, and it accepts a unit only after the tag over the ciphertext verifies. Replicas of one extent hold the same ciphertext, so the payload checksum of two replicas is equal whenever the two units carry the same data. ')
PYEOF
printf 'ask-local selftest prompt, plain english, no emphasis.\n' > "$D/case1-prompt.md"
ASK_LOCAL_FAKE_TEXT="$D/corrupt.txt" bash research/scripts/ask-local.sh "$D/case1-prompt.md" >/dev/null 2>"$D/e1"
rc=$?
[[ $rc -eq 5 ]] || { say ✗ "损坏正文没判红（退出码 $rc，应为 5）"; fail=1; }
if [[ -f "$D/case1-output-void1.md" ]]; then
  grep -q 'resetting resetting' "$D/case1-output-void1.md" \
    || { say ✗ "留存文件在，但内容不是那份被判红的正文"; fail=1; }
else
  say ✗ "判红了却没留下 case1-output-void1.md —— 作废轮的证据又没了"; fail=1
fi

# ② 干净正文 ⇒ 必须退 0，且不许留 void 文件
python3 - "$D/clean.txt" <<PYEOF
import sys
open(sys.argv[1],"w").write('The allocation table is written once per checkpoint and read again on mount. Each unit carries a header that names the tree, the object and the offset. The scan path walks the device in fixed steps and claims every unit whose magic matches and whose header checksum verifies. A unit that fails either test is skipped and reported to the caller as a bad block. The rebuild path runs only when the index trees are gone, and it accepts a unit only after the tag over the ciphertext verifies. Replicas of one extent hold the same ciphertext, so the payload checksum of two replicas is equal whenever the two units carry the same data. The allocation table is written once per checkpoint and read again on mount. Each unit carries a header that names the tree, the object and the offset. The scan path walks the device in fixed steps and claims every unit whose magic matches and whose header checksum verifies. A unit that fails either test is skipped and reported to the caller as a bad block. The rebuild path runs only when the index trees are gone, and it accepts a unit only after the tag over the ciphertext verifies. Replicas of one extent hold the same ciphertext, so the payload checksum of two replicas is equal whenever the two units carry the same data. ')
PYEOF
printf 'ask-local selftest prompt two.\n' > "$D/case2-prompt.md"
ASK_LOCAL_FAKE_TEXT="$D/clean.txt" bash research/scripts/ask-local.sh "$D/case2-prompt.md" >/dev/null 2>"$D/e2"
rc=$?
[[ $rc -eq 0 ]] || { say ✗ "干净正文被判红（退出码 $rc，应为 0）"; fail=1; }
[[ ! -e "$D/case2-output-void1.md" ]] || { say ✗ "干净轮也留了 void 文件"; fail=1; }

# ③ 判别力：两个用例走的是同一条路径，只有正文不同 —— 若①②同判，说明闸没在看
if [[ $fail -eq 0 ]]; then
  say ✓ "ask-local 判红分支自检通过（2 个用例：损坏留证退 5、干净不留证退 0）"
  exit 0
fi
echo "  → 怎么办：看上面哪个用例失败。判红分支的落点在 ask-local.sh 的 save_void，"
echo "    测试缝是 ASK_LOCAL_FAKE_TEXT（指向一份现成正文时跳过网关）。"
exit 1
