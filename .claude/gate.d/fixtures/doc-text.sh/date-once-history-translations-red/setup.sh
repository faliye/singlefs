#!/usr/bin/env bash
# 排除只到两份变更史与 INDEX.md 的英日译文，别的 kb 页译文照判：基准里决策变更史的英语译文与一份实验页的英语译文都干净；
# 这一轮两份各加同一天的第二块（变更史在另一节，实验页在同一节下）。date-once 只点名实验页译文，判红。
# 排除扩到全部 .en.md 时实验页译文不报（退 77）；弄坏开关 date-once-history-translations-judged 下变更史译文也被点名，「1 份」对不上。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p .claude/kb/experiments
cat > .claude/kb/decisions-history.en.md <<'X'
# Sample Decision Change History

## D1 (Sample Decision)

### 2026-10-01

- Item 3 reworded.

## D2 (Other Decision)

- No change yet.
X
cat > .claude/kb/experiments/01-sample-experiment.en.md <<'X'
## E1 Sample Experiment —— partly run

### 2026-09-24: first part

- The first part did A.

## Version History
X
git add -A && git commit -qm base
python3 - <<'PY'
path = '.claude/kb/decisions-history.en.md'
text = open(path, encoding='utf-8').read()
text = text.replace('- No change yet.', '### 2026-10-01\n\n- Item 5 reworded as well.')
open(path, 'w', encoding='utf-8').write(text)
path = '.claude/kb/experiments/01-sample-experiment.en.md'
text = open(path, encoding='utf-8').read()
text = text.replace('## Version History', '### 2026-09-24: second part\n\n- The second part did B.\n\n## Version History')
open(path, 'w', encoding='utf-8').write(text)
PY
