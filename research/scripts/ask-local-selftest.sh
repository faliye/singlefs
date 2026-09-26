#!/usr/bin/env bash
# ask-local.sh 的自检：判红那条分支会不会红，以及红了之后证据留没留下。
#
# 为什么要有它：判红分支此前进不去（要靠本地模型恰好吐出损坏输出才走得到），
# 于是「作废轮的原样输出被留下来」这件事没有任何东西验得了 ——
# 而 2026-09-07 实测就丢了一份。fs-design.md 硬要求 2：每条分支必须能被测试强制进入。
set -uo pipefail
cd "$(dirname "$0")/../.."
D="$(mktemp -d)"; trap 'rm -rf "${D:?}"' EXIT
# 被测脚本可换：自证这份自检会红时，指向一份改回旧写法的副本（ASK_LOCAL_SCRIPT=副本路径）。
# 副本要放在与 research/scripts 同构的位置：它按自己所在目录找两个检测器，oov-check.py 又按 ../data/en-words.txt 找词表，
# 缺了哪样，红的就是「检测器找不到」而不是被改的那一支
ASK_LOCAL="${ASK_LOCAL_SCRIPT:-research/scripts/ask-local.sh}"
# 网关的 key 在测试缝 ASK_LOCAL_FAKE_TEXT 之前就要取；自带一份假的，自证不依赖本机的 ~/code/ai-center
mkdir -p "$D/center"; printf 'AI_CENTER_KEY_VSCODE_CHAT=selftest\n' > "$D/center/.env.tenants"
export AI_CENTER_DIR="$D/center"
fail=0
say() { printf '  %s %s\n' "$1" "$2"; }

# ① 损坏正文 ⇒ 必须退 5，且必须留下 -output-void1.md
python3 - "$D/corrupt.txt" <<PYEOF
import sys
sys.argv[1]
open(sys.argv[1],"w").write('The allocation table is written once per checkpoint and read again on mount. Each unit carries a header that names the tree, the object and the offset. The scan path walks the device in fixed steps and claims every unit whose magic matches and whose header checksum verifies. A unit that fails either test is skipped and reported to the caller as a bad block. The rebuild path runs only when the index trees are gone, and it accepts a unit only after the tag over the ciphertext verifies. Replicas of one extent hold the same ciphertext, so the payload checksum of two replicas is equal whenever the two units carry the same data. The resetting resetting of the batch is fine. The allocation table is written once per checkpoint and read again on mount. Each unit carries a header that names the tree, the object and the offset. The scan path walks the device in fixed steps and claims every unit whose magic matches and whose header checksum verifies. A unit that fails either test is skipped and reported to the caller as a bad block. The rebuild path runs only when the index trees are gone, and it accepts a unit only after the tag over the ciphertext verifies. Replicas of one extent hold the same ciphertext, so the payload checksum of two replicas is equal whenever the two units carry the same data. ')
PYEOF
printf 'ask-local selftest prompt, plain english, no emphasis.\n' > "$D/case1-prompt.md"
ASK_LOCAL_FAKE_TEXT="$D/corrupt.txt" bash "$ASK_LOCAL" "$D/case1-prompt.md" >"$D/o1" 2>"$D/e1"
rc=$?
[[ $rc -eq 5 ]] || { say ✗ "损坏正文没判红（退出码 $rc，应为 5）"; fail=1; }
# 判红那一轮 stdout 必须是空的：此前正文先打到 stdout 再过闸，调用方的重定向文件里就落了一份作废输出，
# 顶着 -output-s1.md 这种合法名字（2026-09-12 实测）
[[ ! -s "$D/o1" ]] || { say ✗ "判红了 stdout 却有 $(wc -c < "$D/o1") 字节正文——作废输出会顶着合法名字落盘"; fail=1; }
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
# 调用方环境里带着同名标记（UNCHECKED=1）也不许影响判定：这一格同时证「运行前清零」那一行有用
UNCHECKED=1 ASK_LOCAL_FAKE_TEXT="$D/clean.txt" bash "$ASK_LOCAL" "$D/case2-prompt.md" >"$D/o2" 2>"$D/e2"
rc=$?
[[ $rc -eq 0 ]] || { say ✗ "干净正文被判红（退出码 $rc，应为 0）"; fail=1; }
python3 -c 'import sys; sys.exit(0 if open(sys.argv[1]).read() == open(sys.argv[2]).read().strip() + "\n" else 1)' "$D/o2" "$D/clean.txt" \
  || { say ✗ "干净正文通过了闸，stdout 却不等于正文（加末尾换行）"; fail=1; }
[[ ! -e "$D/case2-output-void1.md" ]] || { say ✗ "干净轮也留了 void 文件"; fail=1; }

# ④ 检测器没跑成（崩了退 2）或找不到 ⇒ 必须退 6、stdout 为空、正文留成作废副本：退 0 会被只看退出码的调用方当成过了闸
# 强制进入不加测试缝：把被测脚本拷进临时目录，旁边放退 2 的假检测器，或什么都不放（检测器按脚本所在目录找）
mkdir -p "$D/broken" "$D/missing"
cp "$ASK_LOCAL" "$D/broken/ask-local.sh"; cp "$ASK_LOCAL" "$D/missing/ask-local.sh"
for checker in corruption-check.py oov-check.py; do printf 'import sys\nsys.exit(2)\n' > "$D/broken/$checker"; done
case_count=2; assertion_count=5   # ①② 两个用例、5 条断言；下面每个变体加 1 个用例、3 条断言
for variant in broken missing; do
  case_count=$((case_count + 1)); assertion_count=$((assertion_count + 3))
  if [[ $variant == broken ]]; then label="崩了"; else label="找不到"; fi
  printf 'ask-local selftest prompt %s.\n' "$variant" > "$D/case-$variant-prompt.md"
  ASK_LOCAL_FAKE_TEXT="$D/clean.txt" bash "$D/$variant/ask-local.sh" "$D/case-$variant-prompt.md" >"$D/o-$variant" 2>"$D/e-$variant"
  rc=$?
  [[ $rc -eq 6 ]] || { say ✗ "检测器${label}时应退 6，实际 $rc——没验过的一份会被当成过了闸"; fail=1; }
  [[ ! -s "$D/o-$variant" ]] || { say ✗ "检测器${label}时 stdout 却有正文——没验过的输出会顶着合法名字落盘"; fail=1; }
  [[ -f "$D/case-$variant-output-void1.md" ]] || { say ✗ "检测器${label}时没留下作废副本"; fail=1; }
done

# ③ 判别力：两个用例走的是同一条路径，只有正文不同 —— 若①②同判，说明闸没在看
if [[ $fail -eq 0 ]]; then
  say ✓ "ask-local 判红分支自检通过（${case_count} 个用例、${assertion_count} 条断言：损坏留证退 5 且 stdout 为空、干净不留证退 0 且 stdout 等于正文、检测器崩了与找不到各退 6 且 stdout 为空并留证）"
  exit 0
fi
echo "  → 怎么办：看上面哪个用例失败。判红分支的落点在 ask-local.sh 的 save_void，没验过那一支在 UNCHECKED 那一段，"
echo "    测试缝是 ASK_LOCAL_FAKE_TEXT（指向一份现成正文时跳过网关）。"
exit 1
