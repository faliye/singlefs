#!/usr/bin/env bash
# 本该判绿：E953 的登记在树里，第十二节有登记时的 sha1 与修订的 sha2 两行，起跑脚本钉 sha2；
# 这一次修订撤回：装置回退成 sha1 那一版、钉值改成 sha1，第十二节末尾新加一行「R2-1 的改动撤回，装置回退到登记时那一版，sha256 sha1」，已有的两行原样在。
# sha1 最后一次出现在新加的那一行、在 sha2 之后，钉值是第十二节里最后写下的那个：合规的回退放行，成功句把它算进装置变了而新加了修订的那几份。
# 比谁写得更后时取第一次出现的位置的写法（sha1 第一次出现在登记时那一行、在 sha2 之前）判红。
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
registry() { # <号> <第十二节的正文>
  printf '# E%s 第 2 次重跑登记\n\n## 五、跑法\n\n命令照起跑脚本。\n\n## 十二、修订\n\n%s\n\n## 十三、读过的文件\n\n读过装置。\n' "$1" "$2" > "research/prompts/e$1-r2-prereg.md"
}

apparatus 953 v1; sha1=$(sha_of 953)
apparatus 953 v2; sha2=$(sha_of 953)
starter 953 "registered_apparatus_sha256=$sha2"
registry 953 "R2-0 登记时的装置 sha256 $sha1"$'\n\n'"R2-1 装置改了停机条件，sha256 $sha2"
git add -A
git commit -qm '基准：起跑脚本、装置与登记'
apparatus 953 v1
starter 953 "registered_apparatus_sha256=$sha1"
registry 953 "R2-0 登记时的装置 sha256 $sha1"$'\n\n'"R2-1 装置改了停机条件，sha256 $sha2"$'\n\n'"R2-2 R2-1 的改动撤回，装置回退到登记时那一版，sha256 $sha1"
