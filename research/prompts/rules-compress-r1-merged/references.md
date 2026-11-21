# 规则压缩合稿：全仓引用处的改法清单

合稿树 `merged/` 换上之后，合稿树之外的活文件里按旧名引被并掉、被改名的文件与节、引已挪进技能与参考件的细则的每一处。行号是仓里此刻的（`git grep -n --untracked` 现取，量的；原样输出在本目录 `refs-*.txt`）。合稿树里的 51 份换上时整份替换，它们里面的旧引用已在合稿里改掉，不再逐条列（`refs-merged-files-raw.txt` 里落在合稿树文件上的 27 处）。`.claude/kb/` 下的归 `kb-scribe`，`crates/` 下的只改注释、照 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」只改注释的不走代码轮，其余归 `tooling-writer`（`.claude/agent-common.md`「不做」一节写范围那一条）。

## 一、被并掉的 8 份规则：活文件里的引用（66 处）

并法：`verify-before-claiming.md`、`artifact-line-pointer.md` 并进 `evidence-discipline.md`；`exhaustive-interleaving.md` 并进 `fs-design.md`；`service-stop-readback.md` 并进 `command-safety-by-review.md`；`communication-discipline-by-review.md` 并进 `writing-discipline-by-review.md`；`engineering-philosophy.md`、`design-doc-discipline-by-review.md`、`pushback-discipline-by-review.md` 并进 `machine-first.md`。原来只写文件名不写节的，改成并进去的那一节；写了节的，节名照合稿里的节标题（照抄原节名的，新文件里同名）。

| 文件:行 | 现在的写法 | 改成什么 |
|---|---|---|
| `.claude/gate.d/doc-text.sh:2914` | verify-before-claiming.md | `evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `.claude/hooks/ask-user-claim-guard.sh:42` | verify-before-claiming.md | `evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `.claude/kb/checks-owed.en.md:55` | .claude/rules/verify-before-claiming.md | `.claude/rules/evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `.claude/kb/checks-owed.ja.md:55` | .claude/rules/verify-before-claiming.md | `.claude/rules/evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `.claude/kb/checks-owed.md:55` | .claude/rules/verify-before-claiming.md | `.claude/rules/evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `.claude/kb/decisions-history.en.md:504` | .claude/rules/verify-before-claiming.md | `.claude/rules/evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `.claude/kb/decisions-history.en.md:3528` | rules/verify-before-claiming.md | `.claude/rules/evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `.claude/kb/decisions-history.en.md:13489` | .claude/rules/verify-before-claiming.md | `.claude/rules/evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `.claude/kb/decisions-history.en.md:15132` | .claude/rules/verify-before-claiming.md | `.claude/rules/evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `.claude/kb/decisions-history.ja.md:504` | .claude/rules/verify-before-claiming.md「狭いほうの文を確認したのに、口にしたのは広いほうの文」 | `.claude/rules/evidence-discipline.md`「狭いほうの文を確認したのに、口にしたのは広いほうの文」（节名是译文或合稿目标里没有的写法：文件名照改，节名由改它的人对着目标那份的节标题定） |
| `.claude/kb/decisions-history.ja.md:3528` | rules/verify-before-claiming.md | `.claude/rules/evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `.claude/kb/decisions-history.ja.md:13489` | .claude/rules/verify-before-claiming.md「它定了没和它到底怎么定义的是两个问题」 | `.claude/rules/evidence-discipline.md`「它定了没和它到底怎么定义的是两个问题」 |
| `.claude/kb/decisions-history.ja.md:15132` | .claude/rules/verify-before-claiming.md | `.claude/rules/evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `.claude/kb/decisions-history.md:504` | .claude/rules/verify-before-claiming.md「核了窄的那一句，说出口的却是宽的那一句」 | `.claude/rules/evidence-discipline.md`「核了窄的那一句，说出口的却是宽的那一句」 |
| `.claude/kb/decisions-history.md:3528` | rules/verify-before-claiming.md | `.claude/rules/evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `.claude/kb/decisions-history.md:13489` | .claude/rules/verify-before-claiming.md「它定了没和它到底怎么定义的是两个问题」 | `.claude/rules/evidence-discipline.md`「它定了没和它到底怎么定义的是两个问题」 |
| `.claude/kb/decisions-history.md:15132` | .claude/rules/verify-before-claiming.md | `.claude/rules/evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `.claude/kb/experiments-history.en.md:1189` | .claude/rules/verify-before-claiming.md | `.claude/rules/evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `.claude/kb/experiments-history.en.md:2438` | .claude/rules/verify-before-claiming.md | `.claude/rules/evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `.claude/kb/experiments-history.ja.md:1189` | .claude/rules/verify-before-claiming.md「それが決まったかと、それが実際どう定義されているかは二つの問題」 | `.claude/rules/evidence-discipline.md`「それが決まったかと、それが実際どう定義されているかは二つの問題」（节名是译文或合稿目标里没有的写法：文件名照改，节名由改它的人对着目标那份的节标题定） |
| `.claude/kb/experiments-history.ja.md:2438` | .claude/rules/verify-before-claiming.md「それが決まったかと、それが実際どう定義されているかは二つの問題」 | `.claude/rules/evidence-discipline.md`「それが決まったかと、それが実際どう定義されているかは二つの問題」（节名是译文或合稿目标里没有的写法：文件名照改，节名由改它的人对着目标那份的节标题定） |
| `.claude/kb/experiments-history.md:1189` | .claude/rules/verify-before-claiming.md「它定了没和它到底怎么定义的是两个问题」 | `.claude/rules/evidence-discipline.md`「它定了没和它到底怎么定义的是两个问题」 |
| `.claude/kb/experiments-history.md:2438` | .claude/rules/verify-before-claiming.md「它定了没和它到底怎么定义的是两个问题」 | `.claude/rules/evidence-discipline.md`「它定了没和它到底怎么定义的是两个问题」 |
| `.claude/kb/experiments/26-3\343\201\244\343\201\256\345\217\260\345\270\263\343\203\242\343\203\207\343\203\253\343\201\256\346\244\234\347\264\242\343\202\263\343\202\271\343\203\210\343\201\250\347\262\276\345\272\246.ja.md:78` | rules/verify-before-claiming.md | `.claude/rules/evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `.claude/kb/experiments/26-lookup-cost-and-accuracy-of-three-accounting-models.en.md:78` | rules/verify-before-claiming.md | `.claude/rules/evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `.claude/kb/experiments/26-\344\270\211\347\247\215\350\256\260\350\264\246\346\250\241\345\236\213\347\232\204\346\237\245\346\211\276\344\273\243\344\273\267\344\270\216\347\262\276\345\272\246.md:78` | rules/verify-before-claiming.md | `.claude/rules/evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `.claude/kb/experiments/59-can-buffered-messages-recompute-solely-from-units.en.md:13` | verify-before-claiming.md | `evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `.claude/kb/experiments/59-\343\203\220\343\203\203\343\203\225\343\202\241\343\203\241\343\203\203\343\202\273\343\203\274\343\202\270\343\201\257\343\203\246\343\203\213\343\203\203\343\203\210\343\201\256\343\201\277\343\201\213\343\202\211\345\206\215\350\250\210\347\256\227\345\217\257\350\203\275\343\201\213.ja.md:13` | verify-before-claiming.md | `evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `.claude/kb/experiments/59-\347\274\223\345\206\262\346\266\210\346\201\257\350\203\275\344\270\215\350\203\275\345\217\252\344\273\216\345\215\225\345\205\203\351\207\215\347\256\227.md:13` | verify-before-claiming.md | `evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `.claude/kb/experiments/69-incremental-maintenance-cost-of-reverse-index-taking-authoritative-state.en.md:21` | .claude/rules/verify-before-claiming.md | `.claude/rules/evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `.claude/kb/experiments/69-\345\217\215\345\220\221\347\264\242\345\274\225\345\217\226\346\235\203\345\250\201\346\200\201\347\232\204\345\242\236\351\207\217\347\273\264\346\212\244\344\273\243\344\273\267.md:21` | .claude/rules/verify-before-claiming.md | `.claude/rules/evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `.claude/kb/experiments/69-\351\200\206\343\202\244\343\203\263\343\203\207\343\203\203\343\202\257\343\202\271\343\201\214\346\250\251\345\250\201\347\212\266\346\205\213\343\202\222\345\217\226\343\202\213\345\242\227\345\210\206\344\277\235\345\256\210\343\202\263\343\202\271\343\203\210.ja.md:21` | .claude/rules/verify-before-claiming.md | `.claude/rules/evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `.claude/kb/pending-rules.en.md:62` | verify-before-claiming.md「截断过的输出也是窄的那一句」 | `evidence-discipline.md`「截断过的输出也是窄的那一句」 |
| `.claude/kb/pending-rules.en.md:78` | verify-before-claiming.md | `evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `.claude/kb/pending-rules.ja.md:62` | verify-before-claiming.md「截断过的输出也是窄的那一句」 | `evidence-discipline.md`「截断过的输出也是窄的那一句」 |
| `.claude/kb/pending-rules.ja.md:78` | verify-before-claiming.md「切り詰めた出力も狭い主張である」 | `evidence-discipline.md`「切り詰めた出力も狭い主張である」（节名是译文或合稿目标里没有的写法：文件名照改，节名由改它的人对着目标那份的节标题定） |
| `.claude/kb/pending-rules.md:48` | verify-before-claiming.md「截断过的输出也是窄的那一句」 | `evidence-discipline.md`「截断过的输出也是窄的那一句」 |
| `prune-server/control:161` | .claude/rules/service-stop-readback.md | `.claude/rules/command-safety-by-review.md`「停 systemd 服务之后，回读输出判停没停」 |
| `prune-server/control:268` | .claude/rules/service-stop-readback.md | `.claude/rules/command-safety-by-review.md`「停 systemd 服务之后，回读输出判停没停」 |
| `scripts/devices/e125-zoned-wp-probe.sh:4` | verify-before-claiming.md「把定义句原样贴进注释」 | `evidence-discipline.md`「它定了没，和它到底怎么定义的，是两个问题」 |
| `scripts/devices/e72-devtable-probe.sh:4` | verify-before-claiming.md「把定义句原样贴进注释」 | `evidence-discipline.md`「它定了没，和它到底怎么定义的，是两个问题」 |
| `scripts/experiment-apparatus/e104_current_version.rs:8` | verify-before-claiming.md | `evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `scripts/experiment-apparatus/e112_old_writer_unknown_tree.rs:9` | verify-before-claiming.md | `evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `scripts/experiment-apparatus/e113_unknown_tree_full_arms.rs:3` | verify-before-claiming.md | `evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `scripts/experiment-apparatus/e26_accounting.rs:119` | rules/verify-before-claiming.md | `.claude/rules/evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `scripts/experiment-apparatus/e27_snapshot_accounting_risk_paths.rs:8` | rules/verify-before-claiming.md | `.claude/rules/evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `scripts/experiment-apparatus/e34_ring_iomin.rs:3` | verify-before-claiming.md「把定义句原样贴进实验注释」 | `evidence-discipline.md`「它定了没，和它到底怎么定义的，是两个问题」 |
| `scripts/experiment-apparatus/e59_message_recompute.rs:11` | verify-before-claiming.md | `evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `scripts/experiment-apparatus/e60_rebalance.rs:6` | verify-before-claiming.md | `evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `scripts/experiment-apparatus/e71_accounting_keys.rs:3` | verify-before-claiming.md「把定义句原样贴进实验注释」 | `evidence-discipline.md`「它定了没，和它到底怎么定义的，是两个问题」 |
| `scripts/experiment-apparatus/e73_key_range.rs:3` | verify-before-claiming.md「把定义句原样贴进实验注释」 | `evidence-discipline.md`「它定了没，和它到底怎么定义的，是两个问题」 |
| `scripts/experiment-apparatus/e74_allocation_records.rs:3` | verify-before-claiming.md「把定义句原样贴进实验注释」 | `evidence-discipline.md`「它定了没，和它到底怎么定义的，是两个问题」 |
| `scripts/experiment-apparatus/e75_record_size.rs:3` | verify-before-claiming.md「把定义句原样贴进实验注释」 | `evidence-discipline.md`「它定了没，和它到底怎么定义的，是两个问题」 |
| `scripts/experiment-apparatus/e76_payload_checksum.rs:3` | verify-before-claiming.md「把定义句原样贴进实验注释」 | `evidence-discipline.md`「它定了没，和它到底怎么定义的，是两个问题」 |
| `scripts/experiment-apparatus/e77_publish_order.rs:16` | verify-before-claiming.md | `evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `scripts/experiment-apparatus/e78_replay_start.rs:14` | verify-before-claiming.md | `evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `scripts/experiment-apparatus/e79_root_record.rs:17` | verify-before-claiming.md | `evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `scripts/experiment-apparatus/e80_partial_stripe.rs:19` | verify-before-claiming.md | `evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `scripts/experiment-apparatus/e82_admission_overlay.rs:12` | verify-before-claiming.md | `evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `scripts/experiment-apparatus/e83_tombstone_grain.rs:10` | verify-before-claiming.md | `evidence-discipline.md`「陈述外部状态之前，现查一次」 |
| `scripts/experiment-apparatus/e97_entry_encoding.rs:3` | verify-before-claiming.md「把定义句原样贴进实验注释」 | `evidence-discipline.md`「它定了没，和它到底怎么定义的，是两个问题」 |
| `scripts/experiment-apparatus/e98_inode_record.rs:3` | verify-before-claiming.md「把定义句原样贴进实验注释」 | `evidence-discipline.md`「它定了没，和它到底怎么定义的，是两个问题」 |
| `scripts/hosts/peer-host-lib.sh:645` | .claude/rules/service-stop-readback.md | `.claude/rules/command-safety-by-review.md`「停 systemd 服务之后，回读输出判停没停」 |
| `scripts/hosts/run-with-memory-cap.sh:64` | .claude/rules/service-stop-readback.md | `.claude/rules/command-safety-by-review.md`「停 systemd 服务之后，回读输出判停没停」 |
| `task-server/control:44` | .claude/rules/service-stop-readback.md「怎么做」 | `.claude/rules/command-safety-by-review.md`「停 systemd 服务之后，回读输出判停没停」 |
| `task-server/control:440` | .claude/rules/service-stop-readback.md「怎么做」 | `.claude/rules/command-safety-by-review.md`「停 systemd 服务之后，回读输出判停没停」 |

## 二、改了名的节与条目

### `.claude/rules/command-safety-by-review.md`「装置起的样本进程，放进它自己的进程组」改名「装置起的样本进程，逐个按进程号停」（用户定案第 6 条）

按旧节名引的 13 处，节名照改；这几份代码自己对进程组发信号的，做法另见「四」：

| 文件:行 | 现在的写法 | 改成什么 |
|---|---|---|
| `.claude/gate.d/code-tooling.sh:9441` | `.claude/rules/command-safety-by-review.md「装置起的样本进程，放进它自己的进程组」` | `.claude/rules/command-safety-by-review.md「装置起的样本进程，逐个按进程号停」`，做法见「四」 |
| `.claude/gate.d/code-tooling.sh:10399` | 同上 | 同上 |
| `.claude/gate.d/code-tooling.sh:11259` | 同上（拒绝说明里打给人看的） | 同上 |
| `.claude/gate.d/code-tooling.sh:12196` | 同上（拒绝说明里打给人看的） | 同上 |
| `.claude/gate.d/doc-registries.sh:5078` | `command-safety.md「装置起的样本进程，放进它自己的进程组」`（文件名本就写错，项目那份是 `command-safety-by-review.md`） | `.claude/rules/command-safety-by-review.md「装置起的样本进程，逐个按进程号停」`，做法见「四」 |
| `.claude/gate.d/lib/stage-cells.sh:49` | 同 `code-tooling.sh:9441` | 同上，做法见「四」 |
| `.claude/hooks/bash-command-detector.sh:184` | `上游 .claude/rules/command-safety-by-review.md「装置起的样本进程，放进它自己的进程组」` | 节名照改、删「上游」二字（这份规则在项目 `.claude/rules/`）；这一道放行「对自己用新会话起的子进程的进程组发信号」，与新判据相反，见「四」 |
| `.claude/hooks/bash-command-detector.sh:4108` | 同上 | 同上 |
| `scripts/gate/gate-structure-check.py:24` | 「装置起的样本进程，放进它自己的进程组」 | 「装置起的样本进程，逐个按进程号停」，做法见「四」 |
| `scripts/gate/lib_process_group.py:7` | `.claude/rules/command-safety-by-review.md「装置起的样本进程，放进它自己的进程组」` | 同上 |
| `scripts/kb/move-directory-in-use-check.py:341` | `command-safety.md「装置起的样本进程，放进它自己的进程组」` | `.claude/rules/command-safety-by-review.md「装置起的样本进程，逐个按进程号停」`，做法见「四」 |
| `task-server/task_server_lib.py:4529` | `.claude/rules/command-safety-by-review.md「装置起的样本进程，放进它自己的进程组」` | 同上 |
| `records/` 下 2 份 4 行 | — | 只计数，不改 |

### 共用约束的条目

| 文件:行 | 现在的写法 | 改成什么 |
|---|---|---|
| `.claude/hooks/handback-guard.sh:741` | `.claude/agent-common.md「不做」一节停自己起的一条链那一条` | `.claude/agent-common.md「不做」一节「停任务与停进程」` |
| `.claude/hooks/handback-guard.sh:44-46` | 文件头写「共用约束写明 rsync 拷的副本 <根> 填仓根」，原件与合稿里都没有这一句 | 照合稿 `.claude/agent-common.md`「命名检查交主 agent 跑」：实现员一律在副本里改、交补丁，`<根>` 写 `<快照>`；钩子判法要不要跟着改交主 agent（「矛盾总表」共用-10） |

### 此刻就指不到的旧引用（与这一件无关，前面的写手已报）

| 文件:行 | 现在的写法 | 改成什么 |
|---|---|---|
| `.claude/hooks/bash-command-detector.sh:76` | `main-agent.md`「调用里不再加 `&`、`nohup`、`disown`」（原件与合稿里都是「调用里不加」） | `.claude/skills/dispatch/SKILL.md`「派出去之后」，原话「调用里不加 `&`、`nohup`、`disown`」 |
| `.claude/hooks/session-start.sh:94` | `CLAUDE.md「派出去之后要盯住」`（`CLAUDE.md` 里没有这一节） | `.claude/skills/dispatch/SKILL.md`「派出去之后」 |

### 技能与规则里删掉、改名的标题

- `.claude/rules/machine-first.md` 一级标题「机器优先：先分清「可读」与「可验证」」改成「机器优先：实现上 AI 友好，审核上人类友好」：合稿树之外按标题引它的 0 处（`git grep -n '机器优先：先分清' -- . ':!research' ':!records'` 只命中原件自己那一行，量的）。
- `.claude/rules/machine-first.md` 删掉的五个标题、`.claude/rules/fs-design.md`「一个已经成立的实例，和一个不成立的」、`.claude/rules/test-discipline.md`「自证的每一格单跑与整份跑判定相同，失败句不冒充别的格」、`.claude/skills/remind/SKILL.md`「只许这一个」：按需稿写手逐个 `git grep` 过，只命中它们自己那一行（`rules-compress-ondemand/result.md`「删掉的标题在全仓的引用」）。
- `.claude/rules/implementation-workflow.md`「代码轮正文的检查单」那张八格表挪进 `.claude/skills/three-way/SKILL.md`「代码轮正文的检查单」，常驻留一行格名：`.claude/gate.d/doc-process-records.sh:626`「八个格名与 … 那张表逐字相同」、`:2825`「八个格名逐字照那张表」改成「那一行格名」或指技能；门禁的判法不读规则文件（格名硬写在 `CODE_ROUND_CHECKLIST_CELLS`），不改也不红。`.claude/gate.d/stage-owners.tsv:141`「缺的格由主 agent 照 … 补」加指技能同名节。

## 三、挪进技能与参考件的细则：按旧节名引、最好改指的（55 行）

这些引用的节名在合稿常驻稿里都还在（常驻稿留了骨架与指到技能、参考件的一行），换上之后门禁 doc-registries 的 governance-refs 格不会因此红；改了是让读的人一步到细则。行与建议照入口第二遍、规则第二遍两份报告的「⑥」表，行号由合稿这一件逐处回原文件现查过仍是那一行（`moved-pointer-check.txt`，量的）。第四列标出引用处在不在合稿树里。

| 文件:行 | 引的是 | 建议改指 | 引用处在哪 |
|---|---|---|---|
| `.claude/agent-common.md:68` | 合入后验证「跑法照 `.claude/main-agent.md`「写与验分两段」验证段第 1 步」 | `.claude/skills/merge-verify/SKILL.md`「写与验分两段」验证段第 1 步（整条命令在那里） | 合稿树里的（节名在，照旧可用；要一步到细则的照右栏改） |
| `.claude/agents/implementation-writer.md:15` | 「写与验分两段」写段第 2 条、验证段第 2、5 步（副本从段树或快照建） | 写段第 2 条照旧指常驻；验证段第 2、5 步改指 merge-verify 同号（快照当基线的写法在那里） | 合稿树里的（节名在，照旧可用；要一步到细则的照右栏改） |
| `.claude/agents/implementation-writer.md:34` | 「写与验分两段」验证段第 2 步（变异行一律全域） | merge-verify 验证段第 2 步 | 合稿树里的（节名在，照旧可用；要一步到细则的照右栏改） |
| `.claude/agents/implementation-writer.md:40` | 「写与验分两段」验证段第 1 步退 3 那一支 | merge-verify 验证段第 1 步「退 3」那一支 | 合稿树里的（节名在，照旧可用；要一步到细则的照右栏改） |
| `scripts/gate/merge-and-verify.sh:38` | 验证段第 1 步（不带 `--skip-apply` 的一律带 `MERGE_AND_VERIFY_KEEP_COPY=1`） | merge-verify 验证段第 1 步 | 合稿树外 |
| `scripts/gate/merge-and-verify.sh:100` | 验证段第 1 步退 3 那一支 | merge-verify 验证段第 1 步「退 3」那一支 | 合稿树外 |
| `scripts/gate/merge-and-verify.sh:158` | 同上 | 同上 | 合稿树外 |
| `scripts/gate/merge-and-verify.sh:236` | 验证段第 1 步（什么时候跑） | 跑的时机在常驻验证段第 1 步，照旧可以；命令在 merge-verify | 合稿树外 |
| `scripts/gate/merge-and-verify.sh:2725` | 验证段第 1 步退 3 那一支（打给人看的输出） | merge-verify 验证段第 1 步「退 3」那一支 | 合稿树外 |
| `scripts/gate/merge-and-verify.sh:2727` | 验证段第 2 步（打给人看的输出） | merge-verify 验证段第 2 步 | 合稿树外 |
| `scripts/gate/merge-and-verify.sh:196`、`:2476`、`:2649`、`:3319` | 「合入后验证之后判新出现的拒算」第 1 条（拒算补丁目录怎么重做） | merge-verify「合入后验证之后判新出现的拒算」第 1 条（a、b、baseline 三份那一套在那里） | 合稿树外 |
| `scripts/gate/merge-and-verify.sh:3323`、`:3325` | 自证 grep 它自己打出的同一句（`主 agent 从主工作区此刻重做这一份（.claude/main-agent.md「合入后验证之后判新出现的拒算」第 1 条`） | 改 `:2476` 的输出时这两行的 grep 串要一起改，不然自证红（推的：`:3323` 用 `grep -qF` 比的就是这串） | 合稿树外 |
| `.claude/hooks/runner-dispatch-guard.sh:1309` | 第 4 条（撞文件的排在交回之后、或并给同一个） | `.claude/skills/dispatch/SKILL.md`「一轮怎么开、怎么收」第 4 条 | 合稿树外 |
| `.claude/hooks/runner-dispatch-guard.sh:2253` | 「派发提示怎么写」（每份写一行「批次：」） | dispatch「派发提示怎么写」；「批内发现只进下一轮清单」在常驻第 1 条 | 合稿树外 |
| `.claude/hooks/runner-dispatch-guard.sh:2420` | 「派发提示怎么写」（关的是哪几条，常设要求 A5） | dispatch「派发提示怎么写」（那一节的写法）；判据句在常驻第 6 条 | 合稿树外 |
| `.claude/hooks/completion-cleanup-guard.sh:5`、`:1747` | 第 11 条 | 判据在常驻第 11 条，可照旧；完工清理闸怎么判、交付物怎么写在 dispatch 第 11 条 | 合稿树外 |
| `.claude/kb/milestone/03-third-txn.md:199` | 第 11 条（提醒用户跑 `sudo fstrim -v /`） | dispatch 第 11 条 | 合稿树外 |
| `.claude/kb/milestone/03-third-txn.md:238` | 第 11 条（清盘） | 常驻第 11 条可照旧；细则在 dispatch 第 11 条 | 合稿树外 |
| `.claude/kb/milestone/03-third-txn.md:190` | 派发表「一批实现员交回到齐、合入之前」一行（`stream-constants` 命令） | 判据照旧在常驻那一行；装置命令在 dispatch「什么时候派哪个 agent」同名条目 | 合稿树外 |
| `.claude/skills/decide/SKILL.md:12` | 派发表「定案之后写回 kb」那一行 | 规格检查、decision-items-sync 整条在 dispatch 同名条目 | 合稿树里的（节名在，照旧可用；要一步到细则的照右栏改） |
| `scripts/agents/agent-watch.py:49`、`:52`、`:498` | 「派出去之后」（钩子已拒的与判不了的只记不处置） | dispatch「派出去之后」（「读到告警的处置」第一项） | 合稿树外 |
| `scripts/agents/watch.sh:4` | 「派出去之后」（起看门狗） | dispatch「派出去之后」（起法）；什么时候起在常驻 | 合稿树外 |
| `scripts/experiment-apparatus/long-run-lib/binary-identity.sh:17` | 「一轮怎么开、怎么收」（编译目录由 task-server 给、不往仓根 `target/` 编） | dispatch 第 11 条 | 合稿树外 |
| `task-server/task_server_lib.py:2111` | 第 15 条（发给那个会话） | 判据在常驻第 15 条；消息写什么在 dispatch 第 15 条 | 合稿树外 |
| `.claude/agent-common.md:105` | three-way-inference.md「多轮…」里叫停轮登记行的写法 | `.claude/skills/three-way/SKILL.md`「用户叫停、不写判决的轮」 | 合稿树里的（节名在，照旧可用；要一步到细则的照右栏改） |
| `.claude/main-agent.md:27` | 同上（「出处怎么写在那一节」） | 同上 | 合稿树里的（节名在，照旧可用；要一步到细则的照右栏改） |
| `scripts/gate/apply-writer-patch.py:397` | 同上（轮名、日期、为什么撂下、去向与出处的写法） | 同上 | 合稿树外 |
| `.claude/main-agent.md:116` | three-way-inference.md「多轮…」一节里「核查与判决由主 agent 做」那一段 | `.claude/skills/three-way/SKILL.md`「核查与判决由主 agent 做，不派子 agent 去核子 agent」 | 合稿树里的（节名在，照旧可用；要一步到细则的照右栏改） |
| `.claude/agents/three-way-materials.md:16` | three-way-inference.md「引 kb 里的条目要整行抄…」整节（材料员照它写清单与附录） | 加读 `.claude/skills/three-way/SKILL.md`「引 kb 里的条目要整行抄，不许摘句——三处都管」 | 合稿树里的（节名在，照旧可用；要一步到细则的照右栏改） |
| `scripts/agents/agent-handover.py:7` | three-way-inference.md「中途停下、被停或报错的腿」那一段（交接摘要的用法） | `.claude/skills/three-way/SKILL.md`「云端腿的报告要分段落盘」 | 合稿树外 |
| `.claude/gate.d/doc-process-records.sh:158` | implementation-workflow.md 同一小节里「对不上的倒推」的写法 | `.claude/skills/three-way/SKILL.md`「代码轮派腿之前记一份开工快照」 | 合稿树外 |
| `scripts/devices/snapshot-check.sh:9` | implementation-workflow.md「代码轮派腿之前记一份开工快照」的 `sha256sum -c` 与倒推 | 判据还在常驻；倒推做法改指技能同名节 | 合稿树外 |
| `.claude/gate.d/doc-process-records.sh:626` | 「八个格名与 implementation-workflow.md「代码轮正文的检查单」那张表逐字相同」 | 「那张表」改成「那一行格名」，表本身在技能「代码轮正文的检查单」 | 合稿树外 |
| `.claude/gate.d/doc-process-records.sh:2825` | 出路句「八个格名逐字照那张表」 | 同上 | 合稿树外 |
| `.claude/gate.d/stage-owners.tsv:141` | 「缺的格由主 agent 照 implementation-workflow.md「代码轮正文的检查单」补」 | 加指技能「代码轮正文的检查单」（逐格写什么在那里） | 合稿树外 |
| `.claude/main-agent.md:60` | implementation-workflow.md「代码轮正文的检查单」逐格点名 | 加指技能同名节 | 合稿树里的（节名在，照旧可用；要一步到细则的照右栏改） |
| `.claude/agents/implementation-writer.md:47`、`scripts/gate/apply-writer-patch.py:61` | 「走不到的理由」一格腿怎么推（检查单表的第一行） | 技能「代码轮正文的检查单」 | 合稿树外 |
| `scripts/gate/admission.py:42` | implementation-workflow.md「复用门禁的判定…」文件级排除 | `.claude/rules/verification-reference.md`「复用门禁的判定：…」「单元键的组成与排除」 | 合稿树外 |
| `scripts/gate/lib_function_units.py:201` | 同上（crates 变异判法不在文件级排除的射程里） | 同上 | 合稿树外 |
| `.claude/kb/checks-owed.md:569` | 同上（边级排除要配三格自证）；kb，归 kb-scribe | 同上 | 合稿树外 |
| `.claude/agents/tooling-writer.md:16` | implementation-workflow.md「复用门禁的判定…」、verification.md「门禁的结构」（工具员照它写门禁与算键） | 加读 `.claude/rules/verification-reference.md` 的同名两节 | 合稿树里的（节名在，照旧可用；要一步到细则的照右栏改） |
| `.claude/gate.d/code-tooling.sh:3478` | verification.md「门禁的结构」里 stage-owners.tsv 四列 | `.claude/rules/verification-reference.md`「门禁的结构」 | 合稿树外 |
| `.claude/gate.d/code-tooling.sh:5471`、`scripts/gate/gate-structure-check.py:4`、`:322`、`scripts/gate/lib_stage_header.py:4`、`.claude/gate.d/lib/stage-cells.sh:12` | verification.md「门禁的结构」（参数表、格名表、gate-structure-check 判的几样） | 同上 | 合稿树外 |
| `.claude/gate.d/crash-point-replay.sh:240` | verification.md「门禁的结构」里样本根 `.gate-cells` | 同上 | 合稿树外 |
| `.claude/gate.d/formal-verification.sh:50`、`.claude/gate.d/code-tooling.sh:12461` | verification.md「门禁的结构」里 `--list-items` / `--item` 的写法 | 同上（常驻稿只留「按项判的门禁另认这两个参数」） | 合稿树外 |
| `.claude/gate.d/code-source-discipline.sh:53` | verification.md「门禁的结构」「门禁管哪一半」 | `.claude/rules/verification-reference.md`「门禁管哪一半」 | 合稿树外 |
| `scripts/crash/crash-case-check.py:25`、`:617`、`:620` | verification.md「门禁管哪一半」崩溃枚举用例住在验证包那一行 | 同上 | 合稿树外 |
| `.claude/agents/implementation-writer.md:16` | verification.md「定义与名字」…（括注「测试文件第一行声明模块」：原件只在「门禁管哪一半」那张表里） | 括注那一样改指 `.claude/rules/verification-reference.md`「门禁管哪一半」 | 合稿树里的（节名在，照旧可用；要一步到细则的照右栏改） |
| `task-server/task_server.py:219` | verification.md「一个门禁：两种跑法」里「控制测试线程的只有变异那一路」 | `.claude/rules/verification-reference.md`「一个门禁：两种跑法」「测试线程」 | 合稿树外 |
| `crates/singlefs-verification/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:87` | 同上（GPU 测试线程那一条） | 同上；`crates/` 下的注释改动照只改注释算 | 合稿树外 |
| `crates/singlefs-implementation-experiments/src/bin/e171_three_oracle_collusion/main.rs:26` | verification.md「一个门禁：两种跑法」按需领卡那一句 | `.claude/rules/verification-reference.md`「一个门禁：两种跑法」「凡用 GPU 先清场，跑完不复原」 | 合稿树外 |
| `.claude/kb/checks-owed.md:538` | verification.md「一个门禁：两种跑法」里 crash-case-replay 格的 cargo 命令；kb | `.claude/rules/verification-reference.md`「门禁 crash-point-replay 怎么跑」 | 合稿树外 |
| `.claude/kb/verification-build.md:317` | verification.md「一个门禁：两种跑法」的「缩域」一条「写法在」；kb | 判据还在常驻 **缩域**；写法改指 `.claude/rules/verification-reference.md`「缩域与崩溃域上限」 | 合稿树外 |
| `.claude/main-agent.md:12` | verification.md「判定库只有一份」：「判定库的内容、一族「接完」的三件、中转库都在那一节」 | 「判定库的内容」那一样（表与族的名字）改指 `.claude/rules/verification-reference.md`「判定库里的表」；接完与中转库还在常驻 | 合稿树里的（节名在，照旧可用；要一步到细则的照右栏改） |
| `.claude/skills/crash-test/commit-stages.md:3` | implementation-workflow.md「不指定范围的只在提交时跑…」里提交时跑的这六道门禁 | IW 那张表不再列六道，改指 `.claude/rules/verification.md`「一个门禁：两种跑法」提交时只走一条路那一条 | 合稿树里的（节名在，照旧可用；要一步到细则的照右栏改） |

## 四、做法要跟着改的代码处（不只是改名；归治理文件那一批，走三方）

### 用户定案第 6 条：脚本与装置里也不对进程组发信号

合稿里 `.claude/rules/command-safety-by-review.md`「装置起的样本进程，逐个按进程号停」、`.claude/agent-common.md`「样本进程逐个按进程号停」、`.claude/agents/tooling-writer.md` 第 5 步改成逐个按进程号停。仓里此刻对进程组发信号的写法（`git grep -n -E 'os\.killpg|kill -[A-Z0-9]+ -- -|kill -- -'`，排除样本目录，量的：110 行、27 份；原样在 `refs-group-signal-code.txt`），其中 `bash-command-detector.sh` 的 48 行多是它认的写法与自证样例。除这些写法外，`task-server/task_server_lib.py:4529`「停法」、`.claude/gate.d/lib/stage-cells.sh:11`、`:45`、`:395`（每格一个子 shell 放进它自己的进程组）、`scripts/gate/verus-proof-check.py:119`、`:1132`、`:1157`、`scripts/gate/refused-test-registry.py:130` 的文件头与文档串写的就是按组停的做法。

| 文件 | 命中行数 |
|---|---|
| `.claude/hooks/bash-command-detector.sh` | 48 |
| `scripts/mutation/crates-mutation-rows.py` | 7 |
| `.claude/gate.d/lib/stage-cells.sh` | 6 |
| `scripts/kb/move-directory-in-use-check.py` | 5 |
| `scripts/prune-server/prune_server_client_selftest.py` | 5 |
| `.claude/gate.d/code-tooling.sh` | 3 |
| `.claude/gate.d/doc-registries.sh` | 3 |
| `crates/singlefs-verification/tests/gpu_task_switch_diagnostic.py` | 3 |
| `scripts/experiment-apparatus/long_run_item_verdicts.py` | 3 |
| `scripts/experiment-apparatus/e171_three_oracle_collusion/row_driver.py` | 2 |
| `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/row_driver.py` | 2 |
| `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator.py` | 2 |
| `scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/current_tree_row_driver.py` | 2 |
| `scripts/experiment-apparatus/e207_strict_scan_steps_process_sampler.py` | 2 |
| `scripts/gate/gate-structure-check.py` | 2 |
| `scripts/gate/lib_process_group.py` | 2 |
| `scripts/mutation/broad-mutation.py` | 2 |
| `task-server/task_server_selftest.py` | 2 |
| `scripts/agents/tests/test_agents.py` | 1 |
| `scripts/crash/gpu-quiesce-run.sh` | 1 |
| `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/fake_task_server.py` | 1 |
| `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_segment_two.py` | 1 |
| `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_units_revision_8.py` | 1 |
| `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/shard_relay.py` | 1 |
| `scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/full_table_run.py` | 1 |
| `scripts/gate/refused-test-registry.py` | 1 |
| `scripts/gate/verus-proof-check.py` | 1 |

钩子一侧：`.claude/hooks/bash-command-detector.sh` 文件头 ⑥（:184）与 :4108 放行「对自己用新会话起的那一个子进程的进程组发信号」，新判据下这一放行要撤，连同它的自证样例（文件头 :498–:499 的「终止进程:…进程组」几格）。

### 用户定案第 9 条：RefFS 只许 `std`

`.claude/gate.d/checker-independence-and-sync.sh:32` ⑤「singlefs-reference-filesystem（RefFS）的传递闭包（dev-dependencies 也算）⊆ {它自己, singlefs-format}」收严成闭包只有它自己；收严之前合稿写「门禁那一格还许 `singlefs-format`，收严记在欠账」，欠账号要主 agent 登。

### 用户定案第 10 条：`quote-rust-items.py` 进可直接跑的名单

`.claude/hooks/lib_project_code_runs.py:1076` 起的 `TEXT_TOOLS_RUN_DIRECTLY` 加 `scripts/kb/quote-rust-items.py`（照 `:1089` 的 `quote-kb.py` 那一行写）；改钩子库走 `scripts/gate/hook-swap.py`。

### 用户定案第 3 条：并行度缺省一律取登记的线程数

规则里已改；脚本、门禁、钩子与 `crates/` 里按 `nproc`、`os.cpu_count()`、`available_parallelism` 取的写法 121 行、32 份（同一条 `git grep` 量的，含读真实核数做判断而不是定缺省的），存量登在只缩不涨的 `.claude/heavy-thread-exempt`（58 行），由门禁 code-tooling 的 heavy-thread-defaults 格判；哪几处是定缺省、要改，归治理文件那一批逐处判，这一件没逐处判。

## 五、门禁保留表要跟着改的行

| 文件:行 | 现在的写法 | 改成什么 |
|---|---|---|
| `.claude/gate.d/dead-path-kept.tsv:45` | `.claude/main-agent.md〈TAB〉a/.claude/gate.d/refused-tests.tsv〈TAB〉# …` | 第一列改成 `.claude/skills/merge-verify/SKILL.md`（拒算补丁目录那条 `git diff --no-index … a/… b/…` 挪进了技能；合稿里技能有这两个路径、`main-agent.md` 里没有，量的） |
| `.claude/gate.d/dead-path-kept.tsv:46` | `.claude/main-agent.md〈TAB〉b/.claude/gate.d/refused-tests.tsv〈TAB〉# …` | 同上 |

这张表里载体在合稿树里的只有这两行（逐行对合稿树核的，量的）；别的门禁表（`*.tsv`、`.claude/*-exempt`、`.claude/*-exclude` 与登记文件）里没有一行点名被并掉的 8 份文件（同一条 `git grep` 量的）。按内容哈希登记的豁免表 `.claude/agent-def-review-exempt` 里 `.claude/main-agent.md`、`.claude/agent-common.md` 的旧行哈希对不上新内容，照规矩不加新行。

## 六、`CLAUDE.md` 的 `@` 与 read-once 登记（合稿里已改）

- 删掉的 `@` 五行：`@.claude/rules/service-stop-readback.md`、`@.claude/rules/artifact-line-pointer.md`、`@.claude/rules/exhaustive-interleaving.md`、`@.claude/rules/verify-before-claiming.md`、`@.claude/rules/communication-discipline-by-review.md`（仓里 `CLAUDE.md:56`、`:57`、`:58`、`:60`、`:64`）。
- 「另六篇不 `@` 常驻」改成「另三篇」，read-once 那一行（仓里 `CLAUDE.md:71`）删掉 `engineering-philosophy.md`、`design-doc-discipline-by-review.md`、`pushback-discipline-by-review.md`。
- 新加一句讲参考件什么时候读，另起一行 `<!-- doc-lint:read-once verification-reference.md -->`：门禁 code-tooling 的 rules-manifest 格要 `.claude/rules/` 下每一份在 `@` 或 read-once 里（`.claude/gate.d/code-tooling.sh:3351` 起），参考件不 `@`，只能登进 read-once。三份新技能不登（技能目录自动被发现，那一格只管 `.claude/rules/`）。
- 合稿树上照那一格与 rules-named-by-definitions 格的判法现算（量的，判法照抄进 python 只读跑）：规则清单两向相等；没被任何定义按路径点名的规则从仓里此刻的 10 条降到 6 条（`communication-discipline.md`、`pushback-discipline.md` 两份上游的，与 `session-wrapup.md`、`writing-discipline-by-review.md`、`long-run-split.md`、`code-discipline-by-review.md`），是此刻那 10 条的子集；`verification-reference.md` 由 `.claude/agents/tooling-writer.md` 点名。

## 七、要 `git rm` 的文件（8 份，归主 agent）

- `.claude/rules/verify-before-claiming.md`
- `.claude/rules/artifact-line-pointer.md`
- `.claude/rules/exhaustive-interleaving.md`
- `.claude/rules/service-stop-readback.md`
- `.claude/rules/communication-discipline-by-review.md`
- `.claude/rules/engineering-philosophy.md`
- `.claude/rules/design-doc-discipline-by-review.md`
- `.claude/rules/pushback-discipline-by-review.md`

## 八、合稿树整份换上的文件（51 份）

按 `merged/` 下的相对路径逐份换到仓里同一路径；新建的 4 份：`.claude/rules/verification-reference.md`、`.claude/skills/dispatch/SKILL.md`、`.claude/skills/merge-verify/SKILL.md`、`.claude/skills/three-way/SKILL.md`。四份新建与 `.claude/main-agent.md` 要同一步换上（常驻稿点名它们，不在时门禁 governance-refs 判指不到）。`.claude/hooks/agent-write-scope.tsv` 里没有一类子 agent 能写 `.claude/skills/`（`grep -n skills` 无命中，入口第二遍写手量的），换上由主 agent 做，或先给工具员加一行。

## 九、`research/` 与 `records/` 下的旧名（只计数，不逐条列）

| 旧名 | `research/` | `records/` |
|---|---|---|
| 被并掉的 8 份文件名（`git grep -c`，命中行数） | 57 行、43 份 | 26 行、15 份 |
| 「装置起的样本进程，放进它自己的进程组」 | 0 | 4 行、2 份 |

照 `.claude/rules/path-moves.md`「怎么做」第 5 步判：说当时情形的留旧名；并掉不是搬迁，`.claude/gate.d/path-moves.tsv` 登不了（常驻稿写手报告「④」）。
