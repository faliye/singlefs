#!/usr/bin/env bash
# 本该判红：E937 的起跑脚本头一句注释先点名了上一次的登记（research/prompts/e937-r1-prereg.md，沿用它的几何，那一份不在树里也不在历史里），
# 自己的登记写成「跑前登记 research/prompts/e937-r2-prereg.md」；装置改了、自己的登记第十二节没动。
# 登记认写成「跑前登记 <路径>」的那一处，不取注释里头一处点名的。
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
registry() { # <号> <第十二节的正文> [<第五节的补充>]
  printf '# E%s 第 2 次重跑登记\n\n## 五、跑法\n\n命令照起跑脚本。\n\n%s\n\n## 十二、修订\n\n%s\n\n## 十三、读过的文件\n\n读过装置。\n' "$1" "${3:-（第五节没有补充）}" "$2" > "research/prompts/e$1-r2-prereg.md"
}

apparatus 937 v1; sha=$(sha_of 937)
{
  echo '#!/usr/bin/env bash'
  echo '# E937 沿用 research/prompts/e937-r1-prereg.md 的几何与停机条款；'
  echo '# E937 第 2 次重跑一段的跑法（跑前登记 research/prompts/e937-r2-prereg.md 第五节「跑法」）：'
  echo 'apparatus_source=crates/sample/src/bin/e937_apparatus.rs'
  echo "registered_apparatus_sha256=$sha"
} > scripts/experiment-apparatus/e937-r2-segment-run.sh
registry 937 "R2-0 登记时的装置 sha256 $sha"
git add -A
git commit -qm '基准：起跑脚本、装置与登记'
apparatus 937 v2
