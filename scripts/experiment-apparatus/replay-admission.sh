#!/usr/bin/env bash
# admission: always 被 scripts/experiment-apparatus/replay.sh 在同目录 source 进去的那一段，判的是此刻的仓与这一趟的参数，上一次的结论不替这一次作保
# run-condition: none 不单独执行：由 replay.sh 的 source 载入，环境要求同 replay.sh（文件头「GPU」「编译目录」）
# replay-admission.sh：replay.sh 的一段（准入那一段：向准入模块要每一行的准入键、--admission-only 只问准入）。含载入时就执行的语句（replay.sh 载入次序里这一段的顺序流程），接着 source 它的那一行往下走；exit 与 replay.sh 里的 exit 一样退出整趟。
# 这一段按 replay.sh 里 source 它的那一行的位置载入。文件头的用法、复跑表、弄坏开关表、退出码都在 replay.sh。
ADMISSION_MODULE="scripts/gate/admission.py"
if ! admission_rows_text="$(python3 "$ADMISSION_MODULE" replay-row-keys .)"; then
  printf '%s\n' "$admission_rows_text" >&2
  echo "  ✗ 定不下复跑表每一行的准入键（python3 $ADMISSION_MODULE replay-row-keys .），一行都没跑" >&2
  echo "     → 怎么办：照上面准入模块打的那几条改 ADMISSION_ROW_KEYS 或 .claude/gate.d/stage-inputs.tsv；不知道每一行按哪个键问准入、比哪一份产物，就不开跑" >&2
  exit 2
fi
declare -A ADMISSION_REGISTERED=() ADMISSION_ROW_KEY=() ADMISSION_ROW_HEADER_WRITER=()
while IFS=$'\t' read -r row_identity row_key row_header_writer row_registered; do
  [[ -n "$row_identity" ]] || continue
  ADMISSION_ROW_KEY["$row_identity"]="$row_key"
  ADMISSION_ROW_HEADER_WRITER["$row_identity"]="$row_header_writer"
  [[ "$row_registered" == yes ]] && ADMISSION_REGISTERED["$row_key"]=1
done <<<"$admission_rows_text"
admission_rows_unkeyed=()
while IFS='|' read -r exp bin args _stored _kind; do
  [[ -z "$exp" ]] && continue
  [[ -n "${ADMISSION_ROW_KEY["$exp|$bin|$args"]+set}" ]] || admission_rows_unkeyed+=("$exp|$bin|$args")
done <<<"$TABLE"
if (( ${#admission_rows_unkeyed[@]} )); then
  echo "  ✗ 复跑表有 ${#admission_rows_unkeyed[@]} 行准入模块没给出准入键（它只认「编号|二进制|参数|产物|exact 或 timing」形状的行）：" >&2
  printf '      %s\n' "${admission_rows_unkeyed[@]}" >&2   # gate-lint:detail
  echo "     → 怎么办：把那几行改成这个形状（第 5 列只写 exact 或 timing），python3 scripts/gate/admission.py replay-row-keys . 列得出每一行再跑" >&2
  exit 2
fi

if (( ADMISSION_ONLY )); then
  admission_judged=0; admission_unregistered=0; admission_errors=0
  while IFS='|' read -r exp bin row_arguments stored _kind; do
    [[ -z "$exp" ]] && continue
    want_row "$exp" "$bin" "$row_arguments" || continue
    admission_judged=$((admission_judged+1))
    admission_key_of_row "$exp" "$bin" "$row_arguments"
    if [[ -z "${ADMISSION_REGISTERED[$ROW_ADMISSION_KEY]:-}" ]]; then
      admission_unregistered=$((admission_unregistered+1))
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 没接准入 "准入登记表里没有 $ROW_ADMISSION_KEY，照跑（留存产物 $stored）"
      continue
    fi
    if admission_messages="$(python3 "$ADMISSION_MODULE" experiment . "$ROW_ADMISSION_KEY" 2>&1 >/dev/null)"; then admission_exit_code=0; else admission_exit_code=$?; fi
    case "$admission_exit_code" in
      0) admission_verdict=放行 ;; 77) admission_verdict=输入没变 ;; 3) admission_verdict=前提没齐 ;;
      *) admission_verdict=登记有错; admission_errors=$((admission_errors+1)) ;;
    esac
    printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" "$admission_verdict" "准入键 $ROW_ADMISSION_KEY，准入退 $admission_exit_code，留存产物 $stored"
    printf '%s\n' "$admission_messages" | sed 's/^/      /'   # gate-lint:detail
  done <<<"$TABLE"
  echo "只判准入、没编没跑：判了 $admission_judged 行（其中 $admission_unregistered 行没接准入，照跑），准入模块报登记有错 $admission_errors 行"
  (( admission_errors == 0 )) || exit 1
  exit 0
fi
