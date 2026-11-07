#!/usr/bin/env bash
# run-condition: command cargo
# E200（修订6近似树重跑） 第一段：在近似树里 release 编 E171、E170 两个装置 bin（只编不跑），编完把两个二进制拷进树里的 target/release/
#   （近似那一趟第 0 步与各单元按这个位置找，同 E171（三oracle合谋测试） 研究一侧 frozen_tree_lines_revision_8.py 的 target/release/<名字>）。
# 跑法（--cwd 给近似树的 tree/ 目录，整条命令交 task-server/task submit）：
#   task-server/task submit --cwd <造树的输出目录>/tree -- bash <仓根>/scripts/experiment-apparatus/e200_revision_6_segment_one_on_an_approximate_tree/compile_in_the_approximate_tree.sh <新产物路径>
# 编译目录取服务给的 CARGO_TARGET_DIR，不往树里编；没设退 2。不带 gpu 特性（登记第三节：近似树里的装置不走崩溃判段）。
# --locked：Cargo.lock 与清单对不上时编不过、不改树（登记第一节读法 4「不为编过而改」）。诊断写进树旁边的 compile-tree.log（造树输出目录里，不在树里）。
# 拷：两条 cp 各自的退出码收进 COPY_EXIT（不为 0 的取后到的那一个），交给 compile-lines；它再逐字节比树里 target/release/ 下每个二进制与
#   这一趟 $CARGO_TARGET_DIR/release/ 下的那一份：cp 都退 0、两份都与这一趟的构建输出逐字节相同才记 compiled=true（树里原有的旧二进制不算拷好）。
# 产物（approximate_tree_lines.py compile-lines 写）：cargo 退出码、拷的退出码、诊断末 20 行、两个二进制在树里与在编译目录里的 sha256 与逐字节同不同、
#   编之前编之后树的快照摘要。
# 退出码：0 编过、拷好、树没变；1 编不过、没拷好（cp 退非 0，或拷完树里那份与这一趟的构建输出不逐字节相同）或树变了（产物照写）；
#   2 用法错（没给产物路径、产物已存在、当前目录不是近似树、没有 CARGO_TARGET_DIR）。
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PRODUCT="${1:-}"
if [[ -z "$PRODUCT" || -e "$PRODUCT" ]]; then echo "用法：compile_in_the_approximate_tree.sh <还不存在的产物路径>（产物已存在不覆盖）" >&2; exit 2; fi
if [[ ! -f Cargo.toml || ! -f ../manifest.tsv ]]; then echo "当前目录不是造树输出目录里的 tree/（没有 Cargo.toml 或 ../manifest.tsv）" >&2; exit 2; fi
if [[ -z "${CARGO_TARGET_DIR:-}" ]]; then echo "没有 CARGO_TARGET_DIR：编译目录由统一任务服务给，经 task-server/task submit 起" >&2; exit 2; fi
read -r DIGEST_BEFORE _ < <(python3 "$HERE/approximate_tree_lines.py" tree-digest .)
cargo build --release --locked -p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes --bin e170_o3_spec_executor_export 2> ../compile-tree.log
CARGO_EXIT=$?
COPY_EXIT=0
if [[ "$CARGO_EXIT" -eq 0 ]]; then
  mkdir -p target/release   # 建不成的，下面两条 cp 都退非 0
  cp "$CARGO_TARGET_DIR/release/e171_three_oracle_collusion_after_the_oracle_fixes" target/release/e171_three_oracle_collusion_after_the_oracle_fixes || COPY_EXIT=$?
  cp "$CARGO_TARGET_DIR/release/e170_o3_spec_executor_export" target/release/e170_o3_spec_executor_export || COPY_EXIT=$?
fi
python3 "$HERE/approximate_tree_lines.py" compile-lines --tree . --exit-code "$CARGO_EXIT" --copy-exit-code "$COPY_EXIT" --build-output "$CARGO_TARGET_DIR/release" \
  --log ../compile-tree.log --digest-before "$DIGEST_BEFORE" --product "$PRODUCT"
