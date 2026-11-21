#!/usr/bin/env bash
# admission: always 被 scripts/experiment-apparatus/replay.sh 在同目录 source 进去的那一段，判的是此刻的仓与这一趟的参数，上一次的结论不替这一次作保
# run-condition: none 不单独执行：由 replay.sh 的 source 载入，环境要求同 replay.sh（文件头「GPU」「编译目录」）
# replay-row-library.sh：replay.sh 的一段（按行复用的判定库：行键与记录的拉取、E182 片结果库、记判定、开跑前那一判（decide_row））。含载入时就执行的语句（replay.sh 载入次序里这一段的顺序流程），接着 source 它的那一行往下走；exit 与 replay.sh 里的 exit 一样退出整趟。
# 这一段按 replay.sh 里 source 它的那一行的位置载入。文件头的用法、复跑表、弄坏开关表、退出码都在 replay.sh。

# ── 按行复用（文件头「按行复用」）：先算全部行的行键、定哪几行要现跑，再决定编不编 ──────────────────────────────────
declare -A ROW_REUSE_KEY=() ROW_CHANGE_NOTE=() ROW_REFUSAL=() ROW_USES_GPU=()
REPLAY_ROW_LIBRARY=""
REPLAY_ROW_RECORDS=""
# 判定库服务不可用（文件头「按行复用」）：点名退了 75 的那一条，整趟退 75，不退回本地、不当库里没有现跑
replay_prune_server_unavailable() { # <退 75 的那一条命令>
  echo "  ✗ 按行复用：prune-server 不可用（$1 退 75）" >&2
  echo "     → 怎么办：python3 prune-server/control check 看服务，修好、重起服务之后重交这一趟（报给主 agent）；这一趟不退回本地文件、不当库里没有现跑" >&2
  exit 75
}
# 按行复用的库没法用（文件头「按行复用」）：开跑之前整趟退出（不在 git 工作树里退 2，别的退 1），不照「不复用、不记」全跑。
# 弄坏开关 row-library-failure-tolerated 下只打一句、往下全跑
replay_row_library_failure() { # <退出码> <一句为什么> <出路>
  if [[ "$REPLAY_BROKEN" == row-library-failure-tolerated ]]; then
    echo "  ! 按行复用：$2：这一趟一行都不复用、也不记" >&2
    REPLAY_ROW_LIBRARY=""; ROW_REUSE_KEY=()
    return 0
  fi
  echo "  ✗ 按行复用：$2，一行都没跑（退 $1）" >&2
  echo "     → 怎么办：$3" >&2
  exit "$1"
}
# 库名：这棵树的 git common-dir 下那一份，只当名字（键里只用种类名，命名空间只在临时服务底下给；文件头「按行复用」），本地不建
if replay_common_directory="$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" && [[ -n "$replay_common_directory" ]]; then
  REPLAY_ROW_LIBRARY="$replay_common_directory/singlefs-unit-verdicts/replay-row"
  REPLAY_ROW_RECORDS="$OUT_DIR/row-records-from-prune-server"                      # 这一趟的中转目录：replay_finish 收尾删
else
  replay_row_library_failure 2 "$PWD 不是 git 工作树，判定库没有名字" "在 git 工作树里跑；rsync 拷的仓副本先在副本根 git init（键里不带哪棵仓，副本与主仓读写同一份库）"
fi
read_gpu_switch "$OUT_DIR/gpu-switch.log" || exit 78
mkdir -p "$OUT_DIR/compositions"
rm -f -- "$OUT_DIR/prune-server-unavailable"   # remember_row_verdict 记的服务不可用标记：同一个 REPLAY_OUT 里上一趟留下的不算这一趟的
row_inputs_exit=0
if [[ -z "$REPLAY_ROW_LIBRARY" ]]; then
  :   # 只在弄坏开关 row-library-failure-tolerated 下走到（改错的样子：不复用、不记）
elif row_inputs_text="$(python3 "$ADMISSION_MODULE" replay-row-inputs . --gpu "$REPLAY_GPU_SWITCH" --library "$REPLAY_ROW_LIBRARY" --compositions "$OUT_DIR/compositions" ${ONLY[@]+"${ONLY[@]}"})" \
     || { row_inputs_exit=$?; false; }; then
  while IFS=$'\t' read -r row_identity row_reuse_key _row_units row_uses_gpu row_note; do
    [[ -n "$row_identity" ]] || continue
    ROW_USES_GPU["$row_identity"]="$row_uses_gpu"   # 这一行要不要卡（点到 CHECKER_TIER_FEATURES），拒算的行也记（文件头「GPU」）
    if [[ "$REPLAY_BROKEN" == reuse-key-by-row-only ]]; then   # 弄坏开关：键只按复跑表那一行算，不看装置源码与工具链
      row_reuse_key="$(printf '%s' "$row_identity" | sha256sum)"; row_reuse_key="${row_reuse_key%% *}"; row_note=""
    fi
    if [[ "$row_reuse_key" == refused ]]; then
      ROW_REFUSAL["$row_identity"]="$row_note"
      continue
    fi
    ROW_REUSE_KEY["$row_identity"]="$row_reuse_key"
    ROW_CHANGE_NOTE["$row_identity"]="$row_note"
  done <<<"$row_inputs_text"
elif (( row_inputs_exit == 75 )) && [[ "$REPLAY_BROKEN" != prune-server-failure-swallowed ]]; then
  replay_prune_server_unavailable "python3 $ADMISSION_MODULE replay-row-inputs . --library $REPLAY_ROW_LIBRARY"
else
  printf '%s\n' "$row_inputs_text" >&2
  replay_row_library_failure 1 "每一行的行键算不出（python3 $ADMISSION_MODULE replay-row-inputs . 退 $row_inputs_exit，原因在上面）" \
    "单跑 python3 $ADMISSION_MODULE replay-row-inputs . 照它报的修（登记表、工具链、复跑表那几行）"
fi
# 这一趟要的那几把键的记录一次拉进中转目录（文件头「按行复用」）：开跑前那一判与复用哨兵读它
if (( ${#ROW_REUSE_KEY[@]} )); then
  rm -rf -- "${REPLAY_ROW_RECORDS:?}"
  if ! mkdir -p -- "$REPLAY_ROW_RECORDS/keys" "$REPLAY_ROW_RECORDS/items"; then
    replay_row_library_failure 1 "建不了中转目录 $REPLAY_ROW_RECORDS" "看 REPLAY_OUT 那个目录能不能写、盘满没满"
  else
    mapfile -t replay_fetch_prefixes < <(printf 'keys/%s/\n' "${ROW_REUSE_KEY[@]}" | sort -u)
    if replay_fetch_output="$(python3 "$ADMISSION_MODULE" library-files fetch "$REPLAY_ROW_LIBRARY" "$REPLAY_ROW_RECORDS" "${replay_fetch_prefixes[@]}" 2>&1)"; then
      replay_fetch_exit=0
    else
      replay_fetch_exit=$?
    fi
    if (( replay_fetch_exit == 75 )) && [[ "$REPLAY_BROKEN" != prune-server-failure-swallowed ]]; then
      printf '%s\n' "$replay_fetch_output" >&2
      replay_prune_server_unavailable "python3 $ADMISSION_MODULE library-files fetch $REPLAY_ROW_LIBRARY"
    elif (( replay_fetch_exit != 0 )); then
      printf '%s\n' "$replay_fetch_output" >&2
      replay_row_library_failure 1 "判定库里的记录拉不下来（python3 $ADMISSION_MODULE library-files fetch 退 $replay_fetch_exit，原因在上面）" \
        "照上面的报错修 admission.py library-files（文件头「library-files」）"
    fi
  fi
fi

# E182 的片结果库（文件头「E182 的片结果库」）：这一趟点到 driver_e182 或 driver_e182_units_kept 的行，开跑前把两种片拉进各自的中转目录，
#   各行收齐之后 replay_store_e182_shards 并回
REPLAY_E182_SHARD_KINDS=(e182-single-device e182-single-device-units-kept)
REPLAY_E182_SHARDS=""   # 拉过才设：$OUT_DIR/e182-shards-from-prune-server（底下 <种类>/ 一种一份）
replay_e182_wanted=0
while IFS='|' read -r exp bin args _stored _kind; do
  [[ -n "$exp" ]] && want_row "$exp" "$bin" "$args" && [[ "$bin" == @driver_e182 || "$bin" == @driver_e182_units_kept ]] && replay_e182_wanted=1
done <<<"$TABLE"
if (( replay_e182_wanted )) && [[ -n "${replay_common_directory:-}" ]]; then
  REPLAY_E182_SHARDS="$OUT_DIR/e182-shards-from-prune-server"
  rm -rf -- "${REPLAY_E182_SHARDS:?}"
  for replay_e182_kind in "${REPLAY_E182_SHARD_KINDS[@]}"; do
    if ! mkdir -p -- "$REPLAY_E182_SHARDS/$replay_e182_kind"; then
      replay_row_library_failure 1 "建不了 E182 片结果库的中转目录 $REPLAY_E182_SHARDS/$replay_e182_kind" "看 REPLAY_OUT 那个目录能不能写、盘满没满"
      continue
    fi
    replay_e182_exit=0
    replay_e182_output="$(python3 "$ADMISSION_MODULE" library-files fetch "$replay_common_directory/singlefs-unit-verdicts/$replay_e182_kind" "$REPLAY_E182_SHARDS/$replay_e182_kind" 2>&1)" \
      || replay_e182_exit=$?
    if (( replay_e182_exit == 75 )) && [[ "$REPLAY_BROKEN" != prune-server-failure-swallowed ]]; then
      printf '%s\n' "$replay_e182_output" >&2
      replay_prune_server_unavailable "python3 $ADMISSION_MODULE library-files fetch（E182 片结果库 $replay_e182_kind）"
    elif (( replay_e182_exit != 0 )); then
      printf '%s\n' "$replay_e182_output" >&2
      replay_row_library_failure 1 "E182 片结果库 $replay_e182_kind 拉不下来（python3 $ADMISSION_MODULE library-files fetch 退 $replay_e182_exit，原因在上面）" \
        "照上面的报错修 admission.py library-files（文件头「library-files」）"
    fi
  done
fi
# replay_store_e182_shards：各行收齐之后把两种片并回服务（逐字节相同的跳过）；退 75 整趟退 75，别的非 0 整趟退 1（文件头「E182 的片结果库」）
replay_store_e182_shards() {
  [[ -n "${REPLAY_E182_SHARDS:-}" ]] || return 0
  local kind store_exit store_output
  for kind in "${REPLAY_E182_SHARD_KINDS[@]}"; do
    store_exit=0
    store_output="$(python3 "$ADMISSION_MODULE" library-files store "$replay_common_directory/singlefs-unit-verdicts/$kind" "$REPLAY_E182_SHARDS/$kind" 2>&1)" || store_exit=$?
    if (( store_exit == 75 )) && [[ "$REPLAY_BROKEN" != prune-server-failure-swallowed ]]; then
      printf '%s\n' "$store_output" >&2
      replay_prune_server_unavailable "python3 $ADMISSION_MODULE library-files store（E182 片结果库 $kind）"
    elif (( store_exit != 0 )); then
      printf '%s\n' "$store_output" >&2
      echo "  ✗ E182 片结果库 $kind：判完的片并不回服务（library-files store 退 $store_exit，原因在上面），这一趟判的片没记下" >&2
      echo "     → 怎么办：照上面的报错修 admission.py library-files（文件头「library-files」），再重交这一趟（没记下的片下一趟照判）" >&2
      exit 1
    fi
  done
}
# 一行判完之后记进判定库（文件头「按行复用」）：先在中转目录里同目录临时目录写完再改名换上（连同这把键的组成清单与跑法），再经 library-files store
# 写进 prune-server：verdict 排在最后（它落库时同一条记录的别的文件都已落库）；items/<前三列的 sha256> 记这一行最近一次判过的键
# （下一趟键对不上时 admission.py 拿它比出变了的单元）。服务连不上（store 退 75）记一个标记，收齐各行之后整趟退 75；别的记不下只报一行，不判红
remember_row_verdict() { # <记录目录，空串不记> <判定> <结果行> <新产物> <前三列>
  local record="$1" temporary key identity_digest store_output store_exit=0 name
  local -a store_paths=()
  [[ -n "$record" ]] || return 0
  temporary="$record.tmp.$BASHPID"
  key="${record##*/}"
  rm -rf -- "${temporary:?}"
  if mkdir -p -- "$temporary" && cp -- "$4" "$temporary/product" && printf '%s\n' "$2" >"$temporary/verdict" \
     && printf '%s\n' "$3" >"$temporary/line" && TZ=Asia/Tokyo date +%F >"$temporary/finished" \
     && printf '%s\n' "bash scripts/experiment-apparatus/replay.sh（这一行的二进制与参数照复跑表；GPU 开关 ${REPLAY_GPU_SWITCH:-0}）" >"$temporary/how" \
     && { [[ ! -f "$OUT_DIR/compositions/$key.tsv" ]] || cp -- "$OUT_DIR/compositions/$key.tsv" "$temporary/composition"; }; then
    rm -rf -- "${record:?}"
    identity_digest="$(printf '%s' "$5" | sha256sum)"; identity_digest="${identity_digest%% *}"
    if mv -T -- "$temporary" "$record" 2>/dev/null && printf '%s\n' "$key" >"$REPLAY_ROW_RECORDS/items/$identity_digest"; then
      for name in product line finished how composition; do [[ -f "$record/$name" ]] && store_paths+=("keys/$key/$name"); done
      store_paths+=("items/$identity_digest" "keys/$key/verdict")
      store_output="$(python3 "$ADMISSION_MODULE" library-files store "$REPLAY_ROW_LIBRARY" "$REPLAY_ROW_RECORDS" "${store_paths[@]}" 2>&1)" || store_exit=$?
      (( store_exit == 0 )) && return 0
      printf '%s\n' "$store_output" >&2
      if (( store_exit == 75 )) && [[ "$REPLAY_BROKEN" != prune-server-failure-swallowed ]]; then
        : >"$OUT_DIR/prune-server-unavailable"
        echo "  ✗ 按行复用：prune-server 不可用（library-files store 退 75），$record 没记进判定库；收齐各行之后整趟退 75" >&2
        return 0
      fi
    fi
  fi
  rm -rf -- "${temporary:?}"
  echo "  ! 按行复用：记不下 $record（下一趟这一行照跑）" >&2
}
# 开跑前的那一判（准入与按行复用）：照判定写 $OUT_DIR/<tag>.decision（reuse、unchanged、broken、run 之一），准入放行的产物头写 <tag>.header；
# 复用、输入没变、跑不了的那几行同时把结果行与结论写好，replay_one 读到不是 run 的就直接收尾。主循环先把每一行都判完，再定编不编（文件头「按行复用」）。
decide_row() { # <实验号> <tag> <二进制> <留存产物> <准入键> <行键，空串不复用> <行的身份：实验号|二进制|参数>
  local exp="$1" tag="$2" bin="$3" stored="$4" row_key="$5" reuse_key="$6" identity="$7" reuse_record="" reused_verdict reused_title admission_exit_code=0 deferred_root
  rm -f "$OUT_DIR/$tag.decision" "$OUT_DIR/$tag.header" "$OUT_DIR/$tag.admission"
  # 登记了「输入缺时本次未跑」的行（REPLAY_DEFERRED_ROW_ROOT）：导出根不在就不起、不判红，结果行写本次未跑；根在了照常现跑
  if [[ -n "${REPLAY_DEFERRED_ROW_ROOT["$identity"]+set}" && "$REPLAY_BROKEN" != deferred-rows-ignored ]]; then
    deferred_root="$(replay_frozen_root "${REPLAY_DEFERRED_ROW_ROOT["$identity"]}")" || deferred_root=""
    if [[ -z "$deferred_root" || ! -d "$deferred_root" ]]; then
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 本次未跑 "输入缺：导出根 ${REPLAY_DEFERRED_ROW_ROOT["$identity"]} 不在 replay-frozen-roots 下，里程碑四收口做全量时重建再跑（.claude/kb/milestone/04-fourth-txn.md 八·8）；这一行不判红、不判绿" >"$OUT_DIR/$tag.line"
      echo not_run >"$OUT_DIR/$tag.verdict"; echo not_run >"$OUT_DIR/$tag.decision"; return
    fi
  fi
  if [[ -n "${ADMISSION_REGISTERED[$row_key]:-}" ]]; then
    if python3 "$ADMISSION_MODULE" experiment . "$row_key" >"$OUT_DIR/$tag.header" 2>"$OUT_DIR/$tag.admission"; then admission_exit_code=0; else admission_exit_code=$?; fi
    if [[ $admission_exit_code -eq 77 ]]; then
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 输入没变 "实验键与留存产物头上记的相同，本次没跑，沿用 $stored 的结论（见 $OUT_DIR/$tag.admission）" >"$OUT_DIR/$tag.line"
      echo unchanged >"$OUT_DIR/$tag.verdict"; echo "$exp|results/$stored" >"$OUT_DIR/$tag.claim"; echo unchanged >"$OUT_DIR/$tag.decision"; return
    fi
    if [[ $admission_exit_code -ne 0 ]]; then
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 跑不了 "准入没放行（退 $admission_exit_code），见 $OUT_DIR/$tag.admission" >"$OUT_DIR/$tag.line"; echo broken >"$OUT_DIR/$tag.verdict"
      echo broken >"$OUT_DIR/$tag.decision"; return
    fi
  fi
  [[ -n "$reuse_key" && -n "$REPLAY_ROW_LIBRARY" ]] && reuse_record="$REPLAY_ROW_RECORDS/keys/$reuse_key"
  if [[ -n "$REPLAY_FORCED_TAG" && "$tag" == "$REPLAY_FORCED_TAG" ]]; then   # 复用哨兵抽中强制现判：不读库、照现跑
    printf '  · %s 要现跑：复用哨兵抽中强制现判（行键相同，这一趟不读库）\n' "$exp" >&2
    echo run >"$OUT_DIR/$tag.decision"; return
  fi
  if [[ -n "$reuse_record" && -f "$reuse_record/verdict" && -f "$reuse_record/product" ]] && [[ "$REPLAY_BROKEN" != reuse-ignored ]]; then
    reused_verdict="$(cat -- "$reuse_record/verdict")"
    case "$reused_verdict" in pass) reused_title=字节一致 ;; timing_only) reused_title=仅计时不同 ;; *) reused_title="" ;; esac
    if [[ -n "$reused_title" ]]; then
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 复用 "${reused_title}：行键相同（${reuse_key:0:16}…，$(cat -- "$reuse_record/finished" 2>/dev/null || echo 那一趟)判的；留存产物 $stored），这一趟没跑" >"$OUT_DIR/$tag.line"
      echo "reused:$reused_verdict" >"$OUT_DIR/$tag.verdict"; echo "$exp|$reuse_record/product" >"$OUT_DIR/$tag.claim"
      echo reuse >"$OUT_DIR/$tag.decision"; return
    fi
  fi
  # 库里这把行键最近一次判的是红（对不上，或现跑了而跑不了）：不当绿复用，现跑；写一行原因（文件头「按行复用」记红、撤绿、下趟现判）
  if [[ -n "$reuse_record" && -f "$reuse_record/verdict" ]]; then
    case "$(cat -- "$reuse_record/verdict")" in
      drift) printf '  · %s 要现跑：上次判红（%s 判的对不上，行键相同），不当绿复用\n' "$exp" "$(cat -- "$reuse_record/finished" 2>/dev/null || echo 那一趟)" >&2 ;;
      broken) printf '  · %s 要现跑：上次判红（%s 那一趟跑不了，行键相同），不当绿复用\n' "$exp" "$(cat -- "$reuse_record/finished" 2>/dev/null || echo 那一趟)" >&2 ;;
    esac
  fi
  echo run >"$OUT_DIR/$tag.decision"
}
