#!/usr/bin/env bash
# H-B（V2）：crates 退回到一份逐字节相同的旧内容，那份旧内容在 common-dir 里有一格旧绿格，
# 而写那一格的是另一版 54 号（drift=54）或另一版工具链（drift=toolchain）。「旧格只增不减」，那一格一直在。
# 前缀固定：提交 A（crash.rs=C0）的 --full 在旧 54 号 / 旧工具链下绿、写格 H0 → 提交 B（54 号换 new 或工具链升到 2，crash.rs=C1）
# → 这一批把 crates/ 退回 A（git checkout A -- crates/）并暂存 → 照 main-agent.md 第 44 行后台起 --full（出路句三行原样）。
# 放开扫的用户动作（这一趟 --full 起来之后）：
#   during   --full 还在跑时起门禁（54 号第 11 行：前一趟那一格在这一趟跑的过程中照样算数）
#   kill9    54 号那个 bash 进程被 SIGKILL（会话被整组杀掉、机器重启一类），之后起门禁
#   oom      OOM 挑中的是占内存的测试进程（这里是假 cargo 进程）被 SIGKILL，之后起门禁
#   term     54 号那个 bash 进程被 SIGTERM，之后起门禁
#   finish   等 --full 跑完再起门禁
source "$(dirname "$0")/lib.sh"
outpath_file "$MODEL/inputs/54-post.sh" "$RUNS/outpath.sh"

prefix() { # <仓> <arm> <drift>：造到「这一批已暂存」为止
  local r="$1" arm="$2" drift="$3" token
  echo 1 > "$FAKE_TOOLCHAIN_FILE"
  if [[ "$drift" == 54 ]]; then make_repo "$r" "$arm" old ""; token=SINGLE_THREADED; else make_repo "$r" "$arm" new ""; token=TOOLCHAIN_SENSITIVE; fi
  set_crash "$r" c0 "$token"; git -C "$r" add -A
  if [[ "$arm" == post ]]; then (cd "$r" && bash "$RUNS/outpath.sh" > "$r.a-full.log" 2>&1); fi
  echo "  ── 提交 A 之前的门禁（C0，$([[ $drift == 54 ]] && echo 旧 54 号 || echo 工具链 1)）"; gate_staged_54 "$r"
  git -C "$r" commit -q -m A; commit_a="$(git -C "$r" rev-parse HEAD)"
  if [[ "$drift" == 54 ]]; then set_54 "$r" "$arm" new; else echo 2 > "$FAKE_TOOLCHAIN_FILE"; fi
  set_crash "$r" c1 ""; git -C "$r" add -A
  if [[ "$arm" == post ]]; then (cd "$r" && bash "$RUNS/outpath.sh" > "$r.b-full.log" 2>&1); fi
  echo "  ── 提交 B 之前的门禁（C1，$([[ $drift == 54 ]] && echo 新 54 号 || echo 工具链 2)）"; gate_staged_54 "$r"
  git -C "$r" commit -q -m B
  git -C "$r" checkout -q "$commit_a" -- crates/
  echo "  ── 这一批：git checkout A -- crates/，暂存区：$(git -C "$r" diff --cached --name-only | tr '\n' ' ')"
  cells "$r"
}

run_case() { # <arm> <drift> <action>
  local arm="$1" drift="$2" action="$3" r hold pid cargo_pid
  want "$arm-$drift-$action" || return 0
  r="$RUNS/h-b-$arm-$drift-$action"
  echo "════ H-B [$arm] 漂移=$drift 用户动作=$action"
  prefix "$r" "$arm" "$drift"
  if [[ "$arm" == post ]]; then
    hold="$RUNS/tmp/hold-$$-$RANDOM"
    ( cd "$r" && FAKE_CARGO_HOLD="$hold" bash "$RUNS/outpath.sh" > "$r.c-full.log" 2>&1 ) &
    pid=$!
    for _ in $(seq 1 600); do [[ -e "$hold.started" ]] && break; sleep 0.1; done
    stage_pid="$(descendant_matching "$pid" '54-layer0-replay.sh --full')"
    case "$action" in
      during) echo "  ── --full 还在跑（54 号 pid $stage_pid），起门禁"; gate_staged_54 "$r"; : > "$hold.release"; wait "$pid" ;;
      kill9)  kill -KILL "$stage_pid"; : > "$hold.release"; wait "$pid"; echo "  ── 54 号 pid $stage_pid 被 SIGKILL 之后起门禁"; gate_staged_54 "$r" ;;
      oom)    cargo_pid="$(descendant_matching "$stage_pid" 'fake-bin/cargo')"
              kill -KILL "$cargo_pid"; : > "$hold.release"; wait "$pid"; echo "  ── 假 cargo pid $cargo_pid 被 SIGKILL（仿 OOM）之后起门禁"; gate_staged_54 "$r" ;;
      term)   kill -TERM "$stage_pid"; : > "$hold.release"; wait "$pid"; echo "  ── 54 号 pid $stage_pid 被 SIGTERM 之后起门禁"; gate_staged_54 "$r" ;;
      finish) : > "$hold.release"; wait "$pid"; echo "  ── --full 跑完之后起门禁"; gate_staged_54 "$r" ;;
    esac
    echo "  ── 这一趟 --full 的判定行："; grep -E '^  (✓ 全绿|✗)' "$r.c-full.log" | trim 150 || echo "      （没打出判定行）"
    cells "$r"
  else
    echo "  ── 起门禁（分档之前：门禁自己在临时 worktree 里跑全量）"; gate_staged_54 "$r"
  fi
  echo "  ── 真值"; truth_full "$r"
  echo
}

# descendant_matching <pid> <命令行片段>：<pid> 的子孙里第一个命令行含这一段的进程（按 ps 的父子关系走，不按名字全局找）
descendant_matching() {
  local frontier=("$1") next child args
  for _ in 1 2 3 4 5 6; do
    next=()
    for p in "${frontier[@]}"; do
      while read -r child args; do
        [[ -z "$child" ]] && continue
        if [[ "$args" == *"$2"* ]]; then echo "$child"; return 0; fi
        next+=("$child")
      done < <(ps -o pid=,args= --ppid "$p")
    done
    frontier=("${next[@]}")
  done
  return 1
}

for drift in 54 toolchain; do
  for action in during kill9 oom term finish; do run_case post "$drift" "$action"; done
  run_case pre "$drift" none
done
