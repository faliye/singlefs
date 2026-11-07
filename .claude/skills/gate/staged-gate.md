# 提交前的整轮门禁与分诊：主 agent 照这一份跑

子 agent 不交整轮门禁（`.claude/agent-common.md`「子 agent 不交任务」）。跑之前先读：`.claude/singlefs-ai-sop/skills/gate/SKILL.md`（阶段含义、常见假失败）；`.claude/singlefs-ai-sop/rules/session-wrapup.md`「4. 同一个仓里有没有别的会话在飞？」。

## 跑之前定下来

- 这一轮的改动已经按 `scripts/gate/stage-mine.py` 暂存了没有（没暂存就不跑，或者明定「跑全量工作区」）。
- 这一次是提交时跑还是用户要求时跑（只在这两种时候跑；交给服务的命令不带 `SINGLEFS_HEAVY_TESTS` 前缀）。
- 这一轮暂存区 diff 的范围：`git diff --cached --stat` 原样。跑全量工作区时，暂存区多半是空的，这时另列「这一轮改过的文件清单」，归属按清单判。
- 草稿目录。

## 做什么

1. 另有别的 `gate.sh` 在跑就等它结束。
   1b. 自己单跑的编译出来的代码（`cargo test`、测试二进制）、`gate.sh` 与单跑的门禁阶段都经 `task-server/task submit -- <命令>` 交、再用 run_in_background 起 `task-server/task wait <号>`；命令里不套内存包装、`nice`、`capped.sh`，不写上限、线程与挑台参数（服务当场拒）；内存上限、机器、卡与排队由服务定；结局读 `task wait` 的退出码与末行 `TASK_SERVER_RESULT`，整份输出 `task-server/task log <号> --all` 写进草稿目录，跑完 `task-server/task clean <号>`。
2. 只在提交时（或用户要求时）跑：暂存过的跑 `SINGLEFS_RUN_SCOPE='种类=whole; 跑什么=提交前的整轮门禁（gate-staged.sh 起整轮 gate.sh --staged）; 范围=整轮; 为什么=<为什么这么跑、这个范围为什么合适>' task-server/task submit -- bash scripts/gate/gate-staged.sh`（用户要求时 `=user-request`；它跑 `gate.sh --staged`，整轮全绿才前移 `refs/sop/staged-green`，它只作「上次整轮全绿的树」的记录、复用不读它；六道各自按每条判定的单元键查同一份判定库里自己那一族的判定）；跑全量工作区的跑 `SINGLEFS_RUN_SCOPE='种类=whole; 跑什么=整轮门禁 gate.sh; 范围=整轮; 为什么=<为什么这么跑、这个范围为什么合适>' task-server/task submit -- bash .claude/scripts/gate.sh` 不带参数（前缀照带）。`crash-point-replay`、qemu-device-streams、lkmm、crates-mutation-replay、formal-verification、broad-mutation 这几道重阶段在 `gate.sh` 里照各自的复用判定走（`crash-point-replay` 跑快档并逐块核崩溃判定库，没罩住的块报「本次未跑」不判红，`.claude/rules/verification.md`），整轮里不另外直接调它们（提交前单跑它们照 `.claude/skills/crash-test/commit-stages.md`），也不跑全量 `cargo test`（`.claude/hooks/project-code-guard.sh` 拒）；单跑某一道或某一格（`--check <格名>`）也带同一个前缀与范围声明（`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 <文件> 点名的格 <格名>; 范围=<N> 格; 为什么=<为什么这么跑、这个范围为什么合适>'`），同样经服务交。都用 run_in_background 起 `task-server/task wait <号>` 等、结束后读输出。
3. 每个红阶段：抄门禁给的「✗」行与紧跟的「→」下一步原样；看它点名的文件与行在不在暂存区 diff 的块里（`git diff --cached -U0 -- <文件>`）。在块里 ⇒「这一轮」；文件在但行不在块里、或文件不在暂存区 ⇒「不是这一轮」；红句说的是「这一批输入」本身的（`crash-point-replay` 的「有红块」「拒算」这类；没罩住的块它报「本次未跑」、不判红）不按它点名的库路径判：这一轮的改动（`git diff --cached --name-only`；跑全量工作区时是另列的文件清单）碰了 `.claude/gate.d/stage-inputs.tsv` 里那一道那一行登记的路径、或碰了 `crash-point-replay` 脚本本身（`.claude/gate.d/crash-point-replay.sh`；块的单元键含工具链） ⇒「这一轮」，下一步原样抄 `crash-point-replay` 出路里的三行命令，一条都没碰 ⇒「不是这一轮」；服务判出的结局（`task wait` 末行 `TASK_SERVER_RESULT` 的 `type=` 是 `memory-cap-at-maximum`、`never-fits`、`card-fault`、`launcher-lost`、`unit-start-failed`、`peer-run-failed` 这类，不是 `succeeded`、`command-failed`、`not-run`）不按阶段点名的文件判：那一趟没判出结果，判「环境」，原样抄末行与服务给的出路，照它改命令重新交；`card-fault` 照 `.claude/rules/verification.md` 弹窗请用户重置那张卡；formal-verification 的 Verus 超时，✗ 行写「超过 <N> 秒没出结局，按红记」，不归「环境」：按超时判红，照它点名的文件与行归属；工具没装、网关不通、`cargo` 不在 ⇒「环境」，用 `bash .claude/scripts/env.sh` 的对应行佐证；别的会话同时在跑同一个重阶段、进程被外部信号杀掉（输出里一行裸的 `Terminated`）⇒「并发」，贴开跑时与出事前后 `ps` 看到的同名进程。红阶段没有「✗」与「→」可抄的（被杀、崩溃），抄它最后 20 行输出，判「分不清」或「并发」，写明没有下一步可抄。分不清的写「分不清」与原因。
4. 原样抄未实现清单与「由项目本地阶段覆盖」清单。别的会话同时在改仓、跑全量工作区时「工作区跑的过程中没变」那一条红了：照抄开跑、收尾两个指纹，判「不是这一轮」。`gate.sh` 不打印单个阶段的耗时，取不到的不估。

## 写范围

- 草稿目录。门禁自己的 git 写（共享 `gate.sh` 不带 `--staged` 全绿时 `update-ref refs/sop/gate-ok`（带 `--staged` 时不写）、`--staged` 时的临时 worktree、`gate-staged.sh` 整轮全绿时 `update-ref refs/sop/staged-green`）是门禁本身的行为，允许；分诊这一步不另做 git 写。

## 产出

- 一张表：阶段 / 绿·红·77 / 归属（这一轮 · 不是这一轮 · 环境 · 并发 · 分不清）/ 门禁原样的 ✗ 与 → ；然后未实现清单原样。

## 判读边界

- 红了不在这里修，修照 `.claude/main-agent.md`「测试或门禁结果与预期不符」那一行派 `investigator`；归属按暂存区 diff 的块判，块外的红即使与这一轮有关也判不出来，照写。
