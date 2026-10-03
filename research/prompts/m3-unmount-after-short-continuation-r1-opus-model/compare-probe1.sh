#!/usr/bin/env bash
# 用法：compare-probe1.sh <probe1.txt> <narrow-today.txt>
# 把实现员探针的输出与本原型 narrow/today 的输出按 (槽数, 覆盖写, 单元数, 批) 连起来比结局（Empty / Short / 挂载错）。
set -euo pipefail
probe="$1"; mine="$2"
scratch="$(mktemp -d)"
outcome_of() { awk '{o="mount-err"; for(i=1;i<=NF;i++) if($i ~ /^outcome=/){split($i,a,"["); o=a[1]}; print $0 "\t" o}'; }
grep '^PROBE-CONT' "$probe" | outcome_of | awk -F'\t' '{split($1,f," "); print f[2]"_"f[3]"_"f[4]"_"f[5], $2}' | sort > "$scratch/p1"
outcome_of < "$mine" | awk -F'\t' '{split($1,f," "); split(f[4],o,"/"); split(f[5],u,"/"); split(f[6],b,"/"); print f[3]"_"o[1]"_"u[1]"_"b[1], $2}' | sort > "$scratch/mine"
join "$scratch/p1" "$scratch/mine" > "$scratch/joined"
echo "probe1 格数 $(wc -l < "$scratch/p1")，连上 $(wc -l < "$scratch/joined")，结局相同 $(awk '$2==$3' "$scratch/joined" | wc -l)，两边都 Short $(awk '$2=="outcome=Short" && $3=="outcome=Short"' "$scratch/joined" | wc -l)，probe1 Short $(awk '$2=="outcome=Short"' "$scratch/joined" | wc -l)"
rm -r "$scratch"
