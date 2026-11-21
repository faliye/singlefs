#!/usr/bin/env bash
# 本该判绿：与 pinned-apparatus-revision-red 同一批起跑脚本与装置，装置源码这一轮改了，登记第十二节都新加了一行修订。
#   e990：起跑脚本用 apparatus_source= 点名装置，登记第十二节新写了一行。
#   e991：起跑脚本只有 cargo build 的 --bin，登记第十二节新写了一行，第十三节也多了一句（别的节的改动不碍事）。
#   e992：装置改了，登记 e992-r1-prereg.md 不在树里（那一轮已归档），不判，成功句点名。
#   e993：装置没改，登记没动，不判。
# 样本根的 .gate-cells 只点名 pinned-apparatus-revision。
set -euo pipefail
git init -q -b master .
git config user.email selftest@example.invalid
git config user.name selftest
mkdir -p scripts/experiment-apparatus crates/sample/src/bin research/prompts

registry() { # <号> <第五节的话> <第十二节的话> <第十三节的话>
  printf '# E%s 第 1 次重跑登记\n\n## 五、跑法\n\n%s\n\n## 十二、修订\n\n%s\n\n## 十三、读过的文件与跑过的命令\n\n%s\n' "$1" "$2" "$3" "$4" > "research/prompts/e$1-r1-prereg.md"
}
starter_with_source() { # <号>
  {
    echo '#!/usr/bin/env bash'
    echo "# E$1 第 1 次重跑一段的跑法（跑前登记 research/prompts/e$1-r1-prereg.md 第五节「跑法」）："
    echo "apparatus_source=crates/sample/src/bin/e$1_apparatus.rs"
    printf 'registered_apparatus_sha256=%064d\n' "$1"
  } > "scripts/experiment-apparatus/e$1-r1-segment-run.sh"
}

for number in 990 991 992 993; do
  printf 'fn main() { println!("e%s"); }\n' "$number" > "crates/sample/src/bin/e${number}_apparatus.rs"
done
starter_with_source 990
{
  echo '#!/usr/bin/env bash'
  echo '# E991 第 1 次重跑一段的跑法（跑前登记 research/prompts/e991-r1-prereg.md 第五节「跑法」）：'
  echo 'cargo build --release -p sample --bin e991_apparatus'
  printf 'registered_apparatus_sha256=%064d\n' 991
} > scripts/experiment-apparatus/e991-r1-segment-run.sh
starter_with_source 992
starter_with_source 993
registry 990 '命令照 e990 的起跑脚本。' '（装置改完之后、第一份产物之前由执行员写。）' '读过 e990 的装置。'
registry 991 '命令照 e991 的起跑脚本。' '（装置改完之后、第一份产物之前由执行员写。）' '读过 e991 的装置。'
registry 993 '命令照 e993 的起跑脚本。' '（装置改完之后、第一份产物之前由执行员写。）' '读过 e993 的装置。'
git add -A
git commit -qm '基准：四份起跑脚本、三份登记（e992 的登记已归档）'

for number in 990 991 992; do
  printf 'fn main() { println!("e%s 改过了"); }\n' "$number" > "crates/sample/src/bin/e${number}_apparatus.rs"
done
registry 990 '命令照 e990 的起跑脚本。' $'（装置改完之后、第一份产物之前由执行员写。）\n\nR990-1 装置 sha256 改成 0000000000000000000000000000000000000000000000000000000000000990：println 多打了「改过了」三个字。' '读过 e990 的装置。'
registry 991 '命令照 e991 的起跑脚本。' $'（装置改完之后、第一份产物之前由执行员写。）\n\nR991-1 装置 sha256 改成 0000000000000000000000000000000000000000000000000000000000000991：println 多打了「改过了」三个字。' $'读过 e991 的装置。\n\n又读了 e991 的起跑脚本。'
