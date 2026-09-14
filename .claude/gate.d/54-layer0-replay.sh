#!/usr/bin/env bash
# gate-stage: 层 0 崩溃点重放（第一个事务的全部崩溃状态在 release 下逐个跑恢复，计数与 E142 产物逐字比对）
#
# 里程碑「第一个事务」步 7：拿步 5 的录制流按 D13（验证路线） 已定项 4 枚举崩溃状态，每个状态跑三件事：步 6 的恢复与 oracle、
# 池级 checker（23 条不变量）、记录核对器（根在案而记录缺席、恢复自称新态而单元缺席）；三者的计数都由用例钉死。
# 全量 262165 个状态在 debug 下要几分钟，所以平时 `cargo test` 里那条用例标 ignored；这里在 release 下跑它，
# 把用例打印的 `LAYER0 …` 计数行报出来。exhaustive=true 才算全量，不是全量判红——层 0 全量是里程碑出口。
# 判别力：用例自己对着产物的十个计数断言（states / violations / root_persisted … 逐字），oracle 的判别力由同文件的靶向阳性对照证明
# （根槽已持久而某个单元两份都没持久 ⇒ 8 个单元逐个都判红）。本阶段没有 fixtures 样本：判红要 cargo 真跑，装不进 fixtures 目录。
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
[[ -f Cargo.toml && -d crates/singlefs-harness ]] || { echo "  ! 没有 crates/singlefs-harness，本阶段跳过（步 0 之前没有装置）"; exit 0; }

log="$(mktemp)"
if ! cargo test --release -p singlefs-harness --test first_transaction_step_seven_layer0 -- --include-ignored --nocapture >"$log" 2>&1; then
  tail -40 "$log"
  echo "  ✗ 层 0 崩溃点重放的用例判红（上面是 cargo test 的尾部）"
  echo "     → 怎么办：单跑看细节：cargo test --release -p singlefs-harness --test first_transaction_step_seven_layer0 -- --include-ignored --nocapture"
  echo "                oracle 报的第一条违例在断言消息里。改代码还是改断言先想清楚是哪一种——直接改断言等于把测试关掉。"
  rm -f "$log"
  exit 1
fi
line="$(grep '^LAYER0 ' "$log" | head -1)"
rm -f "$log"
if [[ -z "$line" ]]; then
  echo "  ✗ 用例跑过了，却没打印 LAYER0 计数行"
  echo "     → 怎么办：first_transaction_step_seven_layer0.rs 里全量那条用例要 println! 一行以 LAYER0 开头的计数（字段见该用例的文档注释）。"
  exit 1
fi
if [[ "$line" != *"exhaustive=true"* ]]; then
  echo "  ✗ 层 0 不是全量：$line"
  echo "     → 怎么办：枚举到的状态数要等于闭式 1 + Σ(2^|段| − 1)；少了说明某一段没展开子集，去 crash.rs 的 enumerate_layer0 看。"
  exit 1
fi
echo "  ✓ 层 0 崩溃点重放全量跑完（1 条流、每个状态两遍恢复 + checker + 记录核对器）：${line#LAYER0 }"
