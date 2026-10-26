#!/usr/bin/env bash
# m3-w3-r2 辩方腿（Sonnet）的复跑。用法：SINGLEFS_REPO=<仓根> bash rerun.sh <工作目录> <段>
# 段（每段一条前台命令，4 分钟以内；编译与跑都经 scripts/hosts/run-with-memory-cap.sh，编译 6G、跑 4G）：
#   derive        从攻方腿的模型目录（$SINGLEFS_REPO/research/prompts/m3-w3-r1-opus-model）按 tools/derivation 里的五个补丁脚本重新生成
#                 timeline-model/src/main.rs、w3r2_module.rs、make_device.py，与本目录里存的逐字节比（cmp）
#   build-model   编抽象时间线模型（不依赖 crates/）
#   build-P       拷冻结树（FROZEN_TREE，默认 ~/.local/share/singlefs/work/m3-w3-r1/tree）进 <工作目录>/repo、核清单、生成原型装置、编 P 臂
#   build-V1      施加 E175 的 patch-V1.diff、编 V1 臂、还原、再核清单
#   quick         两臂装置快档（P、V1 并行）
#   model-main / model-defense / model-mutation / model-ringwin / model-replay   抽象模型各组
#   model-deep    深一层（长 8、字母表 8 个）：P 臂 0 张误报、V1 臂 C4g 漏 0
#   dev-g-long-v1-idx24   G-long V1 第 24 段逐步打印（两个 Gone 事件那一段）
#   dev-g-long-v1 / dev-f2-v1 / dev-g-long-p / dev-f2-p-1 / dev-f2-p-2   装置全量（V1 臂事件与持久镜像，P 臂误报）
#   compare       把 <工作目录>/out 与 expected/ 比（模型输出去掉 W3TRACE 行排序后比；装置输出比 summarize.py 的摘要）
#   expect        （只在生成 expected 时用）把 <工作目录>/out 整理成 expected/
set -euo pipefail
W=${1:?工作目录}; PART=${2:?段}
R=${SINGLEFS_REPO:?设 SINGLEFS_REPO 为仓根}
M=$(cd "$(dirname "$0")" && pwd)
FROZEN=${FROZEN_TREE:-$HOME/.local/share/singlefs/work/m3-w3-r1/tree}
E=$R/research/results/e175-product-path-inode-watermark-regression-2026-10-08
O=$R/research/prompts/m3-w3-r1-opus-model
CAP="bash $R/scripts/hosts/run-with-memory-cap.sh"
THREADS="bash $R/scripts/hosts/capped.sh 6"
mkdir -p "$W/out" "$W/bin"
export SINGLEFS_REPO
model() { nice -n 19 $CAP 4G "$W/target-model/release/w3r2-timeline-model" "$@"; }
# 并行跑一组模型：model_group <组> <起> <止> <文件前缀>；退出码写 out/<前缀>-<i>.exit，等齐后有非 0 退 1
model_group() {
  local group=$1 lo=$2 hi=$3 prefix=$4 i bad=0
  for i in $(seq "$lo" "$hi"); do
    rm -f "$W/out/$prefix-$i.exit"
    { model "$group" "$i" > "$W/out/$prefix-$i.out" 2> "$W/out/$prefix-$i.err"; echo "$?" > "$W/out/$prefix-$i.exit"; } &
  done
  wait
  for i in $(seq "$lo" "$hi"); do [ "$(cat "$W/out/$prefix-$i.exit" 2>/dev/null)" = 0 ] || bad=1; done
  echo "组 $group $lo..$hi 有非 0=$bad"; return $bad
}
cargo_exe() { # 从 cargo 的 json 消息里取装置可执行文件
  python3 -c 'import sys,json
for l in sys.stdin:
    try: m=json.loads(l)
    except Exception: continue
    if m.get("reason")=="compiler-artifact" and m.get("target",{}).get("name")=="w3_attack_candidates" and m.get("executable"): print(m["executable"])'
}
build_device() {
  (cd "$W/repo" && CARGO_TARGET_DIR=$W/target nice -n 19 $THREADS $CAP 6G cargo test --release --offline -p singlefs-harness \
    --test w3_attack_candidates --no-run --message-format=json 2>>"$W/build.log") | cargo_exe
}
case "$PART" in
  derive)
    D=$W/derive; mkdir -p "$D"
    python3 "$M/tools/derivation/patch_model.py" "$O/timeline-model/src/main.rs" "$D/m1.rs"
    python3 "$M/tools/derivation/patch_model2.py" "$D/m1.rs" "$D/m2.rs"
    python3 "$M/tools/derivation/patch_model3.py" "$D/m2.rs" "$D/m3.rs"
    python3 "$M/tools/derivation/patch_model4.py" "$D/m3.rs" "$D/main.rs"
    python3 "$M/tools/derivation/patch_module.py" "$O/w3_module.rs" "$D/w3r2_module.rs"
    python3 "$M/tools/derivation/mk_make_device.py" "$O/make_device.py" "$D/make_device.py"
    cmp "$D/main.rs" "$M/timeline-model/src/main.rs" && cmp "$D/w3r2_module.rs" "$M/w3r2_module.rs" && cmp "$D/make_device.py" "$M/make_device.py"
    echo "derive 三份与本目录逐字节相同" ;;
  build-model)
    rsync -a --exclude target "$M/timeline-model/" "$W/timeline-model/"
    (cd "$W/timeline-model" && CARGO_TARGET_DIR=$W/target-model nice -n 19 $THREADS $CAP 6G cargo build --release --offline 2>>"$W/build.log")
    sha256sum "$W/target-model/release/w3r2-timeline-model" ;;
  build-P)
    [ ! -e "$W/repo" ] || { echo "工作目录里已有 repo：$W/repo" >&2; exit 2; }
    rsync -a --exclude target --exclude .git "$FROZEN/" "$W/repo/"
    (cd "$W/repo" && sha256sum -c --quiet "$R/research/prompts/m3-w3-r1-snapshot/tree-sha256.txt")
    python3 "$M/make_device.py" "$R/scripts/experiment-apparatus/e175_product_path_inode_watermark_regression/device-source.rs" "$W/repo/crates/singlefs-harness/tests/w3_attack_candidates.rs"
    exe=$(build_device); cp "$exe" "$W/bin/w3-P"
    cp "$W/repo/crates/singlefs-core/src/mount.rs" "$W/mount.rs.pristine"
    sha256sum "$W/bin/w3-P" ;;
  build-V1)
    patch -s "$W/repo/crates/singlefs-core/src/mount.rs" < "$E/patch-V1.diff"
    exe=$(build_device); cp "$exe" "$W/bin/w3-V1"
    cp "$W/mount.rs.pristine" "$W/repo/crates/singlefs-core/src/mount.rs"
    (cd "$W/repo" && sha256sum -c --quiet "$R/research/prompts/m3-w3-r1-snapshot/tree-sha256.txt" && echo "冻结树副本已还原")
    sha256sum "$W/bin/w3-V1" ;;
  quick)
    for arm in P V1; do
      { (cd "$W" && E175_ARM=$arm nice -n 19 $THREADS $CAP 4G "$W/bin/w3-$arm" --test-threads 6 2>&1 | grep '^test result: ' > "$W/out/quick-$arm.txt"); } &
    done
    wait; cat "$W/out/quick-P.txt" "$W/out/quick-V1.txt" ;;
  model-main) model_group main 0 5 model-main ;;
  model-defense) model_group defense 0 5 model-defense ;;
  model-mutation) { model_group mutation 0 5 model-mutation & model_group ring6 0 1 model-ring6 & wait; } ;;
  model-ringwin) bash "$M/tools/ringwin.sh" "$W/target-model/release/w3r2-timeline-model" "$W/out" 4:6 4:7 4:8 6:8 6:9 6:10 8:10 8:11 ;;
  model-replay)
    E23=$(printf 'empty,%.0s' $(seq 1 23)); E24=$(printf 'empty,%.0s' $(seq 1 24))
    model replay V1 24 "create,rollback1,${E23}mount,create" > "$W/out/model-replay-ring24-event.out"
    model replay V1 24 "create,rollback1,create,create,${E24%,}" > "$W/out/model-replay-ring24-persist.out"
    echo "replay 两条跑完" ;;
  model-deep)
    # 深一层（长 8）、字母表收窄到 8 个（含 delete 与墓碑、crash-hide、switch-redo）：P 臂看误报，V1 臂看漏报
    for cfg in 0 2; do
      { W3_OPS=create,delete,rollback1,rollback2,raiseF,mount,crash-hide,switch-redo W3_LONGEST=8 model main "$cfg" > "$W/out/model-deep-$cfg.out" 2> "$W/out/model-deep-$cfg.err"; echo "$?" > "$W/out/model-deep-$cfg.exit"; } &
    done
    wait
    cat "$W/out/model-deep-0.exit" "$W/out/model-deep-2.exit" | tr "\n" " "; echo ;;
  dev-g-long-v1-idx24)
    rm -f "$W/out/dev-G-long-V1-idx24.out"
    bash "$M/tools/dev_run.sh" "$W/bin" V1 G-long "$W/out/dev-G-long-V1-idx24.out" 24 24 ;;
  dev-g-long-v1) bash "$M/tools/par.sh" "$W/bin" "$W/out" V1:G-long:0:15 V1:G-long:16:31 ;;
  dev-f2-v1) bash "$M/tools/par.sh" "$W/bin" "$W/out" V1:F2:0:31 V1:F2:32:63 ;;
  dev-g-long-p) bash "$M/tools/par.sh" "$W/bin" "$W/out" P:G-long:0:15 P:G-long:16:31 P:G-long:32:47 P:G-long:48:63 ;;
  dev-f2-p-1) bash "$M/tools/par.sh" "$W/bin" "$W/out" P:F2:0:31 P:F2:32:63 P:F2:64:95 P:F2:96:127 ;;
  dev-f2-p-2) bash "$M/tools/par.sh" "$W/bin" "$W/out" P:F2:128:159 P:F2:160:191 P:F2:192:223 P:F2:224:255 ;;
  compare) bash "$M/compare.sh" "$M/expected" "$W/out" ;;
  expect) bash "$M/compare.sh" --make "$M/expected" "$W/out" ;;
  *) echo "不认的段：$PART" >&2; exit 2 ;;
esac
echo "段 $PART 跑完"
