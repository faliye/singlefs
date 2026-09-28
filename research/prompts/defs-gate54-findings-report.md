# 工具实现员报告：54 号与双机驱动接发现日志（层 0 放量的发现日志，工具那一半）

报告时刻：2026-09-27。关的是 `records/2026-09-24-里程碑二收尾调度.md`「层 0 放量的发现日志」那一行；接口照 `/tmp/claude-1000/impl-layer0-findings/report.md` 第一节逐字读。

## 结论

出口 1–5 都做到了：
- 54 号 `--full`：每条用例的全量日志与发现日志放 `<common-dir>/singlefs-layer0-logs/<时刻>-<pid>/`，名字 `log.<用例名>` 与 `log.<用例名>.findings.tsv`，跑完不删。单机跑时在 crash-case-command 交的命令外面套 `env SINGLEFS_LAYER0_FINDINGS_FILE=<发现日志>`。跑完先打发现表，再判：没有 summary 的节报「这一趟没跑完」判红；`red_states` 不是 0 判红；定稿行数与 `signatures=` 对不上判红；超过 30 个签名只打前 30 行，再打一行「其余 N 个见 <路径>」。全量日志只给路径。快档不设、不读，还用 `env -u` 清掉调用方的这个变量。
- 双机驱动：三趟各设一个发现日志，每份都是那一趟的日志名加 `.findings.tsv`。`--merged-log` 时 merge 那一份就是 `<54 给的日志>.findings.tsv`，正是 54 号读的那一份。第二台那一片先写在第二台上，跑完拷回本机、放在那一片的日志旁边，第二台上那一份随后删掉。三份路径都打进输出。
- 判别力样本 `.claude/gate.d/fixtures/54-layer0-replay.sh/{red,green}`，`stage-selftest.sh` 两格都判对。
- 全绿标记不带发现表（judged 行文件没动）。

有三处是我定的写法，派发提示里没写：
1. 日志目录：原来的全量日志放在跑完就删的 mktemp 里，而出口 5 要发现日志留着，所以另开了一个持久目录。
2. 读每一节，不只读最后一节。这一趟的目录是新建的，里面的节都是这一趟的；树分裂那条用例一个进程跑七条流，写七节，只读最后一节会漏掉前六条。
3. 发现日志不在时怎么判：全量日志里有 `LAYER0_FINDINGS` 行却没有发现日志，判红（变量没传进用例）；一行都没有，就只打一句说明、不判红。原因：`crash-injection-fast-tier`、`c561-sigma-full` 不走读这个变量的入口（`enumerate_every_subset_of_one_segment` 等），它们不会写发现日志。

推翻条件：
- 真 cargo 跑 `--full` 时，某条走 `enumerate_layer0_in_state_slices_or_one_shard` 的用例全量日志里有 `LAYER0_FINDINGS` 行，54 号却报「没走读 … 的枚举入口」；或者驱动交回的 merge 发现日志不在 `<日志>.findings.tsv`。
- 实现员的定稿格式与第一节不符（字段名、制表符、summary 行名）。

`admission.py` 没改：它的 54 号自证里，假 cargo 的日志没有 `LAYER0_FINDINGS` 行、也不写发现日志，走的是「不判红」那一支，232 格照样全过。

## 改过的文件

```
 .claude/gate.d/54-layer0-replay.sh            | 150 ++++++++++++++++++++++++--
 research/scripts/layer0-shard-run-selftest.sh |  34 ++++++
 research/scripts/layer0-shard-run.sh          |  60 ++++++++---
 3 files changed, 224 insertions(+), 20 deletions(-)
```

新建（未跟踪，30 个文件）：`.claude/gate.d/fixtures/54-layer0-replay.sh/`。`git status --short --untracked-files=all` 数出来的行数：30。

54 号文件头的准入声明两行（第 2、3 行）与 `source …preflight.sh` / `preflight …` 两行没动；驱动与自证文件头的声明、source、preflight 行也没动。已存在的三份脚本都是先在草稿目录里改，再写成同目录临时文件、`mv` 换上，换上后 `cmp` 与草稿相同。例外：开工时 54 号文件头两段说明是用 Edit 就地改的，当时 `ps` 查过，没有 54 号或驱动在跑。

改动的位置（行号现取）：
```
105:layer0_sample_mode=0
220:  env -u SINGLEFS_LAYER0_FINDINGS_FILE cargo test --release -p singlefs-harness --test "$1" -- --nocapture 2>&1 \
255:run_crash_case() {
278:report_crash_case_findings() {
459:layer0_log_directory="$git_common_directory/singlefs-layer0-logs/$(date -u +%Y%m%dT%H%M%SZ)-$$"
507:  case_findings_file="$case_log.findings.tsv"
529:  if report_crash_case_findings "$case_findings_file" "$case_log"; then case_findings_exit=0; else case_findings_exit=$?; fi
557:  if [[ "$case_findings_exit" != 0 ]]; then
605:  exit 3
101:peer_findings_file="$PEER_REPOSITORY_DIRECTORY/runs/$run_label.shard-1-of-2.findings.tsv"
106:  merge_findings_file="$merged_log_file.findings.tsv"
109:  merge_findings_file="$log_prefix.merge.log.findings.tsv"
212:local_findings_file="$local_log.findings.tsv"
213:fetched_peer_findings_file="$peer_log.findings.tsv"
242:if [[ "$layer0_shard_break" == no-peer-findings-copy ]]; then
279:merge_findings_setting=(SINGLEFS_LAYER0_FINDINGS_FILE="$merge_findings_file")
280:if [[ "$layer0_shard_break" == no-merge-findings ]]; then merge_findings_setting=(-u SINGLEFS_LAYER0_FINDINGS_FILE); fi
286:echo "  · ⑥ 三份发现日志：本机 0/2 $local_findings_file；第二台 1/2 $fetched_peer_findings_file；merge $merge_findings_file"
72:write_findings_section() { # write_findings_section <begin 行多出来的字段>：设了 SINGLEFS_LAYER0_FINDINGS_FILE 就追加一节（0 个签名），标准输出打 LAYER0_FINDINGS
168:# ⑦ 发现日志：merge 那一份是 <日志文件>.findings.tsv（54 号读的那一份），两片的在各自日志旁边，第二台上那一份拷回之后删了，三份路径都打进输出
```

## 判红与转绿（原样行）

### 54 号样本（stage-selftest.sh 同一做法；草稿里跑在一个镜像目录上，`.claude/scripts`、`.claude/singlefs-ai-sop`、`research`、`stage-inputs.tsv` 都是指向真仓的链接；换进仓之后再用真的 stage-selftest 跑一遍）

弄坏开关 `GATE_LAYER0_BREAK=<项>` 打开时（草稿执行器 `run-samples.sh`，判错的每一行照抄）：
```
GATE_LAYER0_BREAK=no-findings-file
  ✗ red 输出里找不到「第 1 节（stream=two-signatures states=100）：signatures=2 red_states=3 states=100」
  ✗ red 输出里找不到「#1 pass=journal_consulted_oracle violated=wrong_content segment=2 publish=instance0_txg5_root_write3 states=2 sample_states=17,18」
  ✗ red 输出里找不到「#2 pass=pool_checker violated=I-3.1 segment=all_persisted publish=every_write_persisted states=1 sample_states=99」
  ✗ red 输出里找不到「问题：第 1 节 red_states=3：有状态判红」
  ✗ red 输出里找不到「第 1 节（stream=unfinished states=100）：这一趟没跑完（没有 layer0_findings_summary 行）；死之前找到 1 个签名：」
  ✗ red 输出里找不到「#1 pass=record_checker violated=root_without_record segment=0 publish=after_the_last_root first_state=5」
  ✗ red 输出里找不到「问题：第 1 节这一趟没跑完」
  ✗ red 输出里找不到「✗ crash-case:unfinished 的用例跑过了、日志也判得绿，发现日志却判红」
  ✗ red 输出里找不到「#30 pass=journal_ignored_oracle violated=read_failed segment=30」
  ✗ red 输出里找不到「其余 2 个见」
  ✗ red 输出里找不到「--full 有 3 条崩溃枚举用例判红」
  ✗ green 期望退出 3，实测 1
  ✗ green 输出里找不到「第 1 节（stream=clean-stream states=100）：signatures=0 red_states=0 states=100」
  ✗ green 输出里找不到「没有签名」
  ✗ green 输出里找不到「crash-case:clean-stream 判绿」
  ✗ green 输出里找不到「这一趟跑了判绿 2 条」
  ✗ green 输出里找不到「退 3 不退 0」
判对 0 个，判错 2 个
GATE_LAYER0_BREAK=ignore-unfinished
  ✗ red 输出里找不到「第 1 节（stream=unfinished states=100）：这一趟没跑完（没有 layer0_findings_summary 行）；死之前找到 1 个签名：」
  ✗ red 输出里找不到「#1 pass=record_checker violated=root_without_record segment=0 publish=after_the_last_root first_state=5」
  ✗ red 输出里找不到「问题：第 1 节这一趟没跑完」
  ✗ red 输出里找不到「✗ crash-case:unfinished 的用例跑过了、日志也判得绿，发现日志却判红」
  ✗ red 输出里找不到「--full 有 3 条崩溃枚举用例判红」
判对 1 个，判错 1 个
GATE_LAYER0_BREAK=ignore-red-states
  ✗ red 输出里找不到「问题：第 1 节 red_states=3：有状态判红」
  ✗ red 输出里找不到「✗ crash-case:two-signatures 的用例跑过了、日志也判得绿，发现日志却判红」
  ✗ red 输出里找不到「✗ crash-case:many-signatures 的用例跑过了、日志也判得绿，发现日志却判红」
  ✗ red 输出里找不到「--full 有 3 条崩溃枚举用例判红」
判对 1 个，判错 1 个
GATE_LAYER0_BREAK=no-truncation
  ✗ red 输出里找不到「其余 2 个见」
判对 1 个，判错 1 个
```

改后在仓里跑 `bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh "$PWD/.claude/gate.d"`（退 1，红在 55 号，见下）：
```
  ✓ 54-layer0-replay.sh          red   判得对
  ✓ 54-layer0-replay.sh          green 判得对
  ✗ 55-qemu-first-transaction.sh red   输出里找不到「second-transaction 盘 0：盘上收到的写（23）没有多于第一个事务那段（23）」
  ✗ 55-qemu-first-transaction.sh red   输出里找不到「这一档该在第一个事务之后再发布一次」
  ✗ 55-qemu-first-transaction.sh green 期望退出 3，实测 1
  ✗ 55-qemu-first-transaction.sh green 输出里找不到「预录档（.qemu-prerecorded，没起虚机）」
  ✗ 55-qemu-first-transaction.sh green 输出里找不到「判了 5 档 direct skip-first-transaction-barrier page-cache second-transaction second-instance」
  ✗ 55-qemu-first-transaction.sh green 输出里找不到「样本里没摆的 0 档：无」
  ✗ 55-qemu-first-transaction.sh green 输出里找不到「这一档不算真跑过 QEMU」
  ✗ 2 个样本判错（判对 153 个，0 个阶段没有样本）
rc=1
```
55 号的样本判错不在这一轮的改动里（`.claude/gate.d/55-qemu-first-transaction.sh` 是别的会话在改的，开工时 git status 就是 M），没修。

green 样本的发现表那几行（原样）：
```
  · 这一趟的全量日志与发现日志放 <临时目录>/.git/singlefs-layer0-logs/20260927T023151Z-3642277/（log.<用例名> 与 log.<用例名>.findings.tsv，跑完不删）
    全量日志：<临时目录>/.git/singlefs-layer0-logs/20260927T023151Z-3642277/log.clean-stream；发现日志：<临时目录>/.git/singlefs-layer0-logs/20260927T023151Z-3642277/log.clean-stream.findings.tsv
    发现日志：<临时目录>/.git/singlefs-layer0-logs/20260927T023151Z-3642277/log.clean-stream.findings.tsv（全量日志里 LAYER0_FINDINGS 1 行）
      第 1 节（stream=clean-stream states=100）：signatures=0 red_states=0 states=100
        没有签名
    全量日志：<临时目录>/.git/singlefs-layer0-logs/20260927T023151Z-3642277/log.no-enumeration；发现日志：<临时目录>/.git/singlefs-layer0-logs/20260927T023151Z-3642277/log.no-enumeration.findings.tsv
    发现日志：没有（<临时目录>/.git/singlefs-layer0-logs/20260927T023151Z-3642277/log.no-enumeration.findings.tsv 不在或是空的，全量日志里也没有 LAYER0_FINDINGS 行：这条用例没走读 SINGLEFS_LAYER0_FINDINGS_FILE 的枚举入口）
  ! 样本档（.layer0-sample-tools，假 cargo）：流程判绿，退 3 不退 0——这一趟没跑过真的层 0
```
red 样本里两签名那一条（原样，路径换成了 <临时目录>）：
```
      第 1 节（stream=two-signatures states=100）：signatures=2 red_states=3 states=100
        #1 pass=journal_consulted_oracle violated=wrong_content segment=2 publish=instance0_txg5_root_write3 states=2 sample_states=17,18
        #2 pass=pool_checker violated=I-3.1 segment=all_persisted publish=every_write_persisted states=1 sample_states=99
      问题：第 1 节 red_states=3：有状态判红（不是 0 都判红；读不出数也判红）
     → 怎么办：「这一趟没跑完」看全量日志 <临时目录>/.git/singlefs-layer0-logs/20260927T023150Z-3641958/log.two-signatures 的尾部是 panic 还是被杀；red_states 不是 0 是有状态判红而用例没红，
      第 1 节（stream=unfinished states=100）：这一趟没跑完（没有 layer0_findings_summary 行）；死之前找到 1 个签名：
      问题：第 1 节这一趟没跑完（没有 layer0_findings_summary 行：还在跑、被杀或 panic 判红）
     → 怎么办：「这一趟没跑完」看全量日志 <临时目录>/.git/singlefs-layer0-logs/20260927T023150Z-3641958/log.unfinished 的尾部是 panic 还是被杀；red_states 不是 0 是有状态判红而用例没红，
        其余 2 个见 <临时目录>/.git/singlefs-layer0-logs/20260927T023150Z-3641958/log.many-signatures.findings.tsv
     → 怎么办：「这一趟没跑完」看全量日志 <临时目录>/.git/singlefs-layer0-logs/20260927T023150Z-3641958/log.many-signatures 的尾部是 panic 还是被杀；red_states 不是 0 是有状态判红而用例没红，
  ✗ --full 有 3 条崩溃枚举用例判红：crash-case:two-signatures crash-case:unfinished crash-case:many-signatures（这一趟判绿 0 条、复用 0 条；开跑 2026-09-27，跑完 2026-09-27）
```

### 驱动脚本（layer0-shard-run.sh --selftest，假 cargo；草稿在 rsync 出来的仓副本里跑，这份副本已删）

弄坏开关 `LAYER0_SHARD_RUN_BREAK=<项>`：
```
LAYER0_SHARD_RUN_BREAK=no-peer-findings-copy
  ✗ ⑦ 发现日志：merge 那一份在 <日志文件>.findings.tsv（一节、不带 shard=、有 summary），两片的在各自日志旁边（shard=0/2、1/2），第二台上那一份拷回后删了，三份路径都打进输出；单独跑时三份在 common-dir 下：第二台那一片没拷回到 <临时目录>/merged.log.shard-1-of-2.log.findings.tsv（或没有 shard=1/2 那一节）；单独跑那一趟 common-dir 下 singlefs-layer0-logs/ 里非空的发现日志是 2 份，不是 3 份
  ✗ layer0-shard-run.sh 自证没过：1 格判错（共 11 格）
rc=1
LAYER0_SHARD_RUN_BREAK=no-merge-findings
  ✗ ⑦ 发现日志：merge 那一份在 <日志文件>.findings.tsv（一节、不带 shard=、有 summary），两片的在各自日志旁边（shard=0/2、1/2），第二台上那一份拷回后删了，三份路径都打进输出；单独跑时三份在 common-dir 下：merge 那一份 <临时目录>/merged.log.findings.tsv 不是恰好一节带 summary；单独跑那一趟 common-dir 下 singlefs-layer0-logs/ 里非空的发现日志是 2 份，不是 3 份
  ✗ layer0-shard-run.sh 自证没过：1 格判错（共 11 格）
rc=1
```
不带开关：
```
  ✓ ⑦ 发现日志：merge 那一份在 <日志文件>.findings.tsv（一节、不带 shard=、有 summary），两片的在各自日志旁边（shard=0/2、1/2），第二台上那一份拷回后删了，三份路径都打进输出；单独跑时三份在 common-dir 下
  ✓ layer0-shard-run.sh 自证通过：11 格都对（假 cargo（真 cargo 那一趟本次未跑：带 SINGLEFS_HEAVY_TESTS=commit 或 user-request 才跑）；第二台是本机上的另一个目录，没碰真的第二台）
rc=0
```
装进仓之后，47 号里的 `bash research/scripts/layer0-shard-run.sh --selftest` 与 `python3 research/scripts/admission.py --selftest` 两项都没列进红项（47 号只列出没过的那几项，见下）。

## 第 7 步与出口 4 各项（仓里现跑）

| 命令 | 退出码 | 末行（原样） |
|---|---|---|
| `bash .claude/gate.d/47-research-script-selftests.sh` | 1 |              ✗ 段序列登记表与 E142 产物的 name=segments 行（或第二条流与钉它的用例）对不上，共 2 处： |
| `bash .claude/gate.d/62-stage-owners.sh` | 0 |   ✓ 阶段归属表与门禁目录一致（76 个阶段，归 9 个 agent） |
| `bash .claude/gate.d/63-agent-write-scope.sh` | 0 |   ✓ 写范围闸、Bash 检出 hook、重型测试闸、续派闸、续做闸、交回闸与弹窗断言闸注册着、自证通过，定义的 model 取值认得，按模式找进程的上游钩子注册着、文件在，会话 |
| `bash .claude/gate.d/73-research-gate-lint.sh` | 0 |   ✓ 研究脚本、hook 与 .claude/scripts 的门禁纪律：扫了 3 个目录（research/scripts .claude/hooks .claude/scripts），拒绝都带出路、shell 纪律守住；进程安全连 .claude/gate.d/ 共扫 4 个目录� |
| `doc-lint.sh .` | 0 |   ✓ 文档铁律检查通过（检查 521，跳过 0；DOC_LINT_VERBOSE=1 看全部） |
| `rules-lint（项目本地那一道）` | 0 |   ✓ 规则只写怎么做（语言 zh；扫了 29 份文件 1965 行；没扫 0 个；记录小节 0、论证小节 0、带日期的行 0（另有 8 行的日期只在引号或反引号里）、解释性段落 0、解释性半� |
| `gate-overlap.py` | 0 |   ✓ 相对 262c02d3e47a：新加的门禁与钩子 0 个（无）都写明了比过谁，改过的 4 份脚本对照已有的 134 份没有整段相同 |
| `SINGLEFS_GATE_FULL=1 python3 research/scripts/admission.py --selftest` | 0 |   ✓ admission.py 自证通过：232 格都对（含弄坏开关 skip-unchanged、skip-preconditions、skip-gate-preconditions、ignore-gate-environment、skip-build-environment、ignore-linked-directories、exclude-mentioned-tes |
| `GATE_LINT_DIR=.claude/gate.d gate-lint.sh .` | 1 |   ✗ 门禁自检失败：45 处（共 655 个脚本、1093 条拒绝） |
| `SHELL_LINT_DIR=.claude/gate.d shell-lint.sh .` | 1 |   ✗ shell 纪律检查失败：61 处（共检查 527 个脚本） |
| `preflight-lint.py` | 0 |   ✓ 准入与运行条件：判了 138 个脚本（登记目录 50、脚本 3、钩子 9、门禁阶段 76），都在开头写明了条件并先判；只 exec 共享脚本的包装 7 个，由被转发的脚本判；排除 192  |
| `stage-selftest.sh` | 1 |   ✗ 2 个样本判错（判对 153 个，0 个阶段没有样本） |

红的几项，点名的文件都不在这一轮的改动里，没修：
- 47 号：只有 `ask-local-selftest.sh`（「检测器崩了时应退 6，实际 2」）与 `check-segment-registry.py --selftest`（「段序列登记表与 E142 产物的 name=segments 行…对不上，共 2 处」）两项。
- gate-lint：45 处，本轮文件 0 处（点名的文件逐个列在 gate-lint.log 里，没有一份是这一轮的）。
- shell-lint：61 处，本轮文件 0 处。另外它对每个 fixtures/<阶段>.sh 目录都打「Is a directory」，这是已有的样子，新样本目录也一样。
- stage-selftest：只有 55 号两格（见上）。

派活时本机 load 23（32 核），ps 里有别的会话的 cargo test / cargo build，没有 qemu、vm-bench、fio。

## 新写的 `# gate-similar:` / `# hook-events:` 行

没有。这一轮没有新建门禁阶段或钩子：改的是已有的 54 号，另加一个样本目录。gate-overlap 的输出是「新加的门禁与钩子 0 个」。

## 第二种活改过的定义

没有。

## 看到但没做

- 驱动脚本单独跑时（不带 `--merged-log`）只打出三份发现日志的路径，不判发现表。判发现表只在 54 号那一边，派发提示也只要打路径。
- 发现日志放在 `<common-dir>/singlefs-layer0-logs/` 下，每趟 `--full` 一个目录，没有清理，会越攒越多。要不要加保留条数，由主 agent 定。
- 54 号与驱动之间靠同一个命名约定对上：54 号读 `<传给驱动的日志>.findings.tsv`，驱动写到同一个路径。驱动那一半由驱动自证 ⑦ 钉住。admission.py 自证里的分片格用的是假驱动，不走这条路；两边连起来的那一趟只有真跑 `--full` 才验得到。
- 驱动文件按内容进三条 `shard=across-machines` 用例的输入指纹（`admission.py` 的 SHARD_DRIVER_FILES），所以改了驱动，这三条用例的旧全绿标记就不再作数，下一趟 `--full` 会重跑它们。54 号本身不进指纹。
- crates 那一半（实现员）还没合进来：现在真用例的日志里没有 `LAYER0_FINDINGS` 行，54 号对每条用例都会报「没走读 … 的枚举入口」、不判红。crates 合进来之后才会真的判发现表。

## 没做什么

- 没跑重型测试：54 号 `--full` 真跑、驱动带真 cargo（`SINGLEFS_HEAVY_TESTS`）、55、57、59、87、`gate.sh`、全量 `cargo test`、`check.sh` 都没跑。54 号只在样本档跑过（假 cargo，经 stage-selftest 与草稿执行器）。
- 没走定义三方，没提交。
- 没改 `admission.py`：判法不读 `LAYER0_FINDINGS` 行，它的自证不改也全过。
- 草稿目录里的仓副本（`repo-copy`，264M）与镜像目录已删；剩下的草稿（draft/、orig/、各 log）都在 `/tmp/claude-1000/gate54-findings/` 下，没有编译目录。
