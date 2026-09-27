#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 要 cargo：缺了就在阶段里那一步 cargo test 判红，不交给 gate.sh 预判（用户 2026-09-26 定：项目更严）；此外只要跑门禁本身就要的 bash、git、python3
# gate-stage: research 构建与单测
#
# **为什么要单开一条**：共享门禁的「构建与单测」阶段只看 `crates/`，
# 而本工程还没有 `crates/`，于是那一阶段恒报「项目尚无 Rust 代码，本阶段不适用」。
# 与此同时，**kb 里几乎每一条实测结论都由 `research/` 下那些实验二进制背书**
# （单测有多少条，看这一道成功行现数的那个数），而它们**从来没有被门禁碰过**。
#
# ⚠️ **实测踩过（2026-08-29）**：`Cargo.toml` 里留了一个指向已删源码的 `[[bin]]`，
# `cargo test` 直接报 `can't find bin`——**而门禁全绿**。
# 一个连编都编不过的证据仓库，比没有证据更糟：它看起来还在。
#
#   bash .claude/gate.d/15-research-build.sh [项目根]
set -uo pipefail
research_selftests_passed=0
# 样本：fixtures/15-research-build.sh/red 是一个 research 工作区里有一条单测红着的小仓，必须判红。没有 green：
# 下面几份研究脚本的自检按被判的仓找脚本，绿样本得把那几份脚本整份拷进样本，拷来的副本与真脚本会分叉。
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" || exit 2
R=research
[[ -f "$R/Cargo.toml" ]] || { echo "  ! 没有 $R/Cargo.toml，本阶段无对象可判"; exit 77; }
command -v cargo >/dev/null || {
  echo "  ✗ 没有 cargo，装不了就没法验 research 的证据"
  echo "     → 怎么办：装 Rust 工具链（curl https://sh.rustup.rs -sSf | sh），"
  echo "               或把已装的 cargo 放进 PATH。跳过这一步等于 research/ 的数字没人验过。"
  exit 1; }

# 这里的 cargo test 跑编译出来的代码，经 research/scripts/run-with-memory-cap.sh 放进内存上限里跑（.claude/agent-common.md「跑编译出来的代码经内存包装」那一条），
# 谁跑这一道都一样，外面不再包一层。上限取 GATE_RESEARCH_BUILD_MEMORY_MAX（派发提示给了上限的，跑这一道的 agent 设进它），
# 没设取 research/scripts/replay.sh 的 REPLAY_MEMORY_CAP 默认值 8G。峰值表的键写死：光看「cargo test --release」分不出是哪个工作区。
# 包装自己的结局（退出码 250–254）不是构建与单测的判定，单独判红、单独给出路。
MEMORY_CAP_RUNNER="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/run-with-memory-cap.sh"
RESEARCH_BUILD_MEMORY_MAX="${GATE_RESEARCH_BUILD_MEMORY_MAX:-8G}"
if [[ ! "$RESEARCH_BUILD_MEMORY_MAX" =~ ^[1-9][0-9]*[KMGT]$ ]]; then
  echo "  ✗ GATE_RESEARCH_BUILD_MEMORY_MAX 写成了「$RESEARCH_BUILD_MEMORY_MAX」，内存包装不认（或认成字节数）：这一道没跑"
  echo "     → 怎么办：写成正整数加 K / M / G / T（例 16G），单位必写；不设就用默认 8G"
  exit 1
fi
out="$(cd "$R" && RUN_WITH_MEMORY_CAP_KEY="gate 15-research-build: cargo test --release (research)" \
  bash "$MEMORY_CAP_RUNNER" "$RESEARCH_BUILD_MEMORY_MAX" cargo test --release 2>&1)"
rc=$?
if (( rc >= 250 && rc <= 254 )); then
  tail -8 <<<"$out" | sed 's/^/     /'
  echo "  ✗ research 的构建与单测没判出结果：内存包装 research/scripts/run-with-memory-cap.sh 退 $rc（上限 $RESEARCH_BUILD_MEMORY_MAX），这一次的输出不算判定"
  echo "     → 怎么办：照包装文件头「退出码」一段办：250 是这一道要的比上限多，把 GATE_RESEARCH_BUILD_MEMORY_MAX 设大再跑；"
  echo "               251、252、254 先跑 bash research/scripts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再整道重跑；不拿掉包装跑。"
  echo "               253 是超过限时：把 RUN_WITH_MEMORY_CAP_TIME_LIMIT 拿掉或放宽再跑。"
  exit 1
fi
if (( rc != 0 )); then
  echo "  ✗ research 的构建或单测没过（cargo test 退出码 $rc）"
  grep -E '^error|FAILED|panicked at' <<<"$out" | head -8 | sed 's/^/     /'
  echo "     → 怎么办：先修好——kb 里的实测结论全靠它们背书，编不过就等于那些数字今天没有来源。"
  exit 1
fi
# 条数用一个 awk 数（「test result: ok. N passed; …」的第 4 段），不再串 sed、paste、bc：串里哪一环失败，条数就静默变成「?」
b="$(grep -c '^test result: ok' <<<"$out")"
n="$(awk '/^test result: ok\./ { passed += $4 } END { print passed + 0 }' <<<"$out")"
[[ "$n" =~ ^[0-9]+$ ]] || n="没数出来（awk 输出「$n」）"
# cargo test 退 0 而一个测试批次都没有：单测这一半没有对象可判，不记通过；
# 下面几个脚本的自检照跑（它们红了照判红），跑完退 77
no_test_batches=0
if (( b == 0 )); then
  no_test_batches=1
  echo "  ! 本次无对象可判：cargo test 退 0，但输出里一个「test result: ok」批次都没有，research 的单测这一半没判"
else
  echo "  ✓ research 构建通过，$b 个测试批次、共 $n 个单测全绿"
fi

# 下面几个脚本的自检：脚本找不到就判红——它搬了家或改了名，自检就静默不跑了，而成功行一个字都不提
missing_selftest_target() {
  echo "  ✗ 找不到 $1：它的自检这一次没跑"
  echo "     → 怎么办：它搬了家或改了名，就把这一道里的路径一起改（.claude/rules/path-moves.md）；"
  echo "               它被删了，就把这一道里它那一段自检一起删。别让自检静默跳过。"
  exit 1
}

# ── 本地腿的字词损坏闸，它自己会不会红 ──
# 三方论证的本地腿靠 `ask-local.sh` 里那道闸挡损坏输出，而那道闸此前没人验过。
# 实测（2026-09-03）：一份含 `inaccessibleisabled` 的输出被判绿，差点当成证据用掉。
OOV="$R/scripts/oov-check.py"
if [[ -f "$OOV" ]]; then
  if ! out="$(python3 "$OOV" --selftest 2>&1)"; then
    echo "$out" | sed 's/^/  /'
    echo "     → 怎么办：本地腿的损坏闸判错了样本，修 splice_of 的规则再跑。"
    echo "                闸不准 ⇒ 三方论证里那一腿的输出可信度归零。"
    exit 1
  fi
  echo "$out" | sed 's/^  /  /'
  research_selftests_passed=$((research_selftests_passed + 1))
else
  missing_selftest_target "$OOV"
fi

# ── 并发会话的两件暂存工具，它们自己会不会红 ──
# stage-mine.py 挑错块，别的会话的半成品就被卷进这一次提交；check-staged.sh 把工作区原样拿去跑，别人的红就算到自己头上。
STAGE_MINE="$R/scripts/stage-mine.py"
if [[ -f "$STAGE_MINE" ]]; then
  if ! out="$(python3 "$STAGE_MINE" --selftest 2>&1)"; then
    echo "$out" | sed 's/^/  /'
    echo "     → 怎么办：stage-mine.py 挑块挑错了样本，修切块或下滑的规则再跑。"
    exit 1
  fi
  echo "  ✓ stage-mine.py $(tail -1 <<<"$out")"
  research_selftests_passed=$((research_selftests_passed + 1))
else
  missing_selftest_target "$STAGE_MINE"
fi
CHECK_STAGED="$R/scripts/check-staged.sh"
if [[ -f "$CHECK_STAGED" ]]; then
  if ! out="$(bash "$CHECK_STAGED" --selftest 2>&1)"; then
    echo "$out" | sed 's/^/  /'
    echo "     → 怎么办：check-staged.sh 把没暂存的改动算进来了、或暂存了的没判红，修 run_isolated 再跑。"
    exit 1
  fi
  echo "  ✓ check-staged.sh $(tail -1 <<<"$out")"
  research_selftests_passed=$((research_selftests_passed + 1))
else
  missing_selftest_target "$CHECK_STAGED"
fi
RELABEL="$R/scripts/relabel-item.py"
if [[ -f "$RELABEL" ]]; then
  if ! out="$(python3 "$RELABEL" --selftest 2>&1)"; then
    echo "$out" | sed 's/^/  /'
    echo "     → 怎么办：relabel-item.py 改写之后 22 号的库复判不过，修它的归属或改写再跑。"
    exit 1
  fi
  echo "  ✓ relabel-item.py $(tail -1 <<<"$out")"
  research_selftests_passed=$((research_selftests_passed + 1))
else
  missing_selftest_target "$RELABEL"
fi
CLAIM="$R/scripts/claim-experiment.sh"
if [[ -f "$CLAIM" ]]; then
  if ! out="$(bash "$CLAIM" --selftest 2>&1)"; then
    echo "$out" | sed 's/^/  /'
    echo "     → 怎么办：claim-experiment.sh 放过了已用的号、或同一个号占了两次，修 used_numbers / claim 再跑。"
    exit 1
  fi
  echo "  ✓ claim-experiment.sh $(tail -1 <<<"$out")"
  research_selftests_passed=$((research_selftests_passed + 1))
else
  missing_selftest_target "$CLAIM"
fi
# 单测那一半没有对象（见上面「本次无对象可判」那一行）：自检都绿也不记通过
if (( no_test_batches )); then exit 77; fi
echo "  ✓ research 构建与单测全绿，研究脚本的自证过了 $research_selftests_passed 份"
