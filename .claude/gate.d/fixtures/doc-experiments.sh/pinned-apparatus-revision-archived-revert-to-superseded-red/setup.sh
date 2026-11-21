#!/usr/bin/env bash
# 本该判红：E952 的登记第十二节先写装置 sha1（交回时），后写 sha2（打进主工作区之后，以这一份为准），起跑脚本钉 sha2；
# 登记随后归档（从树里删了，删它的提交在历史里）；这一次装置回退成 sha1 那一版、钉值改成 sha1。
# 取历史里它被删之前的那一版：sha1 写在第十二节里，可 sha2 写在它之后，钉值只认第十二节里最后写下的那个，sha1 是已作废的值。
# 只看钉值在不在第十二节里的写法判绿；登记不在树里就当已归档不判的写法同样判绿。
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

apparatus 952 v1; sha1=$(sha_of 952)
apparatus 952 v2; sha2=$(sha_of 952)
starter 952 "registered_apparatus_sha256=$sha2"
registry 952 "装置 sha256（交回时）$sha1"$'\n\n'"装置 sha256（打进主工作区之后，以这一份为准）$sha2"
git add -A
git commit -qm '基准：起跑脚本、装置与登记'
git rm -q research/prompts/e952-r2-prereg.md
git commit -qm '归档：登记删出工作区'
apparatus 952 v1
starter 952 "registered_apparatus_sha256=$sha1"
