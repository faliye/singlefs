#!/usr/bin/env bash
# probe_group.sh <二进制> <组> <分片数> <标签>：把一组探针场景按下标取余分给若干进程并行跑，日志 $WORK/logs/<标签>-<i>.log，末尾汇总。
set -u
BIN="$1"; GROUP="$2"; SHARDS="$3"; TAG="$4"
REPO_ROOT="${REPO_ROOT:?}"; WORK="${WORK:?}"
mkdir -p "$WORK/logs"
rm -f "$WORK/logs/$TAG"-*.log "$WORK/logs/$TAG"-*.exit
for ((i = 0; i < SHARDS; i++)); do
  {
    PROBE_GROUP="$GROUP" PROBE_SHARD="$i/$SHARDS" nice -n 19 timeout 215 bash "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" 4G "$BIN" --ignored --nocapture --exact defense_probe_group \
      > "$WORK/logs/$TAG-$i.log" 2>&1
    echo "$?" > "$WORK/logs/$TAG-$i.exit"
  } &
done
wait
echo "exits: $(cat "$WORK"/logs/"$TAG"-*.exit | sort | uniq -c | tr '\n' ' ')"
cat "$WORK"/logs/"$TAG"-*.log | grep '^PROBE ' | awk '{c++; for(i=1;i<=NF;i++){if($i ~ /^ending=/ && $i!="ending=Completed")nc++; if($i ~ /^reds=/){split($i,a,"="); r+=a[2]}}} END{print "scenarios="c" not_completed="nc+0" reds="r+0}'
