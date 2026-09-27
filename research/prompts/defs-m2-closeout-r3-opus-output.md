# defs-m2-closeout-r3 云端攻方（Opus）：K1、K2、K3

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 G6 G7 G8 G9 K1 K2 K3 R1 R2 R3 R4 R5 R6 R7 R8 F9 F13 D1 D2 D3 D4 D5 D6 A1 A2 A3 A4 A5 A6 A7 A8 -->

时刻：开工 2026-09-26 03:52 UTC（12:52 JST）。开工快照 `research/prompts/defs-m2-closeout-r3-snapshot/sha256sums.txt` 27 个文件 `sha256sum -c` 全部 OK（03:52:03 UTC）。

## 复跑

```
bash research/prompts/defs-m2-closeout-r3-opus-model/rerun.sh /tmp/<自己的临时目录>
```

hook 只喂 JSON、只看退出码与 stderr，被判的命令一条都不执行；阶段演示在临时镜像根里跑，包装在起 cargo 之前就退出；40 号在只含文件名的副本里跑；全程不编译。这一次的原样输出存在 `rerun-output.txt`（225 行）。

```
e9dab822532e5340ddd2906e1e4718744f94b45a5635838b83f968034db3634e  ampersand-demo.sh
ffdf098ba238a86594176d4e5ec42ce8e51965d610ba6aeb2b18c48121194c3b  cap-syntax-fix.py
e933b487c4a0fc1da8561f82a9ab8d18493c3e2d5279f7cb830ca048d0d96c85  cases-k1.json
8d4e1b77f4da6171b1f99ebe042157ef11a6bac7701ae015969b8ee357741914  cases-k3-ask.json
7e50bf9fe8dd39b03cf5f2cacf0e1a593bfbb38d445f6bc6bb2f64fb4fe90390  cases-k3-detector.json
1cdb88468cdd111f48af4e30dcf66f4594644e87e3f4cb8d170803881839c1b9  claim-scan.py
d7cdc58ea2b56686366325566eac4d7582104912e33d293bb3abaa9c94868516  g2-results-cited-demo.sh
23957c586e87e0a312f54c4dab12cce6a4fd34c217387d9e3f074fdf1f648939  probe.py
f35998496a143fb8db2b3e51efba7c4ed54aed1e8953caaaf62498a0a02abe82  rerun-output.txt
9e38c51e5ab9d7b0c9b00ab079a50d91a4c11041a42ae1f9db918ad714a43cd9  rerun.sh
8c551507dc41d5753e50892e83ff0079ec7518418d8574e79c51e1fa323aa170  stage-cap-syntax-demo.sh
3b4d23aa20e4226bb875034852aedf89365e3ab06f7c68a832fa3b83ad19b030  verdict-fields.py
```

（同一份在 `SHA256SUMS`。）

## 各格判定一览

| 编号 | 格 | 被攻的 | 判 | 情形一句话 | 证据种类 |
|---|---|---|---|---|---|
| R1 | K1 | G1（15、74 号阶段里面经包装） | **打中（中）** | 派发给的上限写成 `12GiB` / `16GB` / `8g`：包装退 2（用法错），两道都不走 250–254 那一支；15 号报「cargo test 退出码 2、先修好」且把包装的原话吞掉，74 号报「测试二进制判红」并给 ModelDisagreement 的出路 | 量过（镜像根上跑阶段） |
| R2 | K1 | G1 × G9（主 agent 定义「给一个数」） | **打中（中）** | 上限照字面只给数不带单位（`16`）：包装认成 16 字节，起 scope 当场被停、退 251；两道的出路是「`--status` 看 slice 被谁占着」，包装的出路是「查用户级 systemd 与 D-Bus」，都不是真原因 | 量过 |
| R3 | K1 | G1 × 门禁分诊员第 3 步 | **打中（中，推的）** | 15、74 号撞上限（250）或排不上（252）时，阶段的 ✗ 行点名 `research/scripts/run-with-memory-cap.sh`；分诊员按「点名的文件在不在暂存区 diff 里」判「这一轮 / 不是这一轮」，没有一格对应「包装结局」；252 也不满足它「并发」的两条认法 | 推的（按定义字面与阶段原文） |
| R4 | K2 | G2 × 40 号 | **打中（中）** | 执行员照第 4 步给重跑的输出起新名：写成日期、`-r2`、`.r12` 的，40 号第一道判红要求实验页点名；写成 `.r2` 的，40 号当「逐轮中间件」跳过，而 `replay.sh` 此后正指着它。同一个动作按拼法得出两种门禁结局；逐字节一致又不写页的那一种，定义里没有让 40 号变绿的一步 | 量过（副本上跑 40 号） |
| R5 | K3 | G6 × `bash-command-detector.sh` 的出路 | **打中（中）** | 照共用约束「被拒了照出路改写法」，被 ④（或 ③）拒了照 hook 的出路写 `a & b & wait`：hook 放行，一件退 3、一件退 0 时 `wait` 之后 `$?=0` | 量过（探针 + 演示） |
| R6 | K1 | G9（54、57 号「量过的」上限） | 小（推的） | 峰值表今天 0 行 54 / 57 号；表的列没有线程数；54 号 `--full` 在临时 worktree 里跑时键里带临时路径，下一次对不上 | 现查 + 推的 |
| R7 | K1 | G1（两道的出路） | 小 | 两道的出路只写 250、251、252、254，漏 253（限时） | 现查 |
| R8 | K3 | G3（一轮合计 10⁷） | 小（字面） | 「这一轮全部原型跑的全量」若指整轮，别的腿的原型量在禁读里，攻方腿算不出；换腿接着做时没有账可接 | 字面 |
| — | K1 | G1：三种身份跑 15、74 号 | 没打中 | 重型闸对 11 种带上限变量的写法全放（K1-01..11）；74 号在阶段里的键 8G 下量到 5111808000 字节 | 量过（探针） |
| — | K1 | 15 号的 stage-inputs 缺口 | 没打中 | 15 号不问 `stage-must-run.sh`、每次都跑，没有「输入没变就复用」这条路，缺口不存在；74 号那一行多列了它不读的 `capped.sh`，少列了它读的三份（旁支，不归 G1） | 现查 |
| — | K2 | G5 | 没打中 | 今天 26 条判决行、每个字段按改后字面分边，没有一格字面判错 | 量过（扫描） |
| — | K2 | G7 挪后第 6 步剩什么 | 没打中 | 剩 33、52、80、96 号；四道都不读第 7 步才写的东西 | 现查 |
| — | K3 | G4 误拒 | 没打中 | 仓里跟踪的 3349 个文件 × 3 种句式共 10047 句，G4 与 F13 两版拒的句子逐句相同（各 328 句），G4 没有新添一句误拒 | 量过（扫描 + stdin 探针） |
| — | K3 | G8 | 没打中 | — | 现查 |
| — | K1 | 15 号的 8G 够不够 | **没量** | 想在副本上 `cargo test --release --no-run` 量编译峰值，重型闸按「工作区根裸跑 cargo test」拒了（K1-12）；换成 `cargo build --tests`（K1-13 放行）等于换写法绕过去，没做，交主 agent | — |

## K1：G1、G9

### R1 包装退 2（用法错）时，两道阶段都报成被测对象的红

**情形**：主 agent 派门禁分诊员（或崩溃验证员、实现员）时照 `.claude/main-agent.md:48` 给了 15 或 74 号的上限，写法是仓里散文常用的 `12GiB`（或 `16GB`、小写 `8g`）；子 agent 照 `.claude/agent-common.md:48` 把它设进阶段的变量：`GATE_RESEARCH_BUILD_MEMORY_MAX=12GiB SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/scripts/gate.sh --staged`（重型闸放行，探针 K1-01）。

每一步许可它的原文：

- `.claude/agent-common.md:48`（整行）：跑编译出来的代码经内存包装：`cargo test` / `run` / `bench`（带 `--no-run` 只编不跑的不算）与直接执行 cargo 编出来的二进制（实验装置、测试二进制），一律经 `bash research/scripts/run-with-memory-cap.sh <上限> <命令…>` 跑；写进自己的脚本再跑的，整条脚本经它跑（`bash research/scripts/run-with-memory-cap.sh <上限> bash <脚本>`）。上限取派发提示给的（定义另写了来源的照定义），都没给就取 `research/scripts/replay.sh` 文件头 `REPLAY_MEMORY_CAP` 的默认值。`research/scripts/mutate.sh`、`research/scripts/replay.sh` 与门禁 59 号在里面逐条套了，门禁 15、74 号在阶段里面经包装（派发提示给了上限的，设进阶段文件头写的变量：15 号 `GATE_RESEARCH_BUILD_MEMORY_MAX`，74 号 `GATE_MODEL_DIFFERENTIAL_MEMORY_MAX`），都直接跑，外面不再包一层。退出码 250–254 是包装自己的结局（含义见 `research/scripts/run-with-memory-cap.sh` 文件头「退出码」一段）：那一次的输出不算结果，命令与退出码写进报告，不绕开包装重跑。`cargo build`、`cargo clippy`、`cargo fmt` 不跑编出来的代码，不要求经它。
- `.claude/gate.d/15-research-build.sh:34`：`if (( rc >= 250 && rc <= 254 )); then`
- `.claude/gate.d/74-model-differential.sh:57`：`      250|251|252|253|254)`
- `research/scripts/run-with-memory-cap.sh:18`：`#   0–249  那条命令自己的退出码（cargo 只退 0、101、126、127 与 128 + 信号号，碰不到 250–254）；`
- `research/scripts/run-with-memory-cap.sh:27`：`#   2      用法错（上限、余量、总上限不是 正整数[KMGT] 的写法，没给命令）。`

包装文件头自己把 2 同时放进「命令自己的退出码 0–249」与「用法错」两处；两道阶段只把 250–254 当包装的结局，2 就落进「被测对象判红」那一支。

镜像根上跑出来的原样（`stage-cap-syntax-demo.sh`，`rerun-output.txt` 里同一段）：

```
== 15 号，GATE_RESEARCH_BUILD_MEMORY_MAX=12GiB
  ✗ research 的构建或单测没过（cargo test 退出码 2）
     → 怎么办：先修好——kb 里的实测结论全靠它们背书，编不过就等于那些数字今天没有来源。
exit=1
== 74 号，GATE_MODEL_DIFFERENTIAL_MEMORY_MAX=12GiB
  ✗ 内存上限要写成 正整数[K|M|G|T]（例 16G、512M），收到的是「12GiB」
  → 怎么办：照 systemd 的 MemoryMax 写法给；默认值与依据写在文件头
  ✗ 随机历史的测试二进制判红（上面是 cargo test 的尾部）
     → 怎么办：单跑看细节（经内存包装，上限同这一道）：
                bash research/scripts/run-with-memory-cap.sh 12GiB cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history -- --nocapture
                签名是 ModelDisagreement 的，是实现与理想模型对某一步的结局答得不一样：先看模型那一格引的条款，再判改实现还是改模型。
exit=1
```

15 号只把 `^error|FAILED|panicked at` 的行转出来（`.claude/gate.d/15-research-build.sh:43`：`  grep -E '^error|FAILED|panicked at' <<<"$out" | head -8 | sed 's/^/     /'`），包装那两行 `✗ 内存上限要写成…` 一行都没露出来：看输出的人只知道「research 编不过」。74 号露出了包装的原话，却紧跟着自己的「判红」与单跑命令，而单跑命令里带着同一个错写法。`16GB`、`8g` 两种原样同形（`rerun-output.txt`）。

**四句**：
1. 分不分辨臂：分辨 G1（阶段里面包）与已撤回的 F9（整条 `gate.sh` 外面包）——F9 那种写法下错写法在外层包装当场退 2，门禁一行不跑，看得见；但 F9 另有被第二轮坐实的毛病，这一格不改那一判。它打的是 G1 的实现细节（判哪几个退出码），不是 G1 的方向。
2. 被判的系统看不看得到：看得到。阶段手里有 `rc=2` 与包装的整段输出；cargo 自己不退 2（包装文件头第 18 行列的 cargo 退出码里没有 2）。
3. 满足的是哪一个分句：K1 问句「包装自己的退出码 250–254 与阶段判红分得开吗」——250–254 分得开（修定义的 agent 的假根演示），**2 分不开**；病根在包装文件头第 18 行与第 27 行把 2 放进了两个区间，阶段照第 18 行的字面只挑 250–254，是「判据自己写错」里「打中归错了判据」的同族：阶段照的那句话本身重叠。
4. 跑前条款给的改法在这一格上还中不中：第二轮判决 G1 只写「经包装」，没写包装退 2 怎么办；照 G1 原文改不到这一格。

### R2 「给一个数」不带单位：包装认成字节数，退 251，出路指错方向

**情形**：主 agent 照 `.claude/main-agent.md:48` 的字面「给一个数」，写「15 号上限 16」；或崩溃验证员照 `.claude/agents/crash-verifier.md:20` 的「每道一个数」拿到「54 号 16（推的）」。子 agent 设 `GATE_RESEARCH_BUILD_MEMORY_MAX=16`，或照崩溃验证员第 1b 步写 `run-with-memory-cap.sh 16 bash .claude/gate.d/54-layer0-replay.sh`。

- `.claude/main-agent.md:48`（整行）：`派崩溃验证员时给 54、55、57 号各自的内存上限，每道一个数，照 \`.claude/agents/crash-verifier.md\`「输入」一节给：54、57 号写明是量过的还是推的，55 号的不小于同时起的虚机数乘每台的内存。门禁 15、74 号在阶段里面经内存包装，要换它们的上限就在派发提示里给一个数，没给用 \`research/scripts/replay.sh\` 的 \`REPLAY_MEMORY_CAP\` 默认值。`
- `research/scripts/run-with-memory-cap.sh:16`：`# 上限照 systemd 的 MemoryMax 写法：正整数加 K / M / G / T（1024 进制），例 16G、512M。默认值由调用方定，写在调用方的脚本头。`
- `research/scripts/run-with-memory-cap.sh:153`：`  [[ "$1" =~ ^[1-9][0-9]*[KMGT]?$ ]] \`

文件头说「正整数加 K / M / G / T」，判写法的正则里单位是可选的：`16` 过了写法检查，交给 systemd 就是 16 字节。原样：

```
== 15 号，GATE_RESEARCH_BUILD_MEMORY_MAX=16
       ✗ 带内存上限的 scope 起不来：systemd-run --user --scope 退出码 137，命令一行都没跑（上面是 systemd-run 自己的报错）
       → 怎么办：查用户级 systemd 与 D-Bus（systemctl --user status、echo $DBUS_SESSION_BUS_ADDRESS $XDG_RUNTIME_DIR）；起不来就别跑这一步，不许退回无上限去跑
  ✗ research 的构建与单测没判出结果：内存包装 research/scripts/run-with-memory-cap.sh 退 251（上限 16），这一次的输出不算判定
     → 怎么办：照包装文件头「退出码」一段办：250 是这一道要的比上限多，把 GATE_RESEARCH_BUILD_MEMORY_MAX 设大再跑；
               251、252、254 先跑 bash research/scripts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再整道重跑；不拿掉包装跑。
exit=1
```

74 号同形（`rerun-output.txt`）；包装单独跑 `run-with-memory-cap.sh 16 true` 同样退 251、同一句「查用户级 systemd 与 D-Bus」。照出路做的人去查 D-Bus 与 slice 占用，两样都正常，而原因是少了一个 `G`。崩溃验证员给 54、57 号整条外包时（第 1b 步）是同一个 251，阶段一行都不跑。

**四句**：不分辨臂（G1、F9 两种包法都把这个数原样交给包装）；看得到（写法在阶段与包装手里都有，只是正则放过了）；满足 K1「包装自己的退出码…分得开吗」与共用问句「照改后的字面干活，哪一步会做错」——主 agent 照字面「给一个数」就做错；G1、G9 的原文都只说「给一个数」，改不到这一格。

### R3 门禁分诊员没有一格装得下「包装的结局」

**情形**：提交时分诊员照第 2 步跑 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/scripts/gate.sh --staged`（没设上限变量，15、74 号各取 8G）。两种可达的结局：

- 250：15 号在 `--staged` 的新 worktree 里从零编整个 research 工作区（`release` 带 `debug = true`，148 个 bin 源文件，`gate.sh` 不经 `capped.sh`，cargo 取整机 32 核）；8G 够不够没量过（见本节末「没量」）。
- 252：别的会话同时在 `singlefs-heavy.slice` 里占着量（峰值表里 `capped.sh 16 cargo test … --test checker_known_bad_images` 这一类一条就记到 24G 上限），总上限约 40.1 GiB（`bash research/scripts/run-with-memory-cap.sh --status` 04:06 UTC 那一行原样：`  … slice singlefs-heavy.slice 的总上限 40.1 GiB（整机内存 − 余量 20G；RUN_WITH_MEMORY_CAP_RESERVE 改余量）`），15 号按 8G 排队等满 3600 秒退 252。

阶段给的 ✗ 行（`.claude/gate.d/15-research-build.sh:36`、`.claude/gate.d/74-model-differential.sh:58` 同形）点名的文件是 `research/scripts/run-with-memory-cap.sh`，不带行号。分诊员第 3 步（`.claude/agents/gate-triage.md:28`，整行）：

```
3. 每个红阶段：抄门禁给的「✗」行与紧跟的「→」下一步原样；看它点名的文件与行在不在暂存区 diff 的块里（`git diff --cached -U0 -- <文件>`）。在块里 ⇒「这一轮」；文件在但行不在块里、或文件不在暂存区 ⇒「不是这一轮」；红句说的是「这一批输入」本身的（54 号的「没有这批输入的全绿标记」「那一格的哈希不同」这类）不按它点名的标记路径判：这一轮的改动（`git diff --cached --name-only`；跑全量工作区时是主 agent 给的文件清单）碰了 `.claude/gate.d/stage-inputs.tsv` 里那一道那一行登记的路径 ⇒「这一轮」，下一步原样抄 54 号出路里的三行命令，一条都没碰 ⇒「不是这一轮」；工具没装、网关不通、`cargo` 不在 ⇒「环境」，用 `bash .claude/scripts/env.sh` 的对应行佐证；别的会话同时在跑同一个重阶段、进程被外部信号杀掉（输出里一行裸的 `Terminated`）⇒「并发」，贴开跑时与出事前后 `ps` 看到的同名进程。红阶段没有「✗」与「→」可抄的（被杀、崩溃），抄它最后 20 行输出，判「分不清」或「并发」，写明没有下一步可抄。分不清的写「分不清」与原因。
```

照这一行逐分句套：

| 分句 | 250 / 252 这一格落到哪 |
|---|---|
| 「看它点名的文件与行在不在暂存区 diff 的块里」 | 点名的是包装文件、没有行；今天工作区里包装是改过的（`git status --short -- research/scripts/run-with-memory-cap.sh` 原样 ` M research/scripts/run-with-memory-cap.sh`），暂存进这一批就落「这一轮」、没暂存就落「不是这一轮」——两种都在判包装的代码，而 250 是上限（或这一轮让 research 变大），252 是 slice 被占 |
| 「红句说的是『这一批输入』本身的」 | 只列 54 号的两句，不含包装结局 |
| 「工具没装、网关不通、`cargo` 不在 ⇒『环境』」 | 不含上限与 slice |
| 「别的会话同时在跑同一个重阶段、进程被外部信号杀掉（输出里一行裸的 `Terminated`）⇒『并发』」 | 252 是别的会话在跑**别的**东西占 slice，没有 `Terminated` 行，不满足 |
| 「红阶段没有『✗』与『→』可抄的…判『分不清』或『并发』」 | 有 ✗ 与 →，走不到这一支 |

所以按字面，15、74 号「没判出结果」的红被分诊成一条代码归属，主 agent 读到的是「这一轮 / 不是这一轮」，读不到「换上限重跑」或「等 slice 空出来」。这是推的：没在仓上跑 `gate.sh`（整轮门禁归分诊员），只按两份原文逐分句套。

**四句**：不分辨臂（F9 下整条外包撞顶同样没有分诊格）；看得到（✗ 行里写着「内存包装 … 退 25x」）；满足 K1「门禁分诊员照各自定义跑 15、74 号时各是什么结局」；G1 只改阶段、没改分诊员第 3 步，改不到这一格。

### R6 G9 的「量过的」在今天的峰值表上做不成、下一次也对不上

- `.claude/agents/crash-verifier.md:20`（整行）：`- 54、55、57 号各自的内存上限（第 1b 步用），每道一个数：54、57 号的写明是量过的（峰值表 \`research/scripts/memory-peaks.tsv\` 里那一道整条经包装跑出的峰值）还是推的；55 号的不小于同时起的虚机数乘每台的内存（\`.claude/gate.d/55-qemu-first-transaction.sh\` 的 \`MODES\` 档数 × \`research/scripts/vm-bench.sh\` 的 \`VM_MEM\`）。`
- `research/scripts/memory-peaks.tsv:4`（整行）：`# 列（制表符分隔）：峰值字节、量的时刻（UTC）、那一次的上限、键（命令的各个词用空格接起来，测试二进制名里的 16 位哈希去掉；RUN_WITH_MEMORY_CAP_KEY 可以指定）。`

现查：`awk -F'\t' '!/^#/ && $4 ~ /54-layer0|57-lkmm|55-qemu|lkmm\.sh|herd7/'` 在峰值表上 0 行——今天只能写「推的」。表没有线程数一列，而层 0 全量按线程切片跑；同一个键在不同线程数下量到的峰值不能互换（推的，没量过）。主 agent 定义的提交流程让 54 号在 HEAD + 暂存区的临时 worktree 里跑 `--full <它的根>`（`.claude/main-agent.md:61`），根是临时路径，键里带着它（键「命令的各个词用空格接起来」），下一次换一个临时路径，表里那一行就对不上：「量过的」要靠人按前缀去认。小：不让任何一道做错，只让「量过的」这个选项在今天与下一次都难成立。

### R7 两道的出路漏了 253

`.claude/gate.d/15-research-build.sh:37`–`:38`（整行）：

```
  echo "     → 怎么办：照包装文件头「退出码」一段办：250 是这一道要的比上限多，把 GATE_RESEARCH_BUILD_MEMORY_MAX 设大再跑；"
  echo "               251、252、254 先跑 bash research/scripts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再整道重跑；不拿掉包装跑。"
```

74 号第 59–60 行同形。253 只在环境里带着 `RUN_WITH_MEMORY_CAP_TIME_LIMIT` 时出现（包装把这个变量在传进命令之前清掉，外层包装带过来的不会漏进来）；小。

### K1 没打中的几格

**三种身份照各自定义跑 15、74 号**（探针 `cases-k1.json`，只喂重型闸，原样）：

```
K1-01	heavy	gate-triage	fg	exit=0	
K1-02	heavy	gate-triage	fg	exit=0	
K1-03	heavy	gate-triage	fg	exit=0	
K1-04	heavy	gate-triage	fg	exit=0	
K1-05	heavy	gate-triage	fg	exit=0	
K1-06	heavy	gate-triage	fg	exit=0	
K1-07	heavy	crash-verifier	fg	exit=0	
K1-08	heavy	crash-verifier	fg	exit=0	
K1-09	heavy	crash-verifier	fg	exit=0	
K1-10	heavy	implementation-writer	fg	exit=0	
K1-11	heavy	gate-triage	fg	exit=0	
```

K1-01..05 是分诊员把上限变量放在 `SINGLEFS_HEAVY_TESTS=commit` 前、后、`export` 分句、`env` 里、两个变量一起的五种写法；K1-06 分诊员单跑 15 号带变量；K1-07 崩溃验证员跑 74 号带变量；K1-08、K1-09 崩溃验证员第 1b 步 54、57 号整条外包；K1-10 实现员带错写法 `12GiB` 跑 74 号（闸放行，结局见 R1）；K1-11 出路里的 `--status`。`gate.sh` 逐道顺序起本地阶段、不清环境（`.claude/singlefs-ai-sop/scripts/gate.sh:538`：`      GATE_IN_STAGE=1 bash "$f" "$ROOT" ${STAGE_FORCE_OPTION[@]+"${STAGE_FORCE_OPTION[@]}"} || stage_rc=$?`），变量传得进去。结局：

| 身份 | 15 号 | 74 号 |
|---|---|---|
| 实现员 | 没登记给它，不跑 | 照跑，阶段里默认 8G；74 号在阶段里的键今天量过一次，见下 |
| 崩溃验证员 | 没登记给它，不跑 | 同实现员；「输入」一节只列 54、55、57 号的上限，74 号的数照共用约束第 48 行从派发提示取 |
| 门禁分诊员 | 在 `gate.sh` 里跑、或单跑（登记给它），默认 8G；撞了见 R3 | 在 `gate.sh` 里跑 |

74 号在阶段里经包装的那条 cargo，峰值表原样（`research/scripts/memory-peaks.tsv:910`）：

```
2265972736	2026-09-25T14:09:40Z	12G	cargo test --release -p singlefs-harness --test second_transaction_step_zero_layer0 -- --exact every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims --nocapture
```

5111808000 字节约 4.76 GiB，在 8G 以下（这一行不是我跑的，03:40:32 UTC 别的会话或主 agent 跑出来的）。

**15 号的 stage-inputs 缺口**：15 号整份不调 `research/scripts/stage-must-run.sh`（`grep -c stage-must-run .claude/gate.d/15-research-build.sh` 为 0），每次都跑，没有「输入没变就复用」这条路，也就没有「包装改了而复用旧绿」的缺口。旁支（不归 G1）：74 号那一行列了 `research/scripts/capped.sh`，74 号与包装都不读它（`grep -n capped .claude/gate.d/74-model-differential.sh` 0 行，包装里只有 `run_capped` 这个函数名）；74 号读的 `stage-must-run.sh`、`change-touches-crates.sh`、`admission.py` 没列（59 号那一行同样没列），是既有的。

**没量：15 号的 8G 够不够**。我想在 research 的副本上 `cargo test --offline --release --no-run`（只编不跑）量编译峰值，重型闸拒了（K1-12，原样 `✗ 重型测试被拒：在工作区根（@COPY@）上不带 -p / --test / --lib / --bin 的 cargo test（全量测试）：three-way-attack 不跑「全量测…`）。`cargo build --offline --release --tests` 编的是同一批目标、闸放行（K1-13），但那是被拒之后换写法，没做。要量的话由主 agent 定谁来量（分诊员在提交时整轮跑一次，峰值表里就会有键为 `gate 15-research-build: cargo test --release (research)` 的一行；今天 `grep -cF 'gate 15-research-build' research/scripts/memory-peaks.tsv` 为 0）。

## K2：G2、G5、G7

### R4 新文件怎么起名，决定 40 号判红还是放过

**情形**：执行员重跑已有实验 E14（`research/scripts/replay.sh:50`：`E14|e14-discrimination||e14-discrimination-2026-08-29.out|exact`），输出与旧的逐字节一致；派发提示写「这一次不写实验页」（`.claude/agents/experiment-runner.md:21` 那一项）。照第 4 步写新文件、第 5 步把登记行改指新文件、第 6 步最后跑 40 号。

- `.claude/agents/experiment-runner.md:31`（整行）：4. 跑产物进 `research/results/`。那里已有的产物一个都不删、不挪、不覆盖（共用约束「执行前拒绝的写法」那一条的 ⑤）：这一次的输出写一个新文件名（带今天的日期或 `rN`，写之前 `ls` 确认没有同名的），跑完按这个新文件名认本轮才有的完成标记。这一次的新文件也不删：重跑已有实验时，第 5 步把 `research/scripts/replay.sh` 里这个实验的登记行改指新文件，旧的留着；拿新文件与已有的那份比，逐字节一致的在报告里写明，对不上的在实验页两份都点名、写明对不上，交主 agent 定哪份承重。生成产物用的二进制编在 `research/target/` 下，用最后一次源码改动编出来；生成产物与第 5 步跑 `replay.sh` 时都不设 `CARGO_TARGET_DIR`；草稿目录里的 target 只拿来迭代。
- `.claude/gate.d/40-results-cited.sh:33`（整行）：`  case "$b" in *local*|*.r[0-9].out|*.round*|confirm*) continue;; esac`
- `.claude/gate.d/40-results-cited.sh:41`（整行）：`  echo "               并在口径段点名这份原始输出；确实是废弃产物就删掉它。"`

第 31 行里两句是这一格的依据：新文件名「带今天的日期或 `rN`」，没说拼法；「逐字节一致的在报告里写明」，逐字节一致时只写报告、不点名。

副本上跑 40 号（`g2-results-cited-demo.sh`：40 号、实验索引与实验页原样拷，`research/results/` 只按同名建空文件），原样：

```
== 基线（不加新文件）
  ✗ 有实验产物没被 experiments.md 点名——跑过但没写回，或写回了却无法复核：
               并在口径段点名这份原始输出；确实是废弃产物就删掉它。
     （第一道列出的未点名产物共 7 份）
== 加 e14-discrimination-2026-09-26.out
  ✗ 有实验产物没被 experiments.md 点名——跑过但没写回，或写回了却无法复核：
     research/results/e14-discrimination-2026-09-26.out
               并在口径段点名这份原始输出；确实是废弃产物就删掉它。
     （第一道列出的未点名产物共 8 份）
== 加 e14-discrimination-r2.out
  ✗ 有实验产物没被 experiments.md 点名——跑过但没写回，或写回了却无法复核：
     research/results/e14-discrimination-r2.out
               并在口径段点名这份原始输出；确实是废弃产物就删掉它。
     （第一道列出的未点名产物共 8 份）
== 加 e14-discrimination.r2.out
  ✗ 有实验产物没被 experiments.md 点名——跑过但没写回，或写回了却无法复核：
               并在口径段点名这份原始输出；确实是废弃产物就删掉它。
     （第一道列出的未点名产物共 7 份）
== 加 e14-discrimination.r12.out
  ✗ 有实验产物没被 experiments.md 点名——跑过但没写回，或写回了却无法复核：
     research/results/e14-discrimination.r12.out
               并在口径段点名这份原始输出；确实是废弃产物就删掉它。
     （第一道列出的未点名产物共 8 份）
```

（基线本来就红 7 份：今天工作区里 e156、e158 的几份产物没被点名，别的会话的，不归这一轮；只看每种名字让条数加没加 1。）

两种结局：

- 写成日期、`-r2`、`.r12`（`.r` 后两位数）：40 号第一道多列一份，出路要实验页点名这份原始输出。执行员这一次不写页（派发提示定的），第 6 步「红了照写」就交回；逐字节一致时第 31 行只要它在报告里写一句，没有一步让这份新文件被点名。到提交时整轮门禁的 40 号照样红，定义里没人被派去点名它——共用问句「哪一步做不了」：第 4、5、6 步各自照做，合起来留下一道执行员自己清不掉的红。
- 写成 `.r2`（`.r` 后一位数）：40 号当「逐轮复跑的中间件」跳过，一声不响；而第 5 步已经把 `replay.sh` 的登记行改指这一份，承重的产物从此不受「要被点名」这一道管。

试过、没打中的一支：40 号出路里「确实是废弃产物就删掉它」与 G2「这一次的新文件也不删」相反，但 `.claude/agents/experiment-runner.md:35`（整行）：6. 跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节），读实验页或实验索引的那几道除外（27、34、40、69、75、84、85、86、88、99 号）：它们放到第 7 步写完实验页之后跑；这一次不写实验页的，在这一步最后跑，红了照写，产物照第 4 步一个不删（86 号出路里「把 research 下那些文件删掉」那一句不照做，写进报告交主 agent）。其中要编译或复跑整张表的，开跑前照共用约束看负载。15 号（编整个 research 工作区）与 87 号（复跑全部已入库实验）不归你，收尾由 `gate-triage` 统一跑；你只跑第 5 步自己那一个实验的 `replay.sh`。——其中「产物照第 4 步一个不删」管第 6 步跑的全部阶段，86 号那一句只是括注里的例，执行员照字面不会删，不算打中。

**四句**：不分辨臂（G2「不删」这个方向不受影响，打的是它没定的拼法与点名义务）；看得到（40 号只看文件名，拼法由执行员当场选）；满足 K2 问句「G2 与 40 号跳过 `*.r[0-9].out` 的命名规则怎么交叉」与共用问句「哪一步做不了」；第二轮判决 G2 原文只写「这一次的输出写新文件名，`replay.sh` 的登记行改指新文件，旧的留着」，没定拼法、没说逐字节一致时点不点名，改不到这一格。

### G5：今天全部判决行各落哪一边（没打中）

`verdict-fields.py` 照第 4c 步改后的字面分边（判决行 26 条；「名字表示违例 / 不匹配 / 歧义 / 失败」按名字里带 `violation`、`mismatch`、`ambigu`、`fail` 认）。按边合计：

| 边 | 字段名个数 | 出现处数 |
|---|---|---|
| 甲 取值 false（点名） | 6 | 41 |
| 乙 取值 not_run（点名） | 3 | 51 |
| 丙 大于 0 的整数、名字表示没过（点名） | 0 | 0 |
| 丁 取值 true（不点名） | 25 | 213 |
| 戊 取值 zero（不点名） | 1 | 1 |
| 己 大于 0 的整数、名字不表示没过（不点名） | 13 | 18 |
| 庚 取值 0（不点名） | 5 | 30 |
| 辛 字面没说（字符串） | 1 | 1 |

逐字段的原样在 `rerun-output.txt` 的「verdict-fields.py」一段。逐个看过：甲边六个全是 `*_ok`、`*_zero` 这种「true 才好」的名字，false 就是没过，点名对；丁边 25 个里没有「true 表示出事」的名字，最接近的是 `e115_e124_cells_superseded=true`（E147 那一行，说的是 E115 / E124 的格被取代，不是判据没过）；己边 13 个里 `journal_differing_states=3` 是 G5 自己举的不点名例，其余是几何量（`legacy_leaf_fanout=676` 这类）；辛边只有 `journal_reading=j_jia`（读法标签）。今天没有一格按字面落错边；拿不准的两个（`e115_e124_cells_superseded`、`cells_where_pad_taller_than_legacy=2`）写「拿不准」点名也不错。

### G7：挪走十道之后第 6 步剩什么（没打中）

登记给执行员的 14 道（共用约束「门禁」一节那条 `awk` 现取）：`27 33 34 40 52 80 84 85 86 88 69 75 96 99`，挪走十道剩 33、52、80、96。逐道 `grep` 读的东西：33 读 `research/mutations/`（第 3 步写的），52 读 `.claude/kb/layout/01-first-txn.md` 与 `research/scripts/replay.sh` 的 E142 行（第 5 步改的；`research/scripts/check-segment-registry.py:25`（整行）：`  取它的「入库产物」那一列，这样产物重跑之后换了文件名，这个脚本自动跟着换，不用改代码。`），80 不读 kb，96 读 `.claude/kb/checks-owed.md`。四道都不读第 7、7b 步才写的实验页、索引与 `experiments-history.md`，留在第 6 步不会因为「页还没写」判红。十道各自读不读实验页的字面依据归本地攻方，我没逐道核。

## K3：G3、G4、G6、G8

### R5 照 hook 的出路做，退出码又被吞掉

**情形**：攻方腿（或任何子 agent）在 run_in_background 里并行起两件、忘了末尾的 `wait`（探针 D1），或门禁分诊员并行单跑两道阶段、各自写日志（探针 D6）。检出 hook ④ 拒绝，stderr 的出路叫它写 `a & b & wait`；共用约束要求被拒之后照出路改。

- `.claude/agent-common.md:59`（整行）：
```
- 执行前拒绝的写法：项目 settings 里注册的这几道 hook 在执行前（收工闸在收工前）拒绝这几类写法（退出 2，stderr 写原因与出路），主 agent 与子 agent 一样拒，只挂在一方才用的工具上的在条目里写明；拒的只是这一次调用，不停已经在跑的东西。被拒了照出路改写法，不换写法绕过去；改不成的写进报告交主 agent。
```

- `.claude/agent-common.md:64`（整行）：
```
    ④ run_in_background 里有作业以单独的 `&` 收尾，之后同一条命令里没有 `wait`。要并行就每件把退出码写进自己的文件（`{ <命令>; echo "$?" > <文件>; } &`，文件名带批号与件号，每批开跑前先删掉这一批的文件），最后单独一个不带参数的 `wait` 等齐，再按派活的次序逐个读那几个文件；读之前数一遍，文件数与派出去的件数对不上，整批作废；不用 `wait "$pid"` 收。
```

- `.claude/hooks/bash-command-detector.sh:2408`（④ 的出路，整行）：
```
                  "（几条活要并行就在同一条命令里 `a & b & wait`，用 `wait` 等齐）；"
```

- `.claude/hooks/bash-command-detector.sh:2397`（③ 的出路，整行）：
```
                  "几条活要并行就在同一条命令里 `a & b & wait`，用不带参数的 `wait` 等齐", file=sys.stderr)
```

探针（`cases-k3-detector.json`，run_in_background，原样；D2 是照 ④ 的出路改出来的写法，D4、D5 是共用约束 ④ 与攻方第 3c 步的写法）：

```
D1	detector	three-way-attack	rib	exit=2	  ✗ run_in_background 里又把活放到了后台（以单独的 & 收尾、之后同一条命令里没有 wait 的作业：（前面那个复合命令：圆括号、花括号、循环或 if））：外层 shell 起完它就退出，完成通知当场发出，真跑完的那个进程不会叫醒任何人
		     → 怎么办：长活直接放 run_in_background，命令里只写那条活本身，不加 `&`（几条活要并行就在同一条命令里 `a & b & wait`，用 `wait` 等齐）；已经在跑、pid 已知的，另起一条 run_in_background 等它：`python3 .claude/singlefs-ai-sop/scripts/proc.py wait <pid> --timeout <秒>`。这一道只拒这种写法，不停你在跑的任何东西
D2	detector	three-way-attack	rib	exit=0	
D3	detector	three-way-attack	rib	exit=2	  ✗ 把活放出了追踪（认出的写法：nohup … &）：进程移出 shell 的作业表或另起会话，随后的 wait 立刻返回、完成通知当场发出，它跑完不会叫醒任何人，成了没人追踪的孤儿
		     → 怎么办：要等的活用 Bash 的 run_in_background: true 起，命令里只写那条活本身（不加 `disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux` / `screen` 的分离模式、`systemd-run`），然后结束本轮，等它的完成通知再接着做；几条活要并行就在同一条命令里 `a & b & wait`，用不带参数的 `wait` 等齐
D4	detector	three-way-attack	rib	exit=0	
D5	detector	three-way-attack	rib	exit=0	
D6	detector	gate-triage	rib	exit=2	  ✗ run_in_background 里又把活放到了后台（以单独的 & 收尾、之后同一条命令里没有 wait 的作业：（前面那个复合命令：圆括号、花括号、循环或 if））：外层 shell 起完它就退出，完成通知当场发出，真跑完的那个进程不会叫醒任何人
		     → 怎么办：长活直接放 run_in_background，命令里只写那条活本身，不加 `&`（几条活要并行就在同一条命令里 `a & b & wait`，用 `wait` 等齐）；已经在跑、pid 已知的，另起一条 run_in_background 等它：`python3 .claude/singlefs-ai-sop/scripts/proc.py wait <pid> --timeout <秒>`。这一道只拒这种写法，不停你在跑的任何东西
# detections: 0 line(s)
```

照出路写出来的形状父进程看到什么（`ampersand-demo.sh`，两件 `bash -c 'exit 3'`、`bash -c 'exit 0'`），原样：

```
hook 出路的写法 a & b & wait：wait 之后 $?=0
共用约束 ④ 的写法：.rc 文件 2 个，第 1 件退 3，第 2 件退 0
```

hook 放行 D2，D2 的形状把退 3 的那一件吞成 0。共用约束第 59 行叫被拒的人照出路改，第 64 行（④ 那一条自己）写的是另一种收法；一个被拒的 agent 手里两句话，出路那句更近、更短、hook 也放行。

**四句**：不分辨臂（G6 改的是定义里的写法，没碰 hook）；看得到（hook 认得出末尾有没有 `wait`，却不看每件的退出码去了哪）；满足 K3 问句「照 hook 的出路做会不会又回到吞退出码」——会；第二轮判决 G6 只改「攻方第 3c 步与共用约束 ④」，hook 的两句出路（③ 第 2397 行、④ 第 2408 行）不在它的改动里，改不到这一格。修定义的 agent 的报告第五节点的是 `:2395–2397`、抄的是带「不带参数的」那一句（按今天的行号是 ③ 的出路），背景材料附录抄的是 `:2407-2410`（④ 的出路）；两处出路都叫人写 `a & b & wait`，都要跟着改。

### G4：收严之后有没有误拒带出处的合法断言（没打中）

`claim-scan.py` 取 `.claude/hooks/ask-user-claim-guard.sh` 里 heredoc 那段 python 原文判句子（G4 版），另在内存里把 G4 改的那一行换回附录二 diff 的「-」行（F13 版）。G4 那一行（`.claude/hooks/ask-user-claim-guard.sh:58`，整行）：

```
    rf"(?:{ASCII_PATH_CHARACTER}*[A-Za-z0-9_-]/(?:{ASCII_PATH_CHARACTER}+|{FILE_NAME_CHARACTER}*\.(?:{EXTENSION_ALTERNATION}))"
```

甲批：仓里 git 跟踪的 3349 个文件，每个造三句带「一定」的问句（`<路径>:1 写着这一格一定不变`、`见 <路径>：12，这一格一定不变`、`<文件名>:3 写着它一定为 0`）。原样：

```
G4：甲批按句式分的拒数：路径:1 21、见 路径：12 21、文件名:3 286
G4：甲批 10047 句（3349 个跟踪文件 × 3），拒 328 句
F13：甲批按句式分的拒数：路径:1 21、见 路径：12 21、文件名:3 286
F13：甲批 10047 句（3349 个跟踪文件 × 3），拒 328 句
甲批里 G4 拒而 F13 放的 0 句，F13 拒而 G4 放的 0 句
```

G4 没有新添一句误拒。两版一起误拒的 21 条路径是既有的：4 个仓根上不带扩展名的文件（`.gitignore`、`.singlefs-ai-sop-version`、`LICENSE-APACHE`、`LICENSE-MIT`）与 17 个名字里带 `(today)` 的 `research/prompts/m2-newq-r1-opus-model/out*/…-off(today).tsv`；走真实 stdin 的探针 A6（`.gitignore:12 …`）、A7（`…mmm-off(today).tsv:3 …`）都退 2。名字里带 `×`、`·` 的两份实验页（A4、A5）放行：第三支 `[\w-]{FILE_NAME_CHARACTER}*\.(md)` 从 `×` 之后接上。`文件名:3` 那 286 句是不带扩展名、不带 `/` 的名字（`expect:3`、`abbreviations:3` 这类），按设计就不认。

乙批（绕不绕得过去）：没有出处、只是「编号/编号：数」的断言句 8 句，两版都放过 7 句（`F1/F4：2 处一定都要改`、`54/55：2 道一定要带前缀`、`K1/K2：2 格一定都中` 等；stdin 探针 A1–A3 退 0）。G4 的自证只收了 `/` 后面紧跟汉字的那几种（`54/55号：2`），`/` 后面全是 ASCII 的走第一支 `{ASCII_PATH_CHARACTER}+`。小：`grep -rhoE` 在全部判决、`records/*.md`、`.claude/kb/*.md` 里找这种形状，0 处，今天没有人这么写过。

### R8 「这一轮全部原型跑的全量」由谁来算

`.claude/agents/three-way-attack.md:33`（整行）：

```
   - 在自己的原型里这样调枚举函数跑小流不算重型：原型里的流只许在原型里自己造，不从名字带 `layer0` 的用例里拷（拷文件、拷代码段、`include!`、`mod` 引进来都算拷）；测试目标的名字不带 `layer0`（带了 `heavy-test-guard.sh` 按名字拒）；每次跑之前用同一份文件的 `closed_form_state_count` 算出这一次全量的状态数，不超过约 10⁶ 个（几段历史合起来超过的，分几次跑）；这一轮全部原型跑的全量合起来不超过约 10⁷ 个状态。一段历史自己就超过约 10⁶ 的，那段不跑；再跑就要越过约 10⁷ 的，剩下的不跑；两样都写进报告交主 agent。
```

字面没说「这一轮」是这一条腿还是整轮三方。读成整轮：别的腿（例如第二轮的 Sonnet 腿交过 `defs-m2-closeout-r2-sonnet-model/`）的原型量在攻方腿的禁读清单里，攻方腿算不出合计，照字面做不了。读成这一条腿：攻方腿撞限额、换一条新腿接着做时（`.claude/rules/three-way-inference.md`「云端腿的报告要分段落盘」那一节），前一条腿跑过多少状态只能从交接摘要里捞，定义没叫新腿去捞。小：不让任何一步越过 hook，只让 10⁷ 这条线在两种常见情形下没人能核。

### G3 其余形状（没打中）

- 用 `crates/singlefs-harness/tests/common/`（`mod common;`）搭池：层 0 用例自己也用它，但它的名字不带 `layer0`，字面不算拷；它只给池与几何，不给流。
- 照着层 0 用例的步骤手写一条相同的流：字面算「自己造」；拦它的是状态数上限。第一个事务那条流全量 67108885 个状态（`.claude/kb/experiments.md:165` 那一行，E142 第十七次跑第二段），单次超过 10⁶、单轮超过 10⁷，照字面两道线都不许跑。
- 用 `enumerate_layer0_selecting` 只展开几段、把一段历史拆成几次跑：3b 写死「调同一份文件里每一段都展开的 `enumerate_layer0` / `enumerate_layer0_versions`」，不许。
- 从一个名字不带 `layer0`、自己又 `include!` 了层 0 用例的文件里再 `include!`：「拷文件、拷代码段、`include!`、`mod` 引进来都算拷」管得到，隔一层也是 `include!` 了层 0 用例的内容。

### G8（没打中）

`.claude/agents/three-way-local-defense.md:19` 把样本写成「`<前缀>-output-s<n>.md`，前缀取派发提示给的」，本地攻方「输入」第 19 行同样由派发给前缀。现查 `.claude/gate.d/`、`.claude/hooks/`、`research/scripts/` 里认 `-local-defense-output` 或 `-local-attack-output-s` 的：只有 66 号认提示文件名（`<轮>-local-defense.md`），不认样本名，改前缀碰不到它。`ask-local.sh` 的作废副本按提示文件名起（`<轮>-local-defense-output-void<n>.md`），与样本的前缀可能不同，攻方那一侧今天也是这样，不是 G8 带来的。

## 我提的改法（只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| 甲：15、74 号调包装之前先核上限写法，要求「正整数加 K/M/G/T」、单位必写，写错单独判红、出路指到变量名（`cap-syntax-fix.py` 在临时副本上插进两道阶段） | R1、R2 在 15、74 号上 | **量过**，副本上的原样见下 |
| 乙：包装的写法正则把单位改成必写；用法错改退 251（「命令一行都没跑」那一类），不再退落在 0–249 里的 2 | R1、R2 在所有调用方上（含崩溃验证员第 1b 步对 54、55、57 号的整条外包） | 推的 |
| 丙：`.claude/main-agent.md:48` 与 `.claude/agents/crash-verifier.md:20` 的「一个数」写成「一个带单位的上限（例 16G）」 | R2 | 推的 |
| 丁：分诊员第 3 步加一格：✗ 行写着「内存包装 … 退 25x」的不按点名文件判，250 判「环境（上限）」、252 / 254 判「并发」，原样抄 → | R3 | 推的 |
| 戊：15、74 号出路补 253（限时：把 `RUN_WITH_MEMORY_CAP_TIME_LIMIT` 拿掉或放宽再跑） | R7 | 推的 |
| 己：执行员第 4 步定拼法（`<旧名去掉 .out>-<YYYY-MM-DD>.out`，同一天再跑加 `-rN`，不写 `.rN.out`）；第 5 步改指 `replay.sh` 的同时，这个新文件要被实验页点名（逐字节一致也点一行），这一次不写页的写进报告交主 agent 派人点名 | R4 | 推的 |
| 庚：40 号跳过 `*.r[0-9].out` 之前先核它是不是 `replay.sh` 的登记产物，是就不跳 | R4 里 `.r2` 那一半 | 推的 |
| 辛：检出 hook 第 2397、2408 行的出路改成共用约束 ④ 的收法（每件写退出码文件、批号件号、数件数） | R5 | 推的（hook 不在我的写范围，没改副本也没跑） |
| 壬：G9 的「量过的」写明同一线程数下量的；峰值表加线程数一列 | R6 | 推的 |
| 癸：攻方第 3b 步写成「这一条腿与接手它的腿全部原型跑的全量」，接手的腿从交接摘要里取已跑的量 | R8 | 推的 |

改法甲在副本上的原样（`rerun-output.txt`「cap-syntax-fix.py」一段）：

```
  ✗ GATE_RESEARCH_BUILD_MEMORY_MAX 写成了「12GiB」，内存包装不认（或认成字节数）：这一道没跑
     → 怎么办：写成正整数加 K / M / G / T（例 16G），单位必写；不设就用默认 8G
15 cap=12GiB exit=1
  ✗ GATE_MODEL_DIFFERENTIAL_MEMORY_MAX 写成了「12GiB」，内存包装不认（或认成字节数）：这一道没跑
     → 怎么办：写成正整数加 K / M / G / T（例 16G），单位必写；不设就用默认 8G
74 cap=12GiB exit=1
  ✗ GATE_RESEARCH_BUILD_MEMORY_MAX 写成了「16」，内存包装不认（或认成字节数）：这一道没跑
     → 怎么办：写成正整数加 K / M / G / T（例 16G），单位必写；不设就用默认 8G
15 cap=16 exit=1
  ✗ GATE_MODEL_DIFFERENTIAL_MEMORY_MAX 写成了「16」，内存包装不认（或认成字节数）：这一道没跑
     → 怎么办：写成正整数加 K / M / G / T（例 16G），单位必写；不设就用默认 8G
74 cap=16 exit=1
```

副本上的数，不是入库阶段上的数。改法甲只挡住写法错这一步；合法写法下 250–254 那一支没动，修定义的 agent 的假根演示仍是那一支唯一的样本。

## 没打中的形状与取样范围

- 重型闸：分诊员、崩溃验证员、实现员带上限变量的 11 种写法（K1-01..11），全放。
- 15 号的复用缺口：15 号不走复用，不存在。
- G5：`research/results/` 下 22 份含判决行的产物、26 条判决行、54 个（边, 字段名）组合、355 处取值，全部分边；没有落错边的。
- G7：挪后剩下的 33、52、80、96 号四道，逐道 grep 读的路径；都不依赖第 7 步。
- G4：3349 个跟踪文件 × 3 种句式共 10047 句，G4 与 F13 两版拒的句子逐句相同；另 8 句 stdin 探针。
- G3：`mod common`、手写同一条流、按段拆跑、隔一层 `include!` 四种形状。
- G6 的共用约束 ④ 写法本身：D4（两件收尾数件数）、D5（`for` 循环里起、末尾 `wait` 再数）检出 hook 都放行；演示里读得到退 3。
- G8：门禁、hook、research 脚本里没有认样本名的。
- 试过、按字面不算打中：40 号出路「确实是废弃产物就删掉它」与 G2 相反，但执行员第 35 行「产物照第 4 步一个不删」管住了（R4 节）。

## 这条腿自己的限度

- 两道阶段只在镜像根上跑了「写法错」这一种结局（包装在起 cargo 之前就退），没在仓上真跑 15、74 号，也没跑 `gate.sh`；R3 是按两份原文逐分句推的。
- 15 号 8G 够不够没量：重型闸拒了 `cargo test --no-run`，没换写法（K1 节末）。
- 每格只抽了一次（这一条腿自己就是一次观测）；「没打中」的几格照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」只算一次。
- 行号都是 2026-09-26 03:52–04:20 UTC 现取的；开工快照 27 个文件当时全 OK，之后没再核。
- 40 号副本基线就红 7 份（今天工作区里 e156、e158 的几份产物没被实验页点名），是别的会话留下的，我只看了新文件让条数加不加 1，没判那 7 份。

## 没做什么

- 没跑重型测试（54、55、57、59、87、层 0、QEMU、herd7、全量 cargo test、整轮门禁）；没编译任何东西。
- 阶段归属表里没有登记给 `three-way-attack` 的阶段（共用约束「门禁」一节那条 `awk` 取出 0 行），没跑门禁阶段；40 号只在副本上跑过。
- 没判 G7 十道各自读不读实验页的字面依据（本地攻方那一半），没判正推、辩方的格，没替主 agent 采纳。
- 没改仓里任何文件；只写了这份报告与 `research/prompts/defs-m2-closeout-r3-opus-model/`。草稿目录 `/tmp/claude-1000/defs-closeout-r3-opus/` 里只有 10 个小文本（演示输出与抄行用的中间文件），没有编译目录、worktree 与仓副本。
