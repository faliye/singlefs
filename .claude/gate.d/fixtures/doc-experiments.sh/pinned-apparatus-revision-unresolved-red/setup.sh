#!/usr/bin/env bash
# 本该判红：两份钉了装置 sha256 的起跑脚本，认不出装置或认不出登记，不管装置有没有变都判红（看不见对象的闸不算闸）。
#   e994：脚本里既没有行首的 apparatus_source=，非注释行里也没有 cargo 命令的 --bin（注释里写的 --bin 不算）：认不出装置源码。
#   e995：装置源码点名了，脚本注释里没有点名跑前登记：认不出登记。
# 样本根的 .gate-cells 只点名 pinned-apparatus-revision。
set -euo pipefail
git init -q -b master .
git config user.email selftest@example.invalid
git config user.name selftest
mkdir -p scripts/experiment-apparatus crates/sample/src/bin research/prompts
printf 'fn main() { println!("e995"); }\n' > crates/sample/src/bin/e995_apparatus.rs
printf '# E994 第 1 次重跑登记\n\n## 十二、修订\n\n（装置改完之后、第一份产物之前由执行员写。）\n' > research/prompts/e994-r1-prereg.md
{
  echo '#!/usr/bin/env bash'
  echo '# E994 第 1 次重跑一段的跑法（跑前登记 research/prompts/e994-r1-prereg.md 第五节「跑法」）；以前编的是 cargo build --release -p sample --bin e994_apparatus'
  echo 'build_the_apparatus'
  printf 'registered_apparatus_sha256=%064d\n' 994
} > scripts/experiment-apparatus/e994-r1-segment-run.sh
{
  echo '#!/usr/bin/env bash'
  echo '# E995 第 1 次重跑一段的跑法：登记没有点名'
  echo 'apparatus_source=crates/sample/src/bin/e995_apparatus.rs'
  printf 'registered_apparatus_sha256=%064d\n' 995
} > scripts/experiment-apparatus/e995-r1-segment-run.sh
git add -A
git commit -qm '基准：两份认不出对象的起跑脚本'
