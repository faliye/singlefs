#!/usr/bin/env bash
# 复跑：在草稿目录建副本（驱动放在路径尾不是 scripts 的位置），跑两只探针。不起 cargo、不碰真库目录。
# 用法：bash rerun.sh <仓根> <草稿目录>
set -euo pipefail
repository="$1" scratch="$2"
model="$repository/research/prompts/tooling-broad-mutation-r1-opus-model"
mkdir -p "$scratch/w/.claude/scripts" "$scratch/w/.claude/singlefs-ai-sop/scripts" "$scratch/w/.claude/hooks" "$scratch/w/drv/bin"
cp "$repository/.claude/scripts/project_preflight.py" "$scratch/w/.claude/scripts/"
cp "$repository/.claude/singlefs-ai-sop/scripts/preflight.py" "$scratch/w/.claude/singlefs-ai-sop/scripts/"
cp "$repository/.claude/hooks/lib_heavy_tests.py" "$repository/.claude/hooks/lib_shell_words.py" "$scratch/w/.claude/hooks/"
cp "$repository/scripts/broad-mutation.py" "$scratch/w/drv/bin/"
cd "$repository"
nice -n 19 python3 "$model/classify_probe.py" "$scratch/w" "$repository"
nice -n 19 python3 "$model/resume_probe.py" "$scratch/w/drv/bin/broad-mutation.py" "$scratch/run"
