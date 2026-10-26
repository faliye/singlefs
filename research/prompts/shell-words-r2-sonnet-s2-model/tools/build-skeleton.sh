#!/usr/bin/env bash
# build-skeleton.sh <仓根> <冻结副本> <草稿目录>：在草稿目录里搭喂钩子用的几棵小树。
#   <草稿目录>/repo        钩子拷贝所在的「仓」：.claude/（钩子、定义、门禁、规则）、scripts/ 下的 .sh .py .tsv、crates/、Cargo.toml，已 git init 并提交；
#                          .claude/hooks/ 下四份被判文件用冻结副本里的（逐字节同开工快照），其余钩子库取仓根现在的
#   <草稿目录>/copy        repo 的整树拷贝（不带 .git）：模拟子 agent 在草稿目录里的仓副本
#   <草稿目录>/stagedrepo  暂存区非空的小 git 仓（b.md 已暂存）；emptyrepo 暂存区为空
#   <草稿目录>/wr/scratch  给 SINGLEFS_WORK_ROOT 指过来的「工作根」（⑫ 的用例用它，免得仓在工作根底下被整个放行）
set -euo pipefail
REPO_ROOT="$1"; FROZEN="$2"; WORK="$3"
mkdir -p "$WORK/repo/.claude" "$WORK/wr/scratch"
cd "$REPO_ROOT"
cp -r .claude/hooks .claude/agents .claude/gate.d .claude/singlefs-ai-sop .claude/scripts .claude/rules .claude/skills "$WORK/repo/.claude/"
cp .claude/main-agent.md .claude/agent-common.md .claude/settings.json .claude/heavy-thread-default "$WORK/repo/.claude/"
cp .claude/*.tsv "$WORK/repo/.claude/" 2>/dev/null || true
for f in "$FROZEN"/.claude/hooks/*; do cp "$f" "$WORK/repo/.claude/hooks/$(basename "$f")"; done
cp Cargo.toml Cargo.lock "$WORK/repo/"
[ -d .cargo ] && cp -r .cargo "$WORK/repo/"
cp -r crates "$WORK/repo/"
rsync -a --include='*/' --include='*.sh' --include='*.py' --include='*.tsv' --exclude='*' --exclude='experiment-apparatus/*/*/*' scripts/ "$WORK/repo/scripts/"
cd "$WORK/repo"
mkdir -p fx research/results
printf '#!/usr/bin/env bash\ncargo test --all\n' > fx/heavy-all.sh
printf '#!/usr/bin/env bash\necho hello\nls -la\n' > fx/light.sh
printf '#!/usr/bin/env bash\nbash .claude/gate.d/doc-text.sh\n' > fx/uses-commit.sh
echo x > fx/existing.sh
chmod +x fx/*.sh
echo tracked > research/results/tracked.txt
git init -q .
git -c user.name=t -c user.email=t@t add -A >/dev/null 2>&1
git -c user.name=t -c user.email=t@t commit -qm skeleton
echo untracked > research/results/untracked.txt
rm -rf "$WORK/copy"; mkdir -p "$WORK/copy"
tar cf - --exclude=.git . | (cd "$WORK/copy" && tar xf -)
for r in stagedrepo emptyrepo; do
  rm -rf "$WORK/$r"; mkdir -p "$WORK/$r"; cd "$WORK/$r"
  git init -q .; echo a > a.md; git add a.md; git -c user.name=t -c user.email=t@t commit -qm a
done
cd "$WORK/stagedrepo"; echo b > b.md; git add b.md
echo "skeleton ready: $WORK"
