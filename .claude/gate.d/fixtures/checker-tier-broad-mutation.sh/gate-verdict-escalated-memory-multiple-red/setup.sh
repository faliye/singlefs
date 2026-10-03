#!/usr/bin/env bash
# 样本现场（情形 escalated-memory-multiple）：gate-verdict 复核升档判的抓到（驱动文件头「升档」）与结果行的校验列（驱动文件头「结果行的校验列」）。
# 库目录这一版 6 条都有结论，第 1 条（编译那一步两趟都超时）、第 2 条（测试那一步两趟都撞顶）是升档判的抓到；这里造那一版的基线（baseline.tsv：步名、结局、秒数、时刻、
# 线程数、峰值）、两趟日志（末尾带驱动那一行、带包装那一句）、登记表（baseline_row 是它取的那一行基线的行号）、结果表（每行带 HMAC-SHA256 校验列）与样本自己的密钥；
# 日志的 sha256 用 sha256sum 现算，校验列用 python3 的 hmac 现算（不调驱动的函数）；--summary 的输出由 summary.sh 打录好的那几行。情形：
#   green 两行都齐、另有一行别的版本的坏行（不看）；count 判定行 escalated=3、ESCALATED 只有 2 行；handwritten 第 1 条的登记行指的日志不在；
#   log-digest 第 1 条升档那趟的日志登记之后又改了；multiple 第 1 条升档那趟的限时 150 不到基线 10.0 秒的 20 倍；old-version 第 1 条的登记行与证据都是上一版的；
#   threads-rerun 换了 --threads 之后（攻方 J2-a 现场）：16 线程下的基线追加在后面、第 1 条在 16 线程下又升档判了一次（登记一行新的，指 16 线程那一行），
#     8 线程的峰值表也被包装改写过，旧登记行指的 8 线程那一行没动，照样复核得过；
#   baseline-row-changed 第 1 条登记行指的那一行基线（第 2 行）的线程数被改成 16；baseline-lie 第 1 条登记的基线 1.0，它指的那一行记的是 10.0；
#   not-larger 第 1 条两趟的限时都是 3600；memory-multiple 第 2 条的基线峰值 20 GiB，升档那趟 36864M 不到它的 2 倍；
#   no-trailer 第 1 条两份日志末尾没有驱动那一行；trailer-wrong 驱动那一行写成别的（attempt=first、exit=0）；no-wrapper 第 1 条两份日志里没有包装那一句；
#   columns 第 1 条的登记行少了末一段；no-escalated-field 判定行没有 escalated=；name-differs ESCALATED 第 1 行的变异名与登记的不同；
#   logs-elsewhere 第 1 条的两份日志挪到 logs/ 下、登记行跟着指过去；
#   result-mac-forged 结果表末尾手追加一行没有校验列的 caught（第 4 条）与一行校验列对不上的 caught（第 5 条）；result-mac-key-missing 样本的密钥不在。
set -euo pipefail
variant="memory-multiple"
current=e5ca1a7e000000000000000000000000000000000000000000000000000000f1
old=01d0000000000000000000000000000000000000000000000000000000000ff0
name_one="crates/singlefs-core/src/lib.rs:2:5: replace the value with MUTATED_1()"
name_two="crates/singlefs-core/src/lib.rs:5:5: replace the value with MUTATED_2()"
home=.broad-sample/home
key=.broad-sample/result.key
mkdir -p "$home/versions/$current/escalation" "$home/versions/$old/escalation"
printf '{"commit": "0123456789abcdef0123456789abcdef01234567", "revision": "HEAD", "fingerprint": "%s", "source_fingerprint": "%s", "files": 6, "export_paths": ["crates", "Cargo.toml", "Cargo.lock", "litmus", ".cargo"]}\n' "$current" "$current" > "$home/tree.source"
peak_two=1073741824; [[ "$variant" == memory-multiple ]] && peak_two=21474836480
for version in "$current" "$old"; do
  printf '# step\toutcome\tseconds\tfinished_at\tthreads\tpeak_bytes\none-build\tok\t10.0\t2026-10-03\t8\t536870912\none-test\tok\t40.0\t2026-10-03\t8\t%s\n' "$peak_two" \
    > "$home/versions/$version/baseline.tsv"
  printf '# 样本：包装记基线峰值的那一份（驱动读完写进 baseline.tsv 那一行，之后不再读它）\n%s\tselftest\t24G\tbroad-mutation baseline one-test\t8\n' "$peak_two" \
    > "$home/versions/$version/baseline-peaks.8.tsv"
done
first_limit=900; escalated_limit=3600
[[ "$variant" == multiple ]] && { first_limit=100; escalated_limit=150; }
[[ "$variant" == not-larger ]] && first_limit=3600
one_version="$current"; [[ "$variant" == old-version ]] && one_version="$old"
one_first="escalation/1.one-build.100.first.log"; one_escalated="escalation/1.one-build.100.escalated.log"
write_log() {   # <版本> <相对路径> <包装那一句，空就不写> <序号> <步名> <哪一趟> <限时或上限> <退出码>，第 9 个参数 notrailer 就不写驱动那一行
  { printf '   Compiling singlefs-core（样本）\n'
    [[ -n "$3" ]] && printf 'run-with-memory-cap: %s（样本）\n' "$3"
    [[ "${9:-}" == notrailer ]] || printf 'BROAD_MUTATION_EVIDENCE version=%s sequence=%s step=%s attempt=%s limit=%s exit=%s\n' "$1" "$4" "$5" "$6" "$7" "$8"; } > "$home/versions/$1/$2"
}
one_first_wrapper="超过限时 $first_limit 秒"; one_escalated_wrapper="超过限时 $escalated_limit 秒"
[[ "$variant" == no-wrapper ]] && { one_first_wrapper=""; one_escalated_wrapper=""; }
trailer_mode=""; [[ "$variant" == no-trailer ]] && trailer_mode=notrailer
if [[ "$variant" == trailer-wrong ]]; then
  write_log "$one_version" "$one_first" "$one_first_wrapper" 1 one-build first "$first_limit" 0
  write_log "$one_version" "$one_escalated" "$one_escalated_wrapper" 1 one-build first "$escalated_limit" 0
else
  write_log "$one_version" "$one_first" "$one_first_wrapper" 1 one-build first "$first_limit" 253 "$trailer_mode"
  write_log "$one_version" "$one_escalated" "$one_escalated_wrapper" 1 one-build escalated "$escalated_limit" 253 "$trailer_mode"
fi
write_log "$current" escalation/2.one-test.200.first.log "撞了内存上限 24G" 2 one-test first 24G 250
write_log "$current" escalation/2.one-test.200.escalated.log "撞了内存上限 36864M" 2 one-test escalated 36864M 250
if [[ "$variant" == logs-elsewhere ]]; then
  mkdir -p "$home/versions/$current/logs"
  mv "$home/versions/$current/$one_first" "$home/versions/$current/logs/1.first.log"
  mv "$home/versions/$current/$one_escalated" "$home/versions/$current/logs/1.escalated.log"
  one_first=logs/1.first.log; one_escalated=logs/1.escalated.log
fi
digest() { sha256sum "$home/versions/$1/$2" | cut -d' ' -f1; }
one_first_digest="$(digest "$one_version" "$one_first")"; one_escalated_digest="$(digest "$one_version" "$one_escalated")"
if [[ "$variant" == handwritten ]]; then
  one_first="escalation/1.one-build.999.first.log"; one_escalated="escalation/1.one-build.999.escalated.log"
  one_first_digest=0000000000000000000000000000000000000000000000000000000000000000; one_escalated_digest="$one_first_digest"
fi
[[ "$variant" == log-digest ]] && printf '登记之后又改了这一份\n' >> "$home/versions/$one_version/$one_escalated"
one_baseline=10.0; [[ "$variant" == baseline-lie ]] && one_baseline=1.0
one_tail=$'\t2026-10-03'; [[ "$variant" == columns ]] && one_tail=""
{
  printf '# version\tsequence\tname\tlabel\tstep\tbaseline\tbaseline_row\tthreads\tfirst_limit\tescalated_limit\trequired_multiple\tfirst_log\tfirst_log_sha256\tescalated_log\tescalated_log_sha256\tfinished_at\n'
  printf '%s\t1\t%s\tcaught-by-timeout\tone-build\t%s\t2\t8\t%s\t%s\t20\t%s\t%s\t%s\t%s%s\n' "$one_version" "$name_one" "$one_baseline" "$first_limit" "$escalated_limit" \
    "$one_first" "$one_first_digest" "$one_escalated" "$one_escalated_digest" "$one_tail"
  printf '%s\t2\t%s\tcaught-by-memory\tone-test\t%s\t3\t8\t24G\t36864M\t2\t%s\t%s\t%s\t%s\t2026-10-03\n' "$current" "$name_two" "$peak_two" \
    escalation/2.one-test.200.first.log "$(digest "$current" escalation/2.one-test.200.first.log)" escalation/2.one-test.200.escalated.log "$(digest "$current" escalation/2.one-test.200.escalated.log)"
  if [[ "$variant" == green ]]; then
    printf '%s\t9\t别的版本的坏行\tcaught-by-timeout\tone-build\t1.0\t2\t8\t1\t2\t20\tescalation/no-such.log\t0\tescalation/no-such.log\t0\t2026-10-03\n' "$old"
  fi
} > "$home/escalated-verdicts.tsv"
result_rows=(
  "1	singlefs-core	$name_one	caught	one	caught-by-timeout	0.0	3600.2	2026-10-03T00:00:00Z	$current	one-build 超时之后升档重跑（900 → 3600），两趟都超时"
  "2	singlefs-core	$name_two	caught	one	caught-by-memory	0.0	12.0	2026-10-03T00:00:00Z	$current	one-test 撞顶之后升档重跑（24G → 36864M），两趟都撞顶"
  "3	singlefs-core	crates/singlefs-core/src/lib.rs:8:5: replace the value with MUTATED_3()	caught	carried	some::test	0.0	0.0	2026-10-03T00:00:00Z	$current	沿用"
  "4	singlefs-core	crates/singlefs-core/src/lib.rs:11:5: replace the value with MUTATED_4()	caught	carried	some::test	0.0	0.0	2026-10-03T00:00:00Z	$current	沿用"
  "5	singlefs-core	crates/singlefs-core/src/lib.rs:14:5: replace the value with MUTATED_5()	caught	carried	some::test	0.0	0.0	2026-10-03T00:00:00Z	$current	沿用"
  "6	singlefs-core	crates/singlefs-core/src/lib.rs:17:5: replace the value with MUTATED_6()	caught	carried	some::test	0.0	0.0	2026-10-03T00:00:00Z	$current	沿用"
)
if [[ "$variant" == threads-rerun ]]; then
  # 攻方 J2-a：同一版换了 --threads 16，驱动在 16 线程下重量基线、追加两行（热编译 1.2 秒）；包装把 8 线程那一份峰值表里的峰值改写成这一次量的；
  # 再照门禁出路 --item 1 --rerun：16 线程下又升档、两趟都超时、证据齐，另写两份日志、登记一行指 16 线程那一行（第 4 行），结果表追加一行
  printf 'one-build\tok\t1.2\t2026-10-04\t16\t536870912\none-test\tok\t30.0\t2026-10-04\t16\t1074790400\n' >> "$home/versions/$current/baseline.tsv"
  printf '# 样本：包装改写过的峰值表\n1074790400\t2026-10-04T00:00:00Z\t24G\tbroad-mutation baseline one-test\t8\n' > "$home/versions/$current/baseline-peaks.8.tsv"
  write_log "$current" escalation/1.one-build.300.first.log "超过限时 900 秒" 1 one-build first 900 253
  write_log "$current" escalation/1.one-build.300.escalated.log "超过限时 3600 秒" 1 one-build escalated 3600 253
  printf '%s\t1\t%s\tcaught-by-timeout\tone-build\t1.2\t4\t16\t900\t3600\t20\t%s\t%s\t%s\t%s\t2026-10-04\n' "$current" "$name_one" \
    escalation/1.one-build.300.first.log "$(digest "$current" escalation/1.one-build.300.first.log)" \
    escalation/1.one-build.300.escalated.log "$(digest "$current" escalation/1.one-build.300.escalated.log)" >> "$home/escalated-verdicts.tsv"
  result_rows+=("1	singlefs-core	$name_one	caught	one	caught-by-timeout	0.0	3600.1	2026-10-04T00:00:00Z	$current	one-build 超时之后升档重跑（900 → 3600），两趟都超时")
fi
if [[ "$variant" == baseline-row-changed ]]; then
  # 登记行指的那一行基线（第 2 行）被改了：线程数 8 → 16，秒数没动
  awk 'BEGIN { FS = OFS = "\t" } NR == 2 { $5 = "16" } { print }' "$home/versions/$current/baseline.tsv" > "$home/versions/$current/baseline.tsv.new"
  mv "$home/versions/$current/baseline.tsv.new" "$home/versions/$current/baseline.tsv"
fi
printf '0f1e2d3c4b5a69788796a5b4c3d2e1f000112233445566778899aabbccddeeff\n' > "$key"
chmod 600 "$key"
python3 - "$key" "$home/versions/$current/results.tsv" "${result_rows[@]}" <<'PYEOF'
import hashlib, hmac, sys
key = bytes.fromhex(open(sys.argv[1]).read().strip())
with open(sys.argv[2], "w", encoding="utf-8") as handle:
    handle.write("# sequence\tpackage\tname\toutcome\tstage\tfirst_failing_test\tbuild_seconds\ttest_seconds\tfinished_at\tfingerprint\tnote\tmac\n")
    for row in sys.argv[3:]:
        handle.write(row + "\t" + hmac.new(key, row.encode("utf-8"), hashlib.sha256).hexdigest() + "\n")
PYEOF
if [[ "$variant" == result-mac-forged ]]; then
  printf '4\tsinglefs-core\tcrates/singlefs-core/src/lib.rs:11:5: replace the value with MUTATED_4()\tcaught\tone\thand::written\t0.0\t0.0\t2026-10-03T00:00:00Z\t%s\t\n' "$current" \
    >> "$home/versions/$current/results.tsv"
  printf '5\tsinglefs-core\tcrates/singlefs-core/src/lib.rs:14:5: replace the value with MUTATED_5()\tcaught\tone\thand::written\t0.0\t0.0\t2026-10-03T00:00:00Z\t%s\t\t%s\n' "$current" \
    0000000000000000000000000000000000000000000000000000000000000000 >> "$home/versions/$current/results.tsv"
fi
[[ "$variant" == result-mac-key-missing ]] && rm -f "$key"
escalated_count=2; [[ "$variant" == count ]] && escalated_count=3
escalated_field=" escalated=${escalated_count}"; [[ "$variant" == no-escalated-field ]] && escalated_field=""
summary_name_one="$name_one"; [[ "$variant" == name-differs ]] && summary_name_one="crates/singlefs-core/src/lib.rs:9:9: some other mutant"
cat > .broad-sample/summary.sh <<SUMMARY
#!/usr/bin/env bash
# 样本：录好的驱动 --summary 输出（这一版 6 条：沿用 4 条、现判 2 条，现判的两条都是升档判的抓到），退 0
printf 'BROAD_MUTATION_SUMMARY fingerprint=${current:0:16} commit=0123456789ab total=6 caught=6 missed=0 unviable=0 timeout=0 memory_cap=0 stage_one_only=0 driver_error=0 not_run=0\n'
printf 'BROAD_MUTATION_GATE total=6 concluded=6 carried=4 fresh=2 missed=0 without_conclusion=0 flipped=0${escalated_field} verdict=green\n'
printf 'ESCALATED\t1\tcaught-by-timeout\t${summary_name_one}\n'
printf 'ESCALATED\t2\tcaught-by-memory\t${name_two}\n'
exit 0
SUMMARY
