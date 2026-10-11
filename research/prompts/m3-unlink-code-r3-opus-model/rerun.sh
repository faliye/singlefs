#!/usr/bin/env bash
# m3-unlink-code-r3 攻方腿（Opus）复跑。用法：ATTACK_DIR=<空目录> bash rerun.sh
# 在冻结副本的三份拷贝上编原型（丙1 原样 / 改之前那一版 / 零轮改法 G-a），按报告里的批次逐批跑，日志进 $ATTACK_DIR/out/。
# 只跑 harness 档与自己的原型 crate（不带 layer0 的名字、不是 checker 档包）；每一批前台跑得完（每批 1–4 分钟，线程上限 10）。
set -eu
REPO=${SINGLEFS_REPO:-/home/fy5090/code/singlefs}
MODEL=$REPO/research/prompts/m3-unlink-code-r3-opus-model
FROZEN=${FROZEN_TREE:-/tmp/claude-1000/m3-unlink-code-r3/tree}
export ATTACK_DIR=${ATTACK_DIR:?给一个空的草稿目录}
mkdir -p "$ATTACK_DIR/out"
cd "$ATTACK_DIR"
for variant in tree tree-held tree-v2; do
  rsync -a --exclude target --exclude .git "$FROZEN/" "$variant/"
  (cd "$variant" && patch -p1 < "$MODEL/patches/workspace-member.patch")
  mkdir -p "$variant/crates/attack-r3-probe/src"
  cp "$MODEL/probe/Cargo.toml" "$variant/crates/attack-r3-probe/Cargo.toml"
  cp "$MODEL/probe/main.rs" "$variant/crates/attack-r3-probe/src/main.rs"
done
(cd tree-held && patch -p1 < "$MODEL/patches/held-before-bing1.patch")
(cd tree-v2 && patch -p1 < "$MODEL/patches/ga-reuse-only-below-every-newest-root.patch")
cp "$MODEL"/scripts/*.sh "$MODEL"/scripts/compare.py .
bash build.sh "$ATTACK_DIR/tree"; bash build-held.sh "$ATTACK_DIR/tree-held"; bash build-v2.sh "$ATTACK_DIR/tree-v2"
IMM=$ATTACK_DIR/target/release/attack-r3-probe; HELD=$ATTACK_DIR/target-held/release/attack-r3-probe; GA=$ATTACK_DIR/target-v2/release/attack-r3-probe
# 一、X 第 1404 行那一份盘面（256 槽、覆盖写 2、14 单元、崩在第 1 批之后）：前缀刀加两种丢盘的 checker 原文
FAMILY=list SHAPES="256:2:14:1;" ENUM=0 CHECK_ALL=1 REMOUNT_CUTS=0 DETAIL_CUTS=0,2,14 bash run.sh "$IMM" out/x-imm-detail.log
FAMILY=list SHAPES="256:2:14:1;" ENUM=0 CHECK_ALL=1 REMOUNT_CUTS=0 DETAIL_CUTS=0,2,14 bash run.sh "$HELD" out/x-held-detail.log
# 二、窄盘网格（单元区 176–352 槽、覆盖写 0–10、文件 1/2/4/8/14 单元、崩在第 0/1/3/7/13 批之后），三种实现各一遍
export ENUM=0 CHECK_ALL=0 REMOUNT_CUTS=0 DETAIL_CUTS= FAMILY=narrow OVERWRITES=0,1,2,3,4,5,6,7,8,9,10 UNITS=1,2,4,8,14 BATCHES=0,1,3,7,13
for pair in "$IMM:sweep-narrow" "$HELD:sweep-narrow-held" "$GA:sweep-narrow-v2"; do
  bash sweep-batch.sh 1 "${pair%%:*}" "${pair#*:}" SLOTS 176 192 208 224 240 256
  bash sweep-batch.sh 2 "${pair%%:*}" "${pair#*:}" SLOTS 272 288 304 320 336 352
done
# 三、交集起点逐刀开 checker 与再挂，丙1 与改之前那一版同刀比
awk '/^START /{match($0,/U=[0-9]+ k=[0-9]+ u=[0-9]+ b=[0-9]+/); s=substr($0,RSTART,RLENGTH)} /^INTERSECTION_BY_RECORDS/{print s}' out/sweep-narrow-[0-9]*.log | sort -u | sed -E 's/U=([0-9]+) k=([0-9]+) u=([0-9]+) b=([0-9]+)/\1:\2:\3:\4/' > hit-shapes.txt
split -n l/20 -d hit-shapes.txt hitpiece.; for f in hitpiece.??; do tr '\n' ';' < "$f" > "$f.joined"; done
export CHECK_ALL=1 REMOUNT_CUTS=1
for half in 0 1; do bash hit-batch.sh 5 "$IMM" hits-imm $half; bash hit-batch.sh 6 "$HELD" hits-held $half; done
python3 compare.py 'out/hits-imm-*.log' 'out/hits-held-*.log'
# 四、层 0 枚举域（每段展开）：h2 = 176:0:4:3，h3 = 176:5:1:1，X 那一份
export CHECK_ALL=0 REMOUNT_CUTS=0 ENUM=1 MAX_STATES=1000000 FAMILY=list SINGLEFS_LAYER0_THREADS=8
SHAPES="176:0:4:3;" bash run.sh "$IMM" out/enum-h2-imm.log; SHAPES="176:0:4:3;" bash run.sh "$HELD" out/enum-h2-held.log
SHAPES="176:5:1:1;" bash run.sh "$IMM" out/enum-h3-imm.log; SHAPES="176:5:1:1;" bash run.sh "$HELD" out/enum-h3-held.log
SHAPES="256:2:14:1;" bash run.sh "$IMM" out/enum-x-imm.log
# 五、卸载—再挂循环（4 GiB 那一处 h4 与两族网格）
export ENUM=0
FAMILY=cycle WIDTHS=4 OVERWRITES=0 REMOUNTS=1 OVERWRITES_AFTER=0 OVERWRITES_AFTER_UNMOUNT=2 CHECK_ALL=1 REMOUNT_CUTS=1 DETAIL_CUTS=18,22 bash run.sh "$IMM" out/h4-imm.log
FAMILY=cycle WIDTHS=4 OVERWRITES=0 REMOUNTS=1 OVERWRITES_AFTER=0 OVERWRITES_AFTER_UNMOUNT=2 CHECK_ALL=1 REMOUNT_CUTS=1 DETAIL_CUTS=18,22 bash run.sh "$HELD" out/h4-held.log
for pair in "$IMM:sweep-cycle" "$HELD:sweep-cycle-held" "$GA:sweep-cycle-v2"; do
  FAMILY=cycle OVERWRITES=0,2,5,9,14 REMOUNTS=0,1 OVERWRITES_AFTER=0,2 OVERWRITES_AFTER_UNMOUNT=0,1,2,4,8 CHECK_ALL=0 bash sweep-batch.sh 3 "${pair%%:*}" "${pair#*:}" WIDTHS 384 256 240 4
  FAMILY=ncycle OVERWRITES=0,2,5,8 UNITS=1,4,14 BATCHES=0,1,3 OVERWRITES_AFTER_UNMOUNT=0,1,2,4 CHECK_ALL=0 bash sweep-batch.sh 4 "${pair%%:*}" "${pair#*:}" SLOTS 176 208 240 272 304 336
done
FAMILY=plain OVERWRITES=0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15 REMOUNTS=0,1,2 OVERWRITES_AFTER=0,1,3 CHECK_ALL=0 bash sweep-batch.sh 7 "$IMM" sweep-plain WIDTHS 384 256 240 4
for g in 'sweep-narrow-[0-9]*' sweep-narrow-held- sweep-narrow-v2- 'sweep-ncycle-[0-9]*' sweep-ncycle-held- sweep-ncycle-v2- 'sweep-cycle-[0-9]*' sweep-cycle-held- sweep-cycle-v2- sweep-plain-; do echo "$g: $(bash summarize.sh out/$g*.log)"; done
# 六、N2 注错扫（窄盘网格挂载、只崩了再挂的窄盘）与 N5 取界、N7 映射树上界
export N2_FAMILY=narrow UNITS=1,4,14 BATCHES=0,1,3
OVERWRITES=0,3 bash n2-batch.sh 8 "$IMM" 176 208 240 272 304 336; OVERWRITES=6,9 bash n2-batch.sh 9 "$IMM" 176 208 240 272 304 336
bash n2sum.sh out/n2-target-*.log
MODE=n2 N2_FAMILY=remount WIDTHS=240 REMOUNTS=6,7,8,9,10,12 bash run.sh "$IMM" out/n2-remount-240.log; bash n2sum.sh out/n2-remount-240.log
for u in 176 192 208 224 240 256 272 288 304 320; do MODE=n5 SLOTS=$u OVERWRITES=0,1,2,3,4,5,6,7,8,9,10 UNITS=1,2,4,8,14 BATCHES=0,1,3,7,13 bash run.sh "$IMM" out/n5-$u.log; done
for i in $(seq 1 10); do MODE=n7 WIDTH=4 OPS=14 MAX_UNITS=1500 SEEDS=$(seq -s, $(( (i-1)*30 + 1 )) $(( i*30 ))) bash run.sh "$IMM" out/n7-4g-$i.log; done
# 七、零轮改法 G-a 下入库的两份 harness 用例
(cd tree-v2 && CARGO_TARGET_DIR=$ATTACK_DIR/target-v2 RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="副本上跑 harness 测试目标" nice -n 19 bash "$REPO/scripts/run-with-memory-cap.sh" 4G bash "$REPO/scripts/capped.sh" 10 cargo test --release --offline -p singlefs-harness --test admission_raises_the_floor_before_refusing --test unlink_mount_continuation_and_the_emptied_version) > out/v2-tests.log 2>&1
