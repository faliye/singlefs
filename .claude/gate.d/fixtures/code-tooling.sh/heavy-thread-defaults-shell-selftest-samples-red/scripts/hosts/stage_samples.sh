#!/usr/bin/env bash
# 样本：同样两行写在名字里不带 selftest 的函数里：单引号里的 ${SAMPLE_DEFAULT_JOBS:-} 照算读，SAMPLE_DEFAULT_JOBS=1 照算写死的数
stage_samples_check() {
  local sample_root="$1"
  printf '%s\n' '#!/usr/bin/env bash' \
    '[[ -z "${SAMPLE_DEFAULT_JOBS:-}" ]] || exit 3' > "$sample_root/stage.sh"
  if run_sample jobs-default 0 SAMPLE_DEFAULT_JOBS=1 -- "$sample_root"; then
    printf '取缺省那一条判对\n'
  fi
}
