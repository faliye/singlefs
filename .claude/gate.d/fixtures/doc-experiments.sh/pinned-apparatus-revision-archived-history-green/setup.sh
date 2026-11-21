#!/usr/bin/env bash
# 本该判绿：E943 的登记已经归档（从树里删了，删它的提交在历史里），装置这一次又改了；取历史里它被删之前的那一版，第十二节里写着起跑脚本的钉值，成功句点名取自历史。
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
registry() { # <号> <第十二节的正文> [<第五节的补充>]
  printf '# E%s 第 2 次重跑登记\n\n## 五、跑法\n\n命令照起跑脚本。\n\n%s\n\n## 十二、修订\n\n%s\n\n## 十三、读过的文件\n\n读过装置。\n' "$1" "${3:-（第五节没有补充）}" "$2" > "research/prompts/e$1-r2-prereg.md"
}

apparatus 943 v1; sha=$(sha_of 943)
starter 943 "registered_apparatus_sha256=$sha"; registry 943 "R2-0 登记时的装置 sha256 $sha"
git add -A
git commit -qm '基准：起跑脚本、装置与登记'
git rm -q research/prompts/e943-r2-prereg.md
git commit -qm '归档：这一轮的登记删出工作区'
apparatus 943 v2
