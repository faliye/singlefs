#!/usr/bin/env bash
# 本该判绿：与 pinned-apparatus-revision-repin-only-red 同一种起点（装置在早先的提交里改了，起跑脚本与登记停在旧钉值），这一次起跑脚本的钉值改成现算值、
# 登记第十二节也新加了一行写着新钉值的修订，旧钉值那一行原样还在。装置源码这次没变，钉值变了触发、核过登记，成功句点名。
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

apparatus 934 v1; old=$(sha_of 934)
starter 934 "registered_apparatus_sha256=$old"; registry 934 "R2-0 登记时的装置 sha256 $old"
git add -A
git commit -qm '基准：起跑脚本、装置与登记'
apparatus 934 v2
git commit -qam '装置早先已经改了，起跑脚本与登记没跟着动'
new=$(sha_of 934)
starter 934 "registered_apparatus_sha256=$new"
registry 934 "R2-0 登记时的装置 sha256 $old"$'\n\n'"R2-1 重钉：装置在早先的提交里改过，新的 sha256 $new"
