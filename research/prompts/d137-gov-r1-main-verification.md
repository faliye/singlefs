# d137-gov-r1：一百三十七节那一轮的治理文件改动——第一轮判决（2026-10-29）

<!-- doc-lint:not-numbers G1 G2 G3 H1 H2 H3 H4 H5 H6 H7 H8 H9 H10 H11 H12 H13 S1 S2 S3 S4 S5 S6 S7 A1 A2 A3 A4 -->

## 一、这一轮的材料与证据

| 项 | 路径 | sha256 前 8 位 |
|---|---|---|
| 正文 | `research/prompts/_d137-gov-r1-body.md` | 54cb4e6b |
| 正推腿（Sonnet，G1、G2） | `research/prompts/d137-gov-r1-sonnet-output.md`（264 行），模型目录 `d137-gov-r1-sonnet-model/` | f3ec5a7c |
| 攻方腿（Opus，G3） | `research/prompts/d137-gov-r1-opus-output.md`，模型目录 `d137-gov-r1-opus-model/` | 101f39df |
| 被判文件快照 | `research/prompts/d137-gov-r1-snapshot/files-sha256.txt`（41 份） | — |

开工快照倒推与引文核对（主 agent 跑的原样命令）：

```
python3 scripts/agents/snapshot-tree.py d137-gov-r1 --out <草稿目录>/d137-gov-r1-tree
python3 scripts/agents/cite-check.py research/prompts/d137-gov-r1-sonnet-output.md research/prompts/d137-gov-r1-opus-output.md --root <草稿目录>/d137-gov-r1-tree --background research/prompts/_d137-gov-r1-body.md
```

快照点名 41 份，倒推出 41 份，sha256 都对上（腿跑着时被判文件没改）。引文核了 33 处、裸行号 37 处：对上 26 处，对不上 0 处，没判 44 处（裸行号没有引文可比 32 处；引的 `scripts/mutation/broad-mutation.py`、`.claude/singlefs-ai-sop/rules/rules-discipline.md`、调度记录不在快照树里）。没判的里，判决用到的几处主 agent 逐处人工核：`.claude/agent-common.md` 第 81 行原文「…记一条检出，它是进程组组长时会 fork、自己马上退出，完成通知当场发出；写成 `setsid --wait`」与腿乙 S1 引的一致；`scripts/agents/agent-watch.py` 第 3476 行条件 `verdict["declared_in_window"] > 0` 与攻方腿引的一致。

主 agent 现喂（主 agent 身份、不带前缀、cwd 在会话草稿目录，喂此刻的 `.claude/hooks/heavy-test-guard.sh`；脚本或代码里起 `cargo test --all`）：

| 喂法 | 退出码 |
|---|---|
| `bash -euo pipefail -c 'cargo test --all'` | 0 |
| 对照 `bash -c 'cargo test --all'` | 2 |
| `trap 'cargo test --all' EXIT; true` | 0 |
| `shopt -s expand_aliases; alias t='cargo test --all'; t` | 0 |
| `echo 'cargo test --all' \| bash /dev/stdin` | 0 |
| `cp /bin/bash ./mysh && ./mysh -c 'cargo test --all'` | 0 |
| `printf 'cargo test --all\n' > e.sh; f(){ bash -c true; }; BASH_ENV=<绝对路径>/e.sh f` | 0 |
| 对照 `BASH_ENV=<一行 cargo test --all 的脚本> bash -c true` | 2 |

## 二、逐条判决

### G1　照用户口径，钩子判得对不对（正推腿）

| # | 腿报的 | 主 agent 现查 | 判 |
|---|---|---|---|
| H1 | 带值短选项合写（`bash -euo pipefail -c …`）整条 `-c` 代码没判，根因 `lib_shell_words.py` 只认整词 `-o` | 现喂退 0，对照退 2 | 打中，采（最重） |
| H2 | `trap '…' EXIT/ERR/DEBUG/0` 的字符串当代码执行，不判 | 现喂退 0 | 打中，采 |
| H3 | `alias` + `shopt -s expand_aliases` 展开成重型 | 现喂退 0 | 打中，采 |
| H4 | 标准输入一族（`/dev/stdin`、`exec <文件` 等） | `bash /dev/stdin` 现喂退 0 | 打中，采；其余几种采为线索 |
| H5 | `cd` 失败或目标算不出之后的相对路径脚本 | 没现喂 | 采为线索 |
| H6 | `~+` 起的脚本与 `BASH_ENV=~+/…` | 没现喂 | 采为线索 |
| H7 | 同一条命令里先写出再起的源（多源 cat、`>>`、`tee`）不在盘上时放 | 没现喂 | 采为线索（与 S3 同一处） |
| H8 | 改名的 shell（`cp /bin/bash`、`ln -s`、`rbash`、`busybox sh`） | `cp /bin/bash ./mysh` 现喂退 0 | 打中，采 |
| H9 | 别的解释器里起 shell（tclsh、gawk system、perl -e、sed e、git 这类） | 没现喂；正文「已知没认」里「别的程序里起 shell」同一类 | 采为线索，并进已知没认那一批 |
| H10 | 引号里的花括号在命令词位置被当展开（误拒） | 没现喂 | 采为线索（偏严误拒） |
| H11 | 引号里的重定向花括号被拒，拒绝说明说「分不出」已过时 | 没现喂 | 采为线索（说明改） |
| H12 | `admission.py` 不在或 python 自己退 2，派发闸当「登记里没有」拒 | 没现喂 | 采为线索 |
| H13 | python3 不在 PATH 时钩子退 78、不记检出，与文件头「照常执行并记检出」不符 | 没现喂 | 采为线索 |

撑住的（腿量过、主 agent 没逐条复跑，照腿的读数记）：认不出一律拒 26 条、三种静默漏拒 7 条、主 agent 前缀规矩 18 条、准入退码 15 格里 14 格、撤旧根两样 29 格、同族脚本逐字节相同拷贝的边界、⑫ 写范围 24 格。

### G2　定义、共用约束、规则与钩子和脚本实际做的一致吗（正推腿）

| # | 腿报的 | 主 agent 现查 | 判 |
|---|---|---|---|
| S1 | `.claude/agent-common.md` 第 81 行写进一句理由 | 原文核过 | 打中，采 |
| S2 | `lib_heavy_tests.py` 第 218、256 行说 case 模式行被切成命令，`lib_shell_words.py` 已折叠 | 没核 | 采为线索（文件头改） |
| S3 | `heavy-test-guard.sh` 第 190 行写「照拒」，代码只在已拒之后加说明 | 没核 | 采为线索（与 H7 一起改） |
| S4 | 检出钩子拒绝说明「分不出花括号在不在引号里」已过时 | 同 H11 | 并进 H11 |
| S5 | 「会话临时目录」与「会话共用的暂存目录」在 `.claude/agent-common.md` 第 39、44 行并存 | 没核 | 采为线索（统一叫法） |
| S6 | 派发闸那一句与 H12 不符 | 同 H12 | 并进 H12 |
| S7 | 「钩子 bash 那一层起不来照常执行并记检出」与 H13 不符 | 同 H13 | 并进 H13 |

撑住的：开工先读引用 74 处、进程组一句四处逐字相同、detached-run.sh 文件头 20 格、检出 ②④⑦⑬⑮ 与判不了照放并记检出 30 格、`--ack` 四种键三处相同、verification、commit-stages、CLAUDE.md 各一句。

### G3　攻：过度外推、内部矛盾、最脆弱的一条（攻方腿）

| # | 腿报的 | 主 agent 现查 | 判 |
|---|---|---|---|
| A1 | 看门狗 `declared_in_window > 0` 就整条低核只记不叫醒，声明一次永不闭合的真卡住叫不醒主 agent（越位：说成稳妥降噪） | `agent-watch.py` 第 3476 行条件核过 | 打中，采 |
| A2 | 认不出一律拒只在命令文本这一层：`BASH_ENV` 包进转发函数静默放行、不记检出 | 现喂退 0，对照退 2 | 打中，采 |
| A3 | `core_usage.py` 第 56 行「超过整趟一半才另注」与看门狗「>0 就静音」门槛不一（内部矛盾） | 看门狗那一侧核过；core_usage 第 56 行没核 | 采（与 A1 同一处改） |
| A4 | 看门狗按主会话记录的 `--label` 认单元归属，与 detached-run「脚本可在会话里起、脱离进程树」对不上，脚本起的本会话长活被当成别人的静音 | 推的，没在多会话环境复现 | 采为线索 |

攻方腿提的改法（静音门槛改成声明占比过半）被攻过零轮，它自己量过对「声明一次永不闭合」那一格不起作用；不照搬，交修补件另定判法。

## 三、怎么办

这是第一轮。照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」，打中的这一批合成一批修，修完开第二轮攻这一批修补（第二轮分工：修补本身、产生结论的方法、从未被看过的地方）。修补按文件聚簇：

- 钩子与共用切词：H1–H11、A2、正文「已知没认」四类（登录 shell 的 profile、系统级启动文件、别的程序里起 shell 即 H9、`bash x.sh` 正文里起 claude）、S2、S3、S4、H13；采为线索的先复现再改，复现不出的写明。
- 派发闸：H12（连同 S6）。
- 看门狗与 core_usage：A1、A3、A4。
- 共用约束与定义：S1、S5，连同上面几件改完要跟着改的句子（S6、S7 等）。

这一批改法都是零轮，第二轮攻它们。

## 回看决策

不涉及决策：被判的是项目的协作钩子、定义、规则与研究脚本，不碰 `.claude/kb/decisions/` 下任何一条。

## 这一轮被判过的文件

- `.claude/hooks/lib_shell_words.py`：G1 打中 H1、H8、H10（共用切词）
- `.claude/hooks/heavy-test-guard.sh`：G1 打中 H1–H8、A2；G2 打中 S3
- `.claude/hooks/bash-command-detector.sh`：G1 打中 H11；G2 打中 S4；H13 采为线索
- `.claude/hooks/lib_heavy_tests.py`：G2 打中 S2
- `.claude/hooks/runner-dispatch-guard.sh`：G1 采为线索 H12
- `.claude/hooks/lib_work_root.py`：G1 撑住（撤旧根两样 29 格）
- `.claude/hooks/lib_prompt_scratch_line.py`：G1 撑住（撤旧根）
- `.claude/hooks/lib_write_scope.py`：G1 撑住（撤旧根、⑫ 写范围）
- `.claude/hooks/write-guard.sh`：G1 撑住（撤旧根）
- `.claude/hooks/handback-guard.sh`：G2 撑住（判不了照放并记检出）
- `.claude/hooks/completion-cleanup-guard.sh`：G1 撑住（会话临时目录照管）
- `.claude/hooks/lib_snapshot_conflicts.py`：G2 撑住（注释与看门狗一致）
- `.claude/hooks/agent-write-scope.tsv`：G1 撑住（子 agent 不再写旧根）
- `.claude/agent-common.md`：G2 打中 S1、S5、S6、S7
- `.claude/main-agent.md`：G2 撑住（`--ack` 写法、只记不叫醒的范围）；A1 牵到「声明了串行段的处置句」
- `.claude/agents/tooling-writer.md`：G2 撑住（开工先读引用）
- `.claude/agents/implementation-writer.md`：G2 撑住（开工先读引用）
- `.claude/agents/experiment-runner.md`：G2 撑住（开工先读引用）
- `.claude/agents/investigator.md`：G2 撑住（开工先读引用）
- `.claude/agents/three-way-forward.md`：G2 撑住（进程组一句、开工先读）
- `.claude/agents/three-way-attack.md`：G2 撑住（进程组一句、开工先读）
- `.claude/agents/three-way-defense.md`：G2 撑住（进程组一句、开工先读）
- `.claude/agents/three-way-materials.md`：G2 撑住（开工先读；第 2 步加的那一句触发条件两条腿都没打中）
- `.claude/agents/mutation-triage.md`：G2 两条腿都没打中
- `.claude/rules/verification.md`：G2 撑住
- `.claude/skills/crash-test/commit-stages.md`：G2 撑住
- `CLAUDE.md`：G2 撑住
- `scripts/agents/agent-watch.py`：G3 打中 A1、A4
- `scripts/agents/watch.sh`：G2 撑住（`--ack` 写法）
- `scripts/hosts/core_usage.py`：G3 打中 A3
- `scripts/agents/copy-originals.py`、`scripts/hosts/run-with-memory-cap.sh`、`scripts/hosts/memory_peaks_lib.py`、`scripts/hosts/detached-run.sh`、`scripts/hosts/clean-session-scratch.py`、`scripts/hosts/peer-host-lib.sh`、`scripts/gate/apply-writer-patch.py`、`scripts/mutation/mutation-shard-run.sh`、`scripts/crash/crash-point-replay-shard-run.sh`、`scripts/crash/crash-amplification-item.sh`：两条腿都没打中（detached-run.sh 文件头 20 格撑住）
