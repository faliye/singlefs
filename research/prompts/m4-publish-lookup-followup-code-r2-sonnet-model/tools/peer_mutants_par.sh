#!/usr/bin/env bash
# peer_mutants_par.sh <工人数> <变异表:行>…：把若干（变异表、行号）轮流分给 N 个工人，每个工人有自己的 crates 副本与编译目录，并行跑；
# 外面已套内存包装（上限给得住 N × 约 2.5 GiB），这里不再套。每个工人逐条：施加变异 -> cargo test 点名的测试 -> 贴结果 -> 还原。
set -u
root=$(pwd); workers=$1; shift
rm -rf "$root/.pristine" "$root/par"; mkdir -p "$root/.pristine" "$root/par" "$root/mutout"; cp -a "$root/crates" "$root/.pristine/"
threads=$((31 / workers))
for ((k = 0; k < workers; k++)); do
  mkdir -p "$root/par/w$k/.claude"; cp -a "$root/crates" "$root/Cargo.toml" "$root/Cargo.lock" "$root/.cargo" "$root/par/w$k/"; cp "$root/.claude/heavy-thread-default" "$root/par/w$k/.claude/"
  find "$root/par/w$k/crates" -type f -exec touch {} +
done
echo "name=peer_mutants_par workers=$workers jobs=$# nproc=$(nproc)"
index=0
for job in "$@"; do
  echo "$job" >> "$root/par/jobs-$((index % workers)).txt"; index=$((index + 1))
done
for ((k = 0; k < workers; k++)); do
  {
    while read -r job; do
      tsv=${job%%:*}; row=${job##*:}
      python3 -I "$root/model/tools/mutate_run.py" --repo "$root" --pristine "$root/.pristine" --work "$root/par/w$k" \
        --target "$root/par/w$k/target" --out "$root/mutout" --tsv "$root/model/mutations/$tsv" --rows "$row" --direct --threads "$threads" --tag "$tsv." >> "$root/par/out-$k.txt" 2>&1
    done < "$root/par/jobs-$k.txt"
    echo "$?" > "$root/par/done-$k.txt"
  } &
done
wait
count=$(ls "$root"/par/done-*.txt | wc -l)
echo "name=peer_mutants_par done_files=$count of $workers"
cat "$root"/par/out-*.txt
