#!/usr/bin/env bash
# 汇总 v1_reach 的输出：每类历史的段数、命中数、分级。用法：bash summarize.sh <v1_reach 的输出>
set -euo pipefail
OUT="$1"
echo "## 每类：段数 / 至少一次固定点命中的段 / 至少一次用户数据被拒的段 / 挂载干跑拒绝的段"
awk -F'\t' '$1=="H"{n[$2]++; if($9>0) fp[$2]++; if($10>0) dr[$2]++} $1=="R" && $9=="data"{key=$2 FS $3 FS $4; if(!(key in seen)){seen[key]=1; da[$2]++}} END{for(c in n) printf "%s\thistories=%d\twith_fixed_point_hit=%d\twith_data_refusal=%d\twith_dry_run_refusal=%d\n", c, n[c], fp[c]+0, da[c]+0, dr[c]+0}' "$OUT" | sort
echo "## 每类：被拒的步按种类（fixed-point / data / dry-run / other）"
awk -F'\t' '$1=="R"{k[$2 FS $9]++} END{for(x in k) print x "\t" k[x]}' "$OUT" | sort
echo "## 每类：固定点命中按分级（T1 式子会先拒用户写 / T2 保留池取这次的固定点量、承诺量 0 时式子放行 / T3 连挂载期承诺量一起也放行 / - 无读数）"
awk -F'\t' '$1=="R" && $9=="fixed-point"{k[$2 FS $28]++} END{for(x in k) print x "\t" k[x]}' "$OUT" | sort
echo "## 每类：固定点命中按被拒的单元"
awk -F'\t' '$1=="R" && $9=="fixed-point"{k[$2 FS $8]++} END{for(x in k) print x "\t" k[x]}' "$OUT" | sort
echo "## 固定点命中时跑的 checker：绿 / 红"
awk -F'\t' '$1=="R" && $9=="fixed-point" && $29!="-"{if($29=="green") g++; else {r++; print "RED\t" $0}} END{print "green=" g+0, "red=" r+0}' "$OUT"
echo "## 每类第一次固定点命中最早的一段（步号最小，同步号取宽度最小）"
awk -F'\t' '$1=="R" && $9=="fixed-point"{key=$2; s=$5+0; if(!(key in best) || s<bs[key] || (s==bs[key] && $3+0<bw[key])){best[key]=$0; bs[key]=s; bw[key]=$3+0}} END{for(k in best) print best[k]}' "$OUT" | sort
echo "## I 类：抬 F 那一步的结局 × 后缀里有没有挂载 / 回退 × 抬 F 之后有没有一次做成的发布（走出去）"
awk -F'\t' '$1=="H" && $2=="I-after-refused-raise"{
  match($4, /raise_at=[0-9]+/); r=substr($4, RSTART+9, RLENGTH-9)+0;
  n=split($13, s, ","); raised="raise-refused"; esc="stuck";
  for(i=1;i<=n;i++){ if(s[i]!="" && s[i]+0==r) raised="raise-applied"; if(s[i]!="" && s[i]+0>r) esc="escaped" }
  m=($4 ~ /suffix=.*(Mount|Rb)/)?"suffix-with-mount":"suffix-in-process";
  k[raised FS m FS esc]++ } END{for(x in k) print x "\t" k[x]}' "$OUT" | sort
echo "## I 类：抬 F 被拒、后缀全在进程里、却走出去了的段（逐段列出）"
awk -F'\t' '$1=="H" && $2=="I-after-refused-raise"{
  match($4, /raise_at=[0-9]+/); r=substr($4, RSTART+9, RLENGTH-9)+0;
  n=split($13, s, ","); ra=0; e=0; for(i=1;i<=n;i++){ if(s[i]!="" && s[i]+0==r) ra=1; if(s[i]!="" && s[i]+0>r) e=1 }
  if(!ra && e && $4 !~ /suffix=.*(Mount|Rb)/ && shown++ < 40) print }' "$OUT"
echo "## 抬 F 被拒之前已经写出的空发布次数（txg_after − txg_before）"
awk -F'\t' '$1=="R" && $6 ~ /^Raise/ && $7 ~ /NoFreeSlot/ {k[$2 FS ($11-$10)]++} END{for(x in k) print x "\t" k[x]}' "$OUT" | sort
echo "## 固定点命中那一刻，按盘上最旧可读根回收得到而实现没收的落点（C518 那一类混杂因素）：每类命中数 / 其中漏收 > 0 的"
awk -F'\t' '$1=="R" && $9=="fixed-point"{n[$2]++; if($20!="-" && $20+0>0) m[$2]++} END{for(c in n) print c "\tfixed_point_hits=" n[c] "\twith_missed_reclaim=" m[c]+0}' "$OUT" | sort
