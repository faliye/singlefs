#!/usr/bin/env bash
# admission: always 被 scripts/experiment-apparatus/replay.sh 在同目录 source 进去的那一段，判的是此刻的仓与这一趟的参数，上一次的结论不替这一次作保
# run-condition: none 不单独执行：由 replay.sh 的 source 载入，环境要求同 replay.sh（文件头「GPU」「编译目录」）
# replay-finish.sh：replay.sh 的一段（收尾（EXIT）、被 TERM / INT 停、监看进程的停法、被停时已判完的行）。只定义函数与载入时赋值的全局量，不改 shell 选项。
# 这一段按 replay.sh 里 source 它的那一行的位置载入。文件头的用法、复跑表、弄坏开关表、退出码都在 replay.sh。
# 驱动在本轮输出目录（$OUT_DIR）里拷 crates/ 去编的副本编出来的编译目录（文件头「编译目录」）：逐个找带签名对得上的 CACHEDIR.TAG（不跟符号链接、不出这个文件系统），
# 经 scripts/hosts/run_build_directory.py remove 删（删之前核本机没有进程开着里面的文件，有就不删、报出进程，交回 1）；调用方给的 CARGO_TARGET_DIR 落在里面的不碰。
# 弄坏开关 REPLAY_BREAK=copy-build-directories-kept：不删
replay_remove_copy_build_directories() {
  local marker directory output failed=0
  [[ -d "${OUT_DIR:-}" && "$REPLAY_BROKEN" != copy-build-directories-kept ]] || return 0
  while IFS= read -r -d '' marker; do
    directory="${marker%/CACHEDIR.TAG}"
    [[ -d "$directory" && "$(head -c 43 -- "$marker" 2>/dev/null)" == "Signature: 8a477f597d28d172789f06886806bc55" ]] || continue
    [[ -n "${CARGO_TARGET_DIR:-}" && ( "$directory" == "$CARGO_TARGET_DIR" || "$directory" == "$CARGO_TARGET_DIR"/* ) ]] && continue
    if ! output="$(python3 scripts/hosts/run_build_directory.py remove "$directory" 2>&1)"; then
      printf '%s\n' "$output" >&2
      failed=1
    fi
  done < <(find "$OUT_DIR" -xdev -type f -name CACHEDIR.TAG -print0 2>/dev/null)
  return "$failed"
}
# 收尾（EXIT）：正常结束、出错、被停都走到这里。删不成（有进程开着、删完回读还在、运行目录删完编译目录之后不是空的）的，本来要退 0 的改退 1。
# 弄坏开关 REPLAY_BREAK=build-directory-kept：这一趟自己的编译目录与运行目录不删
replay_finish() {
  local status=$? problems=0
  trap - EXIT TERM INT
  replay_row_sampler_stop
  # 按行复用的中转目录（文件头「按行复用」）：判定库在 prune-server 里，这一份只给这一趟读写用，收尾删掉，不留给下一趟
  [[ -n "${REPLAY_ROW_RECORDS:-}" ]] && rm -rf -- "$REPLAY_ROW_RECORDS"
  [[ -n "${REPLAY_E182_SHARDS:-}" ]] && rm -rf -- "$REPLAY_E182_SHARDS"   # E182 的片结果库的中转目录（文件头「E182 的片结果库」），并回之后删
  # 驱动开跑前从 git 历史取回、放回 research/results/ 的留存产物（replay_inputs_present）：本趟放回去的，收尾删掉，不留给别的会话看见
  if [[ -n "${OUT_DIR:-}" && -f "$OUT_DIR/restored-archived-products.list" ]]; then
    while IFS= read -r restored_product; do [[ "$restored_product" == research/results/* ]] && rm -f -- "$restored_product"; done <"$OUT_DIR/restored-archived-products.list"
  fi
  replay_remove_copy_build_directories || problems=1
  if [[ -n "$REPLAY_BUILD_RUN_DIRECTORY" && "$REPLAY_BROKEN" == build-directory-kept ]]; then
    echo "  ! 这一趟的编译目录 $REPLAY_BUILD_RUN_DIRECTORY 不删（弄坏开关 build-directory-kept）" >&2
  elif [[ -n "$REPLAY_BUILD_RUN_DIRECTORY" ]]; then
    if ! run_build_directory_end; then
      problems=1
    elif [[ -d "$REPLAY_BUILD_RUN_DIRECTORY" ]] && ! rmdir -- "$REPLAY_BUILD_RUN_DIRECTORY" 2>/dev/null; then
      echo "  ✗ 这一趟的运行目录 $REPLAY_BUILD_RUN_DIRECTORY 删编译目录之后不是空的（或删不成），没删" >&2
      echo "     → 怎么办：ls -la $REPLAY_BUILD_RUN_DIRECTORY 看是谁写进去的；还开着里面文件的进程照上面那几行停掉再手删" >&2
      problems=1
    fi
  fi
  (( problems && status == 0 )) && status=1
  exit "$status"
}
# 被 TERM / INT 停：给自己起的每一行复跑（REPLAY_PIDS 里、/proc 里父进程还是这一趟的那几个）转一次 TERM（它们是 `&` 起的，INT 本来就忽略），
# 逐个等它们退，再退 143 / 130、走 EXIT 收尾。行里手里那一步由 replay_one 自己再转
replay_forward_term() { # <进程号>：只停点名的这一个，而且只在它的父进程还是这一趟时才发（进程号被别的进程复用了不误伤）
  local stat_line parent
  IFS= read -r stat_line 2>/dev/null <"/proc/$1/stat" || return 0
  read -r _ parent _ <<<"${stat_line##*) }"
  if [[ "$parent" == "$$" ]]; then kill -TERM "$1" 2>/dev/null; fi
  return 0
}
replay_stop() { # <退出码>
  local pid
  trap - TERM INT
  for pid in ${REPLAY_PIDS[@]+"${REPLAY_PIDS[@]}"}; do replay_forward_term "$pid"; done
  for pid in ${REPLAY_PIDS[@]+"${REPLAY_PIDS[@]}"}; do wait "$pid" 2>/dev/null; done
  replay_row_sampler_stop
  replay_print_finished_rows
  exit "$1"
}
# 停掉监看进程（replay_row_costs.py sample）：写下 stop，它把这之前收尾的行记完就退；至多等 10 秒，还没退发 TERM。没起过的不做。起它的那一步在主循环上面
replay_row_sampler_stop() {
  local waited=0 stat_line
  [[ -n "${REPLAY_ROW_SAMPLER_PID:-}" ]] || return 0
  : >"$REPLAY_ROW_WATCH/stop"
  while (( waited < 100 )) && IFS= read -r stat_line 2>/dev/null <"/proc/$REPLAY_ROW_SAMPLER_PID/stat" && [[ "${stat_line##*) }" != Z* ]]; do
    sleep 0.1
    waited=$((waited + 1))
  done
  (( waited < 100 )) || kill -TERM "$REPLAY_ROW_SAMPLER_PID" 2>/dev/null
  wait "$REPLAY_ROW_SAMPLER_PID" 2>/dev/null
  REPLAY_ROW_SAMPLER_PID=""
}
# 被停时已判完的行的结果行（文件头「进度」）：照表序打到标准输出。还没开始派行的不打。弄坏开关 finished-rows-lost-when-stopped
replay_print_finished_rows() {
  local tag finished=0
  [[ -n "${REPLAY_DISPATCH_STARTED:-}" && "$REPLAY_BROKEN" != finished-rows-lost-when-stopped ]] || return 0
  for tag in ${REPLAY_ORDER[@]+"${REPLAY_ORDER[@]}"}; do
    [[ -f "$OUT_DIR/$tag.line" ]] || continue
    cat -- "$OUT_DIR/$tag.line"
    finished=$((finished + 1))
  done
  echo "REPLAY_STOPPED finished=$finished dispatched=${#REPLAY_ORDER[@]}"
}
