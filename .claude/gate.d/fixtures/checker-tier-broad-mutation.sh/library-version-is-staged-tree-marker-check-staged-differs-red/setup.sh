#!/usr/bin/env bash
# 样本现场（情形 marker-check-staged-differs）：查标记那一支（样本放 .broad-sample/marker-check；门禁 --marker-check 走同一支），不跑格。
# 暂存区的文件与别的 library-version-is-staged-tree 样本相同，登记了这一道（输入路径 crates/ Cargo.toml Cargo.lock litmus/ .cargo/）。
# 阶段在样本模式下先起 .broad-sample/prepare.py（把驱动的路径给它），它照 admission.py stage-marker-write 写这一道的全绿标记（clean-unmarked 不写），标记进样本自己的 .git。
# staged-differs 照攻方 exp1 的现场：lib.rs 的 A 在工作区、暂存区里是另一份 B；标记按工作区算指纹（写的是 A 那一份），查标记那一支要先核出两边不同、不查标记。
set -euo pipefail
variant="staged-differs"
mkdir -p crates/sample/src litmus .cargo .claude/gate.d .broad-sample
printf '[workspace]\nmembers = ["crates/sample"]\nresolver = "2"\n' > Cargo.toml
printf '# 样本的锁文件\nversion = 4\n' > Cargo.lock
printf '[package]\nname = "sample"\nversion = "0.1.0"\nedition = "2021"\n' > crates/sample/Cargo.toml
printf 'pub fn add(left: u32, right: u32) -> u32 {\n    left + right\n}\n' > crates/sample/src/lib.rs
printf 'C sample\n{}\nP0(int *x) { WRITE_ONCE(*x, 1); }\nexists (x=1)\n' > litmus/sample.litmus
printf '[build]\njobs = 1\n' > .cargo/config.toml
printf 'checker-tier-crates-mutation-replay.sh\tcrates/ Cargo.toml Cargo.lock litmus/\t# 样本：广谱变异的导出范围取这一行\n' > .claude/gate.d/stage-inputs.tsv
printf 'checker-tier-broad-mutation.sh\tcrates/ Cargo.toml Cargo.lock litmus/ .cargo/\t# 样本：查标记之前核这几条路径上工作区等于暂存区\n' >> .claude/gate.d/stage-inputs.tsv
: > .broad-sample/marker-check
printf '%s\n' "$variant" > .broad-sample/variant
cat > .broad-sample/prepare.py <<'PYEOF'
#!/usr/bin/env python3
"""查标记样本的后半：阶段在样本模式下起它（python3 .broad-sample/prepare.py <驱动>），照 admission.py stage-marker-write 写这一道的全绿标记；clean-unmarked 不写。"""
import os, subprocess, sys
with open(os.path.join(".broad-sample", "variant"), encoding="utf-8") as handle:
    variant = handle.read().strip()
if variant != "clean-unmarked":
    admission = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(sys.argv[1]))), "gate", "admission.py")
    done = subprocess.run([sys.executable, admission, "stage-marker-write", os.getcwd(), "checker-tier-broad-mutation.sh"], capture_output=True, text=True)
    if done.returncode != 0:
        print(f"写标记没成：{done.stdout}{done.stderr}", file=sys.stderr)
        sys.exit(1)
PYEOF
git init -q .
git add -A
if [[ "$variant" == staged-differs ]]; then
  cp crates/sample/src/lib.rs .broad-sample/lib-A.rs
  printf 'pub fn add(left: u32, right: u32) -> u32 {\n    left - right\n}\n' > crates/sample/src/lib.rs
  git add crates/sample/src/lib.rs
  cp .broad-sample/lib-A.rs crates/sample/src/lib.rs
fi
