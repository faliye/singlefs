# 长跑脱离会话入口：工具轮第三轮（最终轮）正文（2026-10-01）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 U1 U2 U3 W1 W2 h4 h5 h7 -->

## 一、题面

第二轮判决 `research/prompts/tooling-detached-run-r2-main-verification.md` 的改法 G1–G5 已改完（实现员报告 `/tmp/claude-1000/m3-r1/detached-run-3/report.md`）。这是最终轮（`.claude/rules/three-way-inference.md`「多轮」：第三轮之后停，这一轮里新冒出来的零轮形态写进判决的交用户表、标「零轮」，不再为它开一轮）。

只攻三样：G1–G5 的改法本身；第二轮还开着的格；第一、二轮报出的数能不能核。

主 agent 带前缀跑过验收（`/tmp/claude-1000/-home-fy5090-code-singlefs/8adc3bc8-1d0b-43bd-9a3f-9e63dae04aaf/scratchpad/detached-accept3/summary.txt`）：
- 自证末行「✓ detached-run.sh 自证 33 格都对」；27 个 `DETACHED_RUN_BREAK` 开关与 `LIB_SHELL_WORDS_BREAK=detached-run-force-kept` 各退 1，红的都是点名的格。
- 钩子：对照组都退 0，四个新开关各自退 1；钩子样本 `feed3.py` 9 行全对。
- 第二轮攻方的 6 份复现在新版上跑：
  - w1：「watch 退 124；此刻单元 ActiveState=active」，不再报「都结束了」；
  - w2：「start 退 0」「watch 退 124」；
  - v3：「start 退 1」「marker：不在」；
  - b1：命令照跑、登记不带 `--force`，闸那一半由自证 Ⓟ 与样本核；
  - v4：五种错实现自证都退 1；
  - 钩子：h4、h7 都退 1。
- 残留单元 0。

## 二、实现今天的样子（主 agent 的观测，2026-10-01）

被判的是脚本与钩子，不是 `crates/`：`crates/` 里没有相关实现（`grep -rln 'systemd-run\|detached' crates/` 零命中）。

腿读冻结副本 `/tmp/claude-1000/tooling-detached-run-r3/tree/`（sha256 在 `research/prompts/tooling-detached-run-r3-snapshot/files-sha256.txt`）。这一轮的改动是 `research/prompts/tooling-detached-run-r3-diff.patch`（1 001 行）：拿实现员开工时的备份对现在的文件逐份 diff；5 份备份与第二轮冻结副本逐字节相同，所以 diff 里只有 G1–G5。`agent-common.md`、`main-agent.md` 里与这件事有关的句子，这一轮没改。`main-agent.md` 第 52、61、63 行在第二轮之后被别的会话改过（加第五道重型门禁），与这一轮无关，不判。

逐条落点（实现员报告第二段）：
- **G1**：watch 观察时连单元名一起记，登记被删或挪走、单元还在跑的，列成「还在跑（登记里没有）」；start 遇到 `systemd-run` 退非 0 时回读单元状态，在跑就记 systemd-run-failed、退 0；list 对记了 start-failed 的登记按单元名再查一次。
- **G2**：
  - start 起单元之前先核 `--log` 写得进；写不进就删掉这一次的登记、把挪走的同名旧登记挪回去、退 1。
  - 起完回读时「有退出码没有进程号」判成没起成。
  - 单元里写登记失败的，写进旁路文件 `$XDG_RUNTIME_DIR/singlefs-detached-runs/`，list 和 watch 单列成「登记写不进」。
- **G3**：选「与预检一致」：共用库新加 `detached_run_arguments_after_preflight`，两道钩子切参数前照 preflight 摘掉每一个 `--force`；自证 Ⓟ 把闸切出的词与登记里真跑的命令逐字比。
- **G4**：b、c、f、g 四种错实现各成一个开关与自证格；钩子补 h4（install / mv）、h7（`cd` 之后的相对路径）、h5（heredoc 写出的拷贝）。
- **G5**：
  - 连续 3 次查不到就单列一行报警，watch 不退、不停单元。
  - 前台调 stop 时，Bash 的 timeout 至少（停机时限 + 30）× 1000 毫秒，默认 210000；时限超过 205 秒的用 `run_in_background` 起 stop。只写在脚本文件头和 start 的输出里，没写进规则。
  - `--stop-timeout` 默认 180 秒（推的，没量过）。
- 已知事实：两个脱离单元 `singlefs-detached-broad-mutation-local.service`、`singlefs-detached-broad-mutation-peer.service` 是第一轮那一版起的，在跑；两台 `Linger=yes`。

## 三、格

### U1　G1–G5 照判决做到了没有、合不合规则（云端正推）
逐条核第二轮判决第三节 G1–G5 原文与 diff：每条做到了什么、没做到什么、多做了什么。G5「stop 的超时只写在脚本文件头」与 `.claude/main-agent.md`「派出去之后」一节、`.claude/agent-common.md`「长活可以等」对不对得上，要不要写进规则（写进规则要走定义三方或用户豁免）。核第一、二轮判决里引的数（产物行、复跑输出）在各自的日志里逐字找不找得到。

### U2　G1–G5 改法本身哪一格会错（云端攻方）
不重复前两轮攻过的角度（第一轮 T2–T4、第二轮 V2–V4，见两轮攻方报告）。至少攻：
① G2 回滚：起之前写了一半登记再回滚，与同名旧登记挪回去，在竞争与中途被杀时会不会丢掉旧登记的退出码；旁路文件在 `$XDG_RUNTIME_DIR`（tmpfs，重启清空）的后果。
② G1：start 回读到单元在跑就退 0，`systemd-run` 真失败而同名单元是早先那一次的残留时，会不会把旧单元认成这一次起的。
③ G3 摘 `--force` 与预检逐字一致：`--force=…`、`-- -- --force`、引号里的 `--force`。
④ G5 报警：「连续 3 次」怎么计数，查不到与查得到交替时清不清零。
⑤ 新开关有没有在实际调用里生效的（F4 那一类）。

### U3　第二轮还开着的格（云端攻方，同一条腿、另成一节）
第二轮判决第二节第 13、14 条：单元被 KILL 时内存包装的 scope 会不会活下来（造得出就造）；别的会话的自证单元在 list 里是噪声；1 000 条登记时 list 要 11 秒。

## 四、分工

| 腿 | 格 | 立场 | 理由 |
|---|---|---|---|
| 云端正推（Sonnet） | U1 | 判决对实现、规则对实现、数能不能核 | 第三轮分工照规则「已定项撑不撑得住、清单全不全、数能不能核」；第二轮已派过辩方 |
| 云端攻方（Opus） | U2、U3 | 在副本上造反例 | 两节分开报 |

## 五、跑前写死的判据

- U2 造出 G1–G5 引进的新错（丢退出码、把在跑的报成结束或反过来、把旧单元认成这一次起的、闸看的与真跑的不一致、开关在实际调用里生效） ⇒ 打中，改法写进判决，用这一轮的改动修，修完不再开第四轮；只写进判决交用户表、标「零轮」。
- U1 判出某条 G 没做到 ⇒ 补做。
- U1 判出前两轮判决引的数在日志里找不到 ⇒ 那一条判决改写。
- U3 造得出 scope 活下来 ⇒ 记欠账（第四十三节），这一轮不修。
- 一条腿报「没打中」的格只抽了一次样，不支撑否定结论。

交付：腿的报告写 `research/prompts/tooling-detached-run-r3-<模型>-output.md`，分段落盘，每次写进文件不超过 150 行，第一段排他新建，回复只写短句；攻方的复现脚本、样本与输出放 `research/prompts/tooling-detached-run-r3-opus-model/`。草稿放腿自己的目录 `/tmp/claude-1000/<腿名>/`。引规则写规则文件名加那份文件自己的行号。不在主仓里改任何文件。钩子对子 agent 拒 `detached-run.sh start` / `stop` 与各份 `--selftest`：要真起单元的写成复现脚本（单元名前缀 `singlefs-detached-opusr3-`、临时登记目录、收尾停干净核零残留），交主 agent 跑；钩子的判定在副本里喂 JSON 输入判，不执行。不碰正在跑的 `singlefs-detached-broad-mutation-*` 两个单元与 `~/.local/state/singlefs/detached-runs/`。
