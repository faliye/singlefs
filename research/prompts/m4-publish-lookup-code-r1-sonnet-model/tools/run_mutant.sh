#!/usr/bin/env bash
# 用法：DRAFT=<草稿目录> REPO_ROOT=<仓根> run_mutant.sh <变异名片段> <测试名过滤> <日志前缀>
# 草稿目录里要有：repo/（冻结副本的 crates、Cargo.toml、Cargo.lock、.cargo、.claude/heavy-thread-default）、
#   transaction.rs.orig 与 mounted_session.rs.orig（冻结副本里那两份核心层源文件）、target/（编译目录）。
# 一条变异：还原源文件 -> 套 crates/mutations.tsv 里那一行 -> 编 -> 跑点名的测试 -> 还原。输出落 <日志前缀>.build.log / .run.log。
set -u
: "${DRAFT:?设草稿目录}" "${REPO_ROOT:?设仓根}"
fragment="$1"; filter="$2"; prefix="$3"
tools_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$DRAFT/repo" || exit 2
cp "$DRAFT/transaction.rs.orig" crates/singlefs-core/src/transaction.rs
cp "$DRAFT/mounted_session.rs.orig" crates/singlefs-core/src/mounted_session.rs
python3 -I "$tools_directory/apply_mutation.py" "$DRAFT/repo" "$fragment" || exit 2
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 publish_lookup_by_identity（只编不跑，套了一条变异）; 范围=1 个测试目标; 为什么=自己的副本里看这条变异会不会让点名的测试红，先编' CARGO_TARGET_DIR="$DRAFT/target" timeout 200 nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 bash "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" 8G cargo test --offline -p singlefs-harness --test publish_lookup_by_identity --no-run > "$prefix.build.log" 2>&1
echo "build exit=$?"
binary="$(ls -t "$DRAFT"/target/debug/deps/publish_lookup_by_identity-* | grep -v '\.d$' | head -1)"
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 publish_lookup_by_identity 里点名的一个用例（套了一条变异）; 范围=1 个测试; 为什么=看这条变异会不会让点名的测试红' timeout 200 nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 bash "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" 3G "$binary" "$filter" --nocapture > "$prefix.run.log" 2>&1
echo "test exit=$?"
cp "$DRAFT/transaction.rs.orig" crates/singlefs-core/src/transaction.rs
cp "$DRAFT/mounted_session.rs.orig" crates/singlefs-core/src/mounted_session.rs
