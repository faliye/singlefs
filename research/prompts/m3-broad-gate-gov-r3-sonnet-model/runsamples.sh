#!/usr/bin/env bash
# 用法：runsamples.sh <BREAK 值> <样本名前缀过滤，空=全部>；打每份样本的 期望/实测/缺want
G=/home/fy5090/code/singlefs/.claude/gate.d; S=$G/fixtures/checker-tier-broad-mutation.sh
brk="$1"; filt="${2:-}"
for d in $(ls $S); do
  [[ -n "$filt" && "$d" != *$filt* ]] && continue
  w=$(mktemp -d /tmp/claude-1000/m3-gov-r3-defense/s.XXXXXX); cp -a $S/$d/. $w/
  ( cd $w && env -u GATE_BASE -u GATE_STAGED_FROM -u GATE_DIFF_BASE bash setup.sh >/dev/null 2>&1 )
  out=$(cd $w && env -u GATE_BASE -u GATE_STAGED_FROM -u GATE_DIFF_BASE PYTHONDONTWRITEBYTECODE=1 CHECKER_TIER_BROAD_MUTATION_BREAK="$brk" nice -n 19 bash $G/checker-tier-broad-mutation.sh $w --force 2>&1); got=$?
  want=$(sed -n 's/^exit=//p' $w/expect); miss=0
  while IFS= read -r wl; do grep -qF -- "$wl" <<<"$out" || miss=$((miss+1)); done < <(sed -n 's/^want=//p' $w/expect)
  if [[ "$got" == "$want" && $miss == 0 ]]; then echo "ok   $d"; else echo "FLIP $d 期望$want 实测$got 缺want$miss"; fi
  rm -rf $w
done
