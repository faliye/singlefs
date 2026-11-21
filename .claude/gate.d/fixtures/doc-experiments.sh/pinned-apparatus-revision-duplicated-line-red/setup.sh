#!/usr/bin/env bash
# 本该判红：E932 的装置改了、钉值没改，登记第十二节把已有的一行再抄了一遍——同一行写了几遍按次数算，第二份算「新加的」，
# 其实什么修订也没写：写着钉值的新加行与基准那一版第十二节里已有的行逐字相同，不算新加修订。
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

apparatus 932 v1; sha=$(sha_of 932)
starter 932 "registered_apparatus_sha256=$sha"; registry 932 "R2-0 登记时的装置 sha256 $sha"
git add -A
git commit -qm '基准：起跑脚本、装置与登记'
apparatus 932 v2
registry 932 "R2-0 登记时的装置 sha256 $sha"$'\n'"R2-0 登记时的装置 sha256 $sha"
