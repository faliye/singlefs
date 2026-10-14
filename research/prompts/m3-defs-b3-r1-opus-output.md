# m3-defs-b3-r1 攻方腿（Opus）报告（2026-10-12）

攻击面：D1、D2、D3、D4、D6。只读冻结副本 `~/.local/share/singlefs/work/m3-defs-b3-r1/repo/`（开工核过 `frozen-copy-sha256.txt` 21 份全 OK）。引文的路径写成冻结副本的绝对路径，`cite-check.py` 不论 `--root` 给仓根还是快照根都按冻结副本核。

## 复跑

```
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m3-defs-b3-r1-opus-model/tools/rerun.sh \
  --repo <仓根> --frozen ~/.local/share/singlefs/work/m3-defs-b3-r1/repo --work <新的草稿目录>
```

跑完末尾五行 `name=compare file=<d1|d3|d4|d6|d6-cell> same` 即与 `out/` 逐字节相同。不起 cargo；派发闸经钩子入口喂 JSON（`CLAUDE_PROJECT_DIR` 指到探针自己建的样本根），没跑任何 `--selftest`。这一次最后一趟复跑（草稿目录 `rerun-d`，D3 加了第三格之后）五行都是 `same`。

模型目录各文件 sha256（`find . -type f | sort | xargs sha256sum` 原样）：

```
ec74ed232c4963de582ae0946a9e82390bbb187ef114bef35449df89e9b6bbc4  ./out/d1.txt
e83c7d173192f4f5ef058f3173709745db1dcd410b145f0eec31ddf9c040a18d  ./out/d3.txt
060d7be43642021c6e0b8f75af3e6f1d44eb4a4eff4c5353e91c64ef5355efc0  ./out/d4.txt
86ccd11c914a7e4088d7ea6264c7b1efd2a633e384059cf052a42634ac8bedb6  ./out/d6-cell.txt
014cbb8b87e11e117cd3239cb5140e700f33fe2054d5ced34488ffa68d74f4a4  ./out/d6.txt
9cb000782666edaf7621b58c5a222eaf52f1136fd52b1411047aa649c007b176  ./probes/d1-threads.sh
5d331b95b31576f046969e79f5a281440896de9c294198f079029dbbc471aa54  ./probes/d3-exempt.sh
7a55d7a70e479cdeaae07fdfa2cddf001eb543e572094637c59535fe5e0d029a  ./probes/d4-private-values.py
02cae2dccee76f3fbb737d645e7298f8bfb9e4993415b8d29cb0ed0483f5fc45  ./probes/d6-abandoned-cell.sh
f7f3dab712c450b5959d13e8dcc00adf8d989518b7d023649844bea208d4ba44  ./probes/d6-halted.sh
e8c0efaa7dfc26321b5d6bb53ef890fe0fe86261b407f264e6c040c8c06018c6  ./tools/rerun.sh
```

## 各格判定一览

| 格 | 判定 | 打中的历史 | 量过 / 推的 | 跑前条款哪一行 | 改前的文字中不中 |
|---|---|---|---|---|---|
| D1 | 打中 | A1：用户经 `SINGLEFS_THREAD_CAP` 给了参数时，共用约束许可的两种写法 `capped.sh N` 与 `capped.sh default` 交出不同的数，派发提示与 ⑮ 只认 31 | 量过（`out/d1.txt`） | 第二行「照定义做却违反用户定案」（给了参数照参数） | 不中：`default` 写法是这一批补进共用约束的 |
| D1 | 打中 | A2：用户要临时让出 CPU，没有一条规则管怎么降在跑的子 agent，「派出去之后」那一句还禁止发消息 | 推的（按文字） | 第二行 | 也中（改前同样只为「重活数变了」开口子）：另立一笔账 |
| D1 | 打中 | A3：⑮ 放过「31 ÷ 2」「31（本机），第二台 16」，也放过不写线程上限一行的重活派发 | 量过 | 第二行后半「钩子说拒的钩子不拒」（⑮ 文件头写了公式拒） | ⑮ 属批 2 钩子；改前的定义没写 ⑮ |
| D2 | 打中 | B1：`verification.md` 自己规定的轻型命令与门禁的带特性那一趟都让 GPU 测试二进制按 31 个测试线程跑，新句「`--test-threads 1`」照规则做就违反 | 量过（前缀交 31、四份脚本里 `--test-threads` 0 处） | 规则内部矛盾 | 不中：那一句是这一批新写的 |
| D2 | 打中 | B2：「配置里的机器与卡都用」，而 GPU 用例只借本机配置点名的卡 | 推的（按代码） | 规则与实际跑法矛盾 | 不中 |
| D2 | 打中（弱） | B3：「20 万态量级」没有单位，与「挂钟长不是收窄它的理由」、攻方腿的 10⁶ 上限互相拉扯 | 推的（按文字） | 规则之间矛盾 | 不中 |
| D3 | 打中 | C1：照第 9 条按文件现状登记哈希，别的会话没审过的代码改动一起拿到豁免，门禁那一格判绿 | 量过（`out/d3.txt`） | 第二行（豁免表头「只有用户明说才加一行」是用户定的口径） | 不中：改前没有「照现状登记」这一句（有一回先例，当时另记了别的会话那一部分） |
| D3 | 打中 | C2：`compile-then-swap.py` 只核「开跑之后」主工作区那份没被改，拷出草稿到开跑之间别的会话的改动整份被盖掉；「别人的改动不追查」之后没人发现 | 推的（按代码） | 第二行 | 也中：另立一笔账 |
| D4 | 打中 | E1：仓里已经有两处私有值（records 里的第二台别名、腿报告里贴的内核日志带本机主机名） | 量过（只报文件与行号，不抄值） | 与仓里已有文字冲突 | 不中（这一条是新写的） |
| D4 | 打中 | E2：射程只列第二台；与「整行抄原话」「贴原样输出」两条撞 | 推的（按文字）+ E1 两处实例 | 规则之间矛盾 | 不中 |
| D4 | 形态 | E3：做得成会红的检查，但只按长词判、只判这一次新加的行 | 量过（短词误报 221–2001 份） | — | — |
| D6 | 打中 | F1：halted 文件引自己、引这一轮正文、引规则本身，都把还要判的轮关掉 | 量过（`out/d6.txt`） | 文字没写到（规则只列了文件空、没出处、原话找不到三种不收） | 不中（halted 是这一批写进规则的） |
| D6 | 打中 | F2：出处写 `research/../../…` 指到仓外，⑥ 照收；规则写的是「仓里」 | 量过 | 第二行后半「定义说的拒绝钩子不拒」 | 不中 |
| D6 | 打中 | F3：halted 只关 ⑥、`abandoned-rounds.tsv` 只关门禁那一格，两处登记各管一半；规则只写了前一处 | 量过（`out/d6.txt`、`out/d6-cell.txt`） | 规则没写全 | 不中 |

## D1　线程数

### A1　用户经 `SINGLEFS_THREAD_CAP` 给了参数，两种许可写法交出不同的数（量过）

用户定案的字面：`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/heavy-thread-default:6`：「双机两台各 31、合计 62，给了参数照参数」；同一份文件头写了参数从哪来：`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/heavy-thread-default:3`：「SINGLEFS_THREAD_CAP 设了照它，脚本的参数给了照参数」。

历史（每一步都是规则许可的，复现在 `probes/d1-threads.sh`）：

1. 用户在 shell 配置里 `export SINGLEFS_THREAD_CAP=8`（要让出 CPU，或就是想要 8）。Claude Code 每条 Bash 从用户配置起 shell，主 agent 与每个子 agent 的命令都带着它。
2. 主 agent 照 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/main-agent.md:53`：「机器忙时也固定这个数，不看忙闲」打骨架。骨架不看环境：`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/scripts/agents/dispatch-prompt.py:91`：「临时覆盖（SINGLEFS_THREAD_CAP）不算：骨架填的是登记值本身」。量到 `name=d1 case=skeleton-env8 线程上限：31`。
3. 派发闸 ⑮ 同样不看环境：`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/runner-dispatch-guard.sh:901`：「environment = {name: value for name, value in os.environ.items() if name != THREAD_OVERRIDE_VARIABLE}」。环境里是 8 时，提示写 31 放（`guard15-n31-env8 rc=0`），写 8 拒（`guard15-n8-env8 rc=2`）。
4. 子 agent 照共用约束起命令：`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/agent-common.md:61`：「它把 cargo 与 crates/singlefs-harness 各装置读的线程变量一次设成同一个数」，两种写法都许可。量到：

```
name=d1 case=capped-default-env8 RUST_TEST_THREADS=8 SINGLEFS_RANDOM_HISTORY_THREADS=8
name=d1 case=capped-31-env8 RUST_TEST_THREADS=31 SINGLEFS_RANDOM_HISTORY_THREADS=31
```

原因在 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/scripts/hosts/capped.sh:52`：「if [[ "$1" != default ]]; then printf」——给了数就原样用，只有 `default` 走 `heavy_thread_default`（它先看 `SINGLEFS_THREAD_CAP`）。

错在哪：同一份提示、同一个环境，子 agent 挑哪种写法决定了是 8 还是 31；写 `capped.sh 31` 的那一支把用户经文件头写明的参数通道盖掉。门禁与脚本里自己套 `capped.sh default` 的（`checker-tier-layer0-replay`、崩溃点重放快档、`.claude/cargo-command-prefix`）跟着 8 走，子 agent 手起的跟着提示走，同一台机器上两套数。

用户动作放开扫（只固定「设了环境变量」这一步，值放开）：

```
env=1 default=1 n31=31 differs
env=8 default=8 n31=31 differs
env=16 default=16 n31=31 differs
env=30 default=30 n31=31 differs
env=31 default=31 n31=31 same
env=32 default=32 n31=31 differs
env=64 default=64 n31=31 differs
```

只有用户给的恰好是 31 时两种写法同值。

四句：

- 分不分辨臂：分。被判的是「共用约束许可两种写法、⑮ 与骨架只认登记值」这一形态；只许一种写法、或 ⑮ 与骨架也认环境变量的形态在这段历史上不中。
- 被判的系统当时看不看得到：看得到。⑮ 与骨架就在主 agent 的进程里，环境变量在手边，是代码有意丢掉的（第 3 步那一行）。
- 满足的分句：跑前条款第二行「照定义做却违反用户定案」，定案是文件头那句「SINGLEFS_THREAD_CAP 设了照它」与「给了参数照参数」。要是主 agent 判「环境变量不算用户参数，只有派发提示里那一行算」，这一格就只剩「两种许可写法不同值」，按「文字没写到」记。
- 跑前条款的改法：条款只说「改定义、规则或钩子」。「只留 `capped.sh N`」修第 4 步的分叉，但用户的环境变量照样被盖（推的）；「⑮ 与骨架照 `SINGLEFS_THREAD_CAP` 判」两步都修（推的，只在我的模型上想过、被攻过零轮）。

### A2　用户要临时让出 CPU，在跑的子 agent 没有一条路降线程（推的）

1. 两个子 agent 在跑，提示都是「线程上限：31」；用户说「别的会话的提交阶段在跑，先别起满」。
2. 主 agent 能做的只有发消息，`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/main-agent.md:39`：「除此之外不给在跑的子 agent 发消息」——例外只剩「改了定义、共用约束或规则」。
3. 共用约束这一批删了接收的那半句，改前是 `/home/fy5090/.local/share/singlefs/work/m3-tooling-b3-defs/patch.diff:19`：「主 agent 发消息改了 N，从下一条命令起照新的」。

剩下的只有 A1 那条环境变量，它只管到写 `capped.sh default` 的那一支。

四句：分辨臂——不分（改前 `/home/fy5090/.local/share/singlefs/work/m3-tooling-b3-defs/patch.diff:80` 那一句开口子的只是「重活数变了」，用户请求同样不在里面），照跑前条款第三行另立一笔账；系统看得到——看得到（用户的话在主会话里）；分句——第二行要的「用户定案」这里是用户当场的话，不是定案，按「文字没写到」记；改法——在「派出去之后」那条例外里加「用户要改线程数」并恢复共用约束接收的半句（推的，被攻过零轮）。

### A3　⑮ 放过公式、第二台另一个数、不写那一行（量过）

`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/runner-dispatch-guard.sh:52`：「不等于、写成公式、认不出数的拒」；同一份第 53 行：「没写这一行的不判」。判法只取行首第一个数（第 888 行 `THREAD_LIMIT_LEADING_NUMBER = re.compile` 后接 `.match`）。量到：

```
name=d1 case=guard15-formula-31-div-2 rc=0
name=d1 case=guard15-n31-second-host-16 rc=0
name=d1 case=guard15-no-line rc=0
```

后果：「线程上限：31 ÷ 2」放行，子 agent 照字面读出 15；「31（本机），第二台 16」放行，违反 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/main-agent.md:53`：「双机时两台各取这一个数」；带重活的派发漏写这一行也放行，子 agent 照 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/agent-common.md:61`（只管「有这一行的」）不套 `capped.sh`，cargo 按核数起（本机 `nproc` 32，第二台的核数没量）。

四句：分辨臂——分（改成「整行只许一个正整数」与「带重活的类型缺这一行就拒」的形态不中）；系统看得到——看得到（提示全文就在钩子手里）；分句——前两例是第二行后半「钩子说拒的钩子不拒」（钩子文件头那一句，不是定义文字）；第三例是文字没写到；改法——两处都只在钩子里改（推的，被攻过零轮）。这一格与正推腿的 D5 重叠，我只报读数，不判 D5。

### A4　已在主 agent 清单里的变体（不另算打中）

- GPU 那一趟：`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/gate.d/checker-tier-layer0-replay.sh:212` 前缀是 `capped.sh default`，量到 `name=d1 case=layer0-feature-pass-prefix RUST_TEST_THREADS=31`。另一处清单没写的：GPU 对拍用例 CPU 一侧读的是 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:208`：「const CPU_THREADS_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_JUDGE_CPU_THREADS";」，这个名字不在 `capped.sh` 设的 8 个变量里（`grep -c` 得 0），所以经 `capped.sh 31` 起它仍取 `available_parallelism`（本机 32），不是 31。它登记在排除表「代码批」行，归代码批。
- 变异门禁 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/scripts/mutation/crates-mutation-rows.py:1855`：「by_processor = max(1, min(16, processor_count // 2))」，主 agent 已列。

## D2　verification 四句

### B1　规则自己点名的轻型命令与门禁都让 GPU 测试二进制开 31 个测试线程，新句要的是 1（量过 + 后果推的）

新句：`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/rules/verification.md:59`：「GPU 测试二进制带 --test-threads 1 跑」。

历史：

1. 主 agent 照同一份规则 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/rules/verification.md:46`：「三条命令带 --quick」跑轻型崩溃那一条（`bash scripts/crash/crash-point-replay.sh --quick`）；多机配置 `ENABLE_GPU=1` 时它的快档带 `gpu` 特性。
2. 那一趟的前缀：`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/scripts/crash/crash-point-replay.sh:453`：「thread_prefix=(bash "$replay_script_directory/../hosts/capped.sh" default)」；提交时门禁带特性那一趟同样（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/gate.d/checker-tier-layer0-replay.sh:212`）。量到 `RUST_TEST_THREADS=31`（`out/d1.txt` 的 `layer0-feature-pass-prefix` 行）。
3. 几份起 GPU 测试二进制的脚本里一处 `--test-threads` 都没有（这一次跑的原样输出）：

```
.claude/gate.d/checker-tier-layer0-replay.sh:0
scripts/crash/gpu-quiesce-run.sh:0
scripts/crash/crash-amplification-item.sh:0
scripts/crash/crash-point-replay.sh:0
scripts/crash/crash-amplification.sh:0
```

后果（推的，没跑 GPU，GPU 用例归重型）：带特性那一趟里挂 `gpu` 的测试文件有 16 份，`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 一份就 205 个 `#[test]`；31 个测试线程同时向卡池借显存。卡池借不到时不等，报「本次未跑」后返回成功：`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1433`：「照卡的优先级记在第一张此刻放得下的卡上」，借不到的走 `not_run_line`，记号与没卡同一个（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/crates/singlefs-harness/src/gpu_memory_ledger.rs:575`：「分项驱动与门禁按 [GPU_NOT_RUN_MARKER] 认这一行」，第 567 行值是 `NOT_RUN_NO_GPU_ADAPTER`）；门禁把这种行只记本次未跑、不判红（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/gate.d/checker-tier-layer0-replay.sh:53`：「GPU 对拍没卡时打这一行后返回成功」）。于是照规则第 1 步做，测试线程多到挤占显存的那几条静默变成「本次未跑」；新句要的 `--test-threads 1` 正好会消掉这种同进程里的争抢。哪几条、多少条会这样，要真跑一趟带 GPU 的快档才知道。

四句：分辨臂——分（规则改成「测试线程照登记值、显存争抢靠卡池」或门禁与轻型补 `--test-threads 1`，两种都不中；只是改一边）；系统看得到——看得到（脚本里线程前缀是写死的）；分句——规则内部两句对不上（第 46 行点名的命令违反第 59 行），也是背景第二节主 agent 自己列的冲突，不算新发现，新的是「本次未跑」那一层后果；改法——在门禁与轻型的 GPU 那一趟补 `--test-threads 1`（推的：CPU 一侧挂 GPU 特性的 CPU 用例也跟着串行，挂钟没量），或改规则那一句（推的）。两个都被攻过零轮。

### B2　「配置里的机器与卡都用」，GPU 用例只用本机点名的卡（推的，按代码）

新句：`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/rules/verification.md:60`：「配置里的机器与卡都用」。GPU 对拍用例的卡池：`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1246`：「GPU 用例共用的卡：这台机器点名的每张卡」。轻型与门禁带特性那一趟在本机起 `cargo test`，第二台的卡一张都用不到（崩溃放量分项另有双机驱动，那是放量，不是 GPU 测试二进制）。

历史：主 agent 照第 46 行跑轻型崩溃那一条，配置里写了第二台的卡；第 60 行读成「两台的卡都用」，实际只用本机。四句：分辨臂——分（改成「GPU 对拍用本机配置点名的卡；崩溃放量照多机配置用两台」就不中）；系统看得到——看得到；分句——规则与实际跑法矛盾（正文 D2 要答的那一项）；改法——改句子（推的，零轮）。第 61 行「先用满一张、再分下一张」与第 1433 行的借法对得上，不算打中。

### B3　「20 万态量级」没有单位，与枚举域条款互相拉扯（弱，推的）

`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/rules/verification.md:58`：「日常验证的规模压在 20 万态量级」。对面两句：`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/agent-common.md:62`：「挂钟长不是收窄它的理由」；`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/main-agent.md:10`：「这一条管枚举域，不管跑的时机」；攻方腿定义 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/agents/three-way-attack.md:34`：「不超过约 10⁶ 个」。

历史：实现员给一个新崩溃点补快档崩溃枚举用例，按闭式算出这条用例 50 万个崩溃态。照第 58 行要压到 20 万上下（或「跑之前先报预计要多久」）；照共用约束与第 10 行不许为挂钟收窄枚举域。「态」指崩溃态、模型对拍的步数还是别的，句子没说；「压」是收窄域还是只是报时，也没说。四句：分辨臂——分（写成「日常验证一趟预计超过 N 分钟先报时长；枚举域照崩溃点那一条，不为规模收窄」就不中）；系统看得到——看得到（用例作者手里有闭式态数）；分句——规则之间矛盾；改法——改句子（推的，零轮）。我没在仓里找到轻型崩溃域的总态数（产物里只见 `pipeline_states=67` 与 `=368` 两行），所以说不出现有的轻型域离 20 万多远。

### B4　没打中：「开满」与内存包装

`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/scripts/hosts/run-with-memory-cap.sh:10`：「开跑前只在按预测这台放不下时排队」——包装只按内存排队，不按线程数或并发数压，与「只限一样：不让哪台的内存或显存撞顶」是同一件事。日常一条串行那一层 2026-10-06 已撤（同一份第 104 行起）。造不出两者打架的历史。

## D3　只核自己的改动

### C1　照现状登记哈希，别的会话没审过的代码改动跟着拿到豁免；换成只暂存自己的块又让自己的门禁红（量过）

被判的两句：`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/main-agent.md:27`：「照文件现状的哈希登记」；`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/agent-common.md:64`：「别人的改动不追查、不拦」。豁免表的口径：`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/agent-def-review-exempt:3`：「时才加一行，理由里写明是哪一次、他说了什么」（`.claude/crates-review-exempt` 表头同义）。

历史（`probes/d3-exempt.sh`；判法是冻结副本 `.claude/gate.d/doc-process-records.sh` 里的 `user_exemption_reason` 原样抽出来调，第 494 行按文件此刻的 sha256 认）：

1. 会话 B 在 `crates/<包>/src/x.rs` 改了一行代码（例：`block < size` 改成 `block <= size`），没提交，打算自己开代码轮。
2. 这一批（会话 A）在同一份文件只改一行注释；用户弹窗答「豁免」，说的是这一批的注释。
3. A 照第 9 条与共用约束：只核自己那行注释还在；B 那一块不追查；按文件现状登记哈希。
4. 门禁 crates-adversarial-review 那一格对这份文件：没有判决点名，转去 `user_exemption_reason`，哈希对上，交出理由，判绿。量到：

```
name=d3 case=register-current-hash exempted=yes file_has_other_session_code_change=1
name=d3 case=register-own-change-only-hash exempted=no
name=d3 case=staged-tree-with-current-hash-registered exempted=no
```

第一行：B 的代码改动在用户只豁免了注释的那一行登记下拿到豁免；B 之后不再动这份文件就提交的话，它也不用走三方、也没有用户豁免。

第三行是另一条岔：主 agent 照 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/main-agent.md:71`：「主 agent 用 scripts/gate/stage-mine.py 暂存」，它只挑自己的块（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/scripts/gate/stage-mine.py:4`：「别人的留在工作区」），`gate.sh --staged` 判的树里这份文件是「基线 + 注释」，而登记的是工作区现状的哈希，对不上，这一格判红，这一批的提交被别人的改动挡住——这正是「别人的改动不拦」要避免的。要两头都绿，A 只能把 B 那一块一起提交，又撞上游 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/singlefs-ai-sop/rules/session-wrapup.md:50`：「只提交这一轮的」。

先例：这种登记以前出过一回，当时另记了别人那一块，`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/agent-def-review-exempt:30`：「它那部分的处置由它的会话另记」。新句把「照现状登记」写成做法，「另记」那半句没写。

四句：

- 分辨臂：分。「按这一批暂存那一版（基线 + 自己的块）的哈希登记」这一形态：B 那一块不在登记的那一版里，谁判工作区现状都对不上、B 得自己找去向；`gate.sh --staged` 判暂存树时对得上、A 不红（推的，那一格我没另造样本量）。
- 系统看得到：看得到。主 agent 手里有自己的补丁与 `stage-mine.py --dry-run` 打的「留在工作区 N 块」；门禁那一格看不到「这一版里哪几块是谁的」，只看得到哈希。
- 分句：跑前条款第二行「照定义做却违反用户定案」——豁免只给用户明说的那一次改动（表头），照第 9 条登记之后豁免罩到了用户没说过的代码上。要是主 agent 判表头不算用户定案，这一格记规则之间矛盾（第 9 条对表头、对上游第 4 条）。
- 改法：条款没给具体改法。「按暂存那一版登记」修第一、三行（推的，零轮）；「照现状登记 + 理由里点名别的会话那几块、另派它的去向」只修第一行的「没人知道」，不修第三行（推的）。

### C2　整份换上的路径会吞掉别人的改动，不追查就没人发现（推的，按代码）

`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/scripts/hosts/compile-then-swap.py:20`：「再核主工作区那份在编的这段时间里没被改过」——核的起点是脚本开跑那一刻（第 252 行 `before = read_bytes_or_none(target)`），草稿是执行员更早从主工作区拷的（第 238 行的出路就让它先拷再改）。

历史：执行员拷出 `crates/singlefs-checker-tier/src/bin/e<号>_….rs`，改了一小时；这一小时里会话 B 在主工作区同一份 bin 里改了一行；执行员开跑 `compile-then-swap.py`，编过、开跑之后没人再动，整份换上，B 那一行没了。执行员照新句只核自己的改动还在（在），不追查别人的；B 以后发现自己的改动丢了。

四句：分辨臂——不分：改前的文字同样没要求谁去核，照跑前条款第三行另立一笔账；系统看得到——脚本看不到草稿是什么时候拷的（不传就不知道），执行员看得到；分句——跑前条款第二行要的「用户定案」是「只核自己的」本身，这里照它做反而丢了别人的改动，按「文字没写到」记；改法——脚本多收一个「拷出时那份的 sha256」参数、换之前核主工作区此刻还是它（推的，零轮）。

### C3　没打中的

「这一批的改动被别人冲掉」：第 64 行要求核自己的改动还在，这一半管住了；造不出没人发现的历史。

## D4　私有主机信息不进仓

新句：`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/agent-common.md:47`：「第二台的主机名与别名、IP、第二台上的路径、两台的服务单元名、清场脚本的内容」，同一行末：「真实值只在仓根被 git 忽略的 multi-host.env 与 ~/.config/singlefs/ 里」。

### E1　仓里已经有两处（量过，只报文件与行号，不抄值）

`probes/d4-private-values.py` 读冻结副本根上被忽略的 `multi-host.env`（`.gitignore` 第 20 行 `/multi-host.env`），取每个键的值按词切开，加本机 `hostname`，在冻结副本其余文件里逐份找。只打键名、词的种类与长度、命中的文件。长词的结果（`out/d4.txt` 原样）：

```
name=d4 key=PEER_SSH_HOST word=0 kind=其他词 length=13 files=1
  records/2026-09-30-里程碑三开工第一轮.md
name=d4 key=PEER_REPOSITORY_DIRECTORY word=0 kind=路径 length=41 files=0
name=d4 key=PEER_CARGO_BIN_DIRECTORY word=0 kind=路径 length=23 files=0
name=d4 key=QUIESCE_STOP_COMMAND word=1 kind=路径 length=45 files=0
name=d4 key=QUIESCE_START_COMMAND word=1 kind=路径 length=46 files=0
name=d4 key=QUIESCE_STOPPED_CHECK_COMMAND word=1 kind=路径 length=46 files=0
name=d4 key=QUIESCE_STARTED_CHECK_COMMAND word=1 kind=路径 length=46 files=0
name=d4 key=本机 hostname word=0 kind=主机名 length=12 files=1
  research/prompts/m3-batch10x-code-r1-opus-output.md
```

- `records/2026-09-30-里程碑三开工第一轮.md` 第 1004 行：第二台的 SSH 别名，同一行还引了用户原话里第二台的另一个名字（那个名字不在 `multi-host.env` 里，探针按值找不到它，是我读那一行时看到的）。两份都已进 git（主仓 `git ls-files` 认得这两条路径）。
- `research/prompts/m3-batch10x-code-r1-opus-output.md` 第 295、302 行：腿把 `journalctl` / 内核日志原样贴进报告，行里带本机主机名。
- 第二台上的路径、清场脚本的路径：0 份。

原文不抄进这份报告：抄了就是又进仓一次。

四句：分辨臂——分（这一条要么写明「已在仓里的那几处怎么办」，要么不写；不写的形态在这段历史上留着两处现成的违反）；系统看得到——看得到（值就在本机配置里，探针几秒跑完）；分句——正文 D4 要答的「与仓里已有文字有没有冲突」；改法——一次性清理要动 records 与上一轮的腿报告，后者撞「原样保存的证据不许事后改」（下一节），交用户定（推的）。

### E2　射程只写了第二台，又撞两条「原样」规矩（推的，按文字；E1 是实例）

1. 本机主机名、本机的服务单元之外的本机值（主机名、IP）不在列表里：列表点名的是「第二台的主机名与别名」。E1 第二处就是本机主机名经日志进仓。我没法判本机主机名算不算私有，交主 agent；算的话，这一行的射程少了一半。
2. 贴原样输出：`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/agent-common.md:98`：「能用命令核的事实，贴命令与原样输出」，同一行还要求「输出不截断」。系统日志、`ssh` 报错、`systemctl` 输出原样带主机名与单元名；照这一条贴，就违反第 47 行。
3. 整行抄原话：用户的原话里带主机名时（E1 第一处就是用户自己在弹窗里点名第二台），下面三处都要原话：`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/rules/three-way-inference.md:121`：「加用户叫停的原话」，而且派发闸 ⑥ 要原话「真在那份文件里」；豁免表 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/agent-def-review-exempt:3`：「时才加一行，理由里写明是哪一次、他说了什么」；kb 引用 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/rules/three-way-inference.md:44`：「整条抄进去」。
4. 证据冻结：`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/singlefs-ai-sop/rules/evidence-discipline.md:28`：「发给模型的提示、跑出来的原始输出、当时的日志，在这一轮里一个字都不改」——这一轮里贴进去的私有值，这一轮里改不掉。

历史（第 3 条那种）：用户说「把第二台 <别名> 那轮停了」；主 agent 照第 121 行写 halted，原话里的别名照抄进 `research/prompts/<轮>-halted.md`，⑥ 才收；照第 47 行就不许写。两条规则不能同时满足。

四句：分辨臂——分（加一句「原话、原样输出里的私有值换成『第二台』『本机』，换过的地方标一下」就不中；⑥ 的「原话真在那份文件里」也要跟着认这种替换）；系统看得到——看得到；分句——规则之间矛盾；改法——推的，零轮。

### E3　能不能做成会红的检查：能，形态如下（只答形态，没写代码）

量到的约束：短词误报。值切开之后 4–7 个字的词（清场命令里的选项、显卡配置里的机器标签）在冻结副本里各命中 221–2001 份文件（`out/d4.txt` 里 `length=4`、`5`、`7` 那几行），长词（别名、路径、主机名）只命中上面两份。所以：

- 值从哪来：本机的 `multi-host.env`、`~/.config/singlefs/` 下的私有文件、`hostname`；只取 8 个字以上的词，另许一份私有的「要拦的短词」清单。
- 判什么：这一次改动新加的行（暂存树对基线的 diff），不判全仓——E1 那两份不是这一次加的，判全仓就一直红；那两份的去向交用户。commit message 在提交之后才有，要另挂 `commit-msg` 钩子，门禁看不到。
- 没有本地配置的机器（第二台、别人的克隆）：这一格报本次未跑（退 77），不判绿。
- 自证：样本根放一份假的 `multi-host.env` 与一份新加了假别名的文件，必须判红；去掉那一行判绿；只放 4 个字的假词判绿（证明短词不拦）。
- 管不到的：换了写法的值（大小写、拆开写、换成别称），用户原话里的别称（E1 第一处的另一个名字就不在配置里）。

## D6　叫停轮写 halted

新句：`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/rules/three-way-inference.md:121`：「文件空、没出处、原话在那份文件里找不到的不收」。钩子的认法：出处是 `/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/hooks/runner-dispatch-guard.sh:692` 那条正则认出的 `records/`、`research/`、`.claude/` 起头的 `.md` 路径，第 713 行「with open(os.path.join(project_root, cited), encoding="utf-8") as cited_file:」打开，第 720 行「elif any(quote in cited_text for quote in quotes):」原话在里面就收。

样本（`probes/d6-halted.sh`）：样本根里一轮 `r-victim-r1`，有正文、有开工快照（点名 `.claude/kb/victim.md`）、没有判决；派 kb-scribe 改 `.claude/kb/victim.md`，经钩子入口喂 JSON（提示里重型测试、开工先读、规格检查已判、草稿目录与报告几样都齐，只剩 ⑥ 会拒）。halted 文件每格一种写法：

```
name=d6 case=no-halted rc=2
name=d6 case=legit-user-quote rc=0
name=d6 case=self-cite rc=0
name=d6 case=cite-own-body rc=0
name=d6 case=cite-the-rule-itself rc=0
name=d6 case=cite-outside-repo rc=0
name=d6 case=abandoned-tsv-row-only rc=2
```

每格 halted 的那一行（原样，见探针第 29–33 行）：

- `self-cite`：`出处：research/prompts/r-victim-r1-halted.md「主 agent 自己决定收掉这一轮」`——引的就是 halted 自己，原话当然在里面。
- `cite-own-body`：`出处：research/prompts/_r-victim-r1-body.md「第一轮腿还在跑」`——引这一轮自己的正文。
- `cite-the-rule-itself`：`出处：.claude/rules/three-way-inference.md「用户叫停、不写判决的轮」`——引规则自己那一句。
- `cite-outside-repo`：`出处：research/../../outside-note.md「随便一句十个字以上的话」`——路径以 `research/` 起头，拼上项目根之后落在仓外。

### F1　一份 halted 能把还要判的轮关掉（量过）

历史：

1. 主 agent 开了 `r-victim-r1`，两条腿在跑，开工快照点名 `.claude/kb/victim.md`。
2. 主 agent 想先派书记员改 `.claude/kb/victim.md`，⑥ 拒；拒绝说明的出路里就写着「写 research/prompts/<轮>-halted.md 收掉它」。
3. 主 agent 写一份 halted，出处指自己、指正文、或指规则那一句，任一种都放行（上表）；之后这一轮的快照不再挡任何派发，腿交回时它们判的那几份文件已经被改了。

规则写的三种「不收」，钩子都照做了；这几格过的是规则没写到的缝：原话是谁说的、出处是不是用户说话的地方，规则与钩子都没要求。

四句：

- 分辨臂：分。「出处必须是 `records/` 下的文件、不能是 halted 自己与这一轮的材料」的形态挡得住前三格（推的）；挡不住「引一句别的轮的用户叫停原话」（引 `records/2026-10-01-删除与并发事务.md` 里 m3-batch63 那句就过），那一格只有「原话里点名这一轮」或主 agent 自己核才挡得住。
- 系统看得到：前三格看得到（路径就在手边：是不是 halted 自己、是不是 `_<轮>-` 起头、是不是 `.claude/rules/`）；「这句话是不是用户为这一轮说的」钩子看不到，判别子观测不到，只能靠主 agent。
- 分句：规则的「不收」清单字面都满足，按「文字没写到」记，不是「定义说的拒绝钩子不拒」。
- 改法：条款没给具体改法；上面那条收严只在我脑子里过了一遍、没实现、被攻过零轮。

### F2　出处可以指到仓外（量过）

规则写「仓里 records/、research/、.claude/ 起头的 .md 路径」；`cite-outside-repo rc=0`。第 692 行的正则只看开头三段，不管 `..`；第 713 行 `os.path.join` 之后不核落点还在不在项目根下。四句：分辨臂——分（加一句 `realpath` 落在项目根下就不中）；系统看得到——看得到；分句——跑前条款第二行后半「定义说的拒绝钩子不拒」（「仓里」是定义的限定）；改法——推的，零轮。

### F3　halted 与 abandoned-rounds.tsv 两处登记，各管一半（量过）

- 只登记 tsv、不写 halted：⑥ 照拒（`abandoned-tsv-row-only rc=2`）。
- 只写合格的 halted、不登记 tsv：门禁 doc-process-records 的 abandoned-rounds 格（冻结副本该文件第 687–865 行那段 python，原样抽出在样本根上跑，`probes/d6-abandoned-cell.sh`）判红：

```
name=d6-cell case=halted-without-tsv-row rc=1
     r-stop-r1（_r-stop-r1-body.md）
name=d6-cell case=halted-and-tsv-row rc=0
```

门禁认的只有判决与登记表（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r1/repo/.claude/gate.d/doc-process-records.sh:98`：「③ 认撂下：research/prompts/abandoned-rounds.tsv 一行一轮」），有正文就算发过腿，halted 它不读。第 121 行只写了 halted，照着做的人提交时撞红；仓里唯一一份实例 `m3-batch63-code-r1` 两处都写了（halted 与 tsv 第 9 行），是写的人知道两处。改前的欠账写的是让派发闸去读 tsv：`records/2026-10-01-删除与并发事务.md` 第 1602 行「不读 research/prompts/abandoned-rounds.tsv（撂下的轮照旧挡派发）」；这一批换成另起一份文件，同一件事成了两处登记。

四句：分辨臂——分（「⑥ 读 tsv」或「门禁也认合格的 halted」任一种都不中）；系统看得到——看得到（两样都在 `research/prompts/`）；分句——正文 D6 第三问「是不是同一件事两处登记」：是；改法——二选一都只改钩子或门禁一边（推的，零轮）。另外核过：`scripts/kb/archive-past-rounds.py:49` 的 `KEEP` 只留判决与 tsv，下一次提交归档时 halted 跟着正文、快照一起删，⑥ 那时本来也不再认这一轮（没有正文就不算开着），不出新问题。

## 改法与各格（全部只在我的模型上想过、被攻过零轮）

| 改法 | A1 | A2 | A3 | B1 | B2 | B3 | C1 | C2 | E1 | E2 | F1 | F2 | F3 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| ⑮ 与骨架照 `SINGLEFS_THREAD_CAP` 判、共用约束只留一种写法 | 修（推的） | 半修（推的：环境变量成了正式通道，消息那一半仍缺） | 不碰 | 不碰 | — | — | — | — | — | — | — | — | — |
| 「派出去之后」的例外加「用户要改线程数」，共用约束补回接收的半句 | 不碰 | 修（推的） | — | — | — | — | — | — | — | — | — | — | — |
| ⑮ 整行只许一个正整数、带重活的类型缺这一行就拒 | — | — | 修（推的） | — | — | — | — | — | — | — | — | — | — |
| 门禁与轻型的 GPU 那一趟补 `--test-threads 1` | — | — | — | 修（推的；挂钟没量） | 不碰 | — | — | — | — | — | — | — | — |
| 改 verification 第 58–60 行的句子（GPU 对拍只用本机点名的卡；态数写单位、只要求报时长） | — | — | — | 修（推的） | 修（推的） | 修（推的） | — | — | — | — | — | — | — |
| 豁免哈希按「这一批暂存那一版」登记 | — | — | — | — | — | — | 修（推的） | 不碰 | — | — | — | — | — |
| `compile-then-swap.py` 收「拷出时的 sha256」 | — | — | — | — | — | — | 不碰 | 修（推的） | — | — | — | — | — |
| 原话、原样输出里的私有值换成「第二台」「本机」并标出；⑥ 认这种替换 | — | — | — | — | — | — | — | — | 不碰（已进仓的要另清） | 修（推的） | — | — | — |
| E3 那道门禁格 | — | — | — | — | — | — | — | — | 不碰（只判新加的行） | 只防新的（推的） | — | — | — |
| halted 的出处只认 `records/`、不认 halted 自己与 `_<轮>-` 材料，`realpath` 落在项目根下 | — | — | — | — | — | — | — | — | — | — | 修前三格（推的） | 修（推的） | 不碰 |
| ⑥ 读 `abandoned-rounds.tsv`（或门禁认 halted） | — | — | — | — | — | — | — | — | — | — | 不碰 | 不碰 | 修（推的） |

表里没有一格是「量过」：改法我一个都没实现，量过的只有打中本身（`out/`）。

## 没打中的形状

- D1：`crates/` 里 13 处 `available_parallelism`、变异门禁 `min(16, 核数 // 2)`、E171 编排器——都在主 agent 第二节的清单里，我只补了 GPU 对拍 CPU 一侧那个变量不在 `capped.sh` 名单里（A4），不另算。`capped.sh` 套在另一层 `capped.sh N` 里取 `default` 得 N（文件头第 15 行写明），没造出错数的历史。每台 31 与机器忙也不降是用户定案，几个重活叠起来超卖不算打中。
- D2：「开满」与内存包装（B4）；「先用满一张、再分下一张」与卡池借法（第 1433 行对得上）。
- D3：「这一批的改动被别人冲掉没人发现」造不出（第 64 行要求核自己的）。
- D4：`multi-host.env.example` 里没有真实值（长词 0 份）；资源提醒钩子给的报告用「本机」「第二台」称呼，不打别名（`scripts/hosts/host-resources.py` 第 125 行的 `HOST_NAMES`）。
- D6：归档之后 halted 先没了而快照与正文还在、轮重新「开着」——`archive-past-rounds.py` 同一次把三样一起删，造不出。
- 取样范围：D1 的环境变量值扫了 1、8、16、30、31、32、64 七个；⑮ 喂了 8 种提示；D6 喂了 7 种 halted；D3 一份两会话共写的样本，三种登记；D4 扫了冻结副本全部文件（跳过 `.git`、`target`、`__pycache__` 与两份被忽略的配置）。

## 这条腿自己的限度

- 没跑 GPU，B1 的「本次未跑」后果、B2 的「只用本机卡」都是读代码推的；要坐实得跑带 `gpu` 特性的快档，那是重型。
- C1 的门禁格只抽了判法函数在非 git 的样本根上调，没跑整道 doc-process-records（子 agent 不跑门禁阶段）；`gate.sh --staged` 取哪一版文件算哈希是按它的文件头与 `stage-mine.py` 的文件头推的。
- A2、C2、E2 是按文字推的历史，没有装置。
- E1 只按 `multi-host.env` 的值与本机 `hostname` 找；用户口头用的别称（E1 第一处另一个名字）不在配置里，按值找不到，那一处是我读那一行时看到的。仓里还有没有这类别称，我没法全扫。
- 冻结副本是只读的；所有样本根都建在草稿目录（复跑时建在 `--work`），派发闸喂 JSON 时 `CLAUDE_PROJECT_DIR` 指样本根，没碰冻结副本与主工作区。
- 没读禁读清单里的文件。

## 没做什么

- 不判 D5、D7（正推腿的格）；A3 与 D5 重叠的那部分只报读数。
- 没跑任何 `--selftest`、门禁阶段、cargo；没跑重型。
- 改法一个都没实现。
- 没替主 agent 采纳；副本与样本根上的读数不算入库装置上的数。

## 草稿目录

`~/.local/share/singlefs/work/m3-defs-b3-r1-opus/` 里探针与复跑建的样本根（每份是冻结副本 `.claude/` 与 `scripts/` 的拷贝）已删：`run1` 57M、`run2` 57M、`run3` 40K、`rerun-a`、`rerun-b`、`rerun-c`、`rerun-d` 各 114M，另删 `run4-d4.txt` 476K（第一版 D4 探针逐份列短词命中的输出）。留下的只有几份几百字节的日志与喂法，没有仓副本与编译目录。
