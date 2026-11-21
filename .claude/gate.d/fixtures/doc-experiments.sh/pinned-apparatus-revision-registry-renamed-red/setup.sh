#!/usr/bin/env bash
# 本该判红：E955 与 E956 的登记都在一次提交里被 git mv 改了名（<号>-r2-prereg.md 改成 <号>-r2-prereg-gc-split.md），起跑脚本注释里的「跑前登记」没跟着改。
#   E955：这一次装置改了、钉值没动，新名字的登记没动；改名之前那一版第十二节写着这个钉值，拿它判会放过。
#   E956：这一次装置改了、钉值改成新值，新名字的登记第十二节新加了写着新钉值的一行；拿改名之前那一版判，红句说的是没写新钉值，出路不对。
# 被改名走的登记不是归档：两份都判红、点名新名字，出路是改起跑脚本里的登记路径。
# 不看是不是改名、照已归档的取改名之前那一版判的写法，E955 判绿、E956 红在「没有出现钉值」上，两句点名新名字的都找不到。
# 样本根的 .gate-cells 只点名 pinned-apparatus-revision。
set -euo pipefail
git init -q -b master .
git config user.email selftest@example.invalid
git config user.name selftest
mkdir -p scripts/experiment-apparatus crates/sample/src/bin research/prompts

apparatus() { # <号> <内容>
  printf 'fn main() { println!("%s"); }\n' "$2" > "crates/sample/src/bin/e$1_apparatus.rs"
}
sha_of() { sha256sum "crates/sample/src/bin/e$1_apparatus.rs" | cut -d' ' -f1; }
starter() { # <号> <钉值行>
  {
    echo '#!/usr/bin/env bash'
    echo "# E$1 第 2 次重跑一段的跑法（跑前登记 research/prompts/e$1-r2-prereg.md 第五节「跑法」）："
    echo "apparatus_source=crates/sample/src/bin/e$1_apparatus.rs"
    echo "$2"
  } > "scripts/experiment-apparatus/e$1-r2-segment-run.sh"
}
registry() { # <号> <第十二节的正文> <登记文件名>
  printf '# E%s 第 2 次重跑登记\n\n## 五、跑法\n\n命令照起跑脚本。\n\n## 十二、修订\n\n%s\n\n## 十三、读过的文件\n\n读过装置。\n' "$1" "$2" > "research/prompts/$3"
}

for number in 955 956; do
  apparatus "$number" "e$number 登记时"
  starter "$number" "registered_apparatus_sha256=$(sha_of "$number")"
  registry "$number" "R2-0 登记时的装置 sha256 $(sha_of "$number")" "e$number-r2-prereg.md"
done
git add -A
git commit -qm '基准：起跑脚本、装置与登记'
git mv research/prompts/e955-r2-prereg.md research/prompts/e955-r2-prereg-gc-split.md
git mv research/prompts/e956-r2-prereg.md research/prompts/e956-r2-prereg-gc-split.md
git commit -qm '登记改名（起跑脚本里的路径没跟着改）'
apparatus 955 'e955 改过了'
first=$(git show HEAD:crates/sample/src/bin/e956_apparatus.rs | sha256sum | cut -d' ' -f1)
apparatus 956 'e956 改过了'
starter 956 "registered_apparatus_sha256=$(sha_of 956)"
registry 956 "R2-0 登记时的装置 sha256 $first"$'\n\n'"R2-1 装置改了，sha256 $(sha_of 956)" e956-r2-prereg-gc-split.md
