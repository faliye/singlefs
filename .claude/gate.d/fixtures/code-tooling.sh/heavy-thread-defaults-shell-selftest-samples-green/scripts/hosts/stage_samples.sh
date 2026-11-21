#!/usr/bin/env bash
# 样本：自证函数里喂给样本的环境词（SAMPLE_DEFAULT_JOBS=1）与写给样本阶段的正文（单引号里的 ${SAMPLE_DEFAULT_JOBS:-}），这一份自己不读它们
stage_samples_selftest() {
  local sample_root="$1"
  printf '%s\n' '#!/usr/bin/env bash' \
    '[[ -z "${SAMPLE_DEFAULT_JOBS:-}" ]] || exit 3' > "$sample_root/stage.sh"
  if run_sample jobs-default 0 SAMPLE_DEFAULT_JOBS=1 -- "$sample_root"; then
    printf '取缺省那一条判对\n'
  fi
}
