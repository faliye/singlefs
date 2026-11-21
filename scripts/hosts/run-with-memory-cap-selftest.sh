#!/usr/bin/env bash
# run-with-memory-cap.sh 的自证：run_selftest 与它用的探针函数 bounded_memory_hog_code、selftest_job_code、收尾 selftest_cleanup。
# 被 run-with-memory-cap.sh source 的函数库，只定义函数与 selftest_slices、scratch 两个全局变量，不单独调；
#   自证里读的全局变量（SELF_PATH、SCRIPT_DIRECTORY、BROKEN_JUDGEMENT、各 *_EXIT 常量、峰值表与 slice 的路径）都在 run-with-memory-cap.sh 里定义，source 它的那一刻之后才有。

# ── 自证 ──
# 吃内存的探针：每次拿 16 MiB 写满（真占常驻页），拿到 MEBIBYTES 就停下正常退出；上限 64M 时它在半路被杀
bounded_memory_hog_code() {
  printf '%s\n' "import sys" "held = []" "for _ in range(int(sys.argv[1]) // 16):" "    held.append(b'\\x01' * (16 * 1024 * 1024))" \
    "print('allocated', len(held) * 16, 'MiB')"
}

# 挤掉那一例用的作业：起跑时刻写进第一个文件，等一会儿、拿够内存、再占一会儿，收尾时刻写进第二个文件
selftest_job_code() {
  printf '%s\n' "import sys, time" "start_path, end_path = sys.argv[1], sys.argv[2]" \
    "delay_seconds, mebibytes, hold_seconds = float(sys.argv[3]), int(sys.argv[4]), float(sys.argv[5])" \
    "open(start_path, 'w').write(repr(time.time()))" "time.sleep(delay_seconds)" \
    "held = [bytes([1]) * (1024 * 1024) for _ in range(mebibytes)]" "time.sleep(hold_seconds)" \
    "open(end_path, 'w').write(repr(time.time()))"
}

# 自证的收尾（文件头「自证的收尾」）：自证开的 slice 逐个 stop 加 revert，再删自证的临时目录；EXIT trap 与自证末尾都调它，调过一次之后再调什么都不做。
# selftest_slices 与 scratch 是全局的：TERM / INT 打断时 trap 照样读得到
selftest_slices=() scratch=""
selftest_cleanup() {
  local own_slice
  for own_slice in ${selftest_slices[@]+"${selftest_slices[@]}"}; do
    systemctl --user stop "$own_slice" >/dev/null 2>&1
    systemctl --user revert "$own_slice" >/dev/null 2>&1
  done
  selftest_slices=()
  [[ -z "$scratch" ]] || rm -rf -- "${scratch:?}"
  scratch=""
}

run_selftest() {
  local failures=0 checked=0 output status marker hog_code job_code target slice_counter=0 slice context_directory
  local first_pid second_pid first_status second_status started_seconds elapsed_seconds cgroup_path refused_before
  local -a context=()
  # 头一件事挂收尾（文件头「自证的收尾」）：TERM / INT 停与中途 exit 都经 EXIT 走到 selftest_cleanup；弄坏开关 selftest-cleanup-unhooked 下不挂（只在末尾停）
  if [[ "$BROKEN_JUDGEMENT" != selftest-cleanup-unhooked ]]; then
    trap selftest_cleanup EXIT
    trap 'exit 143' TERM
    trap 'exit 130' INT
  fi
  target="${RUN_WITH_MEMORY_CAP_SELFTEST_TARGET:-$SELF_PATH}"
  # 自证经服务起时环境里带着服务那一层给的两个标记：各例一律清掉
  unset RUN_WITH_MEMORY_CAP_OUTER_UNIT RUN_WITH_MEMORY_CAP_OUTER_CAP
  scratch="$(mktemp -d "${TMPDIR:-/tmp}/run-with-memory-cap-selftest-XXXXXX")"
  # 自证起的每一份包装的标记（文件头「弄坏开关只在自证里认」）：建在临时目录里、export，收尾随 scratch 删
  printf '%s\n' "$SELFTEST_MARK_CONTENT" > "$scratch/selftest-mark"
  export RUN_WITH_MEMORY_CAP_SELFTEST_MARK="$scratch/selftest-mark"
  local selftest_peak_host=192.0.2.1   # 峰值表第七列：自证各例的多机配置过不了配置判法、认不出这一台，经 RUN_WITH_MEMORY_CAP_SELFTEST_HOST 给（文件头「其余环境变量」）
  hog_code="$(bounded_memory_hog_code)"
  job_code="$(selftest_job_code)"
  fail() { echo "  ✗ 自检：$1"; failures=$((failures + 1)); }   # gate-lint:detail
  # 每一例一个新 slice（名字不带 -：- 是 systemd 的层级）、新的峰值表与状态目录；收尾时 stop 加 revert 掉。XDG_STATE_HOME 也指到这一例底下
  fresh_context() { # fresh_context <总上限>
    slice_counter=$((slice_counter + 1))
    slice="singlefs_memory_selftest_$$_${slice_counter}.slice"
    selftest_slices+=("$slice")
    context_directory="$scratch/context-$slice_counter"
    mkdir -p "$context_directory/temporary-root"
    context=(-u CLAUDE_CODE_SESSION_ID -u RUN_WITH_MEMORY_CAP_OUTER_UNIT -u RUN_WITH_MEMORY_CAP_OUTER_CAP
             RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER=selftest-task.1 CAPPED_ADMITTED_BY_TASK_SERVER=selftest-task.1
             RUN_WITH_MEMORY_CAP_SLICE="$slice" RUN_WITH_MEMORY_CAP_SLICE_TOTAL="$1" RUN_WITH_MEMORY_CAP_STATE_DIR="$context_directory/state"
             RUN_WITH_MEMORY_CAP_PEAKS="$context_directory/peaks.tsv" RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT="$context_directory/temporary-root"
             XDG_STATE_HOME="$context_directory/state-home" SINGLEFS_MULTI_HOST_CONFIG="$context_directory/multi-host.env"
             RUN_WITH_MEMORY_CAP_PRIVATE_CONFIG="$context_directory/private-config-absent.env" RUN_WITH_MEMORY_CAP_SELFTEST_HOST="$selftest_peak_host")   # 私有配置在家目录下、按台各一份：自证不读它（第二台的家目录是另一个），要的格自己给
  }
  in_context() { env "${context[@]}" "$@"; }
  peak_row() { # peak_row <键> → 峰值表里这个键那一行「列数|峰值|近几次」；没有就空
    awk -F '\t' -v key="$1" '$4 == key {print NF "|" $1 "|" $6}' "$context_directory/peaks.tsv" 2>/dev/null
  }

  # 「自证开出 slice 之后被停」那两格起的这一趟（文件头「其余环境变量」RUN_WITH_MEMORY_CAP_SELFTEST_INTERRUPT_PROBE）：收尾已挂好，开一个真 slice、
  # 把进程号、slice 名与临时目录写进探针文件（先写 .partial 再改名，起它的那一格见到文件就是写全了），然后等着被停；前台的 sleep 每次只睡 0.1 秒，trap 至多晚这么久
  if [[ -n "$SELFTEST_INTERRUPT_PROBE" ]]; then
    fresh_context 256M
    in_context bash "$target" --check 64M >/dev/null 2>"$scratch/interrupt-probe-check.err" \
      || fail "探针这一趟的 --check 64M 应当起得来（slice 才开得出来），实际起不来：$(cat -- "$scratch/interrupt-probe-check.err")"
    { printf 'pid %s\n' "$$"; printf 'slice %s\n' "${selftest_slices[@]}"; printf 'scratch %s\n' "$scratch"; } > "$SELFTEST_INTERRUPT_PROBE.partial"
    mv -f -- "$SELFTEST_INTERRUPT_PROBE.partial" "$SELFTEST_INTERRUPT_PROBE"
    local probe_tick
    for ((probe_tick = 0; probe_tick < SELFTEST_INTERRUPT_PROBE_SECONDS * 10; probe_tick++)); do sleep 0.1; done
    fail "设了 RUN_WITH_MEMORY_CAP_SELFTEST_INTERRUPT_PROBE 的这一趟自证应当在 $SELFTEST_INTERRUPT_PROBE_SECONDS 秒里被起它的那一格停掉，实际一直没被停"
    echo "  → 怎么办：这一趟只给「自证开出 slice 之后被 TERM / INT 停时」那两格起；手起的去掉 RUN_WITH_MEMORY_CAP_SELFTEST_INTERRUPT_PROBE 再跑 --selftest"
    exit 1
  fi

  # 弄坏开关只在自证里认（文件头那一段；门禁 code-tooling 的 break-switch-outside-selftest 格就这样探）：
  # 没有标记（变量清掉、或指到头一行不对的文件）而设了开关 ⇒ 退 2、点名它只在自证里认；弄坏开关 guard-off 下这一道不拒，探针往下走到「没给命令」退 251、不点名。
  # 带着标记 ⇒ 不拒，往下走到同一处退 251
  local guard_probe_break=nocap guard_probe_mark guard_probe_output guard_probe_status
  [[ "$BROKEN_JUDGEMENT" != guard-off ]] || guard_probe_break=guard-off
  printf '%s\n' "不是 $SELFTEST_MARK_CONTENT" > "$scratch/selftest-mark-wrong"
  for guard_probe_mark in "" "$scratch/selftest-mark-wrong"; do
    checked=$((checked + 1))
    if guard_probe_output="$(env RUN_WITH_MEMORY_CAP_SELFTEST_MARK="$guard_probe_mark" RUN_WITH_MEMORY_CAP_BREAK="$guard_probe_break" \
        bash "$target" --force --break-guard-probe-unknown-option 2>&1 </dev/null)"; then guard_probe_status=0; else guard_probe_status=$?; fi
    if [[ "$guard_probe_status" != 2 ]] || ! grep -qE 'RUN_WITH_MEMORY_CAP_BREAK.*只在自证（--selftest）里认' <<<"$guard_probe_output"; then
      fail "平常跑（没有自证的标记）设了 RUN_WITH_MEMORY_CAP_BREAK 应当退 2、点名它只在自证里认（标记「${guard_probe_mark:-空}」），实际退 $guard_probe_status：$(head -n 3 <<<"$guard_probe_output" | tr '\n' ' ')"
    fi
  done
  checked=$((checked + 1))
  if guard_probe_output="$(env RUN_WITH_MEMORY_CAP_BREAK=nocap bash "$target" --force --break-guard-probe-unknown-option 2>&1 </dev/null)"; then guard_probe_status=0; else guard_probe_status=$?; fi
  if [[ "$guard_probe_status" != "$MEMORY_CAP_UNAVAILABLE_EXIT" ]] || grep -qF '只在自证（--selftest）里认' <<<"$guard_probe_output"; then
    fail "带着自证的标记设了 RUN_WITH_MEMORY_CAP_BREAK 应当不拒、照常往下判（这一条没给命令，退 $MEMORY_CAP_UNAVAILABLE_EXIT），实际退 $guard_probe_status：$(head -n 3 <<<"$guard_probe_output" | tr '\n' ' ')"
  fi

  fresh_context 512M
  checked=$((checked + 1))
  if ! in_context bash "$target" --check 64M 2>"$scratch/check.err"; then
    fail "--check 64M 在这台机器上应当起得来，实际起不来：$(cat "$scratch/check.err")"
  fi

  checked=$((checked + 1))
  if in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-hog bash "$target" 64M python3 -c "$hog_code" 512 >"$scratch/hog.out" 2>"$scratch/hog.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_HIT_EXIT ]] || ! grep -q '撞了内存上限 64M' "$scratch/hog.err"; then
    fail "上限 64M 里分配 512 MiB 的探针应当退 $MEMORY_CAP_HIT_EXIT 并报「撞了内存上限 64M」，实际退 $status：$(cat "$scratch/hog.out" "$scratch/hog.err")"
  fi
  # 撞了自己的上限的那一次记成撞顶（峰值记上限）（文件头「峰值表」一段）
  checked=$((checked + 1))
  output="$(peak_row selftest-hog)"
  [[ "$output" == "7|$((64 * MEBIBYTE))|$((64 * MEBIBYTE)):hit:unreclaimable" ]] \
    || fail "撞了自己的上限的那一次应当在峰值表里记成撞顶、峰值记上限（「7|$((64 * MEBIBYTE))|$((64 * MEBIBYTE)):hit:unreclaimable」），实际「$output」（列数|峰值|近几次）"

  # 吃内存的是孙进程、父进程被杀之后自己还要退 101：照样按 Result 判撞顶
  checked=$((checked + 1))
  if in_context bash "$target" 64M bash -c 'python3 -c "$1" 512; exit 101' hog-parent "$hog_code" >"$scratch/grandchild.out" 2>"$scratch/grandchild.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_HIT_EXIT ]]; then
    fail "吃内存的是孙进程时应当照样退 $MEMORY_CAP_HIT_EXIT（按 scope 的 Result 判，不按父进程的退出码），实际退 $status：$(cat "$scratch/grandchild.err")"
  fi

  # 不是撞顶的 SIGKILL（自己 kill -9 自己）退 137，不许判成撞顶
  checked=$((checked + 1))
  if in_context bash "$target" 64M bash -c 'kill -9 $$' 2>"$scratch/sigkill.err"; then status=0; else status=$?; fi
  if [[ $status -ne 137 || "$(cat "$scratch/sigkill.err")" == *Killed* ]]; then
    fail "上限之内被 kill -9 的命令应当原样退 137、不算撞顶，stderr 里不多出外壳补的「Killed」那一行，实际退 $status：$(cat "$scratch/sigkill.err")"
  fi

  # 命令被外面的信号停（退 128 加 1、2、9、15）而 scope 没撞自己的上限、也没被 slice 挤掉：退出码原样，报「被外面停了（信号 N），这一次的峰值不记」与出路，
  # 峰值表里不留这个键（量到的是半截）；对照：同样的上限里整段跑完的一条照记。弄坏开关 recordkilledpeak 下 TERM、KILL 两格红
  fresh_context 256M
  checked=$((checked + 1))
  if in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-stopped-control bash "$target" 64M bash -c : 2>"$scratch/stopped-control.err"; then status=0; else status=$?; fi
  output="$(peak_row selftest-stopped-control)"
  if [[ $status -ne 0 || ! "$output" =~ ^7\|[0-9]+\| ]]; then
    fail "对照：整段跑完的命令应当把峰值记进表（selftest-stopped-control 那一行），实际退 $status、记的是「$output」：$(cat "$scratch/stopped-control.err")"
  fi
  local stopped_name stopped_number
  for stopped_name in TERM KILL; do
    stopped_number="$(kill -l "$stopped_name")"
    checked=$((checked + 1))
    if in_context env RUN_WITH_MEMORY_CAP_KEY="selftest-stopped-by-$stopped_name" bash "$target" 64M bash -c "kill -$stopped_name \$\$" 2>"$scratch/stopped-$stopped_name.err"; then status=0; else status=$?; fi
    output="$(peak_row "selftest-stopped-by-$stopped_name")"
    if [[ $status -ne $((128 + stopped_number)) || -n "$output" ]] || ! grep -qF "被外面停了（信号 $stopped_number），这一次的峰值不记" "$scratch/stopped-$stopped_name.err" \
        || ! grep -qF "→ 这一条的结果不算数；查是谁发的信号" "$scratch/stopped-$stopped_name.err"; then
      fail "命令被外面的 $stopped_name 停应当原样退 $((128 + stopped_number))、报「被外面停了（信号 $stopped_number），这一次的峰值不记」与出路、峰值表里不留这个键，实际退 $status、峰值表里「$output」：$(cat "$scratch/stopped-$stopped_name.err")"
    fi
  done

  # 正常退出码原样传回
  local wanted
  for wanted in 0 7 101; do
    checked=$((checked + 1))
    if in_context bash "$target" 64M bash -c "exit $wanted"; then status=0; else status=$?; fi
    [[ $status -eq $wanted ]] || fail "命令退 $wanted 时应当原样传回 $wanted，实际 $status"
  done

  # 命令退出之后停自己的 scope（文件头「命令退出之后停自己的 scope」）：命令记下自己的 scope 单元名，setsid 出去一个清了环境的 sleep、记下它的进程号就退 0 ⇒
  # 包装退 0、那个 sleep 不在了、scope 不再 active、stderr 说还留着 1 个进程；弄坏开关 scope-left-after-exit 下 scope 照活着（收尾停这一例自己的 slice 时一起停）
  fresh_context 256M
  checked=$((checked + 1))
  local leftover_pid="" leftover_unit="" leftover_state=""
  if in_context bash "$target" 64M bash -c 'printf "%s\n" "$RUN_WITH_MEMORY_CAP_OUTER_UNIT" > "$1"; setsid env -i sleep 300 < /dev/null > /dev/null 2>&1 & echo "$!" > "$2"; exit 0' \
      scope-leftover "$scratch/leftover.unit" "$scratch/leftover.pid" 2>"$scratch/leftover.err"; then status=0; else status=$?; fi
  leftover_pid="$(cat "$scratch/leftover.pid" 2>/dev/null)"
  leftover_unit="$(cat "$scratch/leftover.unit" 2>/dev/null)"
  [[ -n "$leftover_unit" ]] && leftover_state="$(systemctl --user is-active "$leftover_unit.scope" 2>/dev/null)"
  if [[ $status -ne 0 || ! "$leftover_pid" =~ ^[1-9][0-9]*$ || -d "/proc/${leftover_pid:-0}" || "$leftover_state" == active ]] \
      || ! grep -qF '还留着 1 个进程' "$scratch/leftover.err"; then
    fail "命令退出之后 scope 里留着一个 setsid 出去、清了环境的进程时应当停掉自己的 scope：包装退 0、那个进程不在了、scope 不再 active、stderr 说还留着 1 个进程，实际退 $status、进程「$leftover_pid」$([[ -d "/proc/${leftover_pid:-0}" ]] && echo 还在 || echo 不在)、scope「${leftover_state:-空}」：$(head_characters 400 "$scratch/leftover.err")"
  fi

  # 显存不够（文件头「显存不够」一段）：假 journalctl 按 journal-mode 文件演：oom 打一行 NVRM … NV_ERR_NO_MEMORY，xid 打一行别的 NVRM，
  # failing 退 1（读不到内核日志）；-n 1（看不看得到内核日志）一律打一行。每次调用的参数记进 journal-args
  fresh_context 512M
  local fake_journal="$scratch/fake-journal/journalctl" journal_args since_epoch until_epoch
  mkdir -p "$scratch/fake-journal"
  cat > "$fake_journal" <<'FAKE_JOURNAL'
#!/usr/bin/env bash
here="$(dirname "$0")"
printf '%s\n' "$*" >> "$here/journal-args"
mode="$(cat "$here/journal-mode")"
if [[ "$mode" == failing ]]; then echo "Failed to open journal" >&2; exit 1; fi
case " $* " in *" -n 1 "*) echo "2026-10-06 host kernel: probe"; exit 0 ;; esac
case "$mode" in
  oom) echo "2026-10-06 host kernel: NVRM: GPU1 nvAssertOkFailedNoLog: Assertion failed: Out of memory [NV_ERR_NO_MEMORY] (0x00000051) returned from pmaAllocatePages() @ pool_alloc.c:270" ;;
  xid) echo "2026-10-06 host kernel: NVRM: Xid (PCI:0000:10:00): 109, pid=441575, name=selftest, channel 0x00000003, errorString CTX SWITCH TIMEOUT, Info 0xc002" ;;
esac
exit 0
FAKE_JOURNAL
  chmod +x "$fake_journal"
  journal_case() { # journal_case <模式> <名字> <命令…>：status 与 $scratch/<名字>.err
    printf '%s\n' "$1" > "$scratch/fake-journal/journal-mode"
    : > "$scratch/fake-journal/journal-args"
    local case_name="$2"
    shift 2
    if in_context env RUN_WITH_MEMORY_CAP_JOURNALCTL="$fake_journal" bash "$target" 64M "$@" 2>"$scratch/$case_name.err"; then status=0; else status=$?; fi
  }
  checked=$((checked + 1))
  journal_case oom gpu-oom bash -c 'exit 3'
  if [[ $status -ne $GPU_MEMORY_HIT_EXIT ]] || ! grep -q '显存不够' "$scratch/gpu-oom.err"; then
    fail "命令退 3、起止之间内核报了 NVRM … NV_ERR_NO_MEMORY 时应当退 $GPU_MEMORY_HIT_EXIT、报「显存不够」，实际退 $status：$(cat "$scratch/gpu-oom.err")"
  fi
  checked=$((checked + 1))
  journal_args="$(grep -v -- ' -n 1 ' "$scratch/fake-journal/journal-args" | head -n 1)"
  since_epoch="$(sed -n 's/.*--since @\([0-9][0-9]*\).*/\1/p' <<<"$journal_args")"
  until_epoch="$(sed -n 's/.*--until @\([0-9][0-9]*\).*/\1/p' <<<"$journal_args")"
  if [[ -z "$since_epoch" || -z "$until_epoch" ]] || ((since_epoch > until_epoch || until_epoch - since_epoch > 60)); then
    fail "读内核日志应当按命令起止给 --since @<起> 与 --until @<止>（两者相差不过 60 秒），实际参数「$journal_args」"
  fi
  checked=$((checked + 1))
  journal_case failing gpu-unreadable bash -c 'exit 3'
  if [[ $status -ne 3 ]] || ! grep -q '显存 OOM 没查成' "$scratch/gpu-unreadable.err"; then
    fail "内核日志读不出来时应当照退命令自己的 3、报「显存 OOM 没查成」，实际退 $status：$(cat "$scratch/gpu-unreadable.err")"
  fi
  checked=$((checked + 1))
  journal_case xid gpu-other-nvrm bash -c 'exit 3'
  if [[ $status -ne 3 ]] || grep -q '显存不够' "$scratch/gpu-other-nvrm.err"; then
    fail "起止之间只有别的 NVRM 行（Xid）时应当照退命令自己的 3、不报显存不够，实际退 $status：$(cat "$scratch/gpu-other-nvrm.err")"
  fi
  checked=$((checked + 1))
  journal_case oom gpu-success bash -c 'exit 0'
  if [[ $status -ne 0 || -s "$scratch/fake-journal/journal-args" ]]; then
    fail "命令退 0 时应当照退 0、不读内核日志，实际退 $status、journalctl 被调了 $(wc -l < "$scratch/fake-journal/journal-args") 次：$(cat "$scratch/gpu-success.err")"
  fi
  checked=$((checked + 1))
  journal_case oom gpu-with-cap-hit python3 -c "$hog_code" 512
  if [[ $status -ne $MEMORY_CAP_HIT_EXIT ]] || ! grep -q '显存不够' "$scratch/gpu-with-cap-hit.err"; then
    fail "撞了自己的上限、起止之间内核又报了显存不够时应当照旧退 $MEMORY_CAP_HIT_EXIT、另报一行「显存不够」，实际退 $status：$(cat "$scratch/gpu-with-cap-hit.err")"
  fi

  # 上限真的设上了：scope 里读自己 cgroup 的 memory.max 与 memory.swap.max
  checked=$((checked + 1))
  output="$(in_context bash "$target" 64M bash -c 'group="/sys/fs/cgroup$(cut -d: -f3 /proc/self/cgroup)"; cat "$group/memory.max" "$group/memory.swap.max"' 2>&1)"
  if [[ "$output" != $'67108864\n0' ]]; then
    fail "scope 里的 memory.max、memory.swap.max 应当是 67108864 与 0，实际「$output」"
  fi

  # 总上限（第 ② 层）真的设上了：scope 挂在这一例的 slice 底下，slice 的 memory.max 是 512M
  checked=$((checked + 1))
  output="$(in_context bash "$target" 64M bash -c 'group="$(cut -d: -f3 /proc/self/cgroup)"; echo "$group"; cat "/sys/fs/cgroup$(dirname "$group")/memory.max"' 2>&1)"
  if [[ "$output" != *"/$slice/"*$'\n536870912' ]]; then
    fail "scope 应当挂在 slice $slice 底下、slice 的 memory.max 是 536870912（512M），实际「$output」"
  fi

  # slice 设不上（cgroup 里读回来的不是那个数，这里用一个不存在的 cgroup 根演）：退 251、报 slice、命令没跑，不退回无总上限
  checked=$((checked + 1))
  marker="$scratch/ran-without-slice"
  if in_context env RUN_WITH_MEMORY_CAP_CGROUP_ROOT="$scratch/no-cgroup-here" bash "$target" 64M bash -c ': > "$1"' touch-marker "$marker" 2>"$scratch/noslice.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_UNAVAILABLE_EXIT || -e "$marker" ]] || ! grep -q 'slice' "$scratch/noslice.err"; then
    fail "slice 的总上限设不上时应当退 $MEMORY_CAP_UNAVAILABLE_EXIT、报 slice、命令不跑，实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/noslice.err")"
  fi
  checked=$((checked + 1))
  if in_context env RUN_WITH_MEMORY_CAP_CGROUP_ROOT="$scratch/no-cgroup-here" bash "$target" --check 64M 2>/dev/null; then status=0; else status=$?; fi
  [[ $status -eq $MEMORY_CAP_UNAVAILABLE_EXIT ]] || fail "slice 的总上限设不上时 --check 应当退 $MEMORY_CAP_UNAVAILABLE_EXIT，实际 $status"

  # 总上限从哪来（文件头「总上限」）：每一例一个新 slice，环境里的 RUN_WITH_MEMORY_CAP_SLICE_TOTAL 设成空，多机配置指到这一例自己写的那一份
  # ② 配置写 SLICE_MEMORY_CAP_BY_HOST='127.0.0.1=300M 192.0.2.71=1G' ⇒ slice 的 memory.max 是这一台那一项 300M；--status 报总上限从多机配置来、不报余量。
  #   127.0.0.1 在每一台的网卡上（lo），是这一台；192.0.2.71（文档用的地址段）是另一台。值整个带一层引号（配置判法去一层成对引号，这里同读法）
  fresh_context 256M
  printf '%s\n' "# 自证写的多机配置" "HOSTS=127.0.0.1 192.0.2.71" "SLICE_MEMORY_CAP_BY_HOST='127.0.0.1=300M 192.0.2.71=1G'" > "$context_directory/multi-host.env"
  checked=$((checked + 1))
  output="$(in_context env RUN_WITH_MEMORY_CAP_SLICE_TOTAL= bash "$target" 64M bash -c 'group="$(cut -d: -f3 /proc/self/cgroup)"; cat "/sys/fs/cgroup$(dirname "$group")/memory.max"' 2>"$scratch/config-total.err")"
  [[ "$output" == "$((300 * MEBIBYTE))" ]] \
    || fail "总上限照多机配置 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项（300M）：slice 的 memory.max 应当是 $((300 * MEBIBYTE))，实际「$output」：$(cat "$scratch/config-total.err")"
  checked=$((checked + 1))
  if in_context env RUN_WITH_MEMORY_CAP_SLICE_TOTAL= bash "$target" --status >"$scratch/config-status.out" 2>&1; then status=0; else status=$?; fi
  if [[ $status -ne 0 ]] || ! grep -qF "的 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项 300M" "$scratch/config-status.out" || grep -q '余量' "$scratch/config-status.out"; then
    fail "--status 应当报总上限从多机配置来（「… 的 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项 300M」）、退 0、不再报余量，实际退 $status：$(head_characters 600 "$scratch/config-status.out")"
  fi
  # ③ 配置里没有这个键、slice 已有总上限（上一例设的 300M）⇒ 照它走、不改；配置换成写坏的（这一台那一项写法不对、没有这一台那一项）⇒ 退 251、报写法不对、命令没跑、slice 不改
  printf '%s\n' "# 自证写的多机配置，没有 SLICE_MEMORY_CAP_BY_HOST" "HOSTS=127.0.0.1 192.0.2.71" > "$context_directory/multi-host.env"
  checked=$((checked + 1))
  output="$(in_context env RUN_WITH_MEMORY_CAP_SLICE_TOTAL= bash "$target" 64M bash -c 'group="$(cut -d: -f3 /proc/self/cgroup)"; cat "/sys/fs/cgroup$(dirname "$group")/memory.max"' 2>"$scratch/existing-total.err")"
  [[ "$output" == "$((300 * MEBIBYTE))" ]] \
    || fail "多机配置里没有 SLICE_MEMORY_CAP_BY_HOST、slice 已有总上限时应当照它走（memory.max 仍是 $((300 * MEBIBYTE))）、命令照跑，实际「$output」：$(cat "$scratch/existing-total.err")"
  local bad_case bad_name bad_value bad_text
  for bad_case in "syntax|这一台那一项写法不对|127.0.0.1=50 192.0.2.71=1G" "absent|没有这一台那一项|192.0.2.71=1G"; do
    IFS='|' read -r bad_name bad_value bad_text <<<"$bad_case"
    printf '%s\n' "# 自证写的多机配置，SLICE_MEMORY_CAP_BY_HOST $bad_value" "HOSTS=127.0.0.1 192.0.2.71" "SLICE_MEMORY_CAP_BY_HOST=$bad_text" > "$context_directory/multi-host.env"
    checked=$((checked + 1))
    marker="$scratch/ran-with-bad-slice-cap-$bad_name"
    if in_context env RUN_WITH_MEMORY_CAP_SLICE_TOTAL= bash "$target" 64M bash -c ': > "$1"' touch-marker "$marker" 2>"$scratch/bad-slice-cap-$bad_name.err"; then status=0; else status=$?; fi
    if [[ $status -ne $MEMORY_CAP_UNAVAILABLE_EXIT || -e "$marker" ]] || ! grep -q 'SLICE_MEMORY_CAP_BY_HOST 写法不对' "$scratch/bad-slice-cap-$bad_name.err"; then
      fail "多机配置里 SLICE_MEMORY_CAP_BY_HOST $bad_value时应当退 $MEMORY_CAP_UNAVAILABLE_EXIT、报「SLICE_MEMORY_CAP_BY_HOST 写法不对」、命令不跑，实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/bad-slice-cap-$bad_name.err")"
    fi
  done
  # 按台取：头一项是另一台（HOSTS 与 SLICE_MEMORY_CAP_BY_HOST 都把 192.0.2.71 写在前面）⇒ 总上限照这一台那一项 320M，不取头一项的 1G（新 slice）
  fresh_context 256M
  printf '%s\n' "# 自证写的多机配置，头一项是另一台" "HOSTS=192.0.2.71 127.0.0.1" "SLICE_MEMORY_CAP_BY_HOST=192.0.2.71=1G 127.0.0.1=320M" > "$context_directory/multi-host.env"
  checked=$((checked + 1))
  output="$(in_context env RUN_WITH_MEMORY_CAP_SLICE_TOTAL= bash "$target" 64M bash -c 'group="$(cut -d: -f3 /proc/self/cgroup)"; cat "/sys/fs/cgroup$(dirname "$group")/memory.max"' 2>"$scratch/config-by-host.err")"
  [[ "$output" == "$((320 * MEBIBYTE))" ]] \
    || fail "SLICE_MEMORY_CAP_BY_HOST 里头一项是另一台时总上限照这一台那一项：slice 的 memory.max 应当是 $((320 * MEBIBYTE))（320M），实际「$output」：$(cat "$scratch/config-by-host.err")"
  # 三样都没有：新 slice、配置不在 ⇒ 退 251、出路写加 SLICE_MEMORY_CAP_BY_HOST、命令没跑
  fresh_context 256M
  checked=$((checked + 1))
  marker="$scratch/ran-without-total"
  if in_context env RUN_WITH_MEMORY_CAP_SLICE_TOTAL= bash "$target" 64M bash -c ': > "$1"' touch-marker "$marker" 2>"$scratch/no-total.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_UNAVAILABLE_EXIT || -e "$marker" ]] || ! grep -q '写一行 SLICE_MEMORY_CAP_BY_HOST=' "$scratch/no-total.err"; then
    fail "三样都定不出总上限时（没给 RUN_WITH_MEMORY_CAP_SLICE_TOTAL、多机配置不在、slice 是新的）应当退 $MEMORY_CAP_UNAVAILABLE_EXIT、出路写在多机配置里写一行 SLICE_MEMORY_CAP_BY_HOST=、命令不跑，\
实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/no-total.err")"
  fi

  # slice 上别的版本留下的 memory.high（文件头「总上限」memory.high 那一段）：不碰真的 slice。假 systemctl 只管这一例的 slice：
  # show -p ControlGroup 答假 cgroup 根下的 /selftest-heavy.slice（起跑标记 <目录>.started 在才答，否则答空 = slice 不在）、set-property MemoryMax 与 MemoryHigh=infinity 写进假目录
  # （FAKE_MEMORY_HIGH_STUCK=1 时 MemoryHigh 写不进去）、start 建起跑标记（给了 FAKE_MEMORY_HIGH_AFTER_START 就把 memory.high 换成它：drop-in 起来就带着）；别的调用交给真的。
  # 假 cgroup 根里 user.slice 是指到真 /sys/fs/cgroup/user.slice 的符号链接：scope 由真的 systemd-run 起，外壳与采样子进程照读真的 scope 目录。
  # 五例：留着 47.5G（51002736640）、已是 max、slice 不在（起来之后是 max）、slice 不在而起来之后带着 47.5G、清了读回仍是 47.5G
  local fake_high_root fake_high_slice fake_high_bin fake_high_case fake_high_before fake_high_after fake_high_stuck high_calls high_now
  local leftover_high=51002736640
  fake_high_root="$scratch/fake-memory-high-root" fake_high_bin="$scratch/fake-memory-high-bin"
  fake_high_slice="$fake_high_root/selftest-heavy.slice"
  mkdir -p "$fake_high_bin" "$fake_high_slice"
  ln -s /sys/fs/cgroup/user.slice "$fake_high_root/user.slice"
  cat > "$fake_high_bin/systemctl" <<'FAKE_MEMORY_HIGH_SYSTEMCTL'
#!/usr/bin/env bash
directory="$FAKE_MEMORY_HIGH_DIRECTORY" slice="$FAKE_MEMORY_HIGH_SLICE"
if [[ "${1:-}" == --user && "${*: -1}" == "$slice" ]]; then
  printf '%s\n' "$*" >> "$FAKE_MEMORY_HIGH_CALLS"
  case "${2:-} ${3:-}" in
    "show -p")
      if [[ "$*" == *ControlGroup* ]]; then
        [[ -e "$directory.started" ]] && printf '/%s\n' "${directory##*/}"
        exit 0
      fi ;;
    "start $slice")
      : > "$directory.started"
      [[ -z "${FAKE_MEMORY_HIGH_AFTER_START:-}" ]] || printf '%s\n' "$FAKE_MEMORY_HIGH_AFTER_START" > "$directory/memory.high"
      exit 0 ;;
  esac
fi
if [[ "${1:-} ${2:-} ${3:-} ${4:-}" == "--user set-property --runtime $slice" ]]; then
  printf '%s\n' "$*" >> "$FAKE_MEMORY_HIGH_CALLS"
  case "${5:-}" in
    MemoryMax=*) printf '%s\n' "${5#MemoryMax=}" > "$directory/memory.max" ;;
    MemoryHigh=infinity) [[ -n "${FAKE_MEMORY_HIGH_STUCK:-}" ]] || printf 'max\n' > "$directory/memory.high" ;;
    *) exit 1 ;;
  esac
  exit 0
fi
exec "$FAKE_SYSTEMCTL_REAL" "$@"
FAKE_MEMORY_HIGH_SYSTEMCTL
  chmod +x "$fake_high_bin/systemctl"
  for fake_high_case in "leftover|started|$leftover_high||" "already-max|started|max||" "absent|absent|max||" \
      "absent-then-leftover|absent|max|$leftover_high|" "stuck|started|$leftover_high||1"; do
    IFS='|' read -r fake_high_case fake_high_before high_now fake_high_after fake_high_stuck <<< "$fake_high_case"
    fresh_context 256M
    rm -f -- "$fake_high_slice.started" "$scratch/memory-high-calls"
    printf '%s\n' $((256 * MEBIBYTE)) > "$fake_high_slice/memory.max"
    printf '%s\n' "$high_now" > "$fake_high_slice/memory.high"
    [[ "$fake_high_before" != started ]] || : > "$fake_high_slice.started"
    marker="$scratch/memory-high-$fake_high_case-ran"
    rm -f -- "$marker"
    checked=$((checked + 1))
    if in_context env PATH="$fake_high_bin:$PATH" FAKE_SYSTEMCTL_REAL="$(command -v systemctl)" FAKE_MEMORY_HIGH_SLICE="$slice" FAKE_MEMORY_HIGH_DIRECTORY="$fake_high_slice" \
        FAKE_MEMORY_HIGH_CALLS="$scratch/memory-high-calls" FAKE_MEMORY_HIGH_AFTER_START="$fake_high_after" FAKE_MEMORY_HIGH_STUCK="$fake_high_stuck" \
        RUN_WITH_MEMORY_CAP_CGROUP_ROOT="$fake_high_root" RUN_WITH_MEMORY_CAP_KEY="selftest-memory-high-$fake_high_case" \
        bash "$target" 64M bash -c ': > "$1"' touch-marker "$marker" 2>"$scratch/memory-high-$fake_high_case.err"; then status=0; else status=$?; fi
    high_now="" high_calls=""
    IFS= read -r high_now < "$fake_high_slice/memory.high"
    high_calls="$(grep -c 'MemoryHigh=infinity' "$scratch/memory-high-calls" 2>/dev/null)"
    high_calls="${high_calls:-0}"
    case "$fake_high_case" in
      leftover|absent-then-leftover)
        if [[ $status -ne 0 || ! -e "$marker" || "$high_now" != max || "$high_calls" != 1 ]] || ! grep -qF '清回 max' "$scratch/memory-high-$fake_high_case.err"; then
          fail "$([[ "$fake_high_case" == leftover ]] && echo 'slice 上留着软上限 47.5G 时' || echo 'slice 原先不在、起来之后带着软上限 47.5G 时')应当 set-property MemoryHigh=infinity 一次、\
读回 memory.high 是 max、stderr 打「清回 max」、命令照跑退 0，实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')、memory.high「$high_now」、\
MemoryHigh=infinity 调了 ${high_calls:-0} 次：$(head_characters 600 "$scratch/memory-high-$fake_high_case.err")"
        fi ;;
      already-max|absent)
        if [[ $status -ne 0 || ! -e "$marker" || "$high_now" != max || "$high_calls" != 0 ]] || grep -qF '清回 max' "$scratch/memory-high-$fake_high_case.err"; then
          fail "$([[ "$fake_high_case" == absent ]] && echo 'slice 原先不在（起来之后 memory.high 是 max）时' || echo 'slice 的 memory.high 已是 max 时')应当不调 set-property MemoryHigh、\
不打「清回 max」、命令照跑退 0，实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')、memory.high「$high_now」、MemoryHigh=infinity 调了 ${high_calls:-0} 次：\
$(head_characters 600 "$scratch/memory-high-$fake_high_case.err")"
        fi ;;
      stuck)
        if [[ $status -ne $MEMORY_CAP_UNAVAILABLE_EXIT || -e "$marker" ]] || ! grep -qF '不是 max；命令一行都没跑' "$scratch/memory-high-$fake_high_case.err" \
            || ! grep -qF '→ 怎么办：systemctl --user cat' "$scratch/memory-high-$fake_high_case.err"; then
          fail "清软上限之后读回不是 max 时应当退 $MEMORY_CAP_UNAVAILABLE_EXIT、报「不是 max；命令一行都没跑」与查 drop-in 的出路、命令不跑，\
实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(head_characters 600 "$scratch/memory-high-$fake_high_case.err")"
        fi ;;
    esac
  done
  rm -f -- "$fake_high_root/user.slice"

  # systemd-run 起不来：退 251、报错，而且那条命令一行都没跑（不退回无上限）
  fresh_context 256M
  checked=$((checked + 1))
  marker="$scratch/ran-without-cap"
  if in_context env DBUS_SESSION_BUS_ADDRESS=unix:path=/nonexistent XDG_RUNTIME_DIR=/nonexistent \
      bash "$target" 64M bash -c ': > "$1"' touch-marker "$marker" 2>"$scratch/unavailable.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_UNAVAILABLE_EXIT || -e "$marker" ]] || ! grep -q '起不来' "$scratch/unavailable.err"; then
    fail "D-Bus 连不上时应当退 $MEMORY_CAP_UNAVAILABLE_EXIT、报「起不来」、命令不跑，实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/unavailable.err")"
  fi
  checked=$((checked + 1))
  if in_context env DBUS_SESSION_BUS_ADDRESS=unix:path=/nonexistent XDG_RUNTIME_DIR=/nonexistent \
      bash "$target" --check 64M 2>/dev/null; then status=0; else status=$?; fi
  [[ $status -eq $MEMORY_CAP_UNAVAILABLE_EXIT ]] || fail "D-Bus 连不上时 --check 应当退 $MEMORY_CAP_UNAVAILABLE_EXIT，实际 $status"
  # slice 设得上、而 systemd-run 自己起不来（systemd 不认这个上限：换成字节超过 64 位）：--check 退 251，不退回无上限去跑
  checked=$((checked + 1))
  if in_context bash "$target" --check 99999999999999T 2>"$scratch/scope-refused.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_UNAVAILABLE_EXIT ]] || ! grep -q 'systemd-run' "$scratch/scope-refused.err"; then
    fail "slice 设得上而 systemd-run 不认上限 99999999999999T 时 --check 应当退 $MEMORY_CAP_UNAVAILABLE_EXIT、报 systemd-run，实际退 $status：$(cat "$scratch/scope-refused.err")"
  fi

  # 被总上限挤掉（第 ② 层兜住、退 254，不报成撞了自己的上限）：两条各拿 150 MiB，上限各 200M，合起来超总上限 256M；
  # 挤掉的那一次峰值不记（量到的是半截），没挤掉的那一条整段跑完照记
  fresh_context 256M
  checked=$((checked + 1))
  in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-squeeze-1 bash "$target" 200M python3 -c "$job_code" "$scratch/squeeze-1.start" "$scratch/squeeze-1.end" 0.3 150 1.5 \
    >/dev/null 2>"$scratch/squeeze-1.err" &
  first_pid=$!
  in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-squeeze-2 bash "$target" 200M python3 -c "$job_code" "$scratch/squeeze-2.start" "$scratch/squeeze-2.end" 0.3 150 1.5 \
    >/dev/null 2>"$scratch/squeeze-2.err" &
  second_pid=$!
  wait "$first_pid"; first_status=$?
  wait "$second_pid"; second_status=$?
  if [[ $first_status -ne $SLICE_TOTAL_HIT_EXIT && $second_status -ne $SLICE_TOTAL_HIT_EXIT ]] || [[ $first_status -eq $MEMORY_CAP_HIT_EXIT || $second_status -eq $MEMORY_CAP_HIT_EXIT ]] \
      || ! grep -q '总上限挤掉' "$scratch/squeeze-1.err" "$scratch/squeeze-2.err"; then
    fail "两条合起来超总上限 256M（各自没超上限 200M）时，被杀的那条应当退 $SLICE_TOTAL_HIT_EXIT、报「被总上限挤掉」，实际退 $first_status 与 $second_status：$(cat "$scratch/squeeze-1.err" "$scratch/squeeze-2.err")"
  fi
  checked=$((checked + 1))
  local squeezed_key=""
  [[ $first_status -eq $SLICE_TOTAL_HIT_EXIT ]] && squeezed_key=selftest-squeeze-1
  [[ $second_status -eq $SLICE_TOTAL_HIT_EXIT ]] && squeezed_key=selftest-squeeze-2
  if [[ -n "$squeezed_key" ]]; then
    output="$(peak_row "$squeezed_key")"
    [[ -z "$output" ]] || fail "被总上限挤掉的那一次（$squeezed_key）不该记进峰值表（量到的是半截），实际「$output」"
  fi

  # 外壳写峰值那一步挨得住 systemd 停 scope 时发给 scope 里每个进程的 TERM：命令在 scope 里留一个进程，等命令退出之后 1 到 2 秒里
  # 不停地（不 sleep）给 scope 里除它以外的每个进程发 TERM——外壳那时起的任何外部命令活不过一轮；峰值照样要记进表。
  # 只在这一例自己开的 scope 里发：cgroup 路径要是 …/<这一例的 slice>/singlefs-memory-cap-*.scope（slice 名经 $1 传进去），不对就不发信号、退 0。
  # 不开 scope 的路径（弄坏开关 nocap）下命令就在调用方的 cgroup 里：在 SSH 会话的 session-1.scope 里发 TERM，
  # 会打掉 VSCode 扩展宿主与 Claude 会话；只核最后一段名字时，外面包着一层别的 singlefs-memory-cap scope（整条门禁经包装跑）会打到外层那一整条
  fresh_context 256M
  checked=$((checked + 1))
  if in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-term-storm bash "$target" 64M bash -c '
      group="/sys/fs/cgroup$(cut -d: -f3 /proc/self/cgroup)"
      case "$group" in
        */"$1"/singlefs-memory-cap-*.scope) ;;
        *) echo "不在这一例自己的 slice $1 底下自己开的 scope 里（$group），不发信号" >&2; exit 0 ;;
      esac
      command_pid=$$
      ( while kill -0 "$command_pid" 2>/dev/null; do :; done
        storm_end=$((SECONDS + 2))
        while ((SECONDS < storm_end)); do
          while read -r process_id; do [[ "$process_id" != "$BASHPID" ]] && kill -TERM "$process_id" 2>/dev/null; done < "$group/cgroup.procs"  # process-safety:own-scope 上面 case 核过路径在这一例自己的 slice 底下、是自己开的 singlefs-memory-cap-*.scope
        done ) >/dev/null 2>&1 &
      exit 0' term-storm "$slice" 2>"$scratch/term-storm.err"; then status=0; else status=$?; fi
  output="$(peak_row selftest-term-storm)"
  if [[ ! "$output" =~ ^7\|[0-9]+\| ]]; then
    fail "scope 里一直有进程收 TERM 时，外壳照样要把峰值记进表（selftest-term-storm 那一行），实际退 $status、记的是「$output」：$(cat "$scratch/term-storm.err")"
  fi

  # 起了 scope 之后包装收到 TERM / INT（欠账 C801）：先 systemctl --user stop 自己开的那个 scope 单元（在删标记之前），scope 里的命令随之停，
  # 包装退 143 / 130，标记与 scope 都不留。systemctl 是 PATH 前面的一层转接：记下每一次 stop（单元名、那一刻标记还在不在），再交给真的；
  # 包装经一层 python 起，把 INT 设回默认。命令是 exec sleep 60：没被停的话它一直活着（弄坏开关 term-leaves-scope）；
  # 这一例收尾自己停自己的 slice，不让它留到最后。信号只发给这里 & 起、$! 记下的那个包装。
  scope_stop_process_is_live() { # scope_stop_process_is_live <进程号>：在跑（没有、或已是僵尸都算没在跑）交回 0
    local stat_line
    IFS= read -r stat_line 2>/dev/null < "/proc/$1/stat" || return 1
    stat_line="${stat_line##*) }"
    [[ "${stat_line%% *}" != Z ]]
  }
  scope_stop_case() { # scope_stop_case <TERM|INT> <包装该退的码>
    local stop_signal="$1" expected_status="$2" case_directory="$scratch/scope-stop-$1" wrapper_pid command_pid="" poll wrapper_status=0
    local ready=0 wrapper_exited=0 command_exited=0 scope_left="" marker_left="" stop_line=""
    mkdir -p "$case_directory/bin" "$case_directory/tmp"
    : > "$case_directory/calls"
    printf '%s\n' '#!/usr/bin/env bash' \
      'if [[ "${1:-}" == --user && "${2:-}" == stop ]]; then' \
      '  marker=absent' \
      '  compgen -G "$STUB_TMPDIR/run-with-memory-cap-*.started" >/dev/null && marker=present' \
      '  echo "stop $3 marker=$marker" >> "$STUB_CALLS"' \
      'fi' \
      'exec "$STUB_REAL_SYSTEMCTL" "$@"' > "$case_directory/bin/systemctl"
    chmod +x "$case_directory/bin/systemctl"
    python3 -c 'import os, signal, sys; signal.signal(signal.SIGINT, signal.SIG_DFL); signal.signal(signal.SIGHUP, signal.SIG_DFL); os.execvp(sys.argv[1], sys.argv[1:])' \
      env "${context[@]}" PATH="$case_directory/bin:$PATH" TMPDIR="$case_directory/tmp" STUB_TMPDIR="$case_directory/tmp" STUB_CALLS="$case_directory/calls" \
      STUB_REAL_SYSTEMCTL="$(command -v systemctl)" RUN_WITH_MEMORY_CAP_KEY="selftest-scope-stop-$stop_signal" \
      bash "$target" 64M bash -c 'echo $$ > "$1"; exec sleep 60' scope-stop "$case_directory/command-pid" >"$case_directory/output" 2>&1 &
    wrapper_pid=$!
    for ((poll = 0; poll < 150; poll++)); do   # 至多 15 秒等命令在 scope 里起来（它把自己的进程号写进文件）
      if [[ -s "$case_directory/command-pid" ]]; then ready=1; break; fi
      sleep 0.1
    done
    command_pid="$(cat -- "$case_directory/command-pid" 2>/dev/null)"
    if ((ready)); then
      kill -"$stop_signal" "$wrapper_pid"
      for ((poll = 0; poll < 100; poll++)); do   # 至多 10 秒
        scope_stop_process_is_live "$wrapper_pid" || { wrapper_exited=1; break; }
        sleep 0.1
      done
      ((wrapper_exited)) || kill -KILL "$wrapper_pid" 2>/dev/null
      if [[ "$command_pid" =~ ^[0-9]+$ ]]; then
        for ((poll = 0; poll < 50; poll++)); do   # 至多 5 秒
          scope_stop_process_is_live "$command_pid" || { command_exited=1; break; }
          sleep 0.1
        done
      fi
    else
      kill -KILL "$wrapper_pid" 2>/dev/null
    fi
    wait "$wrapper_pid"; wrapper_status=$?
    scope_left="$(compgen -G "/sys/fs/cgroup/user.slice/user-$(id -u).slice/user@$(id -u).service/$slice/*.scope")"
    marker_left="$(compgen -G "$case_directory/tmp/run-with-memory-cap-*.started")"
    stop_line="$(grep -E "^stop singlefs-memory-cap-$wrapper_pid-[0-9]+\.scope marker=present\$" "$case_directory/calls" | head -n 1)"
    systemctl --user stop "$slice" >/dev/null 2>&1
    if ((ready == 0)); then
      fail "起了 scope 之后给包装发 $stop_signal 这一例没判成：命令 15 秒里没在 scope 里起来，包装退 $wrapper_status：$(cat "$case_directory/output")"
    elif [[ -z "$stop_line" || $wrapper_exited -eq 0 || $command_exited -eq 0 || $wrapper_status -ne $expected_status || -n "$scope_left$marker_left" ]]; then
      fail "起了 scope 之后给包装发 $stop_signal：应当先 systemctl --user stop 自己开的 scope 单元（那一刻标记还在）、scope 里的命令 5 秒内随之停、包装 10 秒内退 $expected_status、标记与 scope 都不留；\
实际 stop 调用「$(cat "$case_directory/calls")」、包装$([[ $wrapper_exited -eq 1 ]] && echo '退了' || echo '10 秒里没退')（$wrapper_status）、\
命令 ${command_pid:-没记下}$([[ $command_exited -eq 1 ]] && echo ' 停了' || echo ' 5 秒里没停')、留下的 scope「${scope_left}」标记「${marker_left}」：$(cat "$case_directory/output")"
    fi
  }
  fresh_context 256M
  checked=$((checked + 1))
  scope_stop_case TERM 143
  fresh_context 256M
  checked=$((checked + 1))
  scope_stop_case INT 130

  # 自证开出 slice 之后被停（文件头「自证的收尾」）：起一趟设了 RUN_WITH_MEMORY_CAP_SELFTEST_INTERRUPT_PROBE 的自证（经一层 python 把 INT 设回默认，& 起的作业 INT 默认被忽略），
  # 等它开出一个真 slice、写下探针文件，核那个 slice 此刻是 active（不然这一格什么都没证），只给 & 起、$! 记下的它发 TERM / INT，10 秒不退再发 KILL；
  # 退了之后核 systemctl --user list-units --all 里 singlefs_memory_selftest_<它的进程号>_*.slice 一个都不是 active、它的临时目录不在。
  # 弄坏开关只传 selftest-cleanup-unhooked 给它（别的开关不传：只红在点名那几格）；它没收干净的，这里替它停、删，slice 记进 selftest_slices 由这一趟收尾再 stop 加 revert
  interrupt_active_slices() { # interrupt_active_slices <那一趟自证的进程号> → 它开的 slice 里此刻 active 的，各带一个前导空格
    systemctl --user list-units --all --plain --no-legend --type=slice "singlefs_memory_selftest_$1_*.slice" 2>/dev/null | awk '$3 == "active" {printf " %s", $1}'
  }
  interrupt_case() { # interrupt_case <TERM|INT> <那一趟自证该退的码>
    local interrupt_signal="$1" expected_status="$2" probe="$scratch/interrupt-probe-$1" probe_pid poll probe_status=0 ready=0 probe_exited=0
    local probe_key probe_value probe_slices="" probe_scratch="" probe_recorded_pid="" active_before="" active_after="" probe_slice
    rm -f -- "$probe" "$probe.partial"
    python3 -c 'import os, signal, sys; signal.signal(signal.SIGINT, signal.SIG_DFL); os.execvp(sys.argv[1], sys.argv[1:])' \
      env RUN_WITH_MEMORY_CAP_SELFTEST_INTERRUPT_PROBE="$probe" \
      RUN_WITH_MEMORY_CAP_BREAK="$([[ "$BROKEN_JUDGEMENT" == selftest-cleanup-unhooked ]] && printf %s "$BROKEN_JUDGEMENT")" \
      bash "$target" --selftest </dev/null >"$scratch/interrupt-$interrupt_signal.out" 2>&1 &
    probe_pid=$!
    for ((poll = 0; poll < 300; poll++)); do   # 至多 30 秒等它写下探针文件
      if [[ -e "$probe" ]]; then ready=1; break; fi
      scope_stop_process_is_live "$probe_pid" || break
      sleep 0.1
    done
    if ((ready)); then
      while read -r probe_key probe_value; do
        case "$probe_key" in
          pid) probe_recorded_pid="$probe_value" ;;
          slice) probe_slices+=" $probe_value" ;;
          scratch) probe_scratch="$probe_value" ;;
        esac
      done < "$probe"
      active_before="$(interrupt_active_slices "$probe_pid")"
      kill -"$interrupt_signal" "$probe_pid"
    fi
    for ((poll = 0; poll < 100; poll++)); do   # 至多 10 秒等它退
      scope_stop_process_is_live "$probe_pid" || { probe_exited=1; break; }
      sleep 0.1
    done
    ((probe_exited)) || kill -KILL "$probe_pid" 2>/dev/null
    wait "$probe_pid"; probe_status=$?
    active_after="$(interrupt_active_slices "$probe_pid")"
    # 它没收干净的替它收：slice 记进这一趟的 selftest_slices 并当场停；临时目录只删 mktemp 起的那种名字
    for probe_slice in $probe_slices $active_after; do
      selftest_slices+=("$probe_slice")
      systemctl --user stop "$probe_slice" >/dev/null 2>&1
    done
    [[ "$probe_scratch" == */run-with-memory-cap-selftest-* && -d "$probe_scratch" ]] && rm -rf -- "${probe_scratch:?}" && probe_scratch+="（还在，这里删了）"
    if ((ready == 0)); then
      fail "自证开出 slice 之后被 $interrupt_signal 停这一格没判成：那一趟自证 30 秒里没写下探针文件，退 $probe_status：$(head_characters 600 "$scratch/interrupt-$interrupt_signal.out")"
    elif [[ "$probe_recorded_pid" != "$probe_pid" || -z "$active_before" ]]; then
      fail "自证开出 slice 之后被 $interrupt_signal 停这一格没判成：探针文件里的进程号「$probe_recorded_pid」应当是起的那一个 $probe_pid、被停之前它开的 slice（${probe_slices# }）应当有 active 的，\
实际 active 的「${active_before# }」：$(head_characters 600 "$scratch/interrupt-$interrupt_signal.out")"
    elif [[ $probe_exited -eq 0 || $probe_status -ne $expected_status || -n "$active_after" || "$probe_scratch" == *还在* ]]; then
      fail "自证开出 slice 之后被 $interrupt_signal 停时应当 10 秒内退 $expected_status、它开的 slice（${probe_slices# }）都停掉不再 active、它的临时目录删掉；\
实际$([[ $probe_exited -eq 1 ]] && echo '退了' || echo '10 秒里没退、发了 KILL')（$probe_status）、还 active 的「${active_after# }」、临时目录「${probe_scratch:-没记下}」：$(head_characters 600 "$scratch/interrupt-$interrupt_signal.out")"
    fi
  }
  checked=$((checked + 1))
  interrupt_case TERM 143
  checked=$((checked + 1))
  interrupt_case INT 130

  # 限时：超过 RUN_WITH_MEMORY_CAP_TIME_LIMIT 退 253，不跑满命令自己的时长
  fresh_context 256M
  checked=$((checked + 1))
  started_seconds="$(date +%s%N)"
  if in_context env RUN_WITH_MEMORY_CAP_TIME_LIMIT=1 RUN_WITH_MEMORY_CAP_KILL_GRACE=2 bash "$target" 64M sleep 4 2>"$scratch/timelimit.err"; then status=0; else status=$?; fi
  elapsed_seconds=$((($(date +%s%N) - started_seconds) / 1000000000))
  if [[ $status -ne $TIME_LIMIT_HIT_EXIT || $elapsed_seconds -ge 4 ]] || ! grep -q '超过限时 1 秒' "$scratch/timelimit.err"; then
    fail "限时 1 秒跑 sleep 4 应当 4 秒之内退 $TIME_LIMIT_HIT_EXIT、报「超过限时 1 秒」，实际退 $status、用了 $elapsed_seconds 秒：$(cat "$scratch/timelimit.err")"
  fi

  # 峰值表的口径（文件头「峰值表」一段）：采样子进程改采假 cgroup 目录（RUN_WITH_MEMORY_CAP_SELFTEST_SAMPLED_CGROUP），收尾时 memory.peak 与 oom_kill 也读它的；命令是 sleep 0.5，scope 的 Result=success。
  # 页缓存涨满上限：memory.current 等于上限 64M、active_file + inactive_file 占 48M，memory.peak 64M、oom_kill 0 ⇒ 记回收不掉的 16M、clean；
  # 内核在 scope 里杀过进程而 Result 没报 oom-kill：memory.current 40M、页缓存 10M，memory.peak 40M、oom_kill 1 ⇒ 记成撞了顶、峰值记上限 64M
  local fake_cgroup sampled_status case_label
  fresh_context 256M
  for case_label in page-cache oom-kill; do
    fake_cgroup="$scratch/fake-cgroup-$case_label"
    mkdir -p "$fake_cgroup"
    if [[ "$case_label" == page-cache ]]; then
      printf '%s\n' $((64 * MEBIBYTE)) > "$fake_cgroup/memory.current"
      printf 'anon %s\nfile %s\nkernel 0\nactive_file %s\ninactive_file %s\nshmem 0\n' $((16 * MEBIBYTE)) $((48 * MEBIBYTE)) $((32 * MEBIBYTE)) $((16 * MEBIBYTE)) > "$fake_cgroup/memory.stat"
      printf '%s\n' $((64 * MEBIBYTE)) > "$fake_cgroup/memory.peak"
      printf 'low 0\nhigh 0\nmax 40\noom 0\noom_kill 0\n' > "$fake_cgroup/memory.events"
    else
      printf '%s\n' $((40 * MEBIBYTE)) > "$fake_cgroup/memory.current"
      printf 'anon %s\nfile %s\nkernel 0\nactive_file %s\ninactive_file %s\nshmem 0\n' $((30 * MEBIBYTE)) $((10 * MEBIBYTE)) $((6 * MEBIBYTE)) $((4 * MEBIBYTE)) > "$fake_cgroup/memory.stat"
      printf '%s\n' $((40 * MEBIBYTE)) > "$fake_cgroup/memory.peak"
      printf 'low 0\nhigh 0\nmax 3\noom 1\noom_kill 1\n' > "$fake_cgroup/memory.events"
    fi
    if in_context env RUN_WITH_MEMORY_CAP_KEY="selftest-sampled-$case_label" RUN_WITH_MEMORY_CAP_SELFTEST_SAMPLED_CGROUP="$fake_cgroup" RUN_WITH_MEMORY_CAP_PEAK_OUTPUT="$scratch/sampled-$case_label.peak" bash "$target" 64M sleep 0.5 \
        2>"$scratch/sampled-$case_label.err"; then sampled_status=0; else sampled_status=$?; fi
    output="$(peak_row "selftest-sampled-$case_label")"
    checked=$((checked + 1))
    if [[ "$case_label" == page-cache ]]; then
      [[ $sampled_status -eq 0 && "$output" == "7|$((16 * MEBIBYTE))|$((16 * MEBIBYTE)):clean:unreclaimable" ]] \
        || fail "页缓存涨满上限（memory.current 等于上限 64M、其中 48M 是页缓存，memory.peak 64M、oom_kill 0）的那一趟应当退 0、峰值表记回收不掉的 16M 且 clean（「7|$((16 * MEBIBYTE))|$((16 * MEBIBYTE)):clean:unreclaimable」），不记上限，\
实际退 $sampled_status、记的「$output」（列数|峰值|近几次）：$(cat "$scratch/sampled-$case_label.err")"
      # 交给调用方的峰值与峰值表同一个口径（文件头「task-server 给这一层的两样」）：扣了页缓存
      checked=$((checked + 1))
      [[ "$(cat "$scratch/sampled-$case_label.peak" 2>/dev/null)" == "$((16 * MEBIBYTE))" ]] \
        || fail "页缓存涨满上限的那一趟，RUN_WITH_MEMORY_CAP_PEAK_OUTPUT 那一份应当是回收不掉的 16M（$((16 * MEBIBYTE))），实际「$(cat "$scratch/sampled-$case_label.peak" 2>/dev/null)」"
    else
      [[ $sampled_status -eq 0 && "$output" == "7|$((64 * MEBIBYTE))|$((64 * MEBIBYTE)):hit:unreclaimable" ]] \
        || fail "scope 的 memory.events 里 oom_kill 1 而 Result 没报 oom-kill（memory.peak 40M、回收不掉的 30M，没到上限 64M）的那一趟应当退 0、峰值表记成撞了顶、峰值记上限（「7|$((64 * MEBIBYTE))|$((64 * MEBIBYTE)):hit:unreclaimable」），\
实际退 $sampled_status、记的「$output」（列数|峰值|近几次）：$(cat "$scratch/sampled-$case_label.err")"
    fi
  done

  # 采样子进程不拖延收尾（文件头「峰值表」一段）：命令结束、超时、撞顶、被停四种收尾，包装退出那一刻它起的采样子进程已经停了，也不是等包装没了才自己退。
  # RUN_WITH_MEMORY_CAP_SELFTEST_SAMPLER_TRACE：包装起采样子进程时记「sampler <进程号> <起始时刻>」，采样子进程自己退时记「exit command-ended」或「exit wrapper-gone」（被包装停的不记）。
  # 被停那一例：命令在 scope 里起来（写下 ran 文件）、记录里有 sampler 那一行之后，只给 & 起、$! 记下的包装（env 直接 exec 成包装）发 TERM
  local sampler_case sampler_label sampler_trace sampler_expected sampler_status sampler_pid sampler_start sampler_now sampler_stat sampler_problem sampler_wrapper_pid sampler_poll
  local -a sampler_line=() sampler_fields=()
  stop_sampler_wrapper() { # <包装的进程号>：只给这一个包装（下面循环里「被停」那一例 & 起、$! 记下的）发 TERM，等它退出；退出码是包装的
    kill -TERM "$1"
    wait "$1"
  }
  for sampler_case in command-ended time-limit cap-hit stopped; do
    fresh_context 256M
    sampler_trace="$scratch/sampler-$sampler_case.trace"
    checked=$((checked + 1))
    case "$sampler_case" in
      command-ended)
        sampler_label="命令结束（sleep 0.5 退 0）" sampler_expected=0
        if in_context env RUN_WITH_MEMORY_CAP_SELFTEST_SAMPLER_TRACE="$sampler_trace" bash "$target" 64M sleep 0.5 \
            2>"$scratch/sampler-$sampler_case.err"; then sampler_status=0; else sampler_status=$?; fi ;;
      time-limit)
        sampler_label="超时（限时 1 秒跑 sleep 30，退 $TIME_LIMIT_HIT_EXIT）" sampler_expected=$TIME_LIMIT_HIT_EXIT
        if in_context env RUN_WITH_MEMORY_CAP_SELFTEST_SAMPLER_TRACE="$sampler_trace" RUN_WITH_MEMORY_CAP_TIME_LIMIT=1 RUN_WITH_MEMORY_CAP_KILL_GRACE=2 bash "$target" 64M sleep 30 \
            2>"$scratch/sampler-$sampler_case.err"; then sampler_status=0; else sampler_status=$?; fi ;;
      cap-hit)
        sampler_label="撞顶（上限 64M 里分配 512 MiB，退 $MEMORY_CAP_HIT_EXIT）" sampler_expected=$MEMORY_CAP_HIT_EXIT
        if in_context env RUN_WITH_MEMORY_CAP_SELFTEST_SAMPLER_TRACE="$sampler_trace" bash "$target" 64M python3 -c "$hog_code" 512 \
            >/dev/null 2>"$scratch/sampler-$sampler_case.err"; then sampler_status=0; else sampler_status=$?; fi ;;
      stopped)
        sampler_label="被停（命令在 scope 里跑着时给包装发 TERM，退 143）" sampler_expected=143
        env "${context[@]}" RUN_WITH_MEMORY_CAP_SELFTEST_SAMPLER_TRACE="$sampler_trace" bash "$target" 64M bash -c ': > "$1"; sleep 30' sampler-stopped "$scratch/sampler-stopped.ran" \
          >/dev/null 2>"$scratch/sampler-$sampler_case.err" &
        sampler_wrapper_pid=$!
        for ((sampler_poll = 0; sampler_poll < 150; sampler_poll++)); do   # 至多 15 秒等命令在 scope 里起来、记录里有 sampler 那一行
          [[ -e "$scratch/sampler-stopped.ran" ]] && grep -q '^sampler ' "$sampler_trace" 2>/dev/null && break
          sleep 0.1
        done
        stop_sampler_wrapper "$sampler_wrapper_pid"; sampler_status=$? ;;
    esac
    sampler_problem="" sampler_line=() sampler_now=""
    read -r -a sampler_line 2>/dev/null < <(grep -m 1 '^sampler ' "$sampler_trace" 2>/dev/null)
    sampler_pid="${sampler_line[1]:-}" sampler_start="${sampler_line[2]:-}"
    if [[ ! "$sampler_pid" =~ ^[0-9]+$ ]]; then
      sampler_problem+="记录里没有 sampler 那一行（包装没起采样子进程）；"
    elif IFS= read -r sampler_stat 2>/dev/null < "/proc/$sampler_pid/stat"; then
      sampler_fields=()
      read -r -a sampler_fields <<<"${sampler_stat##*) }"
      [[ "${sampler_fields[0]:-}" == Z ]] || sampler_now="${sampler_fields[19]:-}"
      [[ -z "$sampler_now" || "$sampler_now" != "$sampler_start" ]] || sampler_problem+="包装退出之后采样子进程 $sampler_pid 还在；"
    fi
    ! grep -q '^exit wrapper-gone' "$sampler_trace" 2>/dev/null || sampler_problem+="采样子进程等包装没了才自己退（记录里有 exit wrapper-gone）；"
    [[ -z "$(compgen -G "$context_directory/state/peaks/*.unreclaimable*")" ]] \
      || sampler_problem+="状态目录里留下了采样的输出（$(cd "$context_directory/state/peaks" 2>/dev/null && compgen -G '*.unreclaimable*' | tr '\n' ' ')）；"
    [[ $sampler_status -eq $sampler_expected ]] || sampler_problem+="包装退 $sampler_status（要 $sampler_expected）；"
    [[ "$sampler_case" != stopped || -e "$scratch/sampler-stopped.ran" ]] || sampler_problem+="命令 15 秒里没在 scope 里起来（发 TERM 时还没写下 ran 文件）；"
    [[ -z "$sampler_problem" ]] \
      || fail "采样子进程不拖延收尾：$sampler_label，包装退出那一刻采样子进程应当已经被包装停了、不留到包装退出之后，实际：$sampler_problem记录「$(tr '\n' ' ' 2>/dev/null < "$sampler_trace")」：$(head_characters 600 "$scratch/sampler-$sampler_case.err")"
  done

  # 撞顶的 scope 结局读早了（文件头「撞顶的 scope 结局读早了」）：假的 systemctl 对「查 scope 结局」的 show 头 N 次报「已收尾、Result=success」（读早了），
  # lag 那一格之后报 failed / oom-kill（systemd 后来记上了），stuck 那一格一直是 success（systemd 没记上）；
  # 头 3 次读早了的（lag）要在再查的几次里落到 oom-kill，退 250 报撞了上限、另报「读早了」一行；一直读早了的（stuck，自证把再查的缩到 20 次）按 memory.events 判，
  # 同样退 250、另报「一直是 success」一行；两样都不报成「被外面停了」。
  local fake_directory real_systemctl settle_name settle_unit
  fake_directory="$scratch/fake-systemctl"
  real_systemctl="$(command -v systemctl)"
  mkdir -p "$fake_directory"
  cat > "$fake_directory/systemctl" <<'FAKE_SYSTEMCTL'
#!/usr/bin/env bash
# 假 systemctl：查 scope 结局的那种 show（带 -p Result、末项是 <单元>.scope）设了 FAKE_SYSTEMCTL_RESULT_RESET 的照真的答、Result 换成 success；否则头 FAKE_SYSTEMCTL_LIES 次报「已收尾、Result=success」（读早了），之后报 FAKE_SYSTEMCTL_THEN 点名的 Result（没给就和真的一样）；别的调用原样交给真的
last="${*: -1}"
if [[ -n "${FAKE_SYSTEMCTL_RESULT_RESET:-}" && "${1:-} ${2:-}" == "--user show" && "$*" == *"-p Result"* && "$last" == *.scope ]]; then
  "${FAKE_SYSTEMCTL_REAL:?}" "$@" | sed 's/^Result=.*/Result=success/'
  exit 0
fi
if [[ "${1:-} ${2:-}" == "--user show" && "$*" == *"-p Result"* && "$last" == *.scope ]]; then
  count=0
  [[ -f "${FAKE_SYSTEMCTL_COUNT:?}" ]] && read -r count < "$FAKE_SYSTEMCTL_COUNT"
  count=$((count + 1))
  printf '%s\n' "$count" > "$FAKE_SYSTEMCTL_COUNT"
  if ((count <= ${FAKE_SYSTEMCTL_LIES:?})); then printf 'LoadState=loaded\nActiveState=inactive\nResult=success\n'; exit 0; fi
  if [[ -n "${FAKE_SYSTEMCTL_THEN:-}" ]]; then printf 'LoadState=loaded\nActiveState=failed\nResult=%s\n' "$FAKE_SYSTEMCTL_THEN"; exit 0; fi
fi
exec "${FAKE_SYSTEMCTL_REAL:?}" "$@"
FAKE_SYSTEMCTL
  chmod +x "$fake_directory/systemctl"
  for settle_name in lag stuck; do
    fresh_context 256M
    checked=$((checked + 1))
    if [[ "$settle_name" == lag ]]; then
      in_context env PATH="$fake_directory:$PATH" FAKE_SYSTEMCTL_REAL="$real_systemctl" FAKE_SYSTEMCTL_COUNT="$fake_directory/count-$settle_name" FAKE_SYSTEMCTL_LIES=3 FAKE_SYSTEMCTL_THEN=oom-kill \
        bash "$target" 64M python3 -c "$hog_code" 512 >/dev/null 2>"$scratch/settle-$settle_name.err"; status=$?
    else
      in_context env PATH="$fake_directory:$PATH" FAKE_SYSTEMCTL_REAL="$real_systemctl" FAKE_SYSTEMCTL_COUNT="$fake_directory/count-$settle_name" FAKE_SYSTEMCTL_LIES=100000 \
        RUN_WITH_MEMORY_CAP_SELFTEST_LATE_ATTEMPTS=20 bash "$target" 64M python3 -c "$hog_code" 512 >/dev/null 2>"$scratch/settle-$settle_name.err"; status=$?
    fi
    # 假的把真的 reset-failed 那一步挡掉了（stuck 那一格从头到尾挡着）：撞顶的 scope 停在 failed，收尾时替它清
    settle_unit="$(grep -o 'singlefs-memory-cap-[0-9]*-[0-9]*' "$scratch/settle-$settle_name.err" | head -n 1)"
    [[ -z "$settle_unit" ]] || systemctl --user reset-failed "$settle_unit.scope" >/dev/null 2>&1
    if [[ "$settle_name" == lag ]]; then
      if [[ $status -ne $MEMORY_CAP_HIT_EXIT ]] || ! grep -q '撞了内存上限 64M' "$scratch/settle-lag.err" || ! grep -q 'Result 读早了读到 success' "$scratch/settle-lag.err" \
          || grep -q '被外面停了' "$scratch/settle-lag.err"; then
        fail "撞顶的 scope 被读早了（头 3 次查 Result 都报已收尾、success，真的是 oom-kill）时，包装应当再查几次、退 $MEMORY_CAP_HIT_EXIT 报「撞了内存上限 64M」并另报「Result 读早了读到 success」、不报「被外面停了」，实际退 $status：$(tr '\n' ' ' < "$scratch/settle-lag.err")"
      fi
    elif [[ $status -ne $MEMORY_CAP_HIT_EXIT ]] || ! grep -q '撞了内存上限 64M' "$scratch/settle-stuck.err" || ! grep -q 'Result 一直是 success' "$scratch/settle-stuck.err" \
        || grep -q '被外面停了' "$scratch/settle-stuck.err"; then
      fail "撞顶的 scope 的 Result 一直读到 success，而它自己的 memory.events 里 oom_kill 不为 0 时，应当按 memory.events 判、退 $MEMORY_CAP_HIT_EXIT 报「撞了内存上限 64M」并另报「Result 一直是 success」、不报「被外面停了」，实际退 $status：$(tr '\n' ' ' < "$scratch/settle-stuck.err")"
    fi
  done
  # failed 而 Result 读到 success（用户级 systemd 重载过配置之后）：假 systemctl 把真 scope 的 Result 换成 success、ActiveState 照真的（failed）。
  # reset-timeout：限时 1 秒被停，journal 里有 Failed with result 'timeout'，应当照限时被停判（退 253）；reset-unknown：journalctl 换成读不了的，原因取不到，应当写明、不冒充限时被停
  printf '#!/usr/bin/env bash\nexit 1\n' > "$fake_directory/journalctl-broken"
  chmod +x "$fake_directory/journalctl-broken"
  for settle_name in reset-timeout reset-unknown; do
    fresh_context 256M
    checked=$((checked + 1))
    if [[ "$settle_name" == reset-timeout ]]; then
      in_context env PATH="$fake_directory:$PATH" FAKE_SYSTEMCTL_REAL="$real_systemctl" FAKE_SYSTEMCTL_RESULT_RESET=1 RUN_WITH_MEMORY_CAP_TIME_LIMIT=1 \
        bash "$target" 64M sleep 30 >/dev/null 2>"$scratch/settle-$settle_name.err"; status=$?
      if [[ $status -ne $TIME_LIMIT_HIT_EXIT ]] || ! grep -q '超过限时 1 秒' "$scratch/settle-reset-timeout.err" || ! grep -q 'journal 里它失败的原因是 timeout' "$scratch/settle-reset-timeout.err" \
          || grep -q '被外面停了' "$scratch/settle-reset-timeout.err"; then
        fail "限时被停的 scope 落到 failed 而 Result 读到 success（重载配置之后）时，包装应当到 journal 里找原因、照限时被停判（退 $TIME_LIMIT_HIT_EXIT 报「超过限时 1 秒」并另报「journal 里它失败的原因是 timeout」、不报「被外面停了」），实际退 $status：$(tr '\n' ' ' < "$scratch/settle-reset-timeout.err")"
      fi
    else
      in_context env PATH="$fake_directory:$PATH" FAKE_SYSTEMCTL_REAL="$real_systemctl" FAKE_SYSTEMCTL_RESULT_RESET=1 RUN_WITH_MEMORY_CAP_TIME_LIMIT=1 \
        RUN_WITH_MEMORY_CAP_JOURNALCTL="$fake_directory/journalctl-broken" bash "$target" 64M sleep 30 >/dev/null 2>"$scratch/settle-$settle_name.err"; status=$?
      if [[ $status -eq $TIME_LIMIT_HIT_EXIT ]] || ! grep -q '原因取不到' "$scratch/settle-reset-unknown.err" || grep -q '超过限时' "$scratch/settle-reset-unknown.err"; then
        fail "限时被停的 scope 落到 failed 而 Result 读到 success、journal 又读不了时，包装应当写明「原因取不到」、不冒充限时被停（不退 $TIME_LIMIT_HIT_EXIT、不报「超过限时」），实际退 $status：$(tr '\n' ' ' < "$scratch/settle-reset-unknown.err")"
      fi
    fi
  done

  # 这一条命令自己的临时目录（文件头「这一条命令自己的临时目录」一段）：假命令把拿到的 TMPDIR 写进 scratch 里的一份文件、往 TMPDIR 里写一份文件，再照各自的结局退；
  # 退 0 的核那一份 TMPDIR 在包装退出之后不在了、RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT 底下一个不剩；退非 0 的（退 7、kill -9、撞顶、超时）核那一份与命令写的文件都在、
  # 里面有标记、stderr 打出它的路径（「留着看现场」那一行）。各例的 root 在 scratch 里，留着的随 scratch 删。
  # 自证可能套在外层包装里跑（外层给了 TMPDIR），各例一律 env -u TMPDIR、-u SINGLEFS_KEEP_TEST_TEMPORARY_FILES 起，只在点名的那一例设
  local case_name case_status case_cell root recorded leftover caller code ending kept_age kept_problems now_seconds
  local -a case_environment=()
  code='printf "%s\n" "${TMPDIR-}" > "$1"; if [[ -n "${TMPDIR-}" ]]; then : > "$TMPDIR/left-by-command"; fi; '
  for case_name in normal nonzero killed cap-hit time-limit; do
    fresh_context 256M
    root="$scratch/temporary-root-$case_name"
    recorded="$scratch/temporary-$case_name.path"
    mkdir -p "$root"
    case_environment=()
    case "$case_name" in
      normal) ending='exit 0'; case_status=0; case_cell="命令正常退出之后，包装给它的临时目录应当删掉" ;;
      nonzero) ending='exit 7'; case_status=7; case_cell="命令退非 0 之后，包装给它的临时目录应当留着、stderr 打出路径" ;;
      killed) ending='kill -9 $$'; case_status=137; case_cell="命令被 kill -9 之后，包装给它的临时目录应当留着、stderr 打出路径" ;;
      cap-hit) ending='python3 -c "$2" 512'; case_status=$MEMORY_CAP_HIT_EXIT; case_cell="命令撞了 64M 上限被杀之后，包装给它的临时目录应当留着、stderr 打出路径" ;;
      time-limit) ending='sleep 30'; case_status=$TIME_LIMIT_HIT_EXIT; case_environment=(RUN_WITH_MEMORY_CAP_TIME_LIMIT=1)
        case_cell="命令超过限时被停之后，包装给它的临时目录应当留着、stderr 打出路径" ;;
    esac
    checked=$((checked + 1))
    if in_context env -u TMPDIR -u SINGLEFS_KEEP_TEST_TEMPORARY_FILES RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT="$root" ${case_environment[@]+"${case_environment[@]}"} \
        bash "$target" 64M bash -c "$code$ending" temporary-case "$recorded" "$hog_code" >/dev/null 2>"$scratch/temporary-$case_name.err"; then status=0; else status=$?; fi
    output="$(cat "$recorded" 2>/dev/null)"
    leftover="$(ls -A "$root" 2>/dev/null)"
    if [[ "$case_name" == normal ]]; then
      if [[ $status -ne $case_status || "$output" != "$root"/singlefs-run-with-memory-cap-* || -e "$output" || -n "$leftover" ]]; then
        fail "$case_cell（退 $case_status、命令拿到的 TMPDIR 在 $root 底下、跑完那一份不在了、$root 底下一个不剩），实际退 $status、命令拿到的 TMPDIR 是「$output」、\
那一份$([[ -n "$output" && -e "$output" ]] && echo "还在（里面：$(ls -A "$output" 2>/dev/null | tr '\n' ' ')）" || echo '不在')、$root 底下剩「$leftover」：$(head_characters 600 "$scratch/temporary-$case_name.err")"
      fi
    elif [[ $status -ne $case_status || "$output" != "$root"/singlefs-run-with-memory-cap-* || ! -e "$output/left-by-command" || ! -f "$output/$KEPT_TEMPORARY_MARKER" ]] \
        || ! grep -qF "这一条命令的临时目录留着看现场：$output" "$scratch/temporary-$case_name.err"; then
      fail "$case_cell（退 $case_status、命令拿到的 TMPDIR 在 $root 底下、跑完那一份与命令写的 left-by-command 都在、里面有标记、stderr 打出它的路径），实际退 $status、命令拿到的 TMPDIR 是「$output」、\
那一份$([[ -n "$output" && -e "$output" ]] && echo "在（里面：$(ls -A "$output" 2>/dev/null | tr '\n' ' ')）" || echo '不在了')：$(head_characters 600 "$scratch/temporary-$case_name.err")"
    fi
  done
  # 退非 0 而命令一个临时文件都没写：没有现场可看，照删、不打「留着看现场」
  fresh_context 256M
  root="$scratch/temporary-root-empty"
  recorded="$scratch/temporary-empty.path"
  mkdir -p "$root"
  checked=$((checked + 1))
  if in_context env -u TMPDIR -u SINGLEFS_KEEP_TEST_TEMPORARY_FILES RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT="$root" \
      bash "$target" 64M bash -c 'printf "%s\n" "${TMPDIR-}" > "$1"; exit 7' temporary-case "$recorded" >/dev/null 2>"$scratch/temporary-empty.err"; then status=0; else status=$?; fi
  output="$(cat "$recorded" 2>/dev/null)"
  leftover="$(ls -A "$root" 2>/dev/null)"
  if [[ $status -ne 7 || "$output" != "$root"/singlefs-run-with-memory-cap-* || -e "$output" || -n "$leftover" ]] || grep -qF '留着看现场' "$scratch/temporary-empty.err"; then
    fail "命令退非 0 而临时目录里什么都没写时，包装给它的临时目录应当照删（退 7、命令拿到的 TMPDIR 在 $root 底下、跑完那一份不在了、stderr 里没有「留着看现场」），实际退 $status、\
命令拿到的 TMPDIR 是「$output」、那一份$([[ -n "$output" && -e "$output" ]] && echo '还在' || echo '不在')、$root 底下剩「$leftover」：$(head_characters 600 "$scratch/temporary-empty.err")"
  fi
  # 判红留着的只留最近 KEPT_TEMPORARY_DIRECTORIES_LIMIT（3）个：root 下先放三份带标记的旧现场（标记的修改时刻距此刻依次是 300、200、100 秒）与一份没标记的（别的包装正在用的那种），
  # 一条退 7 的命令跑完：它那一份留着，旧的里最早的那一份删掉，另两份与没标记的那一份都在
  fresh_context 256M
  root="$scratch/temporary-root-limit"
  recorded="$scratch/temporary-limit.path"
  mkdir -p "$root/singlefs-run-with-memory-cap-1-in-use"
  : > "$root/singlefs-run-with-memory-cap-1-in-use/written-by-a-running-wrapper"
  printf -v now_seconds '%(%s)T' -1
  for kept_age in 300 200 100; do
    mkdir -p "$root/singlefs-run-with-memory-cap-1-kept-$kept_age"
    printf '包装退 1 留下的现场（自证造的）\n' > "$root/singlefs-run-with-memory-cap-1-kept-$kept_age/$KEPT_TEMPORARY_MARKER"
    touch -d "@$((now_seconds - kept_age))" "$root/singlefs-run-with-memory-cap-1-kept-$kept_age/$KEPT_TEMPORARY_MARKER"
  done
  checked=$((checked + 1))
  if in_context env -u TMPDIR -u SINGLEFS_KEEP_TEST_TEMPORARY_FILES RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT="$root" \
      bash "$target" 64M bash -c "${code}exit 7" temporary-case "$recorded" >/dev/null 2>"$scratch/temporary-limit.err"; then status=0; else status=$?; fi
  output="$(cat "$recorded" 2>/dev/null)"
  kept_problems=""
  [[ $status -eq 7 ]] || kept_problems+="退 $status；"
  [[ "$output" == "$root"/singlefs-run-with-memory-cap-* && -f "$output/$KEPT_TEMPORARY_MARKER" && -e "$output/left-by-command" ]] || kept_problems+="这一条那一份（$output）没留着；"
  [[ ! -e "$root/singlefs-run-with-memory-cap-1-kept-300" ]] || kept_problems+="最早那一份旧的（300 秒以前）还在；"
  [[ -e "$root/singlefs-run-with-memory-cap-1-kept-200" && -e "$root/singlefs-run-with-memory-cap-1-kept-100" ]] || kept_problems+="较新的两份旧的有一份不在了；"
  [[ -e "$root/singlefs-run-with-memory-cap-1-in-use/written-by-a-running-wrapper" ]] || kept_problems+="没标记的那一份（别的包装正在用的）被删了；"
  if [[ -n "$kept_problems" ]]; then
    fail "判红留着的只留最近 3 个（一条退 7 的命令跑完：它那一份留着、三份带标记的旧现场里最早那一份删掉、另两份与没标记的那一份都在），实际：${kept_problems}\
$root 底下剩「$(ls -A "$root" 2>/dev/null | tr '\n' ' ')」：$(head_characters 600 "$scratch/temporary-limit.err")"
  fi
  # 设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1：那一份留着（命令写的文件也在），stderr 打出它的路径；看完由这里删
  fresh_context 256M
  root="$scratch/temporary-root-kept"
  recorded="$scratch/temporary-kept.path"
  mkdir -p "$root"
  checked=$((checked + 1))
  if in_context env -u TMPDIR SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT="$root" \
      bash "$target" 64M bash -c "${code}exit 0" temporary-case "$recorded" >/dev/null 2>"$scratch/temporary-kept.err"; then status=0; else status=$?; fi
  output="$(cat "$recorded" 2>/dev/null)"
  if [[ $status -ne 0 || "$output" != "$root"/singlefs-run-with-memory-cap-* || ! -e "$output/left-by-command" ]] \
      || ! grep -qF "SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1：这一条命令的临时目录留着看现场：$output" "$scratch/temporary-kept.err"; then
    fail "设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 时临时目录应当留着、stderr 打出它的路径（退 0、$root 底下那一份与命令写的 left-by-command 都在），\
实际退 $status、命令拿到的 TMPDIR 是「$output」、那一份$([[ -n "$output" && -e "$output" ]] && echo '在' || echo '不在')：$(head_characters 600 "$scratch/temporary-kept.err")"
  fi
  rm -rf -- "${scratch:?}/temporary-root-kept"
  # 调用方自己给了 TMPDIR：命令拿到的就是它，包装不另建、不删；那一份与调用方放进去的、命令写的文件都还在
  fresh_context 256M
  root="$scratch/temporary-root-caller"
  caller="$scratch/temporary-given-by-caller"
  recorded="$scratch/temporary-caller.path"
  mkdir -p "$root" "$caller"
  : > "$caller/placed-by-caller"
  checked=$((checked + 1))
  if in_context env -u SINGLEFS_KEEP_TEST_TEMPORARY_FILES TMPDIR="$caller" RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT="$root" \
      bash "$target" 64M bash -c "${code}exit 0" temporary-case "$recorded" >/dev/null 2>"$scratch/temporary-caller.err"; then status=0; else status=$?; fi
  output="$(cat "$recorded" 2>/dev/null)"
  leftover="$(ls -A "$root" 2>/dev/null)"
  if [[ $status -ne 0 || "$output" != "$caller" || ! -e "$caller/placed-by-caller" || ! -e "$caller/left-by-command" || -n "$leftover" ]]; then
    fail "调用方设了 TMPDIR 时命令应当拿到调用方那一份、跑完那一份与里面的文件都还在（退 0、拿到 $caller、$root 底下不另建），\
实际退 $status、命令拿到的 TMPDIR 是「$output」、调用方那一份$([[ -e "$caller" ]] && echo "在（里面：$(ls -A "$caller" 2>/dev/null | tr '\n' ' ')）" || echo '不在了')、\
$root 底下剩「$leftover」：$(head_characters 600 "$scratch/temporary-caller.err")"
  fi
  # 给了 RUN_WITH_MEMORY_CAP_TEMPORARY_RECORD（文件头「multi-host-run.sh 给这一层的两样」）：命令跑着时那份文件里是它拿到的 TMPDIR 的全路径，
  # 命令里看不到这个变量，包装退出之后那份文件（连同 .partial）不在了
  local record_file record_seen
  fresh_context 256M
  root="$scratch/temporary-root-record"
  recorded="$scratch/temporary-record.path"
  record_file="$scratch/temporary-record.written"
  mkdir -p "$root"
  checked=$((checked + 1))
  if in_context env -u TMPDIR -u SINGLEFS_KEEP_TEST_TEMPORARY_FILES RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT="$root" RUN_WITH_MEMORY_CAP_TEMPORARY_RECORD="$record_file" \
      bash "$target" 64M bash -c "${code}"'{ cat -- "$2" 2>/dev/null || echo 读不到; printf "%s\n" "${RUN_WITH_MEMORY_CAP_TEMPORARY_RECORD-没有}"; } > "$1.record"; exit 0' \
      temporary-case "$recorded" "$record_file" >/dev/null 2>"$scratch/temporary-record.err"; then status=0; else status=$?; fi
  output="$(cat "$recorded" 2>/dev/null)"
  record_seen="$(tr '\n' '|' 2>/dev/null < "$recorded.record")"
  if [[ $status -ne 0 || "$output" != "$root"/singlefs-run-with-memory-cap-* || "$record_seen" != "$output|没有|" || -e "$record_file" || -e "$record_file.partial" ]]; then
    fail "给了 RUN_WITH_MEMORY_CAP_TEMPORARY_RECORD 时包装应当把建的临时目录的全路径写进那份文件、命令里看不到这个变量、退出之后删掉那份文件（退 0、命令跑着时读到「<它的 TMPDIR>|没有|」），\
实际退 $status、命令拿到的 TMPDIR 是「$output」、命令跑着时读到「$record_seen」、那份文件退出之后$([[ -e "$record_file" ]] && echo '还在' || echo '不在了')：$(head_characters 600 "$scratch/temporary-record.err")"
  fi
  # 设了现场目录（文件头「task-server 给这一层的两样」）：要留的那一份整个搬成那个目录（不放标记、root 底下不剩、原路径不在了）、stderr 点名新位置；
  # 退 7 的与设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 退 0 的各一例。命令里看不到这两个变量
  local scene_case scene_directory scene_environment_seen scene_elapsed scene_manifest scene_misplaced scene_not_moved scene_logs_not_moved
  for scene_case in failed kept; do
    fresh_context 256M
    root="$scratch/temporary-root-scene-$scene_case"
    recorded="$scratch/temporary-scene-$scene_case.path"
    scene_directory="$scratch/scene-of-attempt-$scene_case"
    mkdir -p "$root"
    if [[ "$scene_case" == failed ]]; then
      ending='exit 7'; case_status=7; case_environment=(-u SINGLEFS_KEEP_TEST_TEMPORARY_FILES)
      case_cell="设了现场目录、命令退 7 而临时目录里有东西时"
    else
      ending='exit 0'; case_status=0; case_environment=(SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1)
      case_cell="设了现场目录与 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 时"
    fi
    checked=$((checked + 1))
    if in_context env -u TMPDIR "${case_environment[@]}" RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT="$root" RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY="$scene_directory" \
        bash "$target" 64M bash -c "${code}printf '%s|%s\n' \"\${RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY-没有}\" \"\${RUN_WITH_MEMORY_CAP_PEAK_OUTPUT-没有}\" > \"\$1.environment\"; $ending" \
        temporary-case "$recorded" >/dev/null 2>"$scratch/temporary-scene-$scene_case.err"; then status=0; else status=$?; fi
    output="$(cat "$recorded" 2>/dev/null)"
    scene_environment_seen="$(cat "$recorded.environment" 2>/dev/null)"
    leftover="$(ls -A "$root" 2>/dev/null)"
    if [[ $status -ne $case_status || "$output" != "$root"/singlefs-run-with-memory-cap-* || -e "$output" || -n "$leftover" || ! -e "$scene_directory/left-by-command" \
          || -e "$scene_directory/$KEPT_TEMPORARY_MARKER" || "$scene_environment_seen" != "没有|没有" ]] \
        || ! grep -qF "搬进了这一次尝试的现场目录：$scene_directory" "$scratch/temporary-scene-$scene_case.err"; then
      fail "${case_cell}，包装给它的临时目录应当整个搬成现场目录（退 $case_status、$scene_directory 里有命令写的 left-by-command、没有标记、原来那一份不在了、$root 底下一个不剩、stderr 点名现场目录、命令里看不到两个变量），\
实际退 $status、原来那一份「$output」$([[ -n "$output" && -e "$output" ]] && echo '还在' || echo '不在')、现场目录里「$(ls -A "$scene_directory" 2>/dev/null | tr '\n' ' ')」、$root 底下剩「$leftover」、\
命令看到的「$scene_environment_seen」：$(head_characters 600 "$scratch/temporary-scene-$scene_case.err")"
    fi
  done
  # 现场目录搬多少（文件头「现场目录搬多少」）：六例，命令往 TMPDIR 里写好东西再退 7，包装把它搬进现场目录。
  #   scene_run <例名> <命令正文> [<环境 键=值>…]：置 root、recorded、scene_directory、status、output（命令拿到的 TMPDIR）、leftover（root 底下剩的），stderr 在 $scratch/scene-<例名>.err
  scene_run() {
    local run_name="$1" run_body="$2"
    shift 2
    fresh_context 256M
    root="$scratch/temporary-root-scene-$run_name"
    recorded="$scratch/temporary-scene-$run_name.path"
    scene_directory="$scratch/scene-of-attempt-$run_name"
    mkdir -p "$root"
    checked=$((checked + 1))
    if in_context env -u TMPDIR -u SINGLEFS_KEEP_TEST_TEMPORARY_FILES RUN_WITH_MEMORY_CAP_TEMPORARY_ROOT="$root" RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY="$scene_directory" "$@" \
        bash "$target" 64M bash -c "${code}${run_body}; exit 7" temporary-case "$recorded" >/dev/null 2>"$scratch/scene-$run_name.err"; then status=0; else status=$?; fi
    output="$(cat "$recorded" 2>/dev/null)"
    leftover="$(ls -A "$root" 2>/dev/null)"
  }
  manifest_has() { # manifest_has <清单> <状态> <相对路径>：清单里有这一行（第一列状态、第五列路径）交 0
    awk -F'\t' -v state="$2" -v path="$3" '$1 == state && $5 == path { found = 1 } END { exit found ? 0 : 1 }' "$1" 2>/dev/null
  }
  # 超上限（个数上限经自证专用的变量换成 20）：30 个数据文件加两份日志，只搬两份日志，数据文件只记进清单，原来那一份删掉
  scene_run bounded 'mkdir -p "$TMPDIR/bulk" "$TMPDIR/logs"; for index in $(seq 1 30); do printf "%s\n" "$index" > "$TMPDIR/bulk/part-$index.bin"; done
    printf "日志\n" > "$TMPDIR/logs/run.log"; printf "报告\n" > "$TMPDIR/logs/detail.report"' RUN_WITH_MEMORY_CAP_SELFTEST_SCENE_ENTRY_LIMIT=20
  if [[ $status -ne 7 || -n "$leftover" || ! -f "$scene_directory/logs/run.log" || ! -f "$scene_directory/logs/detail.report" || -e "$scene_directory/bulk" ]] \
      || ! manifest_has "$scene_directory/$SCENE_MANIFEST_NAME" over-limit-not-moved bulk/part-1.bin || ! manifest_has "$scene_directory/$SCENE_MANIFEST_NAME" moved logs/run.log; then
    fail "搬现场超上限时（个数上限 20、要搬 33 个）应当只搬日志、别的只记进清单（退 7、现场里有 logs/run.log 与 logs/detail.report、没有 bulk、清单里 bulk/part-1.bin 记 over-limit-not-moved、$root 底下一个不剩），\
实际退 $status、现场里「$(cd "$scene_directory" 2>/dev/null && find . -mindepth 1 -maxdepth 2 | sort | head -n 12 | tr '\n' ' ')」、$root 底下剩「$leftover」：$(head_characters 600 "$scratch/scene-bounded.err")"
  fi
  # 进度行：250 个小文件（个数上限照默认），开搬一行、每 100 个一行、搬完一行
  scene_run progress 'mkdir -p "$TMPDIR/many"; for index in $(seq 1 250); do : > "$TMPDIR/many/entry-$index"; done'
  if [[ $status -ne 7 || "$(find "$scene_directory/many" -type f 2>/dev/null | wc -l)" -ne 250 ]] || ! grep -qF '搬现场进度：开搬' "$scratch/scene-progress.err" \
      || ! grep -qF '搬现场进度：100/' "$scratch/scene-progress.err" || ! grep -qF '搬现场进度：200/' "$scratch/scene-progress.err" || ! grep -qF '搬现场进度：搬完' "$scratch/scene-progress.err"; then
    fail "搬现场的过程应当打进度行（退 7、现场里 250 个文件都在、stderr 里有「搬现场进度：开搬」「搬现场进度：100/」「搬现场进度：200/」「搬现场进度：搬完」），\
实际退 $status、现场里 $(find "$scene_directory/many" -type f 2>/dev/null | wc -l) 个、进度行「$(grep -F '搬现场进度：' "$scratch/scene-progress.err" | tr '\n' ' ')」：$(head_characters 600 "$scratch/scene-progress.err")"
  fi
  # 稀疏的大镜像：truncate 出来的 32M（占盘 0）不搬、清单里记路径与表观字节，旁边的小日志照搬
  scene_run sparse 'mkdir -p "$TMPDIR/replay-out"; truncate -s 32M "$TMPDIR/replay-out/e9.img"; printf "x\n" > "$TMPDIR/replay-out/small.log"'
  if [[ $status -ne 7 || -n "$leftover" || ! -f "$scene_directory/replay-out/small.log" || -e "$scene_directory/replay-out/e9.img" ]] \
      || ! awk -F'\t' '$1 == "sparse-not-moved" && $3 == 33554432 && $5 == "replay-out/e9.img" { found = 1 } END { exit found ? 0 : 1 }' "$scene_directory/$SCENE_MANIFEST_NAME" 2>/dev/null; then
    fail "现场里稀疏的大镜像应当不搬、只记路径与大小（退 7、现场里有 replay-out/small.log、没有 replay-out/e9.img、清单里 e9.img 记 sparse-not-moved 与表观 33554432 字节），\
实际退 $status、现场里「$(cd "$scene_directory" 2>/dev/null && find . -mindepth 1 | sort | tr '\n' ' ')」、清单「$(grep -F e9.img "$scene_directory/$SCENE_MANIFEST_NAME" 2>/dev/null | tr '\t' ' ')」：$(head_characters 600 "$scratch/scene-sparse.err")"
  fi
  # 中途失败：五份读得了的日志在前、一份读不了的（chmod 000）在后，搬到它时失败：搬了一半的临时名删掉、现场目录不建、原来那一份照旧留在 root 下（放标记）
  scene_run failing 'for index in 1 2 3 4 5; do printf "%s\n" "$index" > "$TMPDIR/early-$index.log"; done; printf "锁着\n" > "$TMPDIR/locked.log"; chmod 000 "$TMPDIR/locked.log"'
  if [[ $status -ne 7 || -e "$scene_directory" || -n "$(compgen -G "$scene_directory.partial-*")" || ! -e "$output/locked.log" || ! -f "$output/$KEPT_TEMPORARY_MARKER" ]] \
      || ! grep -qF '搬了一半的临时名已删' "$scratch/scene-failing.err" || ! grep -qF "没搬进这一次尝试的现场目录 $scene_directory" "$scratch/scene-failing.err"; then
    fail "搬现场中途失败时应当不留半份（退 7、$scene_directory 与 $scene_directory.partial-* 都不在、原来那一份连同 locked.log 留在 $root 下、有标记、stderr 说搬了一半的临时名已删），\
实际退 $status、现场目录$([[ -e "$scene_directory" ]] && echo '在' || echo '不在')、临时名「$(compgen -G "$scene_directory.partial-*" | tr '\n' ' ')」、原来那一份「$output」$([[ -n "$output" && -e "$output" ]] && echo '在' || echo '不在')：\
$(head_characters 600 "$scratch/scene-failing.err")"
  fi
  [[ -n "$output" && -e "$output/locked.log" ]] && chmod 600 "$output/locked.log"
  # 并行（自证专用的慢拷贝每个文件睡 0.2 秒）：80 个文件逐个搬不会少于 80 × 0.2 = 16 秒，16 路并行约 1 秒；
  #   结局句「现场搬了 80 个（用了 S 秒）」的 S 应当不到逐个搬的一半（8 秒）：比的是逐个搬的下限，不靠机器快慢
  scene_run parallel 'mkdir -p "$TMPDIR/slow"; for index in $(seq 1 80); do printf "%s\n" "$index" > "$TMPDIR/slow/part-$index.bin"; done' \
    RUN_WITH_MEMORY_CAP_SELFTEST_SCENE_COPY_DELAY=0.2
  # 个数不钉 80：现场里另有这一趟自己带的文件（量到搬的是 81 个），这一格核的是那 80 个都在与用时
  scene_elapsed="$(grep -oE '现场搬了 [0-9]+ 个（用了 [0-9]+[.][0-9]+ 秒）' "$scratch/scene-parallel.err" | grep -oE '用了 [0-9]+[.][0-9]+' | grep -oE '[0-9]+[.][0-9]+' | head -n 1)"
  if [[ $status -ne 7 || -n "$leftover" || "$(find "$scene_directory/slow" -type f 2>/dev/null | wc -l)" -ne 80 || -z "$scene_elapsed" ]] \
      || ! awk -v elapsed="$scene_elapsed" 'BEGIN { exit (elapsed + 0 < 80 * 0.2 / 2) ? 0 : 1 }'; then
    fail "搬现场 80 个每个慢 0.2 秒的文件时应当并行搬、用时不到逐个搬的一半（退 7、现场里 80 个都在、$root 底下一个不剩、结局句「现场搬了 80 个（用了 S 秒）」的 S 不到 8），\
实际退 $status、现场里 $(find "$scene_directory/slow" -type f 2>/dev/null | wc -l) 个、$root 底下剩「$leftover」、用了「$scene_elapsed」秒：$(head_characters 600 "$scratch/scene-parallel.err")"
  fi
  # 总时限（自证专用的变量换成 1 秒，慢拷贝每个文件睡 0.5 秒）：120 个数据文件排在路径前面（a-bulk/）、两份日志排在路径后面（z-logs/），16 路 1 秒只搬得了几十个；
  #   日志应当先搬，已搬的在现场里，没搬的数据文件在清单里记 time-limit-not-moved、不在现场里，照搬成了办（退 7、原来那一份删掉、不放标记、stderr 点名现场目录与到时限那一句）
  scene_run timelimit 'mkdir -p "$TMPDIR/a-bulk" "$TMPDIR/z-logs"; for index in $(seq 1 120); do printf "%s\n" "$index" > "$TMPDIR/a-bulk/part-$index.bin"; done
    printf "日志\n" > "$TMPDIR/z-logs/run.log"; printf "报告\n" > "$TMPDIR/z-logs/detail.report"' \
    RUN_WITH_MEMORY_CAP_SELFTEST_SCENE_COPY_DELAY=0.5 RUN_WITH_MEMORY_CAP_SELFTEST_SCENE_TIME_LIMIT=1
  scene_manifest="$scene_directory/$SCENE_MANIFEST_NAME"
  scene_not_moved="$(awk -F'\t' '$1 == "time-limit-not-moved" { count++ } END { print count + 0 }' "$scene_manifest" 2>/dev/null)"
  scene_logs_not_moved="$(awk -F'\t' '$1 == "time-limit-not-moved" && $5 !~ /[.]bin$/ { count++ } END { print count + 0 }' "$scene_manifest" 2>/dev/null)"
  scene_misplaced="$(awk -F'\t' '$1 == "moved" || $1 == "time-limit-not-moved" { print $1 "\t" $5 }' "$scene_manifest" 2>/dev/null \
    | while IFS=$'\t' read -r manifest_state manifest_path; do
        if [[ "$manifest_state" == moved && ! -f "$scene_directory/$manifest_path" ]] || [[ "$manifest_state" == time-limit-not-moved && -e "$scene_directory/$manifest_path" ]]; then
          printf '%s %s；' "$manifest_state" "$manifest_path"
        fi
      done)"
  if [[ $status -ne 7 || -n "$leftover" || ! -f "$scene_directory/z-logs/run.log" || ! -f "$scene_directory/z-logs/detail.report" || -e "$scene_directory/$KEPT_TEMPORARY_MARKER" \
        || -n "$(compgen -G "$scene_directory.partial-*")" || ! "$scene_not_moved" =~ ^[1-9][0-9]*$ || "$scene_logs_not_moved" != 0 || -n "$scene_misplaced" ]] \
      || ! manifest_has "$scene_manifest" moved z-logs/run.log || ! grep -qF '到时限没搬的' "$scratch/scene-timelimit.err" \
      || ! grep -qF "搬进了这一次尝试的现场目录：$scene_directory" "$scratch/scene-timelimit.err"; then
    fail "搬现场到总时限时应当照搬成了办、日志先搬、没搬的记进清单（退 7、$root 底下一个不剩、现场里有 z-logs/run.log 与 z-logs/detail.report、没有标记、没有临时名、\
清单里至少一行 time-limit-not-moved 且都是 a-bulk/ 下的 .bin、记 moved 的都在现场里、记 time-limit-not-moved 的都不在、stderr 有「到时限没搬的」与「搬进了这一次尝试的现场目录」），\
实际退 $status、$root 底下剩「$leftover」、time-limit-not-moved 「$scene_not_moved」行（其中不是 .bin 的「$scene_logs_not_moved」行）、清单与现场对不上的「$scene_misplaced」、\
现场里「$(cd "$scene_directory" 2>/dev/null && find . -name '*.log' -o -name '*.report' | sort | tr '\n' ' ')」：\
$(head_characters 600 "$scratch/scene-timelimit.err")"
  fi
  # 退出时交的峰值（文件头「task-server 给这一层的两样」）：命令跑 0.1 秒就退，比调用方一轮采样短；回收不掉的用量的峰值写进 RUN_WITH_MEMORY_CAP_PEAK_OUTPUT 那一份
  fresh_context 256M
  checked=$((checked + 1))
  rm -f -- "$scratch/exit-peak"
  if in_context env RUN_WITH_MEMORY_CAP_PEAK_OUTPUT="$scratch/exit-peak" RUN_WITH_MEMORY_CAP_KEY=selftest-exit-peak bash "$target" 64M sleep 0.1 2>"$scratch/exit-peak.err"; then status=0; else status=$?; fi
  output="$(cat "$scratch/exit-peak" 2>/dev/null)"
  if [[ $status -ne 0 || ! "$output" =~ ^[1-9][0-9]*$ || -e "$scratch/exit-peak.partial" ]]; then
    fail "命令跑 0.1 秒就退时，RUN_WITH_MEMORY_CAP_PEAK_OUTPUT 那一份应当是回收不掉的用量的峰值（退 0、一个正整数、不剩 .partial），实际退 $status、读到「$output」：$(head_characters 600 "$scratch/exit-peak.err")"
  fi
  # scope 里的命令抬 oom_score_adj（TASK_OOM_SCORE_ADJUST）：整机吃紧时先挑任务进程，不挑常驻服务。
  # oom_score_adj 只抬得上去、降不回来，自证经服务起时已继承了抬过的值：目标取自己此刻的值 + 1（RUN_WITH_MEMORY_CAP_SELFTEST_OOM_SCORE_ADJUST），读到它才说明是被测包装设的
  fresh_context 256M
  checked=$((checked + 1))
  local own_oom_score="" wanted_oom_score=""
  IFS= read -r own_oom_score < /proc/self/oom_score_adj
  wanted_oom_score=$(( own_oom_score < 1000 ? own_oom_score + 1 : 1000 ))
  if in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-oom-score RUN_WITH_MEMORY_CAP_SELFTEST_OOM_SCORE_ADJUST="$wanted_oom_score" bash "$target" 64M bash -c 'cat /proc/self/oom_score_adj > "$1"' oom "$scratch/oom-score" 2>"$scratch/oom-score.err"; then status=0; else status=$?; fi
  output="$(cat "$scratch/oom-score" 2>/dev/null)"
  if [[ $status -ne 0 || "$output" != "$wanted_oom_score" || "$own_oom_score" -ge 1000 ]]; then
    fail "scope 里的命令 oom_score_adj 应当是被测包装设的 $wanted_oom_score（自证自己此刻是 $own_oom_score；到 1000 判不出），实际退 $status、读到「$output」：$(head_characters 600 "$scratch/oom-score.err")"
  fi

  # 峰值表的「线程数」一列：命令里经 capped.sh 3 跑的记 3；经 capped.sh default 跑的记登记值（登记文件第一条非注释行的数），SINGLEFS_THREAD_CAP 设了的记它；
  # 环境里线程变量都是 2 的记 2；不一样的逐个记「名=值」；都没设、命令也没经 capped.sh 的记「未设（整机 N 核）」；
  local -a thread_variable_unsets=()
  local thread_variable_name expectation expected_key expected_threads registered_threads
  while IFS='=' read -r thread_variable_name _; do thread_variable_unsets+=(-u "$thread_variable_name"); done < <(bash "$SCRIPT_DIRECTORY/capped.sh" --print 1)
  fresh_context 256M
  in_context env "${thread_variable_unsets[@]}" RUN_WITH_MEMORY_CAP_KEY=selftest-threads-capped bash "$target" 64M bash "$SCRIPT_DIRECTORY/capped.sh" 3 true 2>>"$scratch/threads.err"
  in_context env "${thread_variable_unsets[@]}" CARGO_BUILD_JOBS=2 SINGLEFS_THREAD_CAP=2 RUN_WITH_MEMORY_CAP_KEY=selftest-threads-same bash "$target" 64M true 2>>"$scratch/threads.err"
  in_context env "${thread_variable_unsets[@]}" CARGO_BUILD_JOBS=2 SINGLEFS_CRASH_POINT_REPLAY_THREADS=1 RUN_WITH_MEMORY_CAP_KEY=selftest-threads-differing bash "$target" 64M true \
    2>>"$scratch/threads.err"
  in_context env "${thread_variable_unsets[@]}" RUN_WITH_MEMORY_CAP_KEY=selftest-threads-unset bash "$target" 64M true 2>>"$scratch/threads.err"
  registered_threads="$(awk '!/^[[:space:]]*(#|$)/ {print $1; exit}' "$SCRIPT_DIRECTORY/../../.claude/heavy-thread-default" 2>/dev/null)"
  in_context env "${thread_variable_unsets[@]}" RUN_WITH_MEMORY_CAP_KEY=selftest-threads-default bash "$target" 64M bash "$SCRIPT_DIRECTORY/capped.sh" default true 2>>"$scratch/threads.err"
  in_context env "${thread_variable_unsets[@]}" SINGLEFS_THREAD_CAP=5 RUN_WITH_MEMORY_CAP_KEY=selftest-threads-default-cap bash "$target" 64M bash "$SCRIPT_DIRECTORY/capped.sh" default true \
    2>>"$scratch/threads.err"
  for expectation in "selftest-threads-capped|3" "selftest-threads-same|2" "selftest-threads-differing|CARGO_BUILD_JOBS=2,SINGLEFS_CRASH_POINT_REPLAY_THREADS=1" \
      "selftest-threads-unset|未设（整机 $(nproc) 核）" "selftest-threads-default|$registered_threads" "selftest-threads-default-cap|5"; do
    expected_key="${expectation%%|*}" expected_threads="${expectation#*|}"
    checked=$((checked + 1))
    output="$(awk -F '\t' -v key="$expected_key" '$4 == key {print NF "|" $5}' "$context_directory/peaks.tsv" 2>/dev/null)"
    [[ "$output" == "7|$expected_threads" ]] \
      || fail "峰值表里 $expected_key 那一行应当是七列、线程数记「$expected_threads」，实际「$output」（列数|线程数）：$(cat "$scratch/threads.err")"
  done

  # 峰值表两台共用一份（文件头「峰值表」一段）：第七列记是哪一台（自证给的 RUN_WITH_MEMORY_CAP_SELFTEST_HOST），另一台同一个键的行原样留着
  fresh_context 256M
  local other_host_row
  other_host_row="$(printf '%s\t%s\t%s\t%s\t%s\t%s\t%s' $((5 * MEBIBYTE)) 2026-09-25T00:00:00Z 200M selftest-host-column 1 "$((5 * MEBIBYTE)):clean:unreclaimable" 192.0.2.99)"  # clock-times:allow 自检输入假时间戳
  printf '%s\n' "$other_host_row" >> "$context_directory/peaks.tsv"
  checked=$((checked + 1))
  in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-host-column bash "$target" 64M true 2>"$scratch/host-column.err"
  output="$(awk -F '\t' '$4 == "selftest-host-column" {print NF "|" $7}' "$context_directory/peaks.tsv" 2>/dev/null | LC_ALL=C sort | tr '\n' ' ')"
  if [[ "$output" != "7|$selftest_peak_host 7|192.0.2.99 " ]] || ! grep -qxF -- "$other_host_row" "$context_directory/peaks.tsv"; then
    fail "峰值表两台共用一份：另一台同一个键的行原样留着、本台那一行第七列记 $selftest_peak_host，实际这个键的行「$output」（列数|哪一台）：$(cat "$scratch/host-column.err")"
  fi

  # 峰值表里有不是这一版的行（这里写一行四列的）：这一趟的峰值记不进，说一句（点名行号与要几列，不是回溯栈），命令的退出码照旧，表一个字节不动
  fresh_context 256M
  printf '%s\t%s\t%s\t%s\n' $((10 * MEBIBYTE)) 2026-09-25T00:00:00Z 200M selftest-four-columns >> "$context_directory/peaks.tsv"  # clock-times:allow 自检输入假时间戳
  checked=$((checked + 1))
  refused_before="$(cat "$context_directory/peaks.tsv")"
  if in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-refused-row bash "$target" 64M bash -c 'exit 7' 2>"$scratch/refused-row.err"; then status=0; else status=$?; fi
  if [[ $status -ne 7 || "$(cat "$context_directory/peaks.tsv")" != "$refused_before" ]] || ! grep -q '峰值没记进表：.*第 1 行不是这一版的形态：列数不对，要 7 列，这一行 4 列' "$scratch/refused-row.err" \
      || grep -q 'Traceback' "$scratch/refused-row.err"; then
    fail "峰值表里有不是这一版的行（四列）：命令退 7 应当照旧退 7、峰值表不动、stderr 说「峰值没记进表」并点名第 1 行与要 7 列（不是回溯栈），实际退 $status：$(head_characters 600 "$scratch/refused-row.err")"
  fi

  # 只认服务直接起的那一层（文件头「只认服务直接起的那一层」一段）：没有准入标记 ⇒ 按用法错退 251、命令没跑、出路写「任务里面不再套包装」
  fresh_context 256M
  checked=$((checked + 1))
  marker="$scratch/unadmitted-ran"
  if in_context env -u RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER bash "$target" 64M bash -c ': > "$1"' touch-marker "$marker" 2>"$scratch/unadmitted.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_UNAVAILABLE_EXIT || -e "$marker" ]] || ! grep -q '任务里面不再套包装：内存、线程、机器、卡由 task-server 定；删掉这一层' "$scratch/unadmitted.err" 2>/dev/null; then
    fail "没有服务给的准入标记时应当按用法错退 $MEMORY_CAP_UNAVAILABLE_EXIT、命令不跑、出路写「任务里面不再套包装」，实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/unadmitted.err" 2>/dev/null)"
  fi
  # 被服务起的命令里面再调这一份：里层看不到准入标记 ⇒ 按用法错退 251、命令没跑；外层照跑完、退 0
  fresh_context 256M
  checked=$((checked + 1))
  marker="$scratch/nested-ran"
  rm -f -- "$scratch/nested.status"
  if in_context bash "$target" 100M bash -c 'bash "$1" 64M bash -c ": > \"\$1\"" touch-marker "$2" 2>"$3"; echo "$?" > "$4"' nested-outer "$target" "$marker" \
      "$scratch/nested.err" "$scratch/nested.status" 2>/dev/null; then status=0; else status=$?; fi
  output="$(cat "$scratch/nested.status" 2>/dev/null)"
  if [[ $status -ne 0 || "$output" != "$MEMORY_CAP_UNAVAILABLE_EXIT" || -e "$marker" ]]; then
    fail "被服务起的命令里面再调这一份应当按用法错退 $MEMORY_CAP_UNAVAILABLE_EXIT、命令不跑，\
实际外层退 $status、里层退 ${output:-（没记下）}、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/nested.err" 2>/dev/null)"
  fi

  # 已由 task-server 准入（文件头「只认服务直接起的那一层」一段）：设了 RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER，命令仍在这一例 slice 底下自己的 scope 里，看不到这个变量
  fresh_context 256M
  checked=$((checked + 1))
  rm -f -- "$scratch/admitted-seen.txt" "$scratch/admitted-cgroup.txt"
  if in_context env RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER=selftest-task.1 RUN_WITH_MEMORY_CAP_KEY=selftest-admitted-by-task-server \
      bash "$target" 64M bash -c 'printf "%s" "${RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER-没有}" > "$1"; IFS= read -r line < /proc/self/cgroup; printf "%s\n" "${line#0::}" > "$2"' \
      admitted-probe "$scratch/admitted-seen.txt" "$scratch/admitted-cgroup.txt" 2>"$scratch/admitted.err"; then status=0; else status=$?; fi
  output="$(cat "$scratch/admitted-seen.txt" 2>/dev/null)"
  cgroup_path="$(cat "$scratch/admitted-cgroup.txt" 2>/dev/null)"
  if [[ $status -ne 0 || "$output" != 没有 || "$cgroup_path" != */"$slice"/singlefs-memory-cap-*.scope ]]; then
    fail "已由 task-server 准入的一条应当照常在这一例 slice 底下自己的 scope 里跑、退 0，命令里看不到 RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER，\
实际退 $status、命令看到「${output:-（没记下）}」、命令的 cgroup「${cgroup_path:-（没记下）}」：$(cat "$scratch/admitted.err" 2>/dev/null)"
  fi

  # 收尾之后不留 scope：每个 slice 底下没有 scope
  checked=$((checked + 1))
  output=""
  for slice in "${selftest_slices[@]}"; do
    if compgen -G "/sys/fs/cgroup/user.slice/user-$(id -u).slice/user@$(id -u).service/$slice/*.scope" >/dev/null; then output+=" $slice 底下还有 scope"; fi
  done
  [[ -z "$output" ]] || fail "跑完应当不留 scope，实际：$output"

  # --status 的本地模型服务那一项：私有配置里配了 OUTSIDE_SERVICE_CGROUP 就读那里的 memory.stat；文件不在、没有这个键都报「没配」、不读、退出码照旧。
  # cgroup 根指到这一例自己造的假目录，私有配置写在 scratch 里；失败信息不贴 --status 的整段输出（整段输出里会打出单元名）
  fresh_context 256M
  local fake_cgroup_root="$scratch/fake-cgroup" private_case private_config expected_text
  mkdir -p "$fake_cgroup_root/system.slice/selftest-outside.service" "$fake_cgroup_root/user.slice"
  printf 'anon %s\nshmem %s\nfile %s\n' $((3 * MEBIBYTE)) $((2 * MEBIBYTE)) $((900 * MEBIBYTE)) > "$fake_cgroup_root/system.slice/selftest-outside.service/memory.stat"
  printf 'anon %s\n' $((7 * MEBIBYTE)) > "$fake_cgroup_root/user.slice/memory.stat"
  printf '%s\n' "# 自证用的私有配置" "OTHER_KEY=system.slice/other.service" "OUTSIDE_SERVICE_CGROUP=system.slice/selftest-outside.service" > "$scratch/private-set.env"
  printf '%s\n' "# 自证用的私有配置，没有 OUTSIDE_SERVICE_CGROUP" "OTHER_KEY=system.slice/selftest-outside.service" > "$scratch/private-keyless.env"
  for private_case in "set|$scratch/private-set.env|本地模型服务的 anon + shmem 5 MiB、" \
      "absent|$scratch/private-absent.env|本地模型服务的 anon + shmem 没配（" \
      "keyless|$scratch/private-keyless.env|本地模型服务的 anon + shmem 没配（"; do
    IFS='|' read -r private_case private_config expected_text <<< "$private_case"
    checked=$((checked + 1))
    if in_context env RUN_WITH_MEMORY_CAP_CGROUP_ROOT="$fake_cgroup_root" RUN_WITH_MEMORY_CAP_PRIVATE_CONFIG="$private_config" bash "$target" --status \
        >"$scratch/private-$private_case.out" 2>&1; then status=0; else status=$?; fi
    if [[ $status -ne 0 ]] || ! grep -qF -- "$expected_text" "$scratch/private-$private_case.out" \
        || { [[ "$private_case" != set ]] && grep -qF -- "5 MiB、" "$scratch/private-$private_case.out"; }; then
      fail "--status 私有配置 $private_case 那一例应当退 0、「slice 外面」那一行含「$expected_text」$([[ "$private_case" != set ]] && echo '、不含假 cgroup 里的 5 MiB')，\
实际退 $status、$(grep -qF -- "$expected_text" "$scratch/private-$private_case.out" && echo '含' || echo '不含')那一句（整段输出不贴：改前那一份会在里面打出单元名）"
    fi
  done

  # 用法错退 $MEMORY_CAP_UNAVAILABLE_EXIT（「命令一行都没跑」那一类），不落在 0–249 里与命令自己的退出码混在一起
  checked=$((checked + 1))
  if bash "$target" abc true 2>/dev/null; then status=0; else status=$?; fi
  [[ $status -eq $MEMORY_CAP_UNAVAILABLE_EXIT ]] || fail "上限写成 abc 应当退 $MEMORY_CAP_UNAVAILABLE_EXIT，实际 $status"
  checked=$((checked + 1))
  if bash "$target" 64M 2>/dev/null; then status=0; else status=$?; fi
  [[ $status -eq $MEMORY_CAP_UNAVAILABLE_EXIT ]] || fail "没给命令应当退 $MEMORY_CAP_UNAVAILABLE_EXIT，实际 $status"
  # 上限不带单位（「16」）：systemd 当 16 字节；按用法错拒绝，不起 scope、命令不跑、报「单位必写」
  fresh_context 256M
  checked=$((checked + 1))
  marker="$scratch/ran-unitless"
  if in_context bash "$target" 16 bash -c ': > "$1"' touch-marker "$marker" 2>"$scratch/unitless.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_UNAVAILABLE_EXIT || -e "$marker" ]] || ! grep -q '单位必写' "$scratch/unitless.err"; then
    fail "上限写成不带单位的 16 时应当按用法错退 $MEMORY_CAP_UNAVAILABLE_EXIT、报「单位必写」、命令不跑，实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/unitless.err")"
  fi
  # slice 名不是 singlefs 开头（别人的 slice）：退 $MEMORY_CAP_UNAVAILABLE_EXIT，不设上限、不起 scope。要是真去设，名字取一个不碍事的，收尾照样停掉、还原
  local foreign_slice
  checked=$((checked + 1))
  foreign_slice="notsinglefs_selftest_$$.slice"
  selftest_slices+=("$foreign_slice")
  if env RUN_WITH_MEMORY_CAP_SLICE="$foreign_slice" RUN_WITH_MEMORY_CAP_SLICE_TOTAL=256M RUN_WITH_MEMORY_CAP_STATE_DIR="$scratch/foreign-state" \
      RUN_WITH_MEMORY_CAP_PEAKS="$scratch/foreign-peaks.tsv" bash "$target" --check 64M 2>"$scratch/foreign.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_UNAVAILABLE_EXIT ]] || ! grep -q 'singlefs 开头' "$scratch/foreign.err"; then
    fail "slice 名写成不是 singlefs 开头的 $foreign_slice 时应当退 $MEMORY_CAP_UNAVAILABLE_EXIT、报「singlefs 开头」，实际退 $status：$(cat "$scratch/foreign.err")"
  fi

  selftest_cleanup
  if ((failures)); then
    echo "  ✗ run-with-memory-cap.sh 自检 $failures 处不对（查了 $checked 项；测的是 $target）"
    echo "  → 怎么办：照上面逐条改 run_capped / ensure_slice / clear_leftover_memory_high / resolve_slice_total / configured_slice_cap / record_run / scope_result / check_size_syntax / reject_usage / command_threads / show_status / gpu_memory_verdict / 临时目录那几个函数（取键与峰值口径在 memory_peaks_lib.py）；RUN_WITH_MEMORY_CAP_BREAK 设着、或 RUN_WITH_MEMORY_CAP_SELFTEST_TARGET 指着改前那一份的话这里本来就该红"
    exit 1
  fi
  echo "  ✓ run-with-memory-cap.sh 自检通过（查了 $checked 项：平常跑设了弄坏开关退 2 并点名只在自证里认而带着自证的标记照常判、--check 起得来、64M 上限里分配 512 MiB 的探针与它当孙进程时都判撞顶退 $MEMORY_CAP_HIT_EXIT \
且峰值记成撞顶、上限之内 kill -9 原样退 137 且 stderr 不多一行、被外面的 TERM 与 KILL 停原样退且峰值不记、退出码 0 / 7 / 101 原样传回、\
命令退出之后停掉自己的 scope 连同 setsid 出去、清了环境的进程且回读 scope 不再 active、\
命令退非 0 而起止之间内核报了显存不够退 $GPU_MEMORY_HIT_EXIT 且按起止给日志窗口、读不到内核日志报没查成、别的 NVRM 行不算、命令退 0 不读日志、撞了上限又报了显存不够照旧退 $MEMORY_CAP_HIT_EXIT 且另报一行、\
scope 里 memory.max 与 memory.swap.max 设上了、scope 挂在 slice 底下且 slice 的总上限设上了、slice 设不上时退 $MEMORY_CAP_UNAVAILABLE_EXIT 且命令没跑、\
总上限照多机配置 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项（头一项是另一台也照这一台取）且 --status 报它从哪来、配置里没有这个键时照 slice 已有的走、\
这一台那一项写法不对、没有这一台那一项与三样都没有时退 $MEMORY_CAP_UNAVAILABLE_EXIT 且命令没跑、\
slice 上留下的 memory.high 清回 max（留着 47.5G 与 slice 起来之后才带上的清一次并打一行、已是 max 与 slice 原先不在的不清、清了读回不是 max 退 $MEMORY_CAP_UNAVAILABLE_EXIT 且命令没跑）、\
D-Bus 连不上时跑命令与 --check 都退 $MEMORY_CAP_UNAVAILABLE_EXIT 且命令没跑、systemd-run 不认上限时 --check 退 $MEMORY_CAP_UNAVAILABLE_EXIT、被总上限挤掉退 $SLICE_TOTAL_HIT_EXIT 且那一次峰值不记、\
scope 里一直收 TERM 时峰值照记、起了 scope 之后给包装发 TERM / INT 时先停自己开的 scope 且标记与 scope 都不留、\
自证开出 slice 之后被 TERM / INT 停时退 143 / 130 且它开的 slice 都不再 active、临时目录删掉、超过限时退 $TIME_LIMIT_HIT_EXIT、\
峰值表的口径（页缓存涨满上限记回收不掉的那一截、oom_kill 大于 0 而 Result 没报的记撞顶）、采样子进程不拖延收尾（四种收尾）、撞顶的 scope 结局读早了与一直读不到都报撞了上限、\
failed 而 Result 读到 success 时到 journal 找原因、这一条命令自己的临时目录（五种结局、空的照删、只留最近 3 个、SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 留着、调用方给的 TMPDIR 照它的且不删、给了 RUN_WITH_MEMORY_CAP_TEMPORARY_RECORD 就把临时目录的全路径写进它且退出时删掉）、\
峰值表的线程数一列六种各记对、没有服务的准入标记退 $MEMORY_CAP_UNAVAILABLE_EXIT 且命令没跑、被服务起的命令里面再调这一份退 $MEMORY_CAP_UNAVAILABLE_EXIT 且命令没跑、\
已由 task-server 准入的一条命令看不到那个变量、跑完不留 scope、--status 私有配置三种、用法错四种都退 $MEMORY_CAP_UNAVAILABLE_EXIT）"
}
