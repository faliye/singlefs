#!/usr/bin/env bash
# checker-tier-research-build-and-replay.sh 的 research-unit-tests 格按条复用样本的现场：git init（判定库放 git common-dir），
# 照这一版拷一份上一版进 .unit-verdict-sample/prior/、在上一版里删掉第三条单测 adds_zero_to_itself（恰好删中一次，删不中退 1）。
# 样本带一份 scripts/experiments/Cargo.lock（cargo 照这个工作区生成的那一份）：上一版拷的时候就有，两版的依赖行相同；没有它，上一版的键记「Cargo.lock 不在」、
# 这一版 cargo build 现生成一份，两条旧单测都「变了 1 处」、一条都不复用。
set -euo pipefail
prior=.unit-verdict-sample/prior
mkdir -p "$prior/scripts"
cp -a scripts/experiments "$prior/scripts/"
python3 -c '
import sys
path = sys.argv[1]
old = "\n    #[test]\n    fn adds_zero_to_itself() {\n        assert_eq!(super::add(0, 0), 0);\n    }\n"
text = open(path, encoding="utf-8").read()
if text.count(old) != 1:
    sys.exit(f"样本坏了：{path} 里第三条单测命中 {text.count(old)} 次（要恰好 1 次）")
open(path, "w", encoding="utf-8").write(text.replace(old, ""))' "$prior/scripts/experiments/tiny/src/lib.rs"
git init -q .
printf '%s\n' .unit-verdict-sample/ >> .git/info/exclude
