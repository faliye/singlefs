#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 要 cargo：缺了就在阶段里那一步 cargo test 判红，不交给 gate.sh 预判（用户 2026-09-26 定：项目更严）；此外只要跑门禁本身就要的 bash、git、python3
# gate-stage: 模型对拍（里程碑「覆盖写、释放、回退与复用」增补 3 第 2 件：随机历史快档与三个取样点，每一步拿实现的结局与只住内存的理想模型比）
# gate-covers: 模型对拍
#
# 被测的是 `crates/singlefs-harness/tests/random_histories.rs` 里的那几段：快档与几个偏向某种历史的取样点，段名与段数以下面的 SECTIONS 为准（落到墙边的那几段每一步之后也跑池级 checker，已知红第 0 条那一形只记不停）。每段的报告里有一行「模型对拍 N 步：…」（`crates/singlefs-harness/src/history.rs` 的报告渲染），
# 模型模块 `crates/singlefs-harness/src/model.rs` 只用 `singlefs_format` 的常量（D13（验证路线） 已定项 5）。
# 这里在 release 下单跑那一个测试二进制，要求 SECTIONS 里每一段都打出「模型对拍」那一行、步数都大于 0——测试绿而模型一步没判，等于没对拍。
# 判别力：模型对拍自己会不会红，由 `crates/mutations.tsv` 里点名这个测试二进制的那几条变异（门禁 59 号）证明；这个阶段只判「跑了、判过、没报对不上」。
# 样本：被判目录里没有 Cargo.toml 而有 `model-differential-cargo-output.log` 时，不跑 cargo，只判那份录好的输出
# （`.claude/gate.d/fixtures/74-model-differential.sh/` 的红绿样本走这一支）。
# gate-overlap:copy-kept 59-crates-mutation-replay.sh 开头这几行是重阶段都照写的固定写法：preflight 那两行规范要求逐字写在脚本里，要不要跑的判定已经抽成 research/scripts/stage-run-or-skip.sh，剩下的是 cd 进仓与说明为什么先问复用，再抽一层只会多一个要 source 的文件
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2

# 这次改动没碰这道阶段判的东西就退 77（本次未跑），不退 0——`exit 0` 的跳过在汇总里与「判过了」
# 一模一样（`.claude/singlefs-ai-sop/rules/show-me-test.md`）。这是 C8（范围判定）的粗粒度前身：
# 它只摘得掉「零行代码的改动」，摘不出别的，C8 照旧欠着。
# 先问能不能复用上一次整轮全绿的判定：这一道读的那几条路径（`.claude/gate.d/stage-inputs.tsv`）
# 在 `refs/sop/staged-green` 那棵树与这一次的暂存树之间变没变。为什么用树、为什么只一条 ref、
# 为什么不看工作区，写在 research/scripts/stage-must-run.sh 的文件头。
source "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/stage-run-or-skip.sh" || { echo "  ✗ source 不进 research/scripts/stage-run-or-skip.sh，判不出这一道要不要跑"; echo "     → 怎么办：它随仓走，被删了或挪了就从 git 找回来；找回之前这一道按红记。"; exit 1; }
stage_run_or_skip "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/stage-must-run.sh" "复用上一次整轮全绿的判定" \
  "要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；这一道读哪几条路径见 .claude/gate.d/stage-inputs.tsv。" -- "$ROOT" "$(basename "$0")"
stage_run_or_skip "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/change-touches-crates.sh" "这次改动没碰它判的东西" \
  "要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；判据与前缀见 research/scripts/change-touches-crates.sh。" -- "$ROOT" crates/

TEST_BINARY="random_histories"
# 这里的 cargo test 跑编译出来的代码，经 research/scripts/run-with-memory-cap.sh 放进内存上限里跑（.claude/agent-common.md「跑编译出来的代码经内存包装」那一条），
# 谁跑这一道都一样，外面不再包一层。上限取 GATE_MODEL_DIFFERENTIAL_MEMORY_MAX（派发提示给了上限的，跑这一道的 agent 设进它），
# 没设取 research/scripts/replay.sh 的 REPLAY_MEMORY_CAP 默认值 8G。包装自己的结局（退出码 250–254）不是测试的判定，单独判红、单独给出路。
MEMORY_CAP_RUNNER="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/run-with-memory-cap.sh"
MODEL_DIFFERENTIAL_MEMORY_MAX="${GATE_MODEL_DIFFERENTIAL_MEMORY_MAX:-8G}"
SECTIONS=("随机历史快档" "随机历史：偏向抬 F 之后复用的取样点" "随机历史：偏向抬 F 之后回退的取样点" "随机历史：越过原分配记录墙的取样点" "随机历史：小盘上逼近单元区墙的取样点" "随机历史：小盘上逼近单元区墙的取样点（空间准入判着）")

if [[ ! "$MODEL_DIFFERENTIAL_MEMORY_MAX" =~ ^[1-9][0-9]*[KMGT]$ ]]; then
  echo "  ✗ GATE_MODEL_DIFFERENTIAL_MEMORY_MAX 写成了「$MODEL_DIFFERENTIAL_MEMORY_MAX」，内存包装不认（或认成字节数）：这一道没跑"
  echo "     → 怎么办：写成正整数加 K / M / G / T（例 16G），单位必写；不设就用默认 8G"
  exit 1
fi
log="$(mktemp)"
trap 'rm -f "$log"' EXIT
if [[ -f Cargo.toml && -d crates/singlefs-harness ]]; then
  if bash "$MEMORY_CAP_RUNNER" "$MODEL_DIFFERENTIAL_MEMORY_MAX" cargo test --release -p singlefs-harness --test "$TEST_BINARY" -- --nocapture >"$log" 2>&1; then
    cargo_exit=0
  else
    cargo_exit=$?
  fi
  if (( cargo_exit != 0 )); then
    tail -40 "$log"
    rm -f "$log"
    case "$cargo_exit" in
      250|251|252|253|254)
        echo "  ✗ 随机历史的测试二进制没判出结果：内存包装 research/scripts/run-with-memory-cap.sh 退 $cargo_exit（上限 $MODEL_DIFFERENTIAL_MEMORY_MAX），这一次的输出不算判定"
        echo "     → 怎么办：照包装文件头「退出码」一段办：250 是这一道要的比上限多，把 GATE_MODEL_DIFFERENTIAL_MEMORY_MAX 设大再跑；"
        echo "                251、252、254 先跑 bash research/scripts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再整道重跑；不拿掉包装跑。"
        echo "                253 是超过限时：把 RUN_WITH_MEMORY_CAP_TIME_LIMIT 拿掉或放宽再跑。"
        ;;
      *)
        echo "  ✗ 随机历史的测试二进制判红（上面是 cargo test 的尾部）"
        echo "     → 怎么办：单跑看细节（经内存包装，上限同这一道）："
        echo "                bash research/scripts/run-with-memory-cap.sh $MODEL_DIFFERENTIAL_MEMORY_MAX cargo test --release -p singlefs-harness --test $TEST_BINARY -- --nocapture"
        echo "                签名是 ModelDisagreement 的，是实现与理想模型对某一步的结局答得不一样：先看模型那一格引的条款，再判改实现还是改模型。"
        ;;
    esac
    exit 1
  fi
elif [[ -f model-differential-cargo-output.log ]]; then
  cp model-differential-cargo-output.log "$log"
else
  rm -f "$log"
  echo "  ! 没有 crates/singlefs-harness，本阶段跳过（增补 3 第 2 件之前没有模型）"
  exit 77
fi

missing=()
zero=()
reported=()
for section in "${SECTIONS[@]}"; do
  # 一段的报告从它的标题行起、到下一个「── … ──」标题行为止；不按固定行数取，报告长了照样找得到
  line="$(awk -v header="── ${section} ──" '$0 == header { inside = 1; next } inside && /^── .* ──$/ { exit } inside' "$log" | grep -m1 '模型对拍 [0-9]* 步' || true)"
  if [[ -z "$line" ]]; then
    missing+=("$section")
    continue
  fi
  steps="$(sed -n 's/.*模型对拍 \([0-9]*\) 步.*/\1/p' <<<"$line")"
  if [[ -z "$steps" || "$steps" == 0 ]]; then
    zero+=("$section")
    continue
  fi
  reported+=("${section}：$(sed 's/^[[:space:]]*//' <<<"$line")")
done
rm -f "$log"

# 两类都先列完再退出：一个红样本同时带两类，就能证明两支都判得出（代码三方 m2-supp3-item2-code-r1 正推腿报的样本缺口）
failed=0
if (( ${#missing[@]} > 0 )); then
  echo "  ✗ 测试跑过了，这几段却没有「模型对拍 N 步」那一行：$(printf '「%s」' "${missing[@]}")"
  echo "     → 怎么办：history.rs 的报告渲染要在每段报告里打「模型对拍 N 步：…」；测试文件里每一段都要经 print_uncaptured 把报告写进标准输出。"
  echo "                行没了多半是执行器绕开了 judge_by_model，那一段等于没对拍。"
  failed=1
fi
if (( ${#zero[@]} > 0 )); then
  echo "  ✗ 这几段模型一步都没判：$(printf '「%s」' "${zero[@]}")"
  echo "     → 怎么办：测试绿而模型没判过任何一步，等于没对拍（show-me-test.md「扫到 0 项也不是通过」）。"
  echo "                看 history.rs 的执行器每一步调入口之前有没有先问模型（judge_by_model），前提不满足的步不算。"
  failed=1
fi
(( failed == 0 )) || exit 1
echo "  ✓ 模型对拍每一段都判过、实现与模型没有对不上的（查了 ${#SECTIONS[@]} 段）："
for entry in "${reported[@]}"; do
  echo "      $entry"
done
