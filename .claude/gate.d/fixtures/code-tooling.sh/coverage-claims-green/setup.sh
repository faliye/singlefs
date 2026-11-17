#!/usr/bin/env bash
# code-tooling 的 research-script-selftest-coverage 格样本 coverage-claims-green 的现场：scripts 下叫 target 的目录是编译产物，
# 里面代码写着 --selftest 的 .sh 不算 scripts/ 里实现了自证的脚本，这一格不进 target 目录（弄坏开关 coverage-scans-build-output 下进、把它当没人跑的自证报红）。
# 放在样本的文件树里会被 .gitignore 的 target/ 吞掉，带不进提交，所以在这里现造。
set -euo pipefail
mkdir -p scripts/gate/target/release
printf '%s\n' '#!/usr/bin/env bash' '# 样本：编译产物目录里的脚本，代码里写了 --selftest' 'if [[ "${1:-}" == --selftest ]]; then echo "自证"; fi' > scripts/gate/target/release/build-output.sh
