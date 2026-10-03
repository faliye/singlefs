# defs-m3-formal-gate-r2 辩方腿（Sonnet）报告：E3、E4，兼复核第一轮判决五格

日期 2026-10-01。批次 `records/2026-10-01-里程碑三形式化接入.md#4`。被判文件读主仓现行版本；引文件写那份文件自己的行号，行号现查。
探针都是只读的：不起 Verus、不编装置、不起真包装，被判的命令一条都没执行；放在 `research/prompts/defs-m3-formal-gate-r2-sonnet-model/`（`SHA256SUMS`、`rerun.sh`）。

## 一、结论摘要

- E3：包装 250、251、252、254 判「没跑成」（退 1、不写标记、不被复用）站得住，两份判法的实测输出对得上；253 一侧 Verus 判红、E165 判「没跑成」，各有先例，站得住，但交给分诊时只靠 ✗ 行的字面分得开，`gate-triage.md` 里「253 ⇒ 环境」那半句对 Verus 一侧够不着。两处缺口：同一格里「没跑成」与真红并存时，崩溃验证员第 3 步的「那一格记没跑成」会盖掉真红（探针量到一格 15 条 ✗：8 条没跑成、7 条真红）；退 137 这类不在 250–254 里的外部杀进程，E165 一侧仍写成「→ 修实现回归」（同形态，判决没覆盖）。
- E4：`main-agent.md` 那句写实与 `gate_reuse` 的代码对得上，有两处小毛病（「它」指代、`heavy-test-guard.sh` 还留着旧说法）；崩溃验证员第 1 步的停下在今天这个仓里**会发生但不是每次**：同一个探针在同一会话里先后两次，先量到 0 个未跟踪、后量到 2 个（别的会话在 `research/results/` 里新建的 E167 产物）；挪句与补清单没造出新冲突，另有四处同族清单没补（只是注释与提示文字）。
- 第一轮判决五格：D1、D2、D3、D4、D5 都站得住；两处「改法够不着同形的别的臂」：A2 的停下只落在这一道、QEMU 同形没动；D2 的环境进标记只管登记了 `environment=` 的两道、QEMU 没登记。都不推翻判决，记账。

## 二、判定一览

| 条目 | 辩方判定 | 证据（细节在后） | 什么现象会推翻它 |
|---|---|---|---|
| E3-a 250、251、252、254 判「没跑成」、退 1 | 站得住 | 三、1 | 在真起的格里有一种 25x 结局没被判成「没跑成」，或「没跑成」那一趟写出了全绿标记 |
| E3-b 253 Verus 判红、E165 判「没跑成」 | 站得住（先例在）；对分诊够不着一半 | 三、2 | 出现一条 Verus 253 红被分诊判成「这一轮」的实例，或 E165 的 253 在门禁里真出现 |
| E3-c 同一格里没跑成与真红并存 | 够不着（定义字面盖掉真红） | 三、3 | 交回一份混合格的报告，表里写「没跑成」的同时真红那几行也被主 agent 看见 |
| E3-d 退 137 等不在 250–254 的外部杀进程 | 同样打中（判决没覆盖的同形态，低） | 三、4 | 包装保证外部杀进程也退 250 一类 |
| E3-e 「没跑成」不缩短等待 | 推的、没量过 | 三、5 | 在占满的 slice 上量一格从起到报「没跑成」的挂钟 |
| E4-a `main-agent.md` 写实句 | 站得住，两处小毛病 | 四、1 | 构造一次标记不作数而整轮没真跑这一道 |
| E4-b 崩溃验证员第 1 步对 `research/results` 停下 | 站得住；频率是会话内瞬时的 | 四、2 | 连续若干次提交派发里这一步从不触发 |
| E4-c 第 6 步指纹加五条路径 | 站得住（近似） | 四、3 | 第 6 步报「同一份源码」而标记指纹已变 |
| E4-d D4 补清单之后的残留 | 同样打中（同族清单没补，低） | 四、4 | 这些行不再被读成「哪几道写标记 / 谁靠复用」 |
| E4-e D5 挪句与删句 | 站得住 | 四、5 | 有一处还指着被删的那句 |
| 第一轮 D1 谁跑 | 站得住；A2 的改法够不着 QEMU 同形 | 五、1 | 见五、1 |
| 第一轮 D2 复用 | 站得住；QEMU 同形没登记环境 | 五、2 | 见五、2 |
| 第一轮 D3 并行与内存 | 站得住；C2、C3 仍是推的 | 五、3 | 见五、3 |
| 第一轮 D4 清单一致 | 站得住，有残留 | 五、4 | 见四、4 |
| 第一轮 D5 规则写法 | 站得住 | 五、5 | 见四、5 |

## 三、E3：包装 250–254 判「没跑成」，253 判红

### 1. 250、251、252、254（E3-a）

探针 `e3-verus-exit-codes.py`：`MEMORY_CAP_RUNNER` 换成按 `FAKE_EXIT` 退出的桩，走真的 `run_verus` → `outcome_problem` → `problem_howto`。原样（`rerun-output.txt` 第 3–11 行，每行截到 150 字）：

```
退   0 | wrapper=None         timed_out=False | 类 没出判定   | 出路 格自己的出路 | Verus 没出判定（退 0，编不过或起不来
退   1 | wrapper=None         timed_out=False | 类 没出判定   | 出路 格自己的出路 | Verus 没出判定（退 1，编不过或起不来
退 137 | wrapper=None         timed_out=False | 类 没出判定   | 出路 格自己的出路 | Verus 没出判定（退 137，编不过或起不�
退 250 | wrapper=撞了这一条自己的内存上限 timed_out=False | 类 没跑成    | 出路 WRAPPER_HOWTO | 没跑成：内存包装 research
退 251 | wrapper=包装用法错或起不了 scope timed_out=False | 类 没跑成    | 出路 WRAPPER_HOWTO | 没跑成：内存包装 research/sc
退 252 | wrapper=内存不够排不上      timed_out=False | 类 没跑成    | 出路 WRAPPER_HOWTO | 没跑成：内存包装 research/scripts/r
退 253 | wrapper=None         timed_out=True  | 类 超时红    | 出路 格自己的出路 | 超过 600 秒没出结局，按红记（C5：超�
退 254 | wrapper=被总上限挤掉       timed_out=False | 类 没跑成    | 出路 WRAPPER_HOWTO | 没跑成：内存包装 research/scripts/ru
```

E165 一侧 `e3-e165-exit-codes.py`（一行钉住的表、假包装）：退 250、251、252、253、254 都打「✗ 没跑成：内存包装 research/scripts/run-with-memory-cap.sh 退 <码>…」，`main` 退 1；退 0 与 137 打「✗ 钉住的 1 行在今天的实现上：装置退 <码>…」，也退 1（`rerun-output.txt` 第 12–41 行）。

- 退出码上的位置：「没跑成」是红（退 1），不是 77。共享 `gate.sh` 只把 77 记「本次未跑」：「record_stage "$sname" "$stage_rc" 77」（`.claude/singlefs-ai-sop/scripts/gate.sh:666`），1 照红记；阶段只在整道退 0 且没有本次未跑的格时才写标记与记环境：「if ((stage_exit == 0 && sample_mode == 0 && STAGE_CELLS_CHECK_GIVEN == 0))」（`.claude/gate.d/checker-tier-formal-verification.sh:144`）。所以「没跑成」不会被当成通过，也不会留下可复用的标记；`--check` 单跑一格同样不写标记。
- 先例：同一类结局另两道阶段早就判红、不判 77：「没判出结果：内存包装 research/scripts/run-with-memory-cap.sh 退 $rc」（`.claude/gate.d/checker-tier-research-build-and-replay.sh:190`）、「那条用例没判出结果：内存包装 research/scripts/run-with-memory-cap.sh 退 $cargo_exit」（`.claude/gate.d/harness-model-differential-and-scenarios.sh:200`）。措辞有两种（「没判出结果」「没跑成」），`gate-triage.md` 第 29 行两种都点名了，认的是同一串「内存包装 research/scripts/run-with-memory-cap.sh 退 <码>」。
- 汇总里的记法：阶段汇总行只写「N 格里 K 格红」：「格里 %s 格红（%s）」（`.claude/gate.d/lib/stage-cells.sh:274`），不分哪格是没跑成、哪格是真红；分得开靠它上面各格自己的 ✗ 行。

### 2. 253（E3-b）

- Verus 一侧 253 是这份脚本自己给包装的限时：「RUN_WITH_MEMORY_CAP_TIME_LIMIT=str(timeout)」（`research/scripts/verus-proof-check.py:303`）、「WRAPPER_TIMEOUT_EXIT = 253」（`research/scripts/verus-proof-check.py:101`），所以 253 在这里是「Verus 超时」，不在 `WRAPPER_OWN_EXITS` 里：「WRAPPER_OWN_EXITS = {250: "撞了这一条自己的内存上限"」（`research/scripts/verus-proof-check.py:107`），判红：「按红记（C5：超时不当通过）」（`research/scripts/verus-proof-check.py:366`）。E165 一侧不自设限时，253 只能来自外面设的限时，列在它的表里：「253: "超过限时"」（`research/scripts/e165-pin-red-examples.py:78`）。
- 先例：另两道阶段把 253 与别的 25x 一起判「没判出结果」：「250|251|252|253|254)」（`.claude/gate.d/harness-model-differential-and-scenarios.sh:199`），但它们不给包装设限时，253 在那里只能是外面设的。包装文件头对 253 的定义：「253    超过限时（RUN_WITH_MEMORY_CAP_TIME_LIMIT）」（`research/scripts/run-with-memory-cap.sh:28`）。Verus 一侧自己设了限时，253 就是它自己的判据，判红与 C5 一致；这一步站得住。
- 交给分诊分不分得开：分得开，靠字面——「没跑成：内存包装 … 退 <码>」对「超过 600 秒没出结局，按红记」。但 `gate-triage.md` 的 25x 规则写着：「251 与 253 ⇒」环境（`.claude/agents/gate-triage.md:29`），挂在「✗ 行写着内存包装退 25x」之下；Verus 的 253 红行没有「内存包装」字样，E165 的 253 在门禁里不会出现（没人给它设限时）。所以这半句对这一道是死文字，Verus 的 253 红落到「点名的文件与行在不在暂存区 diff 的块里」那条路，红行不点名文件，只能判「分不清」。这是定义允许的结局，不是错；代价是一次真实的负载型超时（推的，没量到过）会被写成「分不清」而不是「并发」。
- 推翻：Verus 在门禁里出现过因负载而超 600 秒的实例，且分诊要求它归「并发」；或 E165 的 253 在没设限时的门禁里出现。

### 3. 同一格里没跑成与真红并存（E3-c）

空真格与必红世界格是一处一份副本、一次 Verus，每一处各自 `outcome_problem`，问题攒进 `problems` 再逐条打：「{label}（第 {requires_line} 行的 requires）：{problem}」（`research/scripts/verus-proof-check.py:602`）。所以一格里可以同时有「没跑成」几行与真红几行。探针 `e3-mixed-cell.py`：真仓的登记与核，Verus 与包装换成桩（奇数次调用退 252、偶数次打一行「验得过」的 JSON）。原样（`rerun-output.txt` 第 43 行）：

```
check_vacuity 退 1；✗ 行 15 条：没跑成 8 条，照样验得过（空真，真红）7 条
```

崩溃验证员第 3 步的字面：「那一格这一趟没判出结果」（`.claude/agents/crash-verifier.md:32`）——「✗ 行写着『没跑成』的，那一格……表里那一格记『没跑成』，不记成红在证明、表或实现上」。按「一格」读，上面那一格会被记成没跑成，7 条真红的空真就从表里消失（原样抄 → 时它们还在，但表里的类已是「没跑成」）。按「一行」读才对。分诊一侧同形：25x 规则按 ✗ 行写，产出表按阶段一行写归属。
推翻：定义改成「按 ✗ 行记，同一格里有真红的行照记红」，或实例里主 agent 读报告时没被表里的类误导。

### 4. 退 137 等外部杀进程（E3-d，判决没覆盖的同形态）

退 137 在两份判法里都不是 25x：Verus 写「Verus 没出判定（退 137，编不过或起不来）」，E165 写「钉住的 1 行在今天的实现上：装置退 137…」并给「→ 今天的实现在钉住的输入上与原实现不同：是改动引入的回归就修实现」（`rerun-output.txt` 第 6 行、第 17–20 行）。包装文件头写了「被杀的是命令本身时退 137，外面 kill -9 同样退 137」，撞自己上限的由 Result 认成 250，外面杀的才是 137。这与第一轮 C1 是同一种「指错方向的出路」，只是触发条件换成外部杀进程；`gate-triage.md` 另有「输出里一行裸的 `Terminated`」那条兜，但 E165 这一行带 ✗ 与 →，兜不到。判决只收了 25x，不算判决错；记账，低。

### 5. 没跑成不缩短等待（E3-e，推的）

带前缀的重阶段在包装里排队：「等满 RUN_WITH_MEMORY_CAP_WAIT_SECONDS（默认 3600）秒还放不下」（`research/scripts/run-with-memory-cap.sh:54`）；`run_verus` 没设等待上限（`verus-proof-check.py:303` 起只设了限时、键与两条理由），`verify_in_parallel` 把每一处都提交出去。slice 被占满时每一处各等满一轮才报 252，必红世界 94 处、并行 8 个，最坏挂钟约 12 轮（推的，没量）。改法只改了标签、没改等待；判决本来也没要求。

## 四、E4：写实句、第 1 步、清单与挪句

### 1. `main-agent.md` 那句写实（E4-a）

字面：「整轮里真跑那一道」（`.claude/main-agent.md:66`）——标记或复用判定作数的不跑，不作数的（崩溃验证员没跑那一道、跑过之后输入或环境变了）整轮里真跑。对代码：先看全绿标记：「if not break_is_set("stage-marker-ignored"):」（`research/scripts/admission.py:1129`），作数就可跳过；没有标记再看暂存树，没有整轮全绿的 ref 就要跑：「还没有过整轮全绿的暂存树」（`research/scripts/admission.py:1147`）；有就比输入、比复用上限、再比环境。阶段在「要跑」时 `stage_cells_run` 逐格真跑（阶段文件第 140–142 行）。字面与代码对得上，「只核标记」在仓里已无一处：`grep -rnF '只核标记'` 对 `.claude`、`research/scripts`、`README.md` 无命中。

两处小毛病，都不会让人做错事：

- 「它不直接调」的「它」：原句里指 gate-triage（`gate-triage.md:28`「你不直接调它们」），改后紧跟在「整轮里真跑……checker-tier-layer0-replay 照跑快档」之后，读成 `gate.sh` 就与「真跑」自相矛盾。
- 不作数的原因只列了两条。代码里还有第三条：复用上限「hours_text = os.environ.get("SINGLEFS_REUSE_HOURS", "") or "24"」（`research/scripts/admission.py:1157`），标记也有同样的上限（`research/scripts/admission.py:2253`）。字面读不出「标记过了 24 小时」也会整轮真跑。
- 钩子的提示文字还留着旧说法：「gate-triage 只跑」（`.claude/hooks/heavy-test-guard.sh:628`）`gate.sh` 整轮与 research-build 那道，括注（转述）说 54 号与另外四道靠输入没变就复用上一次全绿判定。不影响判定，只是给人看的规矩与写实句不同。

### 2. 崩溃验证员第 1 步对 `research/results` 停下（E4-b）

字面：这一道的未跟踪核对由 `crates` 与 `research/results/e165-*-red-examples.tsv` 改成「checker-tier-formal-verification 查 `crates` 与 `research/results`」（`.claude/agents/crash-verifier.md:28`），有就「两样有一样不过就不跑那一道，停下报告」（同一行）；与指纹同一批文件：「"ls-files", "-z", "--cached", "--others", "--exclude-standard"」（`research/scripts/admission.py:477`），这一行登记了整个 `research/results/`：「research/results/ research/mutations/e165_replay_bounded_enumeration_custom.tsv」（`.claude/gate.d/stage-inputs.tsv:31`）。所以停下是对的：不停，崩溃验证员在工作区写的标记在 `--staged` 的临时树里对不上，整轮重跑（第一轮 A2 量过）。

在今天这个仓里走一遍（`e4-step1-today.sh`，只读，照第 1 步的两条核对命令）：

- 同一探针这一趟会话里跑了两次。第一次（输出落在草稿目录、后被第二次覆盖，转述）：这一道 `git diff --quiet` 退 0、未跟踪 0 个，`git status --porcelain research/results` 只有 31 个「A」。
- 第二次（入库的 `rerun-output.txt` 第 49 行）：「checker-tier-formal-verification | git diff --quiet 退 0 | 未跟踪 2 个」（`research/prompts/defs-m3-formal-gate-r2-sonnet-model/rerun-output.txt:49`）；`git ls-files --others --exclude-standard -- crates research/results` 原样：

```
research/results/e167-multi-inflight-crash-cut-python-l11-recount-2026-10-01.out
research/results/e167-multi-inflight-crash-cut-python-l11-recount-2026-10-01.py
```

  不是我写的，是别的会话在两次之间新建的。此刻派崩溃验证员，这一道会停在第 1 步，要等那个会话暂存或撤掉。
- 同一时段 QEMU 那一道的 `git diff --quiet` 先量到退 1（`research/scripts/crash-case-check.py`、`research/scripts/stage-must-run.sh` 没暂存），几分钟后退 0：QEMU 登记整个 `research/scripts/` 与 `research/results/`（`.claude/gate.d/stage-inputs.tsv:29`），第 1 步对它的「停下」是现成的先例，同样是瞬时的。

结论：会停，但不是每次提交都停：停的窗口是别的会话新建产物到它自己暂存之间。这个会话里两次探针之间量到 0 → 2，没法给频率（没量过多次提交）。代价是一次派发延后，换的是不写出对不上的标记；停下后主 agent 的去向是现成的：「9. 本机常有别的会话在同一个仓里干活」（`.claude/main-agent.md:26`）。字面还有一处含糊：「不跑那一道，停下报告」到底停这一道还是整趟，沿袭自改前，不是这一轮引入。
推翻：连续若干次提交派发里这一步一次也没触发（那就是没有代价）；或有一次别的会话的产物在窗口里而这一步没拦住、写了对不上的标记。

### 3. 第 6 步指纹加五条路径（E4-c）

加进去的五条与登记行逐条对得上：`.claude/gate.d/verus-pins.tsv`、`.claude/gate.d/e165-pinned-red-examples.tsv`、`.claude/gate.d/lib/stage-cells.sh`、`research/results`、`research/mutations/e165_replay_bounded_enumeration_custom.tsv`（`.claude/agents/crash-verifier.md:35`），其余登记路径（`crates/ Cargo.toml Cargo.lock` 与四份脚本）原命令已含（`research/scripts` 整个在内）。`git diff HEAD -- research/results` 今天 4171070 字节、0.67 秒，不贵。它仍只是标记指纹的近似：标记指纹另含阶段脚本、登记表、登记行、工具链与环境：「登记路径下逐文件清单 + 登记表 + 阶段脚本自己 + 登记行 + 工具链 + 构建环境」（`research/scripts/admission.py:2212`），第 6 步不含这几样；开跑前第 1 步的 `git diff --quiet` 带了阶段脚本，覆盖开跑那一刻。判决只要补登记路径，够了。

### 4. D4 补清单之后的残留（E4-d）

补过的位置现查都在：`agent-common.md:54` 带 `GATE_VERUS_MEMORY_MAX` 与 `GATE_E165_PINNED_MEMORY_MAX`，`gate-triage.md:27、29`，`implementation-writer.md:58`，`admission.py` 的 71–72 与 1126 一带，`stage-must-run.sh:17`，`code-tooling.sh:669`，`crash-test/SKILL.md:3、17`，`README.md:126`。同一份「判绿写全绿标记的是哪几道」清单还有三处只写前三道：

- 「判绿之后按输入指纹写的全绿标记」那一行常量注释（`research/scripts/admission.py:228`）；
- 全绿标记自证的 docstring：「的全绿标记：没有时判不了」（`research/scripts/admission.py:2763`）；
- 「的全绿标记（用户 2026-09-27 定：照 54 号写标记、只跑变了的）」（`research/scripts/admission.py:2929`）。

加上第 1 点的 `heavy-test-guard.sh:628`。全是注释与提示文字，不影响判定；与 D4 同族，判决点名了 70、1124 两处，没点这三处。

### 5. D5 挪句与删句（E4-e）

`implementation-workflow.md` 的表加了一行：「| Verus 形式化验证 |」（`.claude/rules/implementation-workflow.md:65`），表后改成「三道都在 `gate.sh` 里」（`.claude/rules/implementation-workflow.md:67`），表头引导句写「三样」；被删的那句（「门禁管哪一半」段末的形式化验证一句）所说的四样内容各有归处：判什么在 `verification.md` 的「门禁管哪一半」节、「每次提交代码」表行在 `.claude/rules/implementation-workflow.md:53`、复用在第 32 行与 `verification.md` 第 37 行。指向它的链接现查：表里写的是「`.claude/rules/verification.md`「门禁管哪一半」」，那一节存在（`verification.md` 第 93 行标题）。「今天的实现」改成「在被判那棵树的实现上逐行重放」（`.claude/rules/verification.md:98`）。没有悬空引用、没有新的相对时间词。

## 五、复核第一轮判决五格（`defs-m3-formal-gate-r1-main-verification.md` 第二节）

引的行都是现查的；第一轮腿的数我没复跑（D1 的钩子探针是攻方 E1 的格，D2 的 `d2-reuse.sh` 是 E2 的格），这里只判「够不够得着、是不是同样打中别的臂」。

### 1. D1 谁跑：站得住，A2 的改法够不着 QEMU 同形

- A1（判法脚本绕闸）：同类先例有两种做法。广谱变异的驱动按名字登记进已知脚本表：「"broad-mutation.py": "research/scripts/broad-mutation.py"」（`.claude/hooks/lib_heavy_tests.py:202`）；crates 变异的判法干脆不开入口：「这里不给整张表开命令行入口」（`research/scripts/crates-mutation-rows.py:9`）。判决取了第一种（按子命令认干活用法），与先例同形；同族的别的臂（`crates-mutation-rows.py`、`vm-bench.sh`、`mutate.sh`）这一类没发现没覆盖的直接入口，所以「同样打中别的臂」这一问在 A1 上没有新的臂。
- A2（标记对不上、整轮真跑）：改法是第 1 步对这一道的未跟踪核对改查整个 `research/results`，只落在这一道。QEMU 同形：登记了「research/scripts/ research/results/」（`.claude/gate.d/stage-inputs.tsv:29`），而第 1 步明写对它不查：「checker-tier-qemu-device-streams 按名字读的 `research/results/` 产物这里不查」（`.claude/agents/crash-verifier.md:28`）。所以 QEMU 一旦有别的会话的未跟踪产物，崩溃验证员写的标记在 `--staged` 里照样对不上，整轮里起虚机真跑。这不是改法错，是判决没把同形的臂算进去；`main-agent.md:66` 的写实句已经把这种情形说明白了（不作数的整轮真跑），所以不会让人做错，只是多一次重阶段。记账。
- A3（单次观测、不采）：本轮我在同一会话里量到第二次实例（四、2：未跟踪 0 → 2），A3 不再只是一次观测；它不改判决，因为采的改法本来就在这个位置停下。
- 推翻：QEMU 在别的会话有未跟踪产物时整轮里没有真跑它（那就是我读错了登记）。

### 2. D2 复用：站得住，QEMU 同形没登记环境

- F1 + F2 落在 `admission.py` 与 `verus-proof-check.py`，不在被判的 11 份里，判决写了「零轮」。我读了两处落点：`stage_fingerprint` 对登记了 `environment=` 的阶段把环境摘要并进指纹（`research/scripts/admission.py:2211` 起），`tool_versions` 另哈希四个安装文件与 `VERUS_Z3_PATH`（`research/scripts/verus-proof-check.py:752` 起）。数（0/3 与 3/3）是 3-11 报告里的，我没复跑。
- 登记了 `environment=` 的阶段现查：`awk` 对 `stage-inputs.tsv` 第三列，lkmm 与 formal-verification 为 1，qemu-device-streams 与 crates-mutation-replay 为 0。同形的臂：QEMU 的二进制与内核镜像换了，标记照样复用到 24 小时上限；登记注释对 herd7 的内核树明说靠上限兜：「靠 stage-must-run.sh 的 24 小时复用上限兜」（`.claude/gate.d/stage-inputs.tsv:30`）。crates 变异那一道的工具链在指纹里（`toolchain_line`），QEMU 没有。D2 对 Verus 判了打中、对 QEMU 没提；换 Verus 是登记表钉着版本的东西，换 QEMU 是系统包，风险不同，但这一格没有量。记账，不推翻。
- 顺带（E2 的格，攻方判）：`VERUS_Z3_PATH` 没设与设成空串在 `tool_versions` 里同一个字面：「if z3_override else '未设'」（`research/scripts/verus-proof-check.py:754`），指纹相同。

### 3. D3 并行与内存：站得住

- C1 采：本轮三、1 的探针证明采的改法生效，两份判法对 250、251、252、254 都给「没跑成」与包装文件头的出路，退 1。
- C2（600 秒限时在负载下撞不撞）、C3（抢编译目录）仍是「推的」：判决没量，我也没量。C2 与三、2 同族：负载型超时会落「超时红」，出路是「超时先看是哪条查询慢」，对负载造成的超时是错方向；判决留着是对的（C5：超时不当通过，红是安全方向），但它与 C1 是同一种错指，只差触发条件。翻面观测：在提交时那种负载下量到一次 Verus 超过 200 秒（攻方 C2 自己写的）。
- 带前缀的重阶段在包装里排队、不是立刻退 252：包装文件头「不带 SINGLEFS_HEAVY_TESTS 的算日常」（`research/scripts/run-with-memory-cap.sh:81`），崩溃验证员起的阶段带 `SINGLEFS_HEAVY_TESTS=commit`，里面每一次 Verus 继承它，不算日常；所以 252 要等满 3600 秒才来（三、5）。
- C4 记账：装置那个上限写不成「量过的」，没动。

### 4. D4 清单一致：站得住，有残留

判决点名的各处现查都补上了（四、4）；残留是同族的三处注释与 `heavy-test-guard.sh:628`。另有一处判决没点、3-11 报告自己列了的：`stage-inputs.tsv` 第 31 行的注释写「Verus 的 version、commit 与可执行文件的 sha256 经 environment=…进复用判定」，F2 之后少了 rust_verify、z3、vstd 与 `VERUS_Z3_PATH`；它是每一道复用都比的登记表，我没改也没判，列在这里。

### 5. D5 规则写法：站得住

三条要求（挪回该在的小节、删重复、去相对时间词）都落了，见四、5。

## 六、同类先例一览

| 问题 | 先例或「找不到」 |
|---|---|
| 包装 25x 判红、不判 77 | 找到：`checker-tier-research-build-and-replay.sh:190`、`harness-model-differential-and-scenarios.sh:200`、`:199` |
| 253 单独判红 | 找不到先例：另两道都把 253 与别的 25x 一起判；Verus 一侧自己给限时，是这一道独有 |
| 同一格里没跑成与真红并存时怎么记 | 找不到：先例的两道阶段一次只起一条命令，不会出现混合 |
| 外部杀进程（137）的出路 | 找不到包装一侧的保证；`gate-triage.md:29` 的「裸 Terminated」只兜没有 ✗ 行的 |
| 第 1 步对 `research/results` 停下 | 找到：QEMU 对 `research/scripts/` 的未跟踪与改动已经按同一句停下（本轮三、4 量到先停后放） |
| 判法脚本绕闸 | 找到：`lib_heavy_tests.py:202`、`crates-mutation-rows.py:9` |

## 七、没做什么

- 没跑重型测试、没起 Verus、没编装置、没跑 `--selftest`、没跑 `gate.sh`；没复跑攻方 E1 的钩子探针与 E2 的 `d2-reuse.sh`；不替主 agent 撤销或维持判决。
- 没读禁读清单里的 `defs-m3-formal-gate-r2-opus-output.md` 与它的模型目录。
- 没量：Verus 在负载下的挂钟（E3-e 的最坏 12 轮是推的）；第 1 步停下的频率（只有一次会话内 0 → 2）；QEMU 换二进制后复用旧标记（五、2 是读登记推的）。
- 没做 E2 的格：`environment=` 进指纹后有没有一道永远复用不上，没判。
- 运行记录：探针经 `bash research/scripts/run-with-memory-cap.sh 4G bash research/scripts/capped.sh 4 nice -n 19 …` 跑。共起了四次：第一次退 252（包装自己的结局：同一会话已有主 agent 一条日常在跑，命令没跑，输出不算结果）；第二次按包装的出路加 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON` 重跑，退 0；改了 `rerun.sh` 之后第三次退 251（包装自己的结局：上限 4G 超过这个键上记的峰值 16M 的 2 倍，命令没跑，输出不算结果）；第四次再加 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON` 重跑，退 0，入库的 `rerun-output.txt` 是第四次的输出。起跑前 `ps` 没有 `qemu-system`、`vm-bench`、`e152`、`fio`。
- 草稿目录 `/tmp/claude-1000/defs-m3-formal-gate-r2-sonnet/` 里只有假包装、假 Verus 安装目录、计数文件，没有编译目录、工作树与仓副本，没有要删的；不入库，因为探针脚本与输出都在模型目录。

## 八、交回前核对

```
$ sha256sum -c research/prompts/defs-m3-formal-gate-r2-snapshot/sha256sums.txt | grep -c ': OK$'
20
$ wc -l < research/prompts/defs-m3-formal-gate-r2-snapshot/sha256sums.txt
22
$ sha256sum -c research/prompts/defs-m3-formal-gate-r2-snapshot/sha256sums.txt | grep -v ': OK$'
.claude/hooks/heavy-test-guard.sh: FAILED
.claude/hooks/lib_heavy_tests.py: FAILED
```

22 份里 20 份与快照相同；`heavy-test-guard.sh` 与 `lib_heavy_tests.py` 在开工之后被别的会话改过（本轮引到 `heavy-test-guard.sh` 的行号是交回时现查的现行行号，开工时是 611、交回时是 628；`lib_heavy_tests.py:202` 交回时现查）。我没有改过任何被判文件。

```
$ python3 research/scripts/cite-check.py research/prompts/defs-m3-formal-gate-r2-sonnet-output.md --root . | tail -1
  ✓ 核了 39 处引文，对上 39 处，没判 0 处（逐处列在上面）
```

模型目录 `research/prompts/defs-m3-formal-gate-r2-sonnet-model/`：`e3-verus-exit-codes.py`、`e3-e165-exit-codes.py`、`e3-mixed-cell.py`、`e4-step1-today.sh`、`rerun.sh`、`rerun-output.txt`、`SHA256SUMS`（`sha256sum -c SHA256SUMS` 6 份全 OK）。
