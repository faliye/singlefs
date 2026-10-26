# 第九批工具员：交回闸核命名检查与写回员目标、检出钩子认草稿目录里的 mktemp（landing-audit-b9-handback-guard，2026-10-23）

## 结论

拆分提案第九十节 ⑥、⑦、⑰ 三行的验收标准都达到了。两个钩子各自先在草稿里验、再经 `scripts/gate/hook-swap.py` 换进主工作区，换上之后在主工作区再验：
`bash .claude/hooks/handback-guard.sh --selftest` 退 0，「查了 45 种」；`bash .claude/hooks/bash-command-detector.sh --selftest` 退 0，「查了 750 种」。
新加的开关逐个打开，都红在点名的那几格上。

- ⑥ 交回闸 ⑤：`implementation-writer`、`experiment-runner`、`tooling-writer` 从它自己的会话记录里认 Edit / Write / MultiEdit 改成的 `crates/`、`scripts/` 下 `.rs`。改动在主工作区或带 `.git` 的副本（git worktree）里、而最后一次改 `.rs` 之后没有真跑过两条命名检查的，拒交回，拒绝说明里给出两条命令，`<根>` 已填好。改动在没有 `.git` 的副本（rsync 拷的）里的不判。
- ⑦ 交回闸 ⑥：写回员规格（`kb-spec-check.py` 的 `load_spec` / `parse_markdown_spec` 认得的两种写法）点名的每个目标文件，要么在主工作区存在，要么写进了交回正文或报告里标题含「找不到的目标」的那一节；两样都不是的拒。
- ⑰ 检出钩子 ⑫：`mktemp -d` 不带 `-t`、模板所在目录落在这个子 agent 认领的草稿目录里的，建出的目录算「草稿目录内」，不报检出；模板目录在草稿目录外的照报。赋给变量的写法、直接写在目标里的写法都认。

推翻条件：
- 主工作区里跑上面两条 `--selftest` 有一条不退 0，或某条开关打开后没红在点名的格上，上面的结论就不成立。
- 真用时出现下面任何一种误判，也算推翻：实现员在主工作区改了 `.rs`、改完把两条命名检查都跑过了还被拒；在自己草稿目录里 `$(mktemp -d <草稿目录>/x.XXXXXX)` 还被报「这一刻算不出来」。

## 改过的文件

只改了两份，都经 hook-swap.py 换上（改名换上新 inode）。没有新建文件，也没有改钩子 import 的库。

```
$ git diff --stat -- .claude/hooks/handback-guard.sh .claude/hooks/bash-command-detector.sh
 .claude/hooks/bash-command-detector.sh | 174 +++++++++++-
 .claude/hooks/handback-guard.sh        | 477 ++++++++++++++++++++++++++++++++-
 2 files changed, 631 insertions(+), 20 deletions(-)
```

（两份在暂存区里本来就有别人的改动，`git status` 是 `MM`。上面这份 diff 比的是工作区与暂存区，只有这一批的改动。）

sha256：改前、换上之后
- `.claude/hooks/handback-guard.sh`：`dd5bdf0a54b541aedffa8308e70ad07f4501121598173c1cca002ff3418a4790` → `d755973dfc40933bdcfd0f3310079aac33bf567920e90fc6c6245308c1125a4a`（换了两次：第一次 `d2ad214a…`，第二次补 `$(pwd)` 的认法、一格自证，还补了 `SINGLEFS_WORK_ROOT_BREAK=new-root-unknown` 头注下点名的格）
- `.claude/hooks/bash-command-detector.sh`：`514d663dee0b1f2151c2289ac54aaf3ce69bc8704d5b63adefaa8dcb6e673c7d` → `bdc7e2467a3944fa4564e8250770b5b300b6532becd71a7004c8ce2fde694742`

hook-swap 的记录：
- 每次换上都写了一份反向规格，在 `/home/fy5090/.local/share/singlefs/work/landing-audit-b9-handback-guard/hook-swap/`：handback 第一次 `reverse-20261022T213651Z-4041338.json`，检出钩子 `reverse-20261022T214320Z-109924.json`，handback 第二次 `reverse-20261022T214900Z-298716.json`。
- 喂法：`draft/handback-feeds.json` 6 条，其中 2 条 neutral；`draft/detector-feeds.json` 5 条，其中 2 条 neutral。
- 回滚时要注意：检出钩子的喂法里有 2 条指到已删的样本目录 `feed-scratch`、`other-scratch`，回滚时这两条不能拿来喂。

没有新钩子、没有新门禁阶段，所以没写新的 `# gate-similar:` / `# hook-events:` 行；两份原有的那几行没动。
动手前跑了 `gate-overlap.py --list`，输出在 `gate-overlap-list.txt`：挂 `PreToolUse[SubagentHandback]` 的已有 handback-guard.sh 与上游 handback-scratch-check.sh 两个。⑥ ⑦ 并进前者，⑰ 是在 bash-command-detector.sh 原有的 ⑫ 里改判法。

## ⑥ 交回闸核命名检查（handback-guard.sh 新的 ⑤）

判法写在文件头 ⑤ 一段。几处要点：
- 「真跑过」怎么认：Bash 的工具结果不是错误，或错误内容以「Exit code」开头（命名检查判红也算跑过）。被钩子拒的、超时的、没有结果的都不算。
- 命令怎么认：用 `lib_shell_words` 切出命令位置，用 `lib_heavy_tests.command_under_launcher` 剥掉 two-logs.sh 这类包装，用 `checked_cells` 读 `--check`。四格要点全，不带 `--check` 的整道也算。
- 根怎么认：要解到改动所在的那棵树。根里的 `~`、`$HOME`、`$PWD`、`$(pwd)` 按命令当时的目录展开。
- 时序：同一棵树每改一次 `.rs`，跑过的记录就清零，只认最后一次改之后跑的。

改之前先造了该红的输入，原来的钩子放过了：
- 样本 `fixtures/naming-unrun.json`（会话记录 `fixtures/session/subagents/agent-naming-unrun.jsonl`）：实现员 Edit 了主工作区 `crates/singlefs-core/src/lib.rs`，交回正文写「命名检查两条都跑了、都绿」，会话记录里一条 Bash 都没有。
- 改前 `CLAUDE_PROJECT_DIR=<仓根> bash .claude/hooks/handback-guard.sh < fixtures/naming-unrun.json` 退 0（`fixtures/before-naming-unrun.err` 是空的）。
- 改后同一份输入退 2（草稿正文 `draft/handback-guard-body.py` 喂的，输出在 `fixtures/after-naming-unrun.err`），拒绝说明原样：

```
✗ implementation-writer 改了 crates/ 或 scripts/ 下的 .rs，会话记录里改完之后没跑过共用约束点名的两条命名检查（只认真跑过的命令，报告里写跑了不算）：
  /home/fy5090/code/singlefs：最后改的是 /home/fy5090/code/singlefs/crates/singlefs-core/src/lib.rs；改完之后没跑：code-source-discipline.sh 那四格、naming-lint.sh
→ 怎么办：在改动所在的树上跑下面两条（<根> 已填好），命中在自己改动的文件里的修掉、在别的文件里的照写，两条的命令与末尾原样输出贴进报告，再交回（.claude/agent-common.md「门禁」一节「命名检查交回前跑」）：
    SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-source-discipline.sh 的 vague-names、test-file-names、ordinal-names、data-names 四格; 范围=4 格; 为什么=交回前核自己改动里的命名' bash .claude/gate.d/code-source-discipline.sh --check vague-names,test-file-names,ordinal-names,data-names /home/fy5090/code/singlefs
    bash .claude/singlefs-ai-sop/scripts/naming-lint.sh /home/fy5090/code/singlefs
  在 rsync 拷的副本（没有 .git）里改的这里不判，照那一条把两条命令写进报告「要主 agent 跑的命令」。
```

绿样本 `fixtures/naming-run.json`：改完之后两条都跑了（naming-lint 那条退 1、根写成 `.`）。改前改后都退 0。

自证新加的格（selftest 里的标签），全部退 0 通过：

| 格 | 期望 |
|---|---|
| 改了 .rs 没跑命名检查拒 | 2 |
| 改了 .rs 之后两条命名检查都跑了放行 | 0 |
| 只改 .md 放行 | 0 |
| rsync 副本里改的 .rs 交主 agent 放行 | 0 |
| 执行员改了 scripts/ 下 .rs 没跑命名检查拒 | 2 |
| 命名检查跑在最后一次改 .rs 之前拒 | 2 |
| 只跑了一条命名检查拒 | 2 |
| 命名检查被钩子拒了不算跑过 | 2 |
| 四格点名少一格不算跑过 | 2 |
| git worktree 副本里改的、命名检查根写成副本根放行 | 0 |
| 经 two-logs.sh 包着跑命名检查放行 | 0 |
| cd 之后根写成 $(pwd)、接管道跑命名检查放行 | 0 |
| 回扫员改 .rs 不判命名检查 | 0 |
| 改了 .rs 没跑的拒绝说明给出两条命令（根填改动所在的树） | True |

验收要的四格：「改了 .rs 没跑命名检查拒」「改了 .rs 之后两条命名检查都跑了放行」「只改 .md 放行」「rsync 副本里改的 .rs 交主 agent 放行」。

⑥ 的弄坏开关（`HANDBACK_GUARD_BREAK=`，每条打开后跑 `bash .claude/hooks/handback-guard.sh --selftest`；红的格与文件头点名的逐条对上了）：

| 开关 | 弄坏哪一处 | 红在 |
|---|---|---|
| `no-naming-check` | decide() 不判 ⑤ | 「改了 .rs 没跑命名检查拒」等六格（退 1，失败行 7 条） |
| `naming-root-unmatched` | 认出的根一律当成别的树 | 「改了 .rs 之后两条命名检查都跑了放行」「git worktree 副本里改的…放行」「经 two-logs.sh…放行」「cd 之后根写成 $(pwd)…放行」 |
| `naming-any-file` | 不看后缀 | 「只改 .md 放行」 |
| `naming-copy-judged` | 没有 .git 的副本也判 | 「rsync 副本里改的 .rs 交主 agent 放行」 |
| `naming-refused-counts` | 被钩子拒的也当跑过 | 「命名检查被钩子拒了不算跑过」 |

## ⑦ 写回员规格目标（handback-guard.sh 新的 ⑥）

规格从哪里认：派发提示是它会话记录的第一条，从里面找三处——
- 提示点名的工作根或旧根下的 `.json` / `.md` / `.txt`，「报告：」那一份除外；
- 「草稿目录：」下写回员自己起草的 `spec.json` / `spec.md`（`.claude/agents/kb-scribe.md` 第 0 步的名字）；
- 提示正文本身，按 markdown 写法读。

只收旧串、新串都在的条目，解析直接用 `scripts/kb/kb-spec-check.py` 的 `load_spec` / `parse_markdown_spec`，没另立格式。
「找不到的目标」一节只认 markdown 标题行里含这几个字的那一节，到同级或更高一级的下一个标题为止。文件写成从仓根起的路径、规格里的原写法、绝对路径都认。

改之前的红输入：
- 样本 `fixtures/scribe-unlisted.json`：规格 `fixtures/scribe-spec.json` 点名两份目标，`.claude/kb/decisions.md` 在，`.claude/kb/landing-audit-b9-no-such-target.md` 不在；报告 `fixtures/scribe-unlisted-result.md` 里没有「找不到的目标」一节。
- 改前退 0，改后退 2，说明原样：

```
✗ 写回员规格点名的目标文件在主工作区不在，交回与报告里「找不到的目标」一节也没有它：
  .claude/kb/landing-audit-b9-no-such-target.md（规格 /home/fy5090/.local/share/singlefs/work/landing-audit-b9-handback-guard/fixtures/scribe-spec.json）
→ 怎么办：照 .claude/agents/kb-scribe.md 那一句办：整份不写，报告里加一节「## 找不到的目标」，逐条写规格里的文件（从仓根起的路径）与旧串，以及 `git log --diff-filter=DR --name-only -- <文件名>` 和 `grep -rnF <旧串的一段>` 查到的新位置（查不到写查不到），再交回；目标其实在的，核规格里的路径是不是写错了（从仓根起写）。
```

绿样本 `fixtures/scribe-listed.json`：报告 `fixtures/scribe-listed-result.md` 的「## 找不到的目标」一节写了那份文件。改前改后都退 0。

自证新加的格，全部通过：

| 格 | 期望 |
|---|---|
| 写回员规格目标都在放行 | 0 |
| 写回员规格目标不在、报告没写找不到的目标拒 | 2 |
| 写回员规格目标不在、报告「找不到的目标」一节写了它放行 | 0 |
| 写回员规格目标不在、只在报告别的节提到拒 | 2 |
| 写回员规格写在提示正文里、目标不在拒 | 2 |
| 写回员自己起草的 spec.json 目标不在拒 | 2 |
| 写回员没有规格不判 | 0 |
| 写回员目标不在的拒绝说明列出文件与出路 | True |

开关：

| 开关 | 弄坏哪一处 | 红在 |
|---|---|---|
| `no-scribe-targets` | decide() 不判 ⑥ | 四格「…拒」与拒绝说明那一格（退 1，失败行 5 条） |
| `scribe-section-anywhere` | 报告里哪一节提到都算 | 「写回员规格目标不在、只在报告别的节提到拒」 |
| `scribe-section-unread` | 不读那一节 | 「写回员规格目标不在、报告「找不到的目标」一节写了它放行」 |

全部 16 条 `HANDBACK_GUARD_BREAK` 开关（原有 8 条加新 8 条）逐条复跑过：`tools/replay_switches.py` 照文件头每一条点名的标签与「」里的原文去对失败行，结果是「跑了 16 条开关，不对 0 条」，每条的输出在 `draft/handback-switches/<值>.out`。
原有的 `cite-any-agent` 下，⑤ ⑥ 里应当放行的格也会红，文件头补了一行点名。
`SINGLEFS_WORK_ROOT_BREAK=new-root-unknown` 下，⑥ 有三格认不出工作根下的规格、也红了，文件头补了点名；实测退 1，红的格与头注一致。`legacy-root-unknown` 实测退 1，没变。

## ⑰ 检出钩子认草稿目录里的 mktemp（bash-command-detector.sh 的 ⑫）

「这个子 agent 认领的草稿目录」按两样认，两样都要对上：
1. 归属文件（`scripts/hosts/work-directory-ownership.sh`）：模板目录在工作根或旧根下，第一层那个目录的 `.owner` 里写的 `session:` 等于 hook 输入的 session_id。
2. 写者锁（`.claude/hooks/lib_writer_lock.py` 的 `find_lock_directory` / `read_lock` / `read_claim` / `launch_receipt`）：离模板目录最近的那把锁，认领文件写的是这个 agent_id；还没有认领文件的，主会话记录里它的派发回执与这把锁对得上。

这里只读，不建认领文件。新函数是 `own_scratch_directory`、`mktemp_template_directory` 与 `mktemp_substitutions_expanded`：
- `mktemp_directory_value` 先试模板这一支，再走原来的 `${TMPDIR:-/tmp}` 那一支；
- 赋值时先把值里已经赋过的变量代进去（`$W/rs.XXXXXX` 这种写法）；
- 直接写在目标里的 `$(mktemp -d …)` 在判 ⑫ 之前换掉。

身份（agent_id、session_id、transcript_path）只在 `bash_scope_refusal` 判 ⑫ 的那一刻设上；⑤、⑦、⑨ 走同一段 `overwrite_steps`，判法不变。

改之前的红输入，现场形状就是审计记录里那几条：
- 样本 `fixtures/detector-real-identity.json`：本工具员自己的身份（agent `a249586cb8c87c57c`，会话 412f440a…），命令是 `T=$(mktemp -d <本草稿目录>/real-identity.XXXXXX); cp -a …/lib_work_root.py $T/`。
- 改前喂 `bash .claude/hooks/bash-command-detector.sh`：退 0，检出 1 条。检出原文与审计里的同形：「cp $(mktemp -d …/rs.XXXXXX)/lib_work_root.py 的目标这一刻算不出来（变量、命令替换或通配），写范围没判，没拦」。
- 改后同一份输入：退 0，检出 0 条。

另外三份样本的检出条数，改前 → 改后：

| 样本 | 改前 | 改后 |
|---|---|---|
| 模板在样本 agent 认领的子草稿目录里 | 1 | 0 |
| 直接写进 `$(mktemp -d …)/cell.py` | 1 | 0 |
| 模板在别的 agent 认领的目录里 | 1 | 1（照报） |
| 模板在 `/tmp` 下 | 1 | 1（照报） |

自证新加的格，全部通过：

| 格 | 期望记几条 |
|---|---|
| 模板在自己认领的草稿目录里：T=$(mktemp -d $W/名.XXXXXX) 之后 cp 进 $T/ 不记 | 0 |
| 模板在自己认领的草稿目录里：直接写进 $(mktemp -d …)/文件 不记 | 0 |
| cd 进自己认领的草稿目录之后模板写相对路径不记 | 0 |
| 还没有认领文件、派发回执对得上那把锁：模板在它的草稿目录里不记 | 0 |
| 派发回执对不上那把锁（别的子 agent）：仍记 | 1 |
| 模板在别的子 agent 认领的草稿目录里，仍记 | 1 |
| 模板目录的归属文件是别的会话，仍记 | 1 |
| 模板在 /tmp 下（不是草稿目录），仍记 | 1 |
| hook 输入里没有 agent_id，模板在草稿目录里也仍记 | 1 |
| stdin:模板在自己认领的草稿目录里的 mktemp -d 放行（退出码 0）、不记检出 | (0, 0) |

开关（`BASH_COMMAND_DETECTOR_BREAK=`）：
- `mktemp-scratch-template-unknown`（带目录的模板一律算不出，改前的样子）：退 1，失败行 5 条，就是「不记」那四格加 stdin 那一格，与头注「共 5 格」一致。
- `mktemp-template-owner-unchecked`（不看归属文件与锁）：退 1，失败行 3 条，就是「别的子 agent」「别的会话」「派发回执对不上」三格，与头注「共 3 格」一致。

原有开关的复核：
- `mktemp-directory-unknown`：仍是原来那 7 格，没多没少。
- `disable-check`、`allow-bash-scope-write`、`scope-refuses-undeterminable` 下，新加的 5 格「仍记」都红了（5 格里红 5 格）。这 5 格照文件头原有的写法另记了一行，没算进那几条的「共 N 格」。
- 失败行全文在 `draft/detector-switch-<值>.out`。

## 要主 agent 跑的命令

改了两个登记为 light 的钩子的自证与开关，复跑那一格是门禁 code-tooling 的 break-switch-replay（handback-guard.sh 在 `.claude/gate.d/break-switch-replay-files.tsv` 里；bash-command-detector.sh 不在复跑表，在 `break-switch-headers-exempt.tsv`）。
另外 agent-write-scope 格判 `.claude/hooks/` 下不重定义 `lib_shell_words.py` / `lib_heavy_tests.py` 的函数名，这一批新加的函数名我用 `comm` 对过，只有原有的 `selftest` 同名。工作目录是仓根，提交前一并跑：

```
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 break-switch-replay、agent-write-scope 两格; 范围=2 格; 为什么=第九批改了交回闸与检出钩子的自证和弄坏开关、两份都 import 共用模块' bash .claude/gate.d/code-tooling.sh --check break-switch-replay,agent-write-scope
```

看哪几行：
- break-switch-replay 那一格的 ✓ 行报「复跑了 N 条开关」，handback-guard.sh 的 16 条都在里面，没有「点名的项没在失败行里」；
- agent-write-scope 那一格 ✓、报函数数。

两个钩子的自证，这一批已在主工作区跑绿（见上）；要复核就再跑一遍：

```
bash .claude/hooks/handback-guard.sh --selftest       # 末行「✓ handback-guard 自检通过（查了 45 种）」
bash .claude/hooks/bash-command-detector.sh --selftest   # 「✓ 自检通过（查了 750 种）」与临时目录三格
```

## 射程外与留给别人的

- ⑥ 认不出的写法：
  - Bash 里写 `.rs`（`mv`、`cp`、重定向）不算「改了 .rs」；
  - 改完又被删掉的 git worktree 副本，因为看不到 `.git`，会按 rsync 副本不判；
  - 根写成别的命令替换（`$(git rev-parse --show-toplevel)` 这类）认不出，按没在那棵树上跑算，会误拒，出路是照拒绝说明里给的命令原样再跑一次。
- ⑰ 只认不带 `-t` 的模板。`-p <目录>`、`--tmpdir=<目录>` 指到草稿目录的照旧报；`$(cd $(mktemp -d …) && pwd -P)` 这种套在别的命令替换里的也照旧报（审计检出里有 3 条是这个形状）。
- 共用约束 `.claude/agent-common.md`「执行前拒绝的写法」里交回正文闸那一条，只列了原来四样。这一批照派发不碰共用约束，补两样（⑤ 命名检查、⑥ 写回员目标）的文字交主 agent 或下一批改定义的工具员。
- `.claude/hooks/lib_writer_lock.py` 文件头写「两处用这一份」，现在检出钩子是第三处，只读用 `find_lock_directory`、`read_lock`、`read_claim`、`launch_receipt`。那份库别的钩子也在用，不算「只被这两个钩子用」，这一批没改它的文件头。它的弄坏开关对检出钩子哪几格有影响也没逐条核。
- 拆分提案第九十节 ⑥、⑦、⑰ 三行的「去向」列没改（`records/` 不碰）。

## 没做什么

- 没跑重型测试：checker-tier-crash-point-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay、checker-tier-broad-mutation、`gate.sh`、全量 `cargo test`、`check.sh` 都没跑；
- 没跑门禁阶段与各类 lint（gate-overlap 只跑了 `--list` 查已有的钩子），break-switch-replay 与 agent-write-scope 两格交主 agent；
- 没编译，也没跑命名检查（这一批没改 `.rs`）；
- 没走定义三方（没改定义、共用约束与规则）；
- 没提交，没做任何 git 写操作。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/landing-audit-b9-handback-guard/` 里留着：
  - `draft/`：改前改后的钩子、草稿正文、换钩子规格、喂法、自证与开关的输出；
  - `fixtures/`：样本会话记录与输入；
  - `tools/`：四个辅助脚本；
  - `hook-swap/`：喂法日志与反向规格。
  
  没有仓副本与编译目录：hook-swap 的草稿树它自己删了，⑰ 样本用的两个带锁子目录也删了。
