# 通用跨机脚本与多机配置改名：tooling-writer 报告（2026-09-28）

## 结论
- 条 1：新建 `research/scripts/multi-host-run.sh`，自证 13 格全对；12 个 `MULTI_HOST_RUN_BREAK` 弄坏开关加 1 个配置判法的弄坏开关，打开任一个都判红（原样行见下）。
- 条 2：改名做完。`git grep -n -F -e layer0-shard.env -e SINGLEFS_LAYER0_SHARD_CONFIG -- research/scripts .claude/gate.d` 从 35 处降到 0 处（grep 退 1）；未跟踪文件另用 `grep -rn` 搜，也是 0。
- 追加输入（双机与 GPU 开关）做完。例外是第 47 号 runner 表那一行，**没加上**：auto mode 分类器以「Modify Shared Resources」为由拒了，我没有换别的写法去绕，交主 agent 或用户处理。
- 推翻条件：真配置里写上 `ENABLE_ACROSS_MACHINES=1` 之后判法仍不退 0；或者在不设任何弄坏开关时，任一份自证报 ✗。

## 改过的文件
research/scripts/multi-host-run.sh（新建）、layer0-shard-configuration-check.sh、layer0-shard-run.sh（只改第 42 行注释）、layer0-shard-run-selftest.sh、mutation-shard-run.sh、peer-host-lib.sh（只改第 27 行出路句）、.claude/gate.d/54-layer0-replay.sh（第 28、29、577 行）。54 号在 fixtures 下的样本里没有旧名，没有改。
`git diff --stat`（工作区对暂存区，就是这一轮的改动）当时只显示 multi-host-run.sh 那一份（`193 insertions`）。其余几份在我动手之前已经暂存，我的改动是换 inode 写回的，也落在工作区，`git diff HEAD --stat` 原样如下：
```
 .claude/gate.d/54-layer0-replay.sh                 |  34 +-
 .../scripts/layer0-shard-configuration-check.sh    |  56 +-
 research/scripts/layer0-shard-run-selftest.sh      | 227 +++++++-
 research/scripts/layer0-shard-run.sh               | 252 ++++++--
 research/scripts/multi-host-run.sh                 | 539 +++++++++++++++++
 research/scripts/mutation-shard-run.sh             | 639 +++++++++++++++++++++
 research/scripts/peer-host-lib.sh                  | 213 +++++++
```
（HEAD 对比里含别的会话已暂存的改动，数字不全是这一轮的。）写法：配置判法开头 5 处用 Edit 改（当时 `ps` 看过没有进程在跑它）；其余都用 replace-once.py 或「同目录临时文件再 mv」。

## 条 2 判红与转绿
改名前 `bash research/scripts/layer0-shard-configuration-check.sh`：
```
  ✗ 双机分片不能用：没有配置文件 layer0-shard.env
exit=1
```
改名后（输出经 peer_host_redact 处理），真配置原样判：`双机：关（配置 …/multi-host.env 里 ENABLE_ACROSS_MACHINES 没写或不是 1；…），GPU：关`，**退 3**。原因：按追加输入，没写开关就算关。
给真配置的副本加一行 `ENABLE_ACROSS_MACHINES=1`（副本放在草稿目录，用完已删）：`双机：开，GPU：关；配置 …，第二台 第二台`，**退 0**。
⇒ 出口 2 要求的「退 0」，要在仓根 `multi-host.env` 里写上 `ENABLE_ACROSS_MACHINES=1` 才成立。那份配置归主 agent 或用户，我没碰。

## 各份自证的末行（都不设弄坏开关）
- `bash research/scripts/layer0-shard-run.sh --selftest` 退 0：`✓ layer0-shard-run.sh 自证通过：21 格都对（假 cargo（真 cargo 那一趟本次未跑：…）；第二台是本机上的另一个目录，没碰真的第二台）`。改名刚做完时是 19 格，后来加了 ⑭ ⑮ 两格。
- `bash research/scripts/mutation-shard-run.sh --selftest` 退 0：`✓ mutation-shard-run.sh 自证 18 格都对（…）`
- `bash research/scripts/multi-host-run.sh --selftest` 退 0：`✓ multi-host-run.sh 自证 13 格都对（…）`
- 54 号样本（只放 54 号的阶段目录，交给 stage-selftest.sh）退 0：`✓ 有样本的阶段判得都对（3 个样本），0 个阶段仍未自检`（red、green、static-check-red 三份样本都判对）。
- shell-lint / gate-lint：把这 7 份的暂存版和现版各拷进一个目录分别跑，都退 0。暂存版：`✓ shell 纪律检查通过（共 7 个脚本）`、`✓ 门禁自检通过：7 个脚本（.sh 与 .py）、49 条拒绝都带了出路`；现版：`共 7 个脚本` 与 `51 条拒绝都带了出路`。没有新出现的红。

## 条 1 与开关：每个弄坏开关打开时判红的原样行（截断在 230 字）
- local-reads-configuration：`✗ Ⓐ 默认本机跑：配置是坏的也照跑、退 0、在本机仓根跑、第二台一个目录都没建：退 255，在「」跑…`（共 3 格判错）
- local-without-memory-cap：`✗ Ⓐ 默认本机跑经 run-with-memory-cap.sh（…）：命令记下的 cgroup：0::/user.slice/user-1000.slice/session-1.scope`
- peer-exit-dropped：`✗ Ⓑ --peer 退出码原样带回：第二台那条命令退 7 ⇒ 驱动退 7：退 0；…`
- peer-without-memory-cap：`✗ Ⓑ --peer 经副本里的 run-with-memory-cap.sh 跑（…）：命令记下的 cgroup：0::/user.slice/user-1000.slice/session-1.scope`
- no-peer-cap-limit：`✗ Ⓑ --memory-cap 2G 比配置的 PEER_MEMORY_CAP 1G 大 ⇒ 拒（退 2），命令没跑：退 0，命令跑了；…`
- no-toolchain-check：`✗ Ⓒ 第二台的 rustc -Vv 不同 ⇒ 拒（退 255，…）…：退 0，命令跑了；…`
- fetch-altered：`✗ Ⓓ --fetch 取回的一个文件、一个目录与第二台上算的 sha256 逐字相同：退 0；核对：out/random.bin: FAILED|out/tree/deeper/a.txt: FAILED|…`（Ⓐ 那一格也红）
- no-peer-cleanup：`✗ Ⓔ Ⓑ Ⓓ 那几趟（含退 7 的那一趟）跑完，第二台 runs/ 下什么都不剩：Ⓑ 那边留着：peer-ok peer-ok.exit peer-ok.log peer-ok.pid peer-seven …`（Ⓕ 那一格也红）
- no-peer-stop-on-signal：`✗ Ⓕ 第二台那条命令在跑时收到 TERM ⇒ 退 143，…：退 143，第二台那条命令的进程号「2132045」还活着；…`（自证随后按这个进程号停掉了它；事后 `ps` 查过，一个都不剩）
- no-redact：`✗ Ⓖ 输出里没有配置里的主机名与第二台的目录，…：有的：peer/run.out ；命令的输出：/tmp/tmp.eON5waKKe2/peer/peer-home/runs/peer-…`
- sync-ignored-files：`✗ Ⓑ --peer 跑成：…被 git 忽略的 .env 没拷过去：退 0，在「…/peer-home/runs/peer-ok」跑，…`
- switch-off-falls-back-to-local：`✗ Ⓗ 配置里没写 ENABLE_ACROSS_MACHINES ⇒ --peer 拒（退 255，…）…：退 0，命令跑了；…`
- MULTI_HOST_CONFIGURATION_BREAK=switch-off-is-on（在 multi-host-run 的自证里打开）：Ⓗ 同样判错。
- 配置判法在 layer0-shard-run.sh --selftest 里：switch-off-is-on 打开时 `✗ ⑭ 写了 ENABLE_ACROSS_MACHINES=1 ⇒ 退 0…没写开关 ⇒ 判法报「双机：关」退 3…`；switch-value-unchecked 打开时 `✗ ⑮ 开关写了 0、1 之外的值…：ENABLE_ACROSS_MACHINES=yes 退 3…`。两次都报「1 格判错（共 21 格）」，退 1。
每项打开时整份自证都退 1；上面「各份自证的末行」三条是全部改回之后转绿的行。全部日志在 /tmp/claude-1000/multi-host-run/breaks/ 与 round2.*.log。

## 需要主 agent 知道的判断
- 拷到第二台的树用的是 `git ls-files -co --exclude-standard` 列出的文件，没有照 layer0-shard-run.sh 那样整棵 rsync。原因：本仓仓根有被 git 忽略的 `.env`、`.claude/settings.local.json`，整棵拷会把它们带到第二台。这一条由 Ⓑ 格与 sync-ignored-files 开关钉住。
- 退出码约定：命令自己的退出码原样带回；2 = 用法错（包括 --memory-cap 大于 PEER_MEMORY_CAP、标签已被占用）；255 = 驱动自己失败（配置、工具链、双机没开、起不来、等不到结局，或命令退 0 而点名取回的路径不在）。
- 本机模式不读配置；`--fetch` 在本机模式同样把文件拷进取回目录，默认是 `${TMPDIR:-/tmp}/singlefs-multi-host-run/<标签>`，可用 MULTI_HOST_RUN_FETCH_DIRECTORY 覆盖。
- 新写的 gate-similar 行（multi-host-run.sh 第 35、36 行），分别点名 layer0-shard-run.sh 与 mutation-shard-run.sh，并写明不合并的理由。没有新建门禁或钩子，所以没有 hook-events 行。

## 判法结论格式改了：54 号、59 号与两个驱动现在怎么读
判法现在有三种结局：退 0 = 双机开；**退 3 = 双机关**（配置判得过，只是 ENABLE_ACROSS_MACHINES 不是 1，不连第二台）；退 1 = 判不过。`--emit-assignments` 多打 ENABLE_ACROSS_MACHINES、ENABLE_GPU 两行，没写的打 0。
- 54 号第 510 行只看是不是退 0：退 3 会落进「双机分片：关（双机：关…）；每条用例单机跑」。行为是对的，可以不改；归 singlefs-29。
- 59 号第 144 行用 `!` 判：退 3 会落进「本地配置判不过」那句话。**行为对**（照单机跑），**但措辞不对**（配置其实判得过），建议 singlefs-8f 按退 3 另报「双机：关」。
- mutation-shard-run.sh 与 layer0-shard-run.sh 单独调用、配置是退 3 时，都会拒（退 1）。前者走库里的 load，那一句写的是「判不过」，措辞同样不准，但只有 59 号调它，而 59 号已经先挡掉了这种情况。两份自证的配置里都加了 `ENABLE_ACROSS_MACHINES=1`，都全对。

## 输入指纹作废
layer0-shard-run.sh 与 layer0-shard-configuration-check.sh 都改了内容（admission.py:220 的 SHARD_DRIVER_FILES）。下面三条用例的全绿标记会作废，要在提交时重跑：`crash-case:layer0-first-stream`、`crash-case:layer0-second-stream`、`crash-case:layer0-multi-record-publish-stream`。

## 仓里别处还有旧名（不在我的写范围，没改）
README.md（2 处）、.claude/rules/implementation-workflow.md（1 处）、research/e7-index-bench/src/bin/e162_crash_verdict_block_store.rs（2 处）、records/ 与 research/prompts/ 下若干处。清单用这条命令现取：`git grep -c -F -e layer0-shard.env -e SINGLEFS_LAYER0_SHARD_CONFIG -- . ':!research/scripts' ':!.claude/gate.d'`。

## 没做什么
- **47 号 runner 表没加 `bash research/scripts/multi-host-run.sh --selftest` 那一行**：replace-once 那一步被 auto mode 分类器拒了（Modify Shared Resources），`--check research-script-selftests` 因此也没跑。要加的就是一行，放在第 208 行 `"bash research/scripts/mutation-shard-run.sh --selftest" \` 的下一行：`              "bash research/scripts/multi-host-run.sh --selftest" \`
- 没碰真的第二台（自证里的「第二台」都是本机目录），没跑真 cargo 那一趟。
- 没跑重型测试（54、55、57、59、87 号本身、gate.sh、全量 cargo test、check.sh），没走定义三方，没提交。
- 没改仓根 `multi-host.env`（真配置要加 ENABLE_ACROSS_MACHINES=1 才会退 0）；也没改 54、59 号读开关的逻辑。
- heavy-test-guard 认不认得「`multi-host-run.sh -- cargo test …`」这种套在外面的写法，没查。
- 草稿目录 /tmp/claude-1000/multi-host-run/ 只留日志；阶段目录树与 lint 副本已删。
