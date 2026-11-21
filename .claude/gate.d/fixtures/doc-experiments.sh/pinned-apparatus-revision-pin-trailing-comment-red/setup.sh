#!/usr/bin/env bash
# 本该判红：E935 的起跑脚本钉值行末尾带注释（registered_apparatus_sha256=<64 位>  # 登记第十二节 R2-0），装置改了、登记第十二节没动。
# 带行尾注释的钉值行也认，整份起跑脚本进对象。另放一份照常的起跑脚本（E936，装置没变），不判。
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

apparatus 935 v1; sha=$(sha_of 935)
starter 935 "registered_apparatus_sha256=$sha  # 登记第十二节 R2-0"; registry 935 "R2-0 登记时的装置 sha256 $sha"
apparatus 936 v1; other=$(sha_of 936)
starter 936 "registered_apparatus_sha256=$other"; registry 936 "R2-0 登记时的装置 sha256 $other"
git add -A
git commit -qm '基准：两份起跑脚本、装置与登记'
apparatus 935 v2
