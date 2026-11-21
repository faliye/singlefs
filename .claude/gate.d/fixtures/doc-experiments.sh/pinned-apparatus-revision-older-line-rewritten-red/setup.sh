#!/usr/bin/env bash
# 本该判红：E950 的登记第十二节有两行，登记时那一行写着 sha1、修订那一行写着基准钉值 sha2；装置改成第三版、钉值改成 sha3，
# 把登记时那一行原地改成 sha3、没新加行——基准那一版第十二节的那一行不在了，修订只许新加行，不改写已有的。
# 只核写着基准钉值的行的写法放过登记时那一行（它没写基准钉值），落到「钉值只认第十二节里最后写下的那个」上，红句换成另一句。
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

apparatus 950 v1; sha1=$(sha_of 950)
apparatus 950 v2; sha2=$(sha_of 950)
starter 950 "registered_apparatus_sha256=$sha2"
registry 950 "R2-0 登记时的装置 sha256 $sha1"$'\n\n'"R2-1 装置改了停机条件，sha256 $sha2"
git add -A
git commit -qm '基准：起跑脚本、装置与登记'
apparatus 950 v3; sha3=$(sha_of 950)
starter 950 "registered_apparatus_sha256=$sha3"
registry 950 "R2-0 登记时的装置 sha256 $sha3"$'\n\n'"R2-1 装置改了停机条件，sha256 $sha2"
