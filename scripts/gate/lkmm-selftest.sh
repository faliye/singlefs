#!/usr/bin/env bash
# admission: always 判的是此刻的 .claude/scripts/lkmm.sh、门禁 checker-tier-lkmm 的样本与 scripts/gate/lib_reuse_checks.py，上一次的结论不替这一次作保
# run-condition: command python3 bash
# gate-similar: checker-tier-lkmm.sh 那一道的样本档（litmus-key-samples-*）判门禁接 lkmm.sh 的两趟复用；这一份是 lkmm.sh 自己的自证（复用三问四趟、复用哨兵），
#   lkmm.sh 本身带 --selftest 会被重型测试闸按 herd7 拦（.claude/hooks/lib_heavy_tests.py 按名字认 lkmm.sh），另放一份可以单独起的
# .claude/scripts/lkmm.sh 的自证：复用三问（格名 reuse-three-questions）与复用哨兵（reuse-sentinel），判法与规范输出在 scripts/gate/lib_reuse_checks.py 文件头「一」「二」。
#
#   bash scripts/gate/lkmm-selftest.sh --selftest
#
# 现场：临时目录里照门禁 checker-tier-lkmm 的样本 litmus-key-samples-anchor-body-changed 的 setup.sh 造一对 litmus（publish 与它去掉屏障的对照）、
#   锚点 crates/demo/src/lib.rs :: publish、写出文件名的测试、假 herd7（打固定版本串、照 litmus 里 fake-observation 交判定）与只有模型文件的假内核树；
#   判定库放这一趟的临时目录。不起真 herd7、不编不跑。
# 格：
#   reuse-three-questions  头一趟（空库）、同一份输入第二趟、改没人引的 fn unrelated 的函数体（闭包外）、改锚点 publish 的函数体（闭包内：只有 binding:publish.litmus 现判）
#                          各跑一趟，三条判定（binding:publish.litmus、herd7:publish.litmus、herd7:publish-nofence.litmus）照三问判、打规范输出
#                          「REUSE_SELFTEST script=.claude/scripts/lkmm.sh …」；这几趟哨兵数设 1000（键相同的全当哨兵、不抽强制现判）
#   reuse-sentinel         同一个种子（5）、哨兵 1 条再跑一趟：herd7 那一份抽中一条强制现判、判定与库里相同、整趟绿；
#                          开着 GATE_LKMM_BREAK=reuse-lookup-skipped（一律不读库）跑，复用哨兵判红「复用失灵」
# 弄坏开关（逐个打开重跑这一份，核红在点名的那一格）：
#   GATE_LKMM_BREAK=reuse-lookup-skipped                       lkmm.sh 一律不读库：reuse-three-questions 判错（reuse-sentinel 照常那一趟也判红）
#   REUSE_CHECKS_BREAK=rejudged-sentinel-passes（共用库的开关）  哨兵被现判不红：reuse-sentinel 判错
# 退出码：0 都判对；1 有格判错或开关没红在点名那一格；2 用法错。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

SELFTEST_DIRECTORY="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPOSITORY="$(cd "$SELFTEST_DIRECTORY/../.." && pwd)"
LKMM="$REPOSITORY/.claude/scripts/lkmm.sh"
REUSE_CHECKS="$REPOSITORY/scripts/gate/lib_reuse_checks.py"
SAMPLE_SETUP="$REPOSITORY/.claude/gate.d/fixtures/checker-tier-lkmm.sh/litmus-key-samples-anchor-body-changed/setup.sh"

if [[ "${1:-}" != --selftest || $# -ne 1 ]]; then
  echo "  ✗ 用法：bash scripts/gate/lkmm-selftest.sh --selftest"
  echo "     → 怎么办：这一份只跑 .claude/scripts/lkmm.sh 的复用自证，不带别的参数；真判 litmus 走门禁 checker-tier-lkmm"
  exit 2
fi

work="$(mktemp -d "${TMPDIR:-/tmp}/lkmm-selftest.XXXXXX")" || { echo "  ✗ 建不了临时目录"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
trap 'rm -rf "${work:?}"' EXIT
root="$work/root"; mkdir -p "$root" "$work/bin"
if ! (cd "$root" && bash "$SAMPLE_SETUP") > "$work/setup.log" 2>&1; then
  echo "  ✗ 照样本 $SAMPLE_SETUP 造现场没成：$(tail -n 3 "$work/setup.log" | tr '\n' ' ')"
  echo "     → 怎么办：那一份样本搬了家或改坏了，照门禁 checker-tier-lkmm 的样本改 SAMPLE_SETUP 这一行"
  exit 1
fi
printf '#!/usr/bin/env bash\nexec bash %q "$@"\n' "$root/.lkmm-fake-herd7/herd7" > "$work/bin/herd7"
chmod +x "$work/bin/herd7"

wrong=(); cases=0
cell_wrong() { wrong+=("$1"); echo "    $1 这一格判错：$2"; }   # gate-lint:detail
# run_round <名> [环境…]：在现场上跑一趟 lkmm.sh（假 herd7、这一趟的判定库），输出落 $work/<名>.log、退出码落 $work/<名>.exit
run_round() {
  local name="$1"; shift
  env PATH="$work/bin:$PATH" SINGLEFS_KERNEL_TREE="$root/.lkmm-fake-herd7/linux" SINGLEFS_NO_FETCH=1 "$@" \
    bash "$LKMM" "$root" --verdict-library "$work/library" > "$work/$name.log" 2>&1
  echo "$?" > "$work/$name.exit"
}
replace_in() { # replace_in <文件> <原文> <新文>：原文恰好一处才换
  python3 - "$@" <<'PY'
import sys
path, old, new = sys.argv[1:4]
text = open(path, encoding="utf-8").read()
if text.count(old) != 1:
    sys.exit(f"{path} 里「{old}」命中 {text.count(old)} 处")
open(path, "w", encoding="utf-8").write(text.replace(old, new))
PY
}

# ── reuse-three-questions ──
cases=$((cases + 1))
many=(SINGLEFS_REUSE_SENTINEL_COUNT=1000)
run_round first "${many[@]}"
run_round second "${many[@]}"
replace_in "$root/crates/demo/src/lib.rs" 'pub fn unrelated() {}' $'pub fn unrelated() {\n    let _ = 2;\n}' || cell_wrong reuse-three-questions "改不了 unrelated"
run_round unrelated "${many[@]}"
replace_in "$root/crates/demo/src/lib.rs" 'pub fn publish() {}' $'pub fn publish() {\n    let _ = 1;\n}' || cell_wrong reuse-three-questions "改不了 publish"
run_round unit "${many[@]}"
exits="$(cat "$work/first.exit" "$work/second.exit" "$work/unrelated.exit" "$work/unit.exit" | paste -sd' ')"
if ! python3 "$REUSE_CHECKS" three-questions --script .claude/scripts/lkmm.sh --from-output --first "$work/first.log" --second "$work/second.log" \
       --unrelated "$work/unrelated.log" --unit-change "$work/unit.log" --affected binding:publish.litmus > "$work/three.log" 2>&1 || [[ "$exits" != "0 0 0 0" ]]; then
  cell_wrong reuse-three-questions "四趟退 $exits；$(grep -v '^REUSE_SELFTEST' "$work/three.log" | tr '\n' ' ')；末一趟：$(grep '· ' "$work/unit.log" | tr '\n' ' ')"
fi
grep '^REUSE_SELFTEST' "$work/three.log"

# ── reuse-sentinel ──
cases=$((cases + 1))
one=(SINGLEFS_REUSE_SENTINEL_COUNT=1 SINGLEFS_REUSE_SENTINEL_SEED=5)
run_round sentinel-green "${one[@]}"
run_round sentinel-skipped "${one[@]}" GATE_LKMM_BREAK="${GATE_LKMM_BREAK:+$GATE_LKMM_BREAK,}reuse-lookup-skipped"
if [[ "$(cat "$work/sentinel-green.exit")" != 0 ]] || ! grep -q '复用哨兵（herd7）：种子 5' "$work/sentinel-green.log" \
   || ! grep -q '，强制现判（复用哨兵抽中' "$work/sentinel-green.log" \
   || [[ "$(cat "$work/sentinel-skipped.exit")" != 1 ]] || ! grep -q '复用失灵：' "$work/sentinel-skipped.log"; then
  cell_wrong reuse-sentinel "照常那一趟退 $(cat "$work/sentinel-green.exit")：$(grep -E '复用哨兵|· ' "$work/sentinel-green.log" | tr '\n' ' ')；不读库那一趟退 $(cat "$work/sentinel-skipped.exit")：$(grep -E '复用失灵|复用哨兵' "$work/sentinel-skipped.log" | tr '\n' ' ')"
fi

# ── 弄坏开关：逐个打开重跑这一份，核红在点名的那一格（开着开关的那一趟不再重跑开关）──
breaks_missed=(); breaks_proven=0
if ((${#wrong[@]} == 0)) && [[ -z "${GATE_LKMM_BREAK:-}" && -z "${REUSE_CHECKS_BREAK:-}" ]]; then
  for pair in "GATE_LKMM_BREAK=reuse-lookup-skipped reuse-three-questions" "REUSE_CHECKS_BREAK=rejudged-sentinel-passes reuse-sentinel"; do
    switch="${pair%% *}"; cell="${pair##* }"
    if env "$switch" bash "${BASH_SOURCE[0]}" --selftest > "$work/break.log" 2>&1 || ! grep -q "    $cell 这一格判错" "$work/break.log"; then
      breaks_missed+=("$switch（要红在 $cell）")
    else
      breaks_proven=$((breaks_proven + 1))
    fi
  done
fi
if ((${#wrong[@]})); then
  echo "  ✗ lkmm.sh 的复用自证 $cases 格里 ${#wrong[@]} 格判错：${wrong[*]}"
  echo "     → 怎么办：照那一格的报错看 .claude/scripts/lkmm.sh 的 judge_lookup、sentinel_plan 与 lookup_item；判据在 scripts/gate/lib_reuse_checks.py 文件头"
  exit 1
fi
if ((${#breaks_missed[@]})); then
  echo "  ✗ 弄坏开关打开之后没红在点名那一格：${breaks_missed[*]}"
  echo "     → 怎么办：那个开关管的那一段没有这一份盯着：补一格会被它弄错的自证，或修开关"
  exit 1
fi
if [[ -n "${GATE_LKMM_BREAK:-}${REUSE_CHECKS_BREAK:-}" ]]; then
  echo "  ✓ lkmm.sh 的复用自证 $cases 格都判对（开着弄坏开关，没重跑开关）"
else
  echo "  ✓ lkmm.sh 的复用自证 $cases 格都判对；弄坏开关 $breaks_proven 个，每个都红在点名那一格"
fi
exit 0
