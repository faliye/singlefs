#!/usr/bin/env bash
# m4-publish-trigger-r1 正推腿：P4 的计数与 P3/P5/P7 的结构性 grep。只读冻结副本，输出到标准输出。
# 用法：bash count_assumptions.sh <冻结副本根>
set -euo pipefail
F="${1:?用法：count_assumptions.sh <冻结副本根>}"
cd "$F"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "== A1 会话入口 publish_user_change 的方法调用点（含测试、装置、history）"
grep -rc '\.publish_user_change(' crates --include=*.rs | grep -v ':0$' | sort > "$TMP/a1" || true
awk -F: '{s+=$2; n+=1} END{print "sites=" s " files=" n}' "$TMP/a1"
echo "-- 按 crate/目录分"
awk -F: '{split($1,p,"/"); k=p[2]"/"p[3]; s[k]+=$2; f[k]+=1} END{for(k in s) print k " files=" f[k] " sites=" s[k]}' "$TMP/a1" | sort

echo "== A2 harness 历史的操作种类：HistoryOperation 一共几种、几种经 publish_user_change"
H=crates/singlefs-harness/src/history.rs
ENUM_START=$(grep -n '^pub enum HistoryOperation {' $H | cut -d: -f1)
ENUM_END=$(awk -v s="$ENUM_START" 'NR>s && /^}/ {print NR; exit}' $H)
echo "enum 行区间 $ENUM_START-$ENUM_END"
awk -v s="$ENUM_START" -v e="$ENUM_END" 'NR>s && NR<e && /^    [A-Z][A-Za-z]+(\(|,| \{)/ {x=$0; sub(/[\(,\{ ].*/,"",x); gsub(/^ +/,"",x); sub(/ .*/,"",x); print x}' $H > "$TMP/ops"
echo "variants=$(wc -l < "$TMP/ops")"
echo "-- 含 .publish_user_change( 的 fn"
awk '/^fn [a-z_]+/ {f=$2; sub(/\(.*/,"",f)} /\.publish_user_change\(/ {print f}' $H | sort -u
echo "-- apply_operation 分派里 变体 => 处理函数"
S=$(grep -n '^fn apply_operation(' $H | cut -d: -f1)
sed -n "${S},$((S+80))p" $H | awk '/HistoryOperation::[A-Za-z]+/ {match($0,/HistoryOperation::[A-Za-z]+/); v=substr($0,RSTART,RLENGTH)} /apply_[a-z_]+/ && v!="" {match($0,/apply_[a-z_]+/); print v " => " substr($0,RSTART,RLENGTH); v=""}'

echo "== A3 RefFS 翻译：内容改动后紧跟显式发布点"
echo "publish_with_content 出现次数=$(grep -c 'publish_with_content(' crates/singlefs-harness/src/reference_filesystem_translation.rs)"
grep -n 'operation: Operation::ExplicitPublish' crates/singlefs-harness/src/reference_filesystem_translation.rs | head -3
echo "-- RefFS 里算发布末尾的种类"
grep -n 'EnteredKind::ExplicitPublish | EnteredKind::Fsync => true' crates/singlefs-reference-filesystem/src/lib.rs
grep -n 'EnteredKind::ContentChange => false' crates/singlefs-reference-filesystem/src/lib.rs

echo "== A4 崩溃流里经会话的"
echo "crash_enumeration 文件数=$(ls crates/singlefs-checker-tier/tests/crash_enumeration_*.rs | wc -l)"
echo "-- checker-tier/tests 里含 publish_user_change 的文件"
grep -rl 'publish_user_change' crates/singlefs-checker-tier/tests | sort
echo "-- checker-tier/src 里含 publish_user_change 的文件"
grep -rl 'publish_user_change' crates/singlefs-checker-tier/src | sort

echo "== A5 随机历史入口"
grep -rl 'run_history_campaign\|execute_history_handing_out_every_step_image' crates --include=*.rs | sort

echo "== B1 发布计划一次装几个改动（P3）"
grep -n 'pub file: Option<FileVersionPlan' crates/singlefs-core/src/transaction.rs
grep -n 'pub new_inode_records' crates/singlefs-core/src/transaction.rs
grep -n 'pub enum ChangeOfAFileOtherThanTheFirst' crates/singlefs-core/src/transaction.rs
grep -n 'WholeWrite { inode: InodeNumber, data_units: u64 },' crates/singlefs-core/src/transaction.rs

echo "== B2 一次发布至多几条 journal 记录；每次准入至多几次发布；格式常量"
grep -n 'pub fn journal_record_limit_of_one_publish' crates/singlefs-core/src/transaction.rs
grep -n 'pub const PUBLISHES_PER_ADMISSION_AT_MOST' crates/singlefs-core/src/mount.rs
grep -n 'pub const JOURNAL_SAFETY_FACTOR\|pub const JOURNAL_RECORD_BYTES\|pub const DATA_UNIT_BYTES\|pub const DATA_UNIT_HEADER_BYTES\|pub const DATA_UNIT_PAYLOAD_OFFSET' crates/singlefs-format/src/lib.rs

echo "== C1 核心层没有的东西（零命中）"
for pat in 'Instant::now' 'SystemTime' 'thread::spawn' 'fn fsync' 'MountOption' 'mount_options' 'T_redo' 'retry_budget' 'InFlightDepth'; do
  printf 'grep -rn -- %s crates/singlefs-core/src | wc -l = %s\n' "'$pat'" "$(grep -rn -- "$pat" crates/singlefs-core/src | wc -l)"
done
echo "会话里的实例切换入口个数=$(grep -rn 'fn [a-z_]*instance_switch\|fn [a-z_]*switch_instance\|fn redo_' crates/singlefs-core/src | grep -v 'admission.rs' | wc -l)"

echo "== C2 调用方给钟的先例（P1）"
grep -n 'pub struct CallerMonotonicClockReading' crates/singlefs-core/src/compaction_plan.rs
grep -n 'pub trait CallerOfTheBackgroundCompaction' crates/singlefs-core/src/compaction_plan.rs
grep -n 'pub fn run_compaction_in_the_background' crates/singlefs-core/src/mounted_session.rs
grep -n 'pub struct SystemRuntimeConfiguration' crates/singlefs-core/src/system_configuration.rs
grep -n 'pub fn time_threshold_milliseconds\|pub fn dirty_threshold_bytes' crates/singlefs-core/src/system_configuration.rs

echo "== C3 只供测试的开关先例（P6）"
grep -n 'pub struct CompactionSwitches' crates/singlefs-core/src/compaction_plan.rs
grep -n 'pub struct DeletionSwitches' crates/singlefs-core/src/unlink.rs
grep -n 'fn mount_writable_with_every_test_only_switch' crates/singlefs-core/src/mount.rs

echo "== A6 调用方读返回值假设「每次改动都有已落盘的版本」的位置（Y 方案才受影响）"
echo "versions_published 在 core/src 之外的出现处=$(grep -rn 'versions_published' crates --include=*.rs | grep -v 'crates/singlefs-core/src' | wc -l) 文件数=$(grep -rln 'versions_published' crates --include=*.rs | grep -v 'crates/singlefs-core/src' | wc -l)"
echo "roots_carrying_the_change 出现处=$(grep -rn 'roots_carrying_the_change' crates --include=*.rs | wc -l) 文件=$(grep -rl 'roots_carrying_the_change' crates --include=*.rs | sort | tr '\n' ' ')"

echo "== D1 会话字段公开、卸载与回退是收字段的自由函数（待发队列要放在哪）"
grep -n 'pub struct MountedSession {' -A3 crates/singlefs-core/src/mounted_session.rs | head -4
grep -n 'pub fn unmount<Device: BlockDevice>(' -A5 crates/singlefs-core/src/mount.rs | head -6

echo "== D2 测试开关装在哪、产品入口固定什么（P6）"
grep -n 'deletion_switches: DeletionSwitches,' crates/singlefs-core/src/allocator.rs
grep -n 'pub fn mount_writable<Device: BlockDevice>(' -A4 crates/singlefs-core/src/mount.rs | head -5
grep -n 'SkippedByTheTestOnlySwitch' crates/singlefs-core/src/mount.rs | head -2
echo "-- 非 tests 目录里引用 mount_writable_with_test_only 的文件"
grep -rl 'mount_writable_with_test_only' crates/*/src | sort
