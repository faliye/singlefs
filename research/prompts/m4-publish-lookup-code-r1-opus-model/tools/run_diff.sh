#!/usr/bin/env bash
# run_diff.sh <批号> <上限> <件…>：件写成 名字|测试名|环境赋值（空格分隔），每件在 before 与 after 两份二进制上各跑一次，
# 每份退出码写进 runs/d<批号>-<件号>-<副本>.rc，wait 等齐后数文件，再逐件比 M4 开头的行。
S=/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r1-opus
batch=$1; cap=$2; shift 2
rm -f $S/runs/d$batch-*.rc
i=0
for spec in "$@"; do
  i=$((i+1))
  IFS='|' read -r name test envs <<< "$spec"
  for copy in before after; do
    B=$(ls $S/target-$copy/${PROFILE:-release}/deps/m4_publish_lookup_attack_differential-* | grep -v '\.d$' | head -1)
    { cd $S/$copy/crates/singlefs-harness && env $envs SINGLEFS_RUN_SCOPE="种类=target; 跑什么=singlefs-harness 的测试目标 m4_publish_lookup_attack_differential; 范围=1 个测试目标; 为什么=攻方腿改前改后差分 $name" timeout 228 nice -n 19 bash /home/fy5090/code/singlefs/scripts/hosts/capped.sh 31 bash /home/fy5090/code/singlefs/scripts/hosts/run-with-memory-cap.sh $cap $B --exact $test --nocapture > $S/runs/d$batch-$i-$copy-$name.log 2>&1; echo "$?" > $S/runs/d$batch-$i-$copy.rc; } &
  done
done
wait
n=$(ls $S/runs/d$batch-*.rc 2>/dev/null | wc -l)
echo "batch=$batch pieces=$i runs=$((i*2)) rc_files=$n"
[ "$n" -eq "$((i*2))" ] || { echo "整批作废"; exit 1; }
i=0
for spec in "$@"; do
  i=$((i+1)); IFS='|' read -r name test envs <<< "$spec"
  rb=$(cat $S/runs/d$batch-$i-before.rc); ra=$(cat $S/runs/d$batch-$i-after.rc)
  lb=$(grep -c '^M4' $S/runs/d$batch-$i-before-$name.log); la=$(grep -c '^M4' $S/runs/d$batch-$i-after-$name.log)
  if diff -q <(grep '^M4' $S/runs/d$batch-$i-before-$name.log) <(grep '^M4' $S/runs/d$batch-$i-after-$name.log) >/dev/null; then same=identical; else same=DIFFERENT; fi
  echo "piece=$i name=$name rc_before=$rb rc_after=$ra lines_before=$lb lines_after=$la $same $(grep -E '^test result' $S/runs/d$batch-$i-after-$name.log | cut -c1-60)"
done
