#!/usr/bin/env bash
# S3 样本树：heavy-threads.sh、heavy_threads.py、capped.sh、登记与排除表从 <副本根> 拷，scripts/experiments/ 下放九份喂法。
# 用法：make_tree.sh <副本根> <新目录>
set -eu
root="$1" tree="$2"
mkdir -p "$tree/scripts/hosts" "$tree/scripts/experiments" "$tree/.claude"
cp "$root/scripts/hosts/heavy-threads.sh" "$root/scripts/hosts/heavy_threads.py" "$root/scripts/hosts/capped.sh" "$tree/scripts/hosts/"
cp "$root/.claude/heavy-thread-default" "$tree/.claude/"
grep -E '^scripts/hosts/(capped\.sh|heavy-threads\.sh|heavy_threads\.py):' "$root/.claude/heavy-thread-exempt" > "$tree/.claude/heavy-thread-exempt"
x="$tree/scripts/experiments"
src='source "$(dirname "$0")/../hosts/heavy-threads.sh"'
tail='[[ -n "${SAMPLE_SWEEP_THREADS:-}" ]] || SAMPLE_SWEEP_THREADS=$default_threads'
# 甲：登记位取不到时用 := 补一个字面数（变量本身只有一处 = 赋值，来自登记位）
printf '%s\n' '#!/usr/bin/env bash' "$src" 'default_threads="$(heavy_thread_default)" || true' ': "${default_threads:=8}"' "$tail" > "$x/colon_equals_fallback.sh"
# 乙：被 source 的同目录配置文件里给了 default_threads=8，本文件只在它空的时候从登记位取
printf '%s\n' '#!/usr/bin/env bash' '# 本目录几份扫描脚本共用的默认值' 'default_threads=8' > "$x/sample_defaults.sh"
printf '%s\n' '#!/usr/bin/env bash' "$src" 'source "$(dirname "$0")/sample_defaults.sh"' '[[ -n "${default_threads:-}" ]] || default_threads="$(heavy_thread_default)"' "$tail" > "$x/sourced_literal.sh"
# 丙：上限变量没设时退回另一个变量，那个变量是字面数
printf '%s\n' '#!/usr/bin/env bash' 'fallback_threads=8' 'default_threads="${SINGLEFS_THREAD_CAP:-$fallback_threads}"' "$tail" > "$x/cap_fallback_via_variable.sh"
# 丁：登记值除以一个变量（不写字面 / 2）
printf '%s\n' '#!/usr/bin/env bash' "$src" 'divisor=2' 'default_threads="$(( $(heavy_thread_default) / divisor ))"' "$tail" > "$x/registry_divided_by_variable.sh"
# 戊：照 run-with-memory-cap.sh 那样把取值的 stderr 丢掉（取的仍是登记值）
printf '%s\n' '#!/usr/bin/env bash' "$src" 'default_threads="$(heavy_thread_default 2>/dev/null)" || exit 2' "$tail" > "$x/stderr_silenced.sh"
# 己：经一个自己的小函数取登记值
printf '%s\n' '#!/usr/bin/env bash' "$src" 'pick_threads() { local threads; threads="$(heavy_thread_default)" || return 2; printf "%s" "$threads"; }' 'default_threads="$(pick_threads)"' "$tail" > "$x/via_own_function.sh"
# 庚：同一个变量两处都从登记位取（对照，应绿）
printf '%s\n' '#!/usr/bin/env bash' "$src" 'default_threads="$(heavy_thread_default)"' 'if [[ -n "${1:-}" ]]; then default_threads="$(heavy_thread_default)"; fi' "$tail" > "$x/assigned_twice_registry.sh"
# 辛：条件赋：一支登记位、一支字面数（对照，应红）
printf '%s\n' '#!/usr/bin/env bash' "$src" 'if [[ -n "${1:-}" ]]; then default_threads="$(heavy_thread_default)"; else default_threads=8; fi' "$tail" > "$x/conditional_literal.sh"
