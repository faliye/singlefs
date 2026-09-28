#!/usr/bin/env bash
# 模型 M1（gate-shrink-r1 云端攻方）：方案 A 把 10、20、30、32、43 五道并成 doc-kb 一道之后，
# 一个格的红样本里 want= 那几句，会不会由并进来的别的格替它打出来。
# 做法：五道里每一道 X 的每一份样本（照 stage-selftest.sh 的办法拷进临时目录、跑 setup.sh），
# 在同一棵样本树上跑另外四道 Y（带 --force，与 stage-selftest.sh 同），记下 Y 的退出码，
# 并逐条看 X 的 want= 在 Y 的输出里出不出现。只读仓里的门禁脚本，不改它们；临时目录跑完即删。
# 用法：bash m1_want_cross_hits.sh <仓根> <输出目录>
# 输出：<输出目录>/m1-rows.tsv，一行一条 want：X、样本、want 原文、X 期望的退出码、别的四道里退 1 的有哪几道、want 出现在哪几道的输出里
set -uo pipefail
repository_root="$(cd "${1:?仓根}" && pwd)"
output_directory="${2:?输出目录}"
mkdir -p "$output_directory"
stages=(10-references-and-invariants.sh 20-doc-decision-documents.sh 30-decision-history-entries.sh 32-doc-field-and-layout-registry.sh 43-checks-owed-and-closeout.sh)
rows_file="$output_directory/m1-rows.tsv"
: > "$rows_file"
printf '%s\t%s\t%s\t%s\t%s\t%s\n' own_stage sample want own_expected_exit other_stages_exit_1 other_stages_printing_want >> "$rows_file"
for own_stage in "${stages[@]}"; do
  fixture_root="$repository_root/.claude/gate.d/fixtures/$own_stage"
  for sample_directory in "$fixture_root"/*/; do
    sample_directory="${sample_directory%/}"
    sample_name="$(basename "$sample_directory")"
    [[ -f "$sample_directory/expect" ]] || continue
    expected_exit="$(sed -n 's/^exit=//p' "$sample_directory/expect")"
    declare -A other_output=() other_exit=()
    for other_stage in "${stages[@]}"; do
      [[ "$other_stage" == "$own_stage" ]] && continue
      work_directory="$(mktemp -d "${TMPDIR:-/tmp}/m1-work.XXXXXX")"
      cp -a "$sample_directory/." "$work_directory/"
      if [[ -f "$work_directory/setup.sh" ]]; then
        ( cd "$work_directory" && env -u GATE_BASE -u GATE_STAGED_FROM -u GATE_DIFF_BASE bash setup.sh >/dev/null 2>&1 ) || { other_exit[$other_stage]=setup-failed; rm -rf "${work_directory:?}"; continue; }
      fi
      stage_exit=0
      stage_output="$(cd "$work_directory" && env -u GATE_BASE -u GATE_STAGED_FROM -u GATE_DIFF_BASE -u GATE_NOT_RUN_FILE bash "$repository_root/.claude/gate.d/$other_stage" "$work_directory" --force 2>&1)" || stage_exit=$?
      rm -rf "${work_directory:?}"
      other_output[$other_stage]="$stage_output"
      other_exit[$other_stage]="$stage_exit"
      printf '%s\n' "$stage_output" > "$output_directory/out__${own_stage%.sh}__${sample_name}__${other_stage%.sh}.log"
    done
    exit_one_list=""
    for other_stage in "${!other_exit[@]}"; do [[ "${other_exit[$other_stage]}" == 1 ]] && exit_one_list+="${other_stage%%-*},"; done
    while IFS= read -r want_text; do
      [[ -z "$want_text" ]] && continue
      printing_list=""
      for other_stage in "${!other_output[@]}"; do
        if grep -qF -- "$want_text" <<<"${other_output[$other_stage]}"; then printing_list+="${other_stage%%-*},"; fi
      done
      printf '%s\t%s\t%s\t%s\t%s\t%s\n' "${own_stage%%-*}" "$sample_name" "$want_text" "$expected_exit" "${exit_one_list:-none}" "${printing_list:-none}" >> "$rows_file"
    done < <(sed -n 's/^want=//p' "$sample_directory/expect")
    unset other_output other_exit
  done
done
echo "rows=$(($(wc -l < "$rows_file") - 1))"
