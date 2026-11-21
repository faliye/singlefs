#!/usr/bin/env bash
# 本该判红：E941 的装置改了、起跑脚本的钉值也改成了新值，新钉值只写在登记第五节新加的一行里，第十二节一个字没动——修订要写在第十二节。
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

apparatus 941 v1; old=$(sha_of 941)
starter 941 "registered_apparatus_sha256=$old"; registry 941 "R2-0 登记时的装置 sha256 $old"
git add -A
git commit -qm '基准：起跑脚本、装置与登记'
apparatus 941 v2; new=$(sha_of 941)
starter 941 "registered_apparatus_sha256=$new"
registry 941 "R2-0 登记时的装置 sha256 $old" "新装置 sha256 $new"
