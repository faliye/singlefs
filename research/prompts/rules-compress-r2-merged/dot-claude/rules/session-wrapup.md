<!-- doc-lint:rule-definition -->
# 收尾：每轮工作结束前必做

用户用 `halt` 终止的那一轮，这几条都不做。

## 0. 报进度——排在最前，不许省

一行三件事：**现在在哪个里程碑 / 门禁过不过 / 还缺什么证据**。门禁那一格照 `.claude/rules/evidence-discipline.md`「陈述外部状态之前，现查一次」表里「门禁现在过不过」一行引；里程碑与证据两格现查（读记录与产物），不从上一轮的笔记里抄。

同时说清这一轮给这个数加了多少：出了个零结论、推翻了一条决策、或只是弄明白了某个证据现在还拿不到，就写 0；不把别轮的成果算进这一轮。

## 1. 这轮敲的脚本/命令，有没有该收归 `scripts/` 的？

- 判据是「会不会被抄第二遍」，不是「写得好不好」。
- 踩过的坑做成会红的检查，不做成提醒句，立的时机照 `.claude/singlefs-ai-sop/rules/sop-first.md`（边界）：先记欠账、按批立。
- 已经没人用的，删。

## 2. 这轮用到的 skill / 规则，有没有该改的？

有哪一步靠记性撑住、判据事后才补、同样的样板抄了第三遍，就当场改那个文件，不留到「下次」。改哪里看这条管什么：

- 管协作（证据、文档、决策落点、门禁反馈）：改 singlefs-ai-sop，并抬 `VERSION`（哪些路径要抬，见 `.claude/singlefs-ai-sop/scripts/version-discipline.sh` 的 `GOVERNED`）。
- 管文件系统怎么设计：改项目本地的 `.claude/kb/`、`.claude/rules/` 或项目 `CLAUDE.md`。
- 拿不准就放项目本地；「别的项目会不会也需要」不是判据。

不写进会话私有的记忆（Claude Code 的 memory 这类）：别人可能撞上的坑与约定写进项目或这套规范；私有记忆只放用户个人的偏好（什么时候提交、用什么语言回复这类）。

## 3. 决策有没有变动？

这一轮推翻或定下了任何设计决策，当场写进 `.claude/kb/decisions.md`，带上依据。掂量完给的说法（「这条现在不该定」「还是维持原方案」）实际推翻了 `.claude/kb/decisions.md` 里已写的某句话的，同样算一次推翻，当场记、带依据。

## 4. 同一个仓里有没有别的会话在飞？

几个会话同时在一个仓里干活时，收尾前逐条对一遍：

- 提交前把改动分成「这一轮的」与「不是这一轮的」，只提交这一轮的。
- 门禁红了，判红之前先看失败涉及的文件在不在这一轮的改动里：不在就如实说「红了，但不是这一轮的」，不顺手修，照 `.claude/rules/command-safety-by-review.md`「遇到问题，先找有没有别的 agent 在管这一块」找管它的那一个。分不清是谁的红、确要跑整轮门禁的，先照 `.claude/rules/verification.md`「一个门禁：两种跑法」「提交之外」那一条弹窗问用户，同意了才跑 `gate.sh --staged`（经服务交 `scripts/gate/gate-staged.sh`，照 `.claude/skills/gate/staged-gate.md`）：它在临时 worktree 上只拿 HEAD 与暂存区跑，这时候红的就是这一次提交带进来的。不带 `--staged` 的 `gate.sh` 开跑与收尾各记一次工作区指纹，对不上就判红；等改动停下再跑，或者用 `--staged`。
- 公共编号先到先得：实验号这类大家共用的编号，取号之前先看已有的最大号，取号在写第一个文件的同一条命令里现查；历史条目编号照 `.claude/singlefs-ai-sop/rules/kb-discipline.md`（8. 正文只写现状，历史放文末的「历史版本」）。项目用了 `history-ordinal.sh` 不认的编号形态，才在项目侧补一道会红的检查。
- 改公共文件只做定点替换，不整篇重写。新建文件用排他方式（`set -o noclobber`、`open(路径, 'x')`）；能在工具层拦（写入前的钩子拒绝覆盖未跟踪的文件）就在工具层拦。
- 「只提交这些路径」不等于 `git commit -- <路径>`：带路径的 commit 提交的是这些路径在工作区里的内容。暂存前先确认 `git diff --cached --name-only` 为空；暂存完、提交前再核一遍名单与逐文件的 cached diff，然后不带路径地 `git commit`。
- 清索引（`git reset`、`git restore --staged .`）之前先 `git diff --cached --name-only` 看有没有别人暂存的；只清自己的路径：`git restore --staged -- <路径…>`。
