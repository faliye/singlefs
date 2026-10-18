#!/usr/bin/env bash
# 这一轮正推腿复跑要跑的步骤（被 rerun.sh 的 rerun_steps 读进去，也能单独读进来分步跑）。
# 读它之前要先设好 REPO_ROOT、FROZEN、MODEL、WORK 四个变量（rerun.sh 的 parse_arguments 设）。
# 路径只用 $REPO_ROOT、$FROZEN、$MODEL、$WORK 四个变量引，不写死、不从 $0 推；cargo 经 "$REPO_ROOT/scripts/hosts/capped.sh" 起，
# 输出写进 $WORK，最后打出与第二轮入库的汇总哪一份逐行比。
# 第二轮两腿的模型、补丁、驱动脚本不拷进这一份模型目录，直接读仓里 research/prompts/c834-r2-opus-model/ 与 c834-r2-sonnet-model/（只读）。
# 环境变量：RERUN_THREADS（线程数，默认 31）、RERUN_TARGET_DIR（编译目录，默认 $WORK/target；librocksdb-sys 现编很慢，多次复跑可指同一个）、
# RERUN_STEPS（空格分隔的步名，默认 all；分步跑时每步前台 4 分钟内：
#   prepare build_opus build_sonnet opus_nine opus_batch:<0..5> sonnet_batch:<0..9> summarize）。
c834r3_run_steps() {
  local cap="$REPO_ROOT/scripts/hosts/capped.sh" threads="${RERUN_THREADS:-31}"
  local prompts="$REPO_ROOT/research/prompts"
  local opus_model="$prompts/c834-r2-opus-model" sonnet_model="$prompts/c834-r2-sonnet-model"
  local target="${RERUN_TARGET_DIR:-$WORK/target}" steps="${RERUN_STEPS:-all}"
  local bindgen_arguments
  bindgen_arguments="$(sed -n 's/^CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS="\([^"]*\)".*/\1/p' "$REPO_ROOT/scripts/crash/crash-amplification-lib.sh")"
  [[ -n "$bindgen_arguments" ]] || { echo "name=rerun_failed reason=no_bindgen_arguments"; return 1; }
  export BINDGEN_EXTRA_CLANG_ARGS="$bindgen_arguments"
  mkdir -p "$WORK/runs-opus" "$WORK/runs-sonnet" "$WORK/out"
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
  wants() { [[ "$steps" == all ]] || [[ " $steps " == *" $1 "* ]]; }

  if wants prepare; then
    local leg
    for leg in opus sonnet; do
      # 已经装过候选开关的树不再装一遍（补丁脚本对同一棵树只许装一次）：分步跑、接着跑都走得通
      if ! grep -q 'c834_candidate_says_refuse' "$WORK/$leg/tree-candidates/crates/singlefs-core/src/mount.rs" 2> /dev/null; then
        mkdir -p "$WORK/$leg/tree-candidates"
        rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/$leg/tree-candidates/"
        python3 "$prompts/c834-r2-$leg-model/patch/apply_candidates.py" "$WORK/$leg/tree-candidates"
      fi
      rsync -a --exclude target --exclude out "$prompts/c834-r2-$leg-model/" "$WORK/$leg/model/"
    done
    python3 "$MODEL/tools/pieces.py" opus 600 > "$WORK/pieces-opus.txt"
    python3 "$MODEL/tools/pieces.py" sonnet 600 > "$WORK/pieces-sonnet.txt"
    echo "name=rerun_prepared pieces_opus=$(wc -l < "$WORK/pieces-opus.txt") pieces_sonnet=$(wc -l < "$WORK/pieces-sonnet.txt")"
  fi
  if wants build_opus; then
    (cd "$WORK/opus/model" && nice -n 19 bash "$cap" "$threads" cargo build --release --target-dir "$target")
  fi
  if wants build_sonnet; then
    (cd "$WORK/sonnet/model" && nice -n 19 bash "$cap" "$threads" cargo build --release --target-dir "$target")
  fi
  local opus_binary="$target/release/c834-r2-attack-model" sonnet_binary="$target/release/c834-attack-model"
  if wants opus_nine; then
    nice -n 19 bash "$cap" "$threads" "$opus_binary" nine > "$WORK/runs-opus/nine.txt"
  fi
  local batch batches
  for ((batch = 0; batch < $(( ($(wc -l < "$WORK/pieces-opus.txt" 2> /dev/null || echo 0) + threads - 1) / threads )); batch++)); do
    if wants "opus_batch:$batch"; then
      bash "$cap" "$threads" bash "$opus_model/tools/run-batch.sh" "$opus_binary" "$WORK/pieces-opus.txt" "$batch" "$threads" "$WORK/runs-opus" > "$WORK/runs-opus/batch-$batch.log"
      if grep -q 'rc=[1-9]' "$WORK/runs-opus/batch-$batch.log" || ! grep -q 'launched=' "$WORK/runs-opus/batch-$batch.log"; then
        echo "name=rerun_piece_failed leg=opus batch=$batch"
        return 1
      fi
      echo "name=rerun_batch leg=opus batch=$batch $(head -n 1 "$WORK/runs-opus/batch-$batch.log")"
    fi
  done
  for ((batch = 0; batch < $(( ($(wc -l < "$WORK/pieces-sonnet.txt" 2> /dev/null || echo 0) + threads - 1) / threads )); batch++)); do
    if wants "sonnet_batch:$batch"; then
      bash "$cap" "$threads" bash "$sonnet_model/tools/run-batch-faults.sh" "$sonnet_binary" "$WORK/pieces-sonnet.txt" "$batch" "$threads" "$WORK/runs-sonnet" > "$WORK/runs-sonnet/batch-$batch.log"
      if grep -q 'rc=[1-9]' "$WORK/runs-sonnet/batch-$batch.log" || ! grep -q 'launched=' "$WORK/runs-sonnet/batch-$batch.log"; then
        echo "name=rerun_piece_failed leg=sonnet batch=$batch"
        return 1
      fi
      echo "name=rerun_batch leg=sonnet batch=$batch $(head -n 1 "$WORK/runs-sonnet/batch-$batch.log")"
    fi
  done
  if wants summarize; then
    local status=0
    python3 "$opus_model/tools/aggregate.py" "$WORK"/runs-opus/*.out > "$WORK/out/opus-aggregate.txt"
    python3 "$sonnet_model/tools/aggregate.py" "$WORK"/runs-sonnet/*.out > "$WORK/out/sonnet-aggregate.txt"
    python3 "$MODEL/tools/compare.py" opus "$WORK/out/opus-aggregate.txt" "$opus_model/out/agg-first.txt" "$opus_model/out/agg-three.txt" "$opus_model/out/agg-partial.txt" > "$WORK/out/compare-opus.txt" || status=1
    diff "$opus_model/out/nine.txt" "$WORK/runs-opus/nine.txt" > "$WORK/out/compare-nine.txt" || status=1
    python3 "$MODEL/tools/compare.py" sonnet "$WORK/out/sonnet-aggregate.txt" "$sonnet_model/out/aggregate-all.txt" > "$WORK/out/compare-sonnet.txt" || status=1
    python3 "$MODEL/tools/tail-compare.py" "$sonnet_model/tools/aggregate.py" "$sonnet_model/out/aggregate-mapping-tail.txt" "$WORK"/runs-sonnet/*.out > "$WORK/out/compare-tail.txt" || status=1
    python3 "$MODEL/tools/flags.py" "$sonnet_model/out/aggregate-all.txt" "$WORK"/runs-sonnet/*.out > "$WORK/out/flags.txt" || status=1
    python3 "$MODEL/tools/middle.py" "$sonnet_model/tools/aggregate.py" "$WORK"/runs-sonnet/*.out > "$WORK/out/mapping-middle.txt"
    python3 "$MODEL/tools/cells.py" opus pointer_per_kind "$WORK/out/opus-aggregate.txt" > "$WORK/out/cells-opus-pointer.txt"
    python3 "$MODEL/tools/cells.py" sonnet mapping_root_only "$WORK/out/sonnet-aggregate.txt" > "$WORK/out/cells-sonnet-mapping.txt"
    python3 "$MODEL/tools/cells.py" sonnet pointer_slots_fixed "$WORK/out/sonnet-aggregate.txt" > "$WORK/out/cells-sonnet-pointer.txt"
    python3 "$MODEL/tools/tally.py" opus "$WORK"/runs-opus/b*.out > "$WORK/out/tally.txt"
    python3 "$MODEL/tools/tally.py" sonnet "$WORK"/runs-sonnet/b*.out >> "$WORK/out/tally.txt"
    local name
    for name in compare-opus compare-sonnet compare-tail; do
      echo "name=rerun_compare $name $(head -n 1 "$WORK/out/$name.txt")"
    done
    echo "name=rerun_compare compare-nine $([[ -s "$WORK/out/compare-nine.txt" ]] && echo differs || echo same)"
    grep -E 'pair_total|pointer_vs_reference' "$WORK/out/flags.txt" || true
    # 这一轮入库的 out/ 里留着的汇总：本次复跑要逐行同（例子序号去掉，辩方汇总里先到先得）
    local mine
    for mine in sonnet-aggregate flags tally cells-opus-pointer cells-sonnet-mapping cells-sonnet-pointer mapping-middle; do
      if [[ -f "$MODEL/out/$mine.txt" ]]; then
        if diff <(sed -E 's/ example_ordinal=[0-9]+//' "$MODEL/out/$mine.txt") <(sed -E 's/ example_ordinal=[0-9]+//' "$WORK/out/$mine.txt") > "$WORK/out/$mine.diff"; then
          echo "name=rerun_compare $mine same"
        else
          echo "name=rerun_compare $mine differs（见 $WORK/out/$mine.diff）"
          status=1
        fi
      fi
    done
    return "$status"
  fi
}
