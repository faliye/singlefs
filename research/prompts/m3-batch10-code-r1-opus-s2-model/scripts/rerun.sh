#!/usr/bin/env bash
# m3-batch10-code-r1 攻方第二次抽样（Opus）复跑。用法：bash rerun.sh <阶段> <空的工作目录 W>；`rerun.sh list W` 列阶段，按列出的次序逐个跑。
# 原型 = m3-batch17-code-r1 攻方原型（probe/main.rs，原样）+ 本目录 patches/main.patch + probe/k.rs；臂 jia 是冻结副本原样，hi / noq3 / rr2 各加一份补丁，
# p10c 是第 10c 批之前那棵树（删除与并发那条线的拷贝）。只编跑自己的原型（名字不带 layer0，不是 checker 档包的测试）；经内存包装、线程不超过 15。
# 每个阶段前台在 4 分钟以内（本机负载 10–18 时量的）；`rerun.sh filter W` 把 W/out 里的判定行抽到 W/out-k，`rerun.sh compare W` 与本目录 out/ 比（去掉 elapsed 字段）。
set -eu
REPO=${SINGLEFS_REPO:-/home/fy5090/code/singlefs}
HERE=$(cd "$(dirname "$0")/.." && pwd)
B17=${B17_MODEL:-$REPO/research/prompts/m3-batch17-code-r1-opus-model}
FROZEN=${FROZEN_TREE:-/tmp/claude-1000/m3-batch10-code-r1/tree}
PRE10C=${PRE10C_TREE:-/tmp/claude-1000/-home-fy5090-code-singlefs/e594a7a0-d003-4a02-9455-a9dc350ed330/scratchpad/m3b17-today/tree}
STAGE=${1:?阶段}; export W=${2:?工作目录}
mkdir -p "$W/out" "$W/pieces"
S=$HERE/scripts
arm() { # arm <臂> <源树> [补丁…]
  local a=$1 src=$2; shift 2
  rsync -a --delete --exclude target --exclude .git "$src/" "$W/$a/"
  (cd "$W/$a" && patch -s -p0 < "$B17/patches/workspace-member.patch")
  mkdir -p "$W/$a/crates/attack-b17-probe/src"
  cp "$B17/probe/Cargo.toml" "$W/$a/crates/attack-b17-probe/Cargo.toml"
  cp "$B17/probe/main.rs" "$W/$a/crates/attack-b17-probe/src/main.rs"
  (cd "$W/$a" && patch -s -p1 < "$HERE/patches/main.patch")
  cp "$HERE/probe/k.rs" "$W/$a/crates/attack-b17-probe/src/k.rs"
  for p in "$@"; do (cd "$W/$a" && patch -s -p1 < "$HERE/patches/$p"); done
  if [ "$a" = p10c ]; then # 第 10e 批之前没有 root_ring_of_the_rebuilt_allocator：只打印实例表那一半；第 10d 批之前没有掉电接缝：去掉 MODE=k4
    sed -i '/^\/\/\/ MODE=k4/,$d' "$W/$a/crates/attack-b17-probe/src/k.rs"
    sed -i '/if mode == "k4" {/,+3d' "$W/$a/crates/attack-b17-probe/src/main.rs"
    sed -i 's/let rereads = format!("ring:{:?}\/table:{:?}", output.rereads.root_ring_of_the_rebuilt_allocator, output.rereads.instance_table_of_the_newest_root);/let rereads = format!("ring:NA\/table:{:?}", output.rereads.instance_table_of_the_newest_root);/' "$W/$a/crates/attack-b17-probe/src/k.rs"
  fi
  ATTACK_DIR=$W ATTACK_THREADS=15 bash "$B17/scripts/build.sh" "$a" > "$W/build-$a.log" 2>&1
}
P() { local name=$1; shift; printf '%s\n' "#!/usr/bin/env bash" "$*" > "$W/pieces/$name.sh"; echo "$W/pieces/$name.sh"; }
A='w:all/-:z;w:-/all:z;w:all/-:e;w:-/all:e;w:but1/all:z;w:all/but1:z;w:but1/but1:z;w:only1/-:z;w:-/only1:z;w:only1/only1:e;w:but1/-:g;w:-/but1:g'
B='w:-/all:z:p;w:all/-:e:p;w:but1/-:z:p;w:-/only1:e:p;w:only1/only1:z:p;w:but1/but1:e:p;w:-/but1:g:p;w:only1/-:g:p'
C='P5:all/all:e+P6:all/-:e;P5:all/all:e+P6:-/all:e;P5:all/all:z+P6:but1/all:z;P5:all/all:z+P6:all/but1:z;P5:all/all:e+P6:only1/-:e;P5:all/all:e+P6:-/only1:z;P5:all/all:z+P6:but1/but1:e;P5:all/all:g+P6:only1/only1:g;P5:all/-:e;P5:-/all:e;P5:but1/all:z;P5:all/but1:g;P5:all/all:e+r144-end:all/-:e:p;P5:all/all:z+r144-end:-/all:z:p'
C1='P5:all/all:e+P6:all/-:e;P5:all/all:e+P6:-/all:e;P5:all/all:z+P6:but1/all:z;P5:all/all:z+P6:all/but1:z;P5:all/all:e+P6:only1/-:e;P5:all/all:e+P6:-/only1:z;P5:all/all:z+P6:but1/but1:e;P5:all/all:e+P6:only1/only1:z;P5:all/-:e;P5:-/all:e;P5:but1/all:z;P5:all/but1:z;P5:all/all:e+r144-end:all/-:e:p;P5:all/all:z+r144-end:-/all:z:p;P5:all/all:e+r144-end:but1/-:z:p;P5:all/all:e+r144-end:-/only1:e:p;P5:all/all:e+P6:all/all:e'
SW="MODE=k SCREEN=1 K_SCREEN_AT=mountroots K_SURVIVOR=1 K_UNMOUNT_AFTER=1 K_CONTINUE=1 AFTER_J=26 K_DETAIL=I-7.9 K_PRINT=6"
STARTS=("plainlist|4:30:1:3;|now|0" "plainlist|384:30:0:0;|now|0" "plainlist|384:12:2:3;|now|0" "plainlist|256:20:1:2;|now|0" "plainlist|240:16:0:0;|now|0" "list|216:7:5:4;|now|0" "list|280:4:3:1;|now|0" "list|320:4:3:1;|now|0" "list|248:4:3:1;|now|0" "list|192:4:3:1;|unmount|17" "list|264:6:4:0;|unmount|15" "list|192:9:3:1;|unmount|8")
s6() { # 第 lo..hi 件（每件一个起点的一半计划）
  local list=() i=0
  for st in "${STARTS[@]}"; do IFS='|' read -r fam sh src cut <<< "$st"; for half in AB C; do i=$((i+1)); [ $i -lt "$1" ] || [ $i -gt "$2" ] && continue
    if [ $half = AB ]; then PL="$A;$B"; else PL="$C"; fi
    list+=("$(P s6-$i "bash $S/piece.sh s6-$i 3 3G $SW K_SOURCE=$src CUT_LIST=$cut FAMILY=$fam 'SHAPES=$sh' 'K_PLANS=$PL'")"); done; done
  bash "$S/runbatch.sh" "6$1" "${list[@]}"
}
L0() { local name=$1; shift; bash "$S/piece.sh" "$name" 15 6G MODE=k SINGLEFS_LAYER0_THREADS=15 AFTER_J=0 "$@"; grep -h '^KENUM' "$W/out/$name.log" | cut -c1-300; }
run_stage() { case "$1" in
  build-jia) arm jia "$FROZEN" ;;
  build-hi) arm hi "$FROZEN" hi.patch ;;
  build-noq3) arm noq3 "$FROZEN" noq3.patch ;;
  build-rr2) arm rr2 "$FROZEN" rr2.patch ;;
  build-p10c) arm p10c "$PRE10C" ;;
  k2c) bash "$S/piece.sh" k2c-192k9-n8 1 512M MODE=fx FAMILY=list 'SHAPES=192:9:3:1;' 'FX_PLANS=w:but1/all:z;w:all/but1:z;w:but1/all:e;w:all/but1:e;w:-/all:z;w:all/-:z' CUT_LIST=8 AFTER_J=0 SCREEN=1 FX_PRINT=6 ;;
  b1) i=0; L=(); for st in "plainlist 4:30:1:3;" "plainlist 384:30:0:0;" "list 216:7:5:4;" "list 280:4:3:1;"; do i=$((i+1)); L+=("$(P b1-$i "bash $S/piece.sh b1-$i 3 3G MODE=k K_SOURCE=now SCREEN=1 K_SCREEN_AT=mountroots K_UNMOUNT_AFTER=1 AFTER_J=26 K_PRINT=6 FAMILY=${st%% *} 'SHAPES=${st#* }' 'K_PLANS=$C1'")"); done; bash "$S/runbatch.sh" 1 "${L[@]}" ;;
  b2) i=0; L=(); for a in jia hi; do for sh in "384:30:0:0;" "256:30:1:3;"; do i=$((i+1)); L+=("$(P b2-$i "ARM=$a bash $S/piece.sh b2-$i 3 3G MODE=k K_SOURCE=now SCREEN=1 K_SCREEN_AT=roots K_CONTINUE=1 AFTER_J=40 K_PRINT=4 FAMILY=plainlist 'SHAPES=$sh' 'K_PLANS=P5:all/all:e+r144-end:all/-:e:p;P5:all/all:e+r144-end:-/only1:e:p;w:-/only1:e:p;w:only1/-:z:p;w:all/-:e:p;w:-/all:z:p'")"); done; done; bash "$S/runbatch.sh" 2 "${L[@]}" ;;
  b3|b4|b5) case "$1" in b3) PL='clean;w:-/only1:e:p;w:only1/-:e:p;w:-/but1:e:p;w:all/-:e:p'; CY=8; J=20; SH=("384:30:0:0;" "384:12:2:3;");;
      b4) PL='w:-/only1:g:p;w:only1/-:g:p;w:-/but1:g:p;w:all/-:g:p;w:only1/only1:e:p'; CY=6; J=20; SH=("384:30:0:0;" "384:12:2:3;");;
      b5) PL='clean;w:-/only1:g:p;w:only1/-:e:p'; CY=14; J=12; SH=("384:36:0:0;" "384:44:0:0;");; esac
    i=0; L=(); for a in jia noq3; do for sh in "${SH[@]}"; do i=$((i+1)); L+=("$(P $1-$i "ARM=$a bash $S/piece.sh $1-$i 3 3G MODE=k K_SOURCE=now K_CONTINUE=1 K_CYCLES=$CY AFTER_J=$J K_ERRLEN=400 FAMILY=plainlist 'SHAPES=$sh' 'K_PLANS=$PL'")"); done; done; bash "$S/runbatch.sh" "${1#b}" "${L[@]}" ;;
  enospc) bash "$S/piece.sh" k-enospc 3 2G MODE=k K_SOURCE=now K_ERRLEN=3000 FAMILY=plainlist 'SHAPES=384:30:0:0;' 'K_PLANS=w:-/only1:e:p' AFTER_J=16 ;;
  s6-a) s6 1 4 ;; s6-b) s6 5 8 ;; s6-c) s6 9 13 ;; s6-d) s6 14 18 ;; s6-e) s6 19 23 ;; s6-f) s6 24 24 ;;
  rr2) L=(); for k in 11 12 23 24; do sed -e "s#piece.sh s6-$k #piece.sh rr2-$k #" -e 's#^bash #ARM=rr2 bash #' "$W/pieces/s6-$k.sh" > "$W/pieces/rr2-$k.sh"; L+=("$W/pieces/rr2-$k.sh"); done; bash "$S/runbatch.sh" 66 "${L[@]}" ;;
  k2a) bash "$S/piece.sh" k2a 3 2G MODE=k K_SOURCE=now FAMILY=plainlist 'SHAPES=4:30:1:3;' 'K_PLANS=r0-24:-/only1:e+P5:all/all:e;r0-48:-/only1:e+r48-200:all/all:e' AFTER_J=0 ;;
  k2b) bash "$S/piece.sh" k2b 3 2G MODE=k K_SOURCE=now FAMILY=plainlist 'SHAPES=4:30:1:3;' 'K_PLANS=w:all/all:e;P5:all/all:e+P6:all/all:e;w:all/all:z;P5:all/all:z+P6:all/all:z;P5:all/all:g+P6:all/all:g' AFTER_J=0 ;;
  k3-p10c) ARM=p10c bash "$S/piece.sh" k3-p10c 12 3G MODE=k K_SOURCE=unmount CUT_LIST=8 SCREEN=1 K_DETAIL=I-7.9 K_PRINT=3 FAMILY=list 'SHAPES=192:9:3:1;' 'K_PLANS=w:all/but1:z;w:but1/all:z' AFTER_J=0 ;;
  k3-i79) bash "$S/piece.sh" k3-i79 12 3G MODE=k K_SOURCE=now SCREEN=1 K_SCREEN_AT=roots K_DETAIL=I-7.9 K_PRINT=3 FAMILY=plainlist 'SHAPES=256:30:1:3;' 'K_PLANS=w:-/all:z:p;w:-/all:z;w:all/-:z;w:all/-:z:p' AFTER_J=40 K_CONTINUE=1 ;;
  k4) i=0; L=(); for w in 4 384; do for b in 0 1 2 3 4 5 6; do i=$((i+1)); f=$((21+b*40)); [ $w = 384 ] && f=$((1+b*40)); L+=("$(P k4-$i "bash $S/piece.sh k4-$i 1 2G MODE=k4 K4_WEIGHTS=orphans K4_WIDTH=$w K4_STEPS=40 K4_SEED_FROM=$f K4_SEED_TO=$((f+39))")"); done; done; bash "$S/runbatch.sh" 7 "${L[@]}" ;;
  l0-L1) L0 l0-t1 K_SOURCE=now FAMILY=list 'SHAPES=248:4:3:1;' 'K_PLANS=w:but1/all:z' 'K_ENUM=now@w:but1/all:z' ;;
  l0-t8) L0 k-t8 K_SOURCE=now FAMILY=plainlist 'SHAPES=384:30:1:3;' 'K_PLANS=w:-/all:z:p' 'K_ENUM=*@w:-/all:z:p' ;;
  l0-L5) L0 l0-L5 K_SOURCE=now FAMILY=plainlist 'SHAPES=384:12:2:3;' 'K_PLANS=P5:all/all:e+P6:all/-:e' 'K_ENUM=now@P5:all/all:e+P6:all/-:e' ;;
  l0-L6) L0 l0-L6 K_SOURCE=now FAMILY=plainlist 'SHAPES=384:12:2:3;' 'K_PLANS=P5:all/all:z+r144-end:-/all:z:p' 'K_ENUM=now@P5:all/all:z+r144-end:-/all:z:p' ;;
  l0-L7) L0 l0-L7 K_SOURCE=now FAMILY=plainlist 'SHAPES=384:12:2:3;' 'K_PLANS=w:all/-:e' 'K_ENUM=now@w:all/-:e' ;;
  l0-L4a|l0-L4b|l0-L3a|l0-L3b) case "$1" in *a) R="K_SEG_FROM=0 K_SEG_TO=6";; *) R="K_SEG_FROM=6 K_SEG_TO=1000";; esac
    case "$1" in l0-L4*) PL='w:but1/but1:z';; *) PL='P5:all/all:e+P6:-/only1:z';; esac
    L0 "$1" K_SOURCE=now $R FAMILY=list 'SHAPES=216:7:5:4;' "K_PLANS=$PL" "K_ENUM=now@$PL" ;;
  l0-L2a|l0-L2b) case "$1" in *a) R="K_SEG_FROM=0 K_SEG_TO=6";; *) R="K_SEG_FROM=6 K_SEG_TO=1000";; esac
    L0 "$1" K_SOURCE=unmount CUT_LIST=17 $R FAMILY=list 'SHAPES=192:4:3:1;' 'K_PLANS=w:all/-:z' 'K_ENUM=17@w:all/-:z' ;;
  l0-U1a|l0-U1b|l0-U2a|l0-U2b) case "$1" in *a) R="K_SEG_FROM=0 K_SEG_TO=6";; *) R="K_SEG_FROM=6 K_SEG_TO=1000";; esac
    case "$1" in l0-U1*) PL='w:but1/but1:e:p';; *) PL='w:only1/-:g:p';; esac
    L0 "$1" K_SOURCE=now K_UNMOUNT_AFTER=1 K_CONTINUE=1 AFTER_J=26 $R FAMILY=list 'SHAPES=216:7:5:4;' "K_PLANS=$PL" "K_ENUM=now@$PL" ;;
  filter) mkdir -p "$W/out-k"; for f in "$W"/out/*.log; do grep -E '^(K|FX)|LAYER0_FINDINGS|LAYER0_PARALLEL_FINISHED' "$f" | sed -E 's/ elapsed(_seconds)?=[0-9.]+s?//g' > "$W/out-k/$(basename "$f" .log).txt"; done; ls "$W/out-k" | wc -l ;;
  compare) n=0; d=0; for f in "$HERE"/out/*.txt; do n=$((n+1)); g="$W/out-k/$(basename "$f")"; if ! diff -q <(sed -E 's/ elapsed(_seconds)?=[0-9.]+s?//g' "$f") "$g" > /dev/null 2>&1; then d=$((d+1)); echo "差：$(basename "$f")"; fi; done; echo "compare: $n 份，$d 份不同"; [ "$d" = 0 ] ;;
  *) echo "不认得的阶段 $1"; exit 2 ;;
esac; }
STAGES="build-jia build-hi build-noq3 build-rr2 build-p10c k2c b1 b2 b3 b4 b5 enospc s6-a s6-b s6-c s6-d s6-e s6-f rr2 k2a k2b k3-p10c k3-i79 k4 l0-L1 l0-L5 l0-L6 l0-L7 l0-L4a l0-L4b l0-L3a l0-L3b l0-L2a l0-L2b l0-U1a l0-U1b l0-U2a l0-U2b filter"
case "$STAGE" in list) echo $STAGES ;; *) run_stage "$STAGE" ;; esac
