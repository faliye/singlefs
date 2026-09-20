#!/usr/bin/env bash
# gate-stage: 模型对拍（里程碑「第二个事务」增补 3 第 2 件：随机历史快档与三个取样点，每一步拿实现的结局与只住内存的理想模型比）
# gate-covers: 模型对拍
#
# 被测的是 `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs` 那五段：快档、偏向抬 F 之后复用的取样点、
# 偏向抬 F 之后回退的取样点、逼近分配记录墙的取样点、小盘上逼近单元区墙的取样点（后两段每一步之后也跑池级 checker，已知红第 0 条那一形只记不停）。每段的报告里有一行「模型对拍 N 步：…」（`crates/singlefs-harness/src/history.rs` 的报告渲染），
# 模型模块 `crates/singlefs-harness/src/model.rs` 只用 `singlefs_format` 的常量（D13（验证路线） 已定项 5）。
# 这里在 release 下单跑那一个测试二进制，要求五段都打出「模型对拍」那一行、步数都大于 0——测试绿而模型一步没判，等于没对拍。
# 判别力：模型对拍自己会不会红，由 `crates/mutations.tsv` 第 146–178 行（门禁 59 号）证明；这个阶段只判「跑了、判过、没报对不上」。
# 样本：被判目录里没有 Cargo.toml 而有 `model-differential-cargo-output.log` 时，不跑 cargo，只判那份录好的输出
# （`.claude/gate.d/fixtures/74-model-differential.sh/` 的红绿样本走这一支）。
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2

# 这次改动没碰这道阶段判的东西就退 77（本次未跑），不退 0——`exit 0` 的跳过在汇总里与「判过了」
# 一模一样（`.claude/singlefs-ai-sop/rules/show-me-test.md`）。这是 C8（范围判定）的粗粒度前身：
# 它只摘得掉「零行代码的改动」，摘不出别的，C8 照旧欠着。
scope_reason="$(bash "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/change-touches-crates.sh" "$ROOT" crates/)"
scope_rc=$?
if [[ "$scope_rc" != 0 ]]; then
  echo "  ! 本阶段跳过（这次改动没碰它判的东西）：$scope_reason"
  echo "     → 要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；判据与前缀见 research/scripts/change-touches-crates.sh。"
  exit 77
fi

TEST_BINARY="second_transaction_supplement_three_random_history"
SECTIONS=("随机历史快档" "随机历史：偏向抬 F 之后复用的取样点" "随机历史：偏向抬 F 之后回退的取样点" "随机历史：逼近分配记录墙的取样点" "随机历史：小盘上逼近单元区墙的取样点")

log="$(mktemp)"
if [[ -f Cargo.toml && -d crates/singlefs-harness ]]; then
  if ! cargo test --release -p singlefs-harness --test "$TEST_BINARY" -- --nocapture >"$log" 2>&1; then
    tail -40 "$log"
    echo "  ✗ 随机历史的测试二进制判红（上面是 cargo test 的尾部）"
    echo "     → 怎么办：单跑看细节：cargo test --release -p singlefs-harness --test $TEST_BINARY -- --nocapture"
    echo "                签名是 ModelDisagreement 的，是实现与理想模型对某一步的结局答得不一样：先看模型那一格引的条款，再判改实现还是改模型。"
    rm -f "$log"
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
