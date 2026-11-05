#!/usr/bin/env bash
# 门禁 lkmm 样本的现场：git init（判定库的名字取被判那棵树的 git common-dir，不是 git 工作树的根 .claude/scripts/lkmm.sh 退 2、一条都不判）
set -euo pipefail
git init -q .
# 假 herd7 与只有模型文件的假内核树（门禁样本只走假 herd7；litmus 里没写 fake-observation 的当 Never）
mkdir -p .lkmm-fake-herd7/linux/tools/memory-model
cat > .lkmm-fake-herd7/herd7 <<'HERD'
#!/usr/bin/env bash
# 样本用的假 herd7：-version 打固定版本串；-conf <cfg> <litmus> 照 litmus 里「fake-observation: <判定>」那一行交 Observation（只给 门禁 lkmm 的样本用）
if [[ "${1:-}" == -version ]]; then echo "7.57+fake, Rev: sample"; exit 0; fi
[[ -f linux-kernel.cfg ]] || { echo "假 herd7：没在 tools/memory-model 里跑" >&2; exit 2; }
litmus="${!#}"
name="$(sed -n '1s/^C[[:space:]]*//p' "$litmus")"
observation="$(sed -n 's/.*fake-observation:[[:space:]]*\([A-Za-z]*\).*/\1/p' "$litmus" | head -1)"
echo "Test $name Allowed"
echo "States 1"
echo "Observation $name ${observation:-Never} 0 1"
HERD
printf '"Linux-kernel memory consistency model (样本)"\nmodel linux-kernel.cat\nbell linux-kernel.bell\n' > .lkmm-fake-herd7/linux/tools/memory-model/linux-kernel.cfg
printf '"样本的 cat"\n' > .lkmm-fake-herd7/linux/tools/memory-model/linux-kernel.cat
printf '"样本的 bell"\n' > .lkmm-fake-herd7/linux/tools/memory-model/linux-kernel.bell
