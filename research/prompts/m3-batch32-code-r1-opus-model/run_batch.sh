#!/usr/bin/env bash
# 用法：run_batch.sh <批号> <探针二进制> <输出目录> <臂:测试名>...（每件单线程、经内存包装 4G，同批并行）
# 探针借的造镜像函数（build_pool 一族）在 std::env::temp_dir() 下建文件盘：每件的 TMPDIR 设成输出目录里自己的子目录，跑完删掉。
set -u
batch="$1"; binary="$2"; out="$3"; shift 3
tools_repo="${SINGLEFS_TOOLS:?要 SINGLEFS_TOOLS 指到有 research/scripts 的仓}"
mkdir -p "$out"
rm -f "$out"/b"$batch"-*.rc
count=0
for item in "$@"; do
  count=$((count + 1))
  arm="${item%%:*}"; test_name="${item#*:}"
  scratch="$out/tmp-$batch-$count"
  {
    mkdir -p "$scratch"
    TMPDIR="$scratch" OPUS_B32_KEY_ARM="$arm" nice -n 19 bash "$tools_repo/research/scripts/run-with-memory-cap.sh" 4G \
      bash "$tools_repo/research/scripts/capped.sh" 1 "$binary" "$test_name" --exact --nocapture --test-threads=1 \
      > "$out/$arm.$test_name.log" 2>&1
    rc=$?
    rm -rf "${scratch:?}"
    echo "$rc" > "$out/b$batch-$count.rc"
  } &
done
wait
found=$(ls "$out"/b"$batch"-*.rc 2>/dev/null | wc -l)
if [ "$found" -ne "$count" ]; then echo "批 $batch 作废：rc 文件 $found 个，派出 $count 件"; exit 9; fi
status=0
for i in $(seq 1 "$count"); do
  rc=$(cat "$out/b$batch-$i.rc"); echo "批 $batch 件 $i rc=$rc"
  [ "$rc" = 0 ] || status=1
done
exit $status
