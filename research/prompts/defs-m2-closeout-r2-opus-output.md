# defs-m2-closeout-r2 云端攻方腿（Opus）报告：E1、E2

<!-- doc-lint:not-numbers D1 D2 D3 D4 O1 O2 O3 O4 O5 O6 O7 O8 O9 O10 O11 O12 O13 O14 O15 O16 O17 O18 F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 F11 F12 F13 F14 F15 F16 E1 E2 E3 H1 H2 H3 H4 H5 H6 H7 H8 H9 G1 -->

写于 2026-09-26 02:36 UTC（JST 11:36）。攻击面：E1（F1–F16 改后字面）、E2（修定义的 agent 自己报的线索）。E3 与 F14 逐格对表不归这条腿。
被判的文件一个没改；开工时与收尾时（02:5x UTC）`sha256sum -c research/prompts/defs-m2-closeout-r2-snapshot/sha256sums.txt` 都是 24 个全 OK。
喂 hook 的探针只喂 JSON、只看退出码，被判的命令一条都没执行；检出记录写进草稿目录下自己的临时文件。

## 复跑

```
cd /home/fy5090/code/singlefs && bash research/prompts/defs-m2-closeout-r2-opus-model/rerun.sh /tmp/claude-1000/defs-m2-closeout-r2-opus
```

它依次跑：三份探针用例（`probe.py`，F4 那几格在一份只含 Cargo 清单与 `crates/` 源码的临时副本里判）、F13 改法 G1 的副本自证、两个包装演示（各在自己开的临时 slice、总上限 600M 里跑 `true` 与一次 400 MiB 分配，跑完 `systemctl --user stop` + `revert` 自己的 slice）、40 / 86 号的仓副本演示、`.rc` 读旧值演示、F5 判决行扫描。第一次整份复跑的原样输出在 `rerun-output.txt`（114 行，退出码 0）。

模型目录 `research/prompts/defs-m2-closeout-r2-opus-model/` 的 `SHA256SUMS`（原样）：

```
987ac8add87ddf5c5488c10f94f5d019557864233066588b3a966ca91ea4c26d  cases-dispatch.json
e38511fcb7e32dd75e059d1ea3e8e8515d7704cfe07c2f8825335d229598f438  cases-f13.json
9d6eab865f5c60736d2e2d4ce258960af99c9fa7b52f3f98f9fdddef939aa0fb  cases-hooks.json
fbcef7a4f5840b80373ca0c584c9a7eb022295a766f352b024d590a61b0551c5  f13-fix-g1.sh
bb36f1fbfe2f36d90703caafc420e83604580bf19a1b6345b24fa4620c3bfe8e  nest-demo.sh
2907c7383c2065a941251bd072375c944a469ea4dd4ff618a7bef5e50dbca588  oom-demo.sh
9828e5e5ed0d4fc5f4ffdc0acdc9148346ea4cf778cc52bc6a365edb6d830800  probe.py
b561fb2546c7daab06dd6fc298170f4646a1bb71576a4b3008ffdc7a960497b3  quotes.list
d0942cb75cbea1fa80fe7266c905c5cf5b679394c94bd9b9b49ea6dc42824c44  rc-stale-demo.sh
ea445cef3ad89594c200d1335625d4ee9b4b63dee1663b3729303f608813ab81  rerun.sh
d2703c2bfa0fcd8109a79b2652677c5d56d125439824563d81cf763548373699  rerun-output.txt
11ae8555b79a85a0c32a67903579c568486cbd808858e55e1cbcbaa1c61b0def  stage-demo.sh
8a2f80d9bc41677a254ac4a83f2bb332962fc448d76c809adca9439750325493  verdict-scan.py
```

## 各格判定一览

编号 H1–H9 是这条腿的打中；「量过」指我在自己的模型或副本上跑出原样输出（在 `rerun-output.txt` 里），「推的」指按代码或数据推、没跑。行号一律是被引文件自己的行号，整行原文在附录 A。

| 编号 | 格 | 被攻的改法 | 判 | 一句话 | 依据的性质 |
|---|---|---|---|---|---|
| H1 | E1 | F9（`gate.sh` 整条经包装） | 打中（大） | 外层 scope 撞顶时 systemd 停整个 scope，后面的阶段一步不跑、退 250 整份作废；外层按上限占着账，里层 59 / 87 的包装排不上（退 252）；上限下界（`check.sh` 的 `cargo test --all` 与 55 号六台虚机都在外层里）与上界（总上限 − 里层上限）之间的窗口可能是空的；F9 与主 agent 定义都没给这个上限怎么取 | 机制量过（私有 slice 缩小版）；下界来自峰值表实测行；32 线程时的数推的 |
| H2 | E1 | F7（逐字节一致就删新文件） | 打中（中） | 这一轮改过装置源码或变异表、而输出逐字节不变的重跑，照 F7 删掉新文件之后，最新的产物比源码旧，执行员自己登记的 69 号判红；69 号的出路是「跑一次把产物存进来」，照 F7 再删，出不去 | 按 69 号判据与样本推的（69 号在非 git 副本里退 77，没在副本上跑） |
| H3 | E1 | F4（原型里调枚举函数不算重型） | 打中（中） | 界只看测试目标名字与「每次」状态数：把一条层 0 流原样拷成不带 `layer0` 的名字就放行（量过）；10⁶ 比多数单条层 0 流的全量还大（1039、22……），没有总量上限，「分几次跑」可以拼回整套；派发闸同时拒掉主 agent 用自然写法转述这一条（D1–D3 量过） | hook 与派发闸量过；状态数取自测试断言 |
| H4 | E1 | F13（弹窗闸认中文文件名） | 打中（中） | `/` 之后的文件名不再要求扩展名，「O3/O8两格：2 格一定该升成打中」这类没有出处的断言句改前拒、改后放；6 句量过；改法 G1 在副本上 30 种自证全过、6 句照拒 | 量过 |
| H5 | E1 | F5（违例类计数点名） | 打中（小） | 按名字里四个词认：`journal_differing_states=3`（6 处）漏掉；`control_violations_ok=true`、`g6_violations_zero=true`、`text_number_mismatches_zero=true`、`layer0_violations=zero` 共 36 处取值「不是 0」照字面要点名，「计数都是 0」那一句对 E142 的产物永远写不出 | 扫描量过；哪些计数算坏是推的 |
| H6 | E1 | F1 × F2（`.rc` 写法 × 分小批） | 打中（小） | 分批复用同一组 `.rc` 名，一件在写 `.rc` 之前被停（照共用约束一次停一个），「按起的次序逐个读」读到上一批的 0；F1 也没带上 command-safety 那条「派出去多少项就收回多少项」 | 演示量过；什么时候会被停是推的 |
| H7 | E2 | F16（只挪了 84 号） | 打中（中） | 40、86 号同样判「实验页有没有写」：新实验在第 6 步（第 7 步写页之前）两道都点名这个实验号；补上实验页与索引行之后点名消失 | 仓副本量过 |
| H8 | E1 | F10（实现员跑登记给它的 74 号） | 打中（小） | 74 号经 hook 直接放行、不经包装，里面的 `cargo test --release` 不在任何上限里；F9 修的正是这一类成本（O9），F10 把它变成实现员的必做步；74 号还登记给崩溃验证员，它的第 1b 步只包 54、55、57 | hook 量过；峰值 4.41–4.70 GiB 来自峰值表 |
| H9 | E2 | F6（本地辩方读攻方「输入」） | 小，推的 | 攻方「输入」要主 agent 给「样本文件名前缀」，辩方的文件名形态把样本名写死；F6 的换名只管带 `-local-attack` 的名字，前缀那一项两边各说各的；作废副本的名字跟提示文件名走（`ask-local.sh:67`） | 推的；门禁与脚本里除 66 号认提示文件名之外没有读样本名的（grep） |
| — | E2 | F8（55 号下限只算客机内存） | 没打中，推的 | 客机内存不预占（`vm-bench.sh:133` 没有 `-mem-prealloc`），6×2048 MiB 多半是松的上界而不是紧的下界；qemu 自己的开销没量过、峰值表里没有 55 号的行 | 推的 |
| — | E2 | F10 的 8G | 没打中 | 峰值表里整道 74 号经包装跑过：4.41 / 4.59 GiB（`capped.sh` 12 / 16、上限 16G / 24G），02:32 UTC 另一个会话在 8G 上限下（`capped.sh` 10）跑完、峰值 4.70 GiB（这一行顶掉了同一个键 02:1x UTC 我读到的 4.22 GiB，峰值表一个键只留最近一次） | 峰值表实测（别的会话量的，只在本机成立） |
| — | E1 | F3、F11、F12、F15 | 没打中 | 见「没打中的形状」 | — |
| — | E1 | F14 | 不归这条腿 | 本地攻方逐格对；这条腿只从命令侧加了 P1–P3 三格（F4 用） | — |

## E1

### H1：F9 让整轮门禁进一个 scope，三件事叠在一起

**被判的字面**：`gate-triage.md:21`（输入加「`gate.sh` 整条经内存包装的上限」）、`gate-triage.md:27`（「`gate.sh` 整条经内存包装跑，上限取输入给的，退出码 250–254 照那一条办」）、`gate-triage.md:28`（命令 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh <上限> bash .claude/scripts/gate.sh --staged`）、`main-agent.md:48`（「派门禁分诊员时给 `gate.sh` 整条经内存包装的上限」）。四处都没写这个上限怎么取。hook 放行这条命令（G1、G2、G3 三格 exit=0）。

**① 一个阶段撞顶，整轮作废，后面的阶段不跑。** 包装给 scope 设 `OOMPolicy=stop`（`run-with-memory-cap.sh:649`）。缩小版（`oom-demo.sh`，外层上限 150M，第 2 步分配 400 MiB，第 4 步在失败之后 2 秒）原样：

```
wrapped (cap 150M):
  step1 ran
  wrapper_exit=250
unwrapped (ulimit -v 300000 KiB stands in for one failing stage):
  step1 ran
  step2 exit=1
  step3 ran
  step4 ran (2 s after the failing step)
  exit=0
```

同一脚本另外四次跑：三次紧接着的第 3 步打出来了、隔 2 秒的第 4 步五次里一次都没打出来，包装五次都退 250。套到门禁上：`check.sh`（`gate.sh:457`）里一个测试二进制撞了外层上限，systemd 停掉整个 `gate.sh`，之后的几十道阶段不跑、汇总不打；而 `gate-triage.md:27` 让 250 照共用约束办——「那一次的输出不算结果……不绕开包装重跑」，分诊员手里没有任何一行可分诊。F9 之前同样的撞顶只让那一道红、其余照跑。

**② 外层按上限占着账，里层排不上。** 包装文件头 `run-with-memory-cap.sh:72` 写明「外层照它要的量占着账」，「要的量」是峰值表里这条命令上一次的峰值、表里没有就按上限算（`run-with-memory-cap.sh:38`）。缩小版（`nest-demo.sh`，slice 总上限 600M，里层上限 300M 只跑 `true`、至多等 6 秒）原样：

```
  inner_cap=300M inner_key=nest-inner-a inner_exit=0
outer_cap=200M outer_key=nest-outer-a outer_exit=0
  inner_cap=300M inner_key=nest-inner-b inner_exit=252
  内存不够排不上：等了 6 秒（上限 RUN_WITH_MEMORY_CAP_WAIT_SECONDS=6），slice 已占 401 MiB（回收不掉的用量 9 MiB + 账上还没涨到量的 392 MiB），这一条要 300 MiB（峰值表里没有这条，按它的上限 300M 算），总上限 600 MiB
outer_cap=400M outer_key=nest-outer-b outer_exit=0
  inner_cap=300M inner_key=nest-inner-c inner_exit=252
  内存不够排不上：等了 6 秒（上限 RUN_WITH_MEMORY_CAP_WAIT_SECONDS=6），slice 已占 300 MiB（回收不掉的用量 8 MiB + 账上还没涨到量的 292 MiB），这一条要 300 MiB（峰值表里没有这条，按它的上限 300M 算），总上限 600 MiB
outer_cap=300M outer_key=nest-outer-c outer_exit=0
  inner_cap=300M inner_key=nest-inner-d inner_exit=0
outer_cap=300M outer_key=nest-outer-c outer_exit=0
```

即：第一次跑，外层上限 + 里层上限 + slice 里已有的 > 总上限，里层就排不上（真门禁里等满默认 3600 秒退 252）；峰值表记下外层峰值之后，占账变成那个峰值。本机总上限此刻 40.1 GiB（`run-with-memory-cap.sh --status`：「slice singlefs-heavy.slice 的总上限 40.1 GiB」）；门禁里再经包装的有 59 号每条变异（默认 4G，`59-crates-mutation-replay.sh:43`）与 87 号经 `replay.sh` 的每一条（`REPLAY_MEMORY_CAP` 默认 8G，`replay.sh:24`）。所以 `gate.sh` 的上限第一次跑时不能超过约 40.1 − 8 − slice 里已有的；峰值表的峰值「含页缓存……偏大不偏小」（`run-with-memory-cap.sh:56`），之后占账也不会小多少（推的）。87 号是逐条串行复跑，每一条排不上都要等满一小时。

**③ 外层里要装下的东西，下界可能已经顶到上界。** `gate-triage.md:28` 直接起 `gate.sh --staged`，不经 `gate-staged.sh`，于是没有 `SINGLEFS_STAGED_TREE`：`stage-must-run.sh:28`「没有这个变量（有人直接跑 gate.sh）一律当成要跑」、`admission.py:745` 同句。55 号的六台虚机（`55-qemu-first-transaction.sh:39` 的六档、`:207` 并行起，每台 `VM_MEM` 默认 2048，`vm-bench.sh:14`）只要这一批碰了它的前缀就在外层 scope 里跑；它们不经包装，占的是外层。`check.sh` 的 `cargo test --all`（`check.sh:47`）也在外层里，没有线程上限（gate-triage 的命令里没有 `capped.sh`）。峰值表里同一个测试二进制在有线程上限时的实测（命令与原样输出，节选到这个二进制与 74 号；表不进 git，只在本机成立）：

```
$ grep -v '^#' research/scripts/memory-peaks.tsv | awk -F'\t' '$4 ~ /checker_known_bad_images|74-model-differential/ {printf "%s\t%.2fGiB\t%s\t%s\n", $2, $1/1073741824, $3, $4}' | sort
（整份 46 行，02:4x UTC 现跑；与这一格有关的原样摘出下面 8 行。其余 38 行：36 行在 `--` 后面点名一两个用例，峰值 1.28–3.60 GiB；1 行只编不跑 0.33 GiB；1 行是 2026-09-25T15:18:10Z 不带 `capped.sh`、4G 上限下整个二进制撞顶 4.00 GiB）
2026-09-25T21:56:40Z	16.00GiB	16G	bash research/scripts/capped.sh 12 cargo test --offline -p singlefs-harness --test checker_known_bad_images
2026-09-25T22:18:59Z	16.00GiB	16G	bash /home/fy5090/code/singlefs/research/scripts/capped.sh 12 cargo test --offline -p singlefs-harness --test checker_known_bad_images
2026-09-26T01:11:17Z	23.66GiB	24G	bash research/scripts/capped.sh 16 cargo test --offline -p singlefs-harness --test checker_known_bad_images
2026-09-26T01:25:03Z	3.33GiB	24G	bash /home/fy5090/code/singlefs/research/scripts/capped.sh 16 cargo test --offline --no-run -p singlefs-core -p singlefs-harness --lib --test second_transaction_step_four_rollback --test second_transaction_step_five_reuse --test checker_known_bad_images --test rollback_floor_written_into_the_system_
2026-09-26T01:26:35Z	24.00GiB	24G	bash /home/fy5090/code/singlefs/research/scripts/capped.sh 16 cargo test --offline -p singlefs-harness --test checker_known_bad_images
2026-09-25T22:32:37Z	4.41GiB	16G	bash research/scripts/capped.sh 12 bash .claude/gate.d/74-model-differential.sh
2026-09-26T02:03:05Z	4.59GiB	24G	bash research/scripts/capped.sh 16 bash .claude/gate.d/74-model-differential.sh
2026-09-26T02:32:19Z	4.70GiB	8G	bash research/scripts/capped.sh 10 bash .claude/gate.d/74-model-differential.sh
```

（「峰值 = 上限」的行是撞了自己的上限，包装按上限记，`run-with-memory-cap.sh:56`。）线程 12 撞 16G 两次、线程 16 一次 23.66 GiB 一次撞 24G；只编不跑（`--no-run`）3.33 GiB，所以大头在跑、不在编。本机 `nproc` = 32，`check.sh` 不设线程上限时按 32 跑：按每线程约 1.4–1.5 GiB 线性外推约 45–48 GiB（推的，没量过），比 slice 总上限 40.1 GiB 还大——那时外层上限取多少都撞，①就必然发生；就算不外推，下界也已在 24 GiB 以上（量过的撞顶），而②给的上界约 31–32 GiB，窗口至多几 GiB，还没算 55 号的 12 GiB 客机内存与别的会话同时占的（02:1x UTC 我看 `--status` 时账上就有别的会话一条 8.0 GiB）。

**四句**：
1. 分不分辨臂：分辨。①②只在「整条经包装」这条臂上发生；F9 之前（`gate.sh` 不经包装）没有外层 scope、没有外层占账。③是两条臂共用的前提（`check.sh` 不限线程），但只有 F9 这条臂把它变成整轮作废。
2. 系统当时看不看得到：给上限的是主 agent，它能看峰值表与 `--status`，但定义里没有一句要它看，也没给算法；分诊员拿到的只有一个数。
3. 满足的是字面的哪一分句：E1「F9 `gate.sh` 整条经内存包装在 scope 里有没有别的影响」。
4. 跑前条款的改法（F9 本身，判决第三节只给了这一个）在打中的格上：①②就是它引出来的，改法本身在这几格上是中的。

**改法（只在我的模型上量过机制、被攻过零轮；各修哪一格见文末改法表）**：甲、撤回 F9，`gate.sh` 照旧不经包装，O9 那一格（`check.sh`、15、74 号的 cargo 不在上限里）另立账；乙、保留 F9，同时给算法并限线程：`gate.sh` 整条经 `capped.sh N` 再经包装，上限 ≥ max(55 号六台 × `VM_MEM` + 余量, 峰值表里 N 线程下 `cargo test --all` 的峰值)，且 ≤ slice 总上限 − 8G − 余量，窗口空就报主 agent、不开跑；丙、分诊员改跑 `gate-staged.sh`（hook 放行，G3 exit=0），55、57、59 在输入没变时复用，外层里少掉虚机那一块（只修③的一半）。

### H2：F7「逐字节一致就删掉新文件」删掉的正是 69 号要的那一份

**被判的字面**：`experiment-runner.md:31`「重跑已有实验时拿新文件与已有的那份比：逐字节一致就删掉这一次的新文件、不新存」。

**情形**：变异分诊之后补断言（`main-agent.md:59` 那一行：「要补取样点、补断言的：……`research/` 那侧 `experiment-designer` 写重跑登记 → `experiment-runner`」），执行员照第 2 步改装置源码（加单测、加钉绝对值的断言、`naming-lint.sh` 要它改名）或照第 3 步追加变异表，装置输出一个字节不变。照 F7：新文件逐字节一致 → 删掉。第 6 步跑登记给它的 69 号（`stage-owners.tsv:58`）：判据一「改动范围里每个还在盘上的 `research/e7-index-bench/src/bin/e<号>_*.rs` 或 `research/mutations/e<号>_*.tsv`，`research/results/` 下要有一份这个实验号的产物……而且它的 mtime 不早于那份源码」（`69-evidence-in-repo.sh:16`），留下的只有旧的那份 ⇒ 红。红句的出路是「跑一次把产物存进 {results_dir}/，并在 research/scripts/replay.sh 把这个实验的登记行指到它」（`69-evidence-in-repo.sh:259`）；照做再跑一次，还是逐字节一致，F7 再删。例外只有两类：只改了 `//` 注释的源码，与 `.claude/gate.d/stage-inputs.tsv` 登记了指纹的实验（今天只有 E142 两行）。

69 号判红的形状由它自己的样本钉着：`fixtures/69-evidence-in-repo.sh/red/setup.sh` 第 10、12、26 行放「装置改过、产物是旧的」那一格，`expect` 要「research/results/ 里却没有一份不比它旧的产物」。我没在副本上跑 69 号：它要 git（非 git 目录退 77），而我不做 git 写操作，所以这一格是按判据与样本推的。

这一半在 F7 之前就有（旧字面「与它逐字节一致就不新存」同样不留新文件）；F7 改的正是这一句，把「不新存」写成了「删掉」，没碰到 69 号。第一轮 O7 修的是它与共用约束 ⑤ 的冲突，这一格是它与 69 号的冲突，没被攻过。

**四句**：分辨臂——分辨：「删掉 / 不新存」这条臂中，「这一轮改过装置或变异表时留着新文件」那条臂不中。系统看得到判别它的东西——看得到：执行员自己改的装置，自己知道。字面分句：E1「F7 ……会不会删到不该删的」。改法在这一格上：F7 本身就是打中的那一句。

**改法（推的，没实现没跑）**：「逐字节一致、而且这一轮没改这个实验的装置源码与变异表（只改 `//` 注释的不算），才删新文件；改过的，留新文件、实验页两份都点名、写明逐字节相同」。或者把实验登记进 `stage-inputs.tsv` 走指纹（69 号那一半就按指纹判、不看 mtime）。

附带（小，推的）：F7 说新文件名带「日期或 `rN`」，没说 `rN` 怎么接；40 号把 `*.r[0-9].out` 当「逐轮复跑的中间件」跳过（`40-results-cited.sh:33`），写成 `e<号>-<简称>.r2.out` 的新产物 40 号不要求点名。

### H3：F4 的「小流」界照字面能越过

**被判的字面**：`three-way-attack.md:33`（测试目标名字不带 `layer0`；每次跑前用 `closed_form_state_count` 算这一次全量状态数，不超过约 10⁶；几段合起来超过的分几次跑；一段自己超过的不跑、交主 agent）。

**① 名字是唯一的界。** 在一份只含 Cargo 清单与 `crates/` 源码的临时副本里，把 `second_transaction_step_three_acquisition_barrier_layer0.rs` 原样拷成 `tests/proto_acquisition_barrier.rs`、把 `second_transaction_step_zero_layer0.rs` 拷成 `tests/proto_step_zero.rs`，用攻方腿的身份喂 `heavy-test-guard.sh`（原样，`@COPY@` 是那份副本）：

```
P1	heavy	three-way-attack	fg	exit=0	
P2	heavy	three-way-attack	fg	exit=2	✗ 重型测试被拒：cargo test --test second_transaction_step_three_acquisition_barrier_layer0（层 0）：three-way-attack 不跑「层 0」
P3	heavy	three-way-attack	fg	exit=0	
```

（P1：`nice -n 19 bash …/run-with-memory-cap.sh 8G cargo test --release -p singlefs-harness --test proto_acquisition_barrier -- --nocapture`；P2 同一条换回原名；P3 换成 `proto_step_zero`。）拷来的是入库的层 0 流本身，不是「自己原型里的小流」；字面只要求名字不带 `layer0`，三条都满足。

**② 10⁶ 比单条层 0 流的全量还大。** 入库断言里的状态数：`second_transaction_step_three_acquisition_barrier_layer0.rs:195` 是 `1 + 15 + 1023`（1039 个），`second_transaction_step_three_formatted_pool_layer0.rs:157` 是 22 个；超过 10⁶ 的是 `second_transaction_step_zero_layer0.rs:590` 的闭式 2,104,413 与 `second_transaction_parallel_line_one_layer0.rs:318` 的 5,505,123。字面按「每次」算、没有总量，「几段历史合起来超过的，分几次跑」把全量拼回来的路是开着的；唯一的总量约束是 `three-way-attack.md:31` 的「估出来超过 40 分钟，先缩」，那是时长、不是状态数。

**③ 派发闸拒掉主 agent 转述这一条。** 派发闸的「层 0」名字表是 `layer\s*0(?!\d)`，左边不要词界（`runner-dispatch-guard.sh:126`）。原样：

```
D1	dispatch	three-way-attack	fg	exit=2	✗ 派 three-way-attack 的提示要它跑「层 0」：「照定义第 3b 步，在原型里调 `enumerate_layer0` 跑小流，每次不超过约 10⁶ 个状态」
D2	dispatch	three-way-attack	fg	exit=2	✗ 派 three-way-attack 的提示要它跑「层 0」：「崩溃状态照层 0 的枚举域取，用 enumerate_layer0_versions 跑每段历史」
D3	dispatch	three-way-attack	fg	exit=2	✗ 派 three-way-attack 的提示要它跑「层 0」：「原型用 closed_form_state_count 先算状态数，再用 enumerate_layer0 跑」
D4	dispatch	three-way-attack	fg	exit=0	
```

D2 差不多就是 F3 写进 3b 的原句。主 agent 只能不点函数名（D4 那种写法）或改写成闸认可的句式；这一格是派发闸文件头自己承认的「会误拒」，小。

另一处字面张力（小）：「一段历史自己就超过的，那段不跑」按状态数砍掉一段历史的全部崩溃状态，而 `agent-common.md:53` 写「崩溃点测试不衡量时间成本，也不为省时间缩范围」。F4 让它交主 agent，没有自己砍，所以只记张力。

**四句**：分辨臂——①②分辨「按名字 + 每次上限」这条臂与「按来源（自己造的历史）+ 总量上限」那条；③不分辨（任何允许原型调枚举函数的写法都会撞派发闸，病根在闸的名字表）。系统看得到——`heavy-test-guard.sh` 只看得到名字，看不到测试内容是不是拷来的，所以这一界只能写进定义、靠腿守。字面分句：E1「F4『原型里调枚举函数跑小流不算重型』的界能不能被照字面越过」。改法在打中的格上：F4 本身就是这一句。

**改法（推的）**：3b 写明原型的历史由腿自己造，不拷、不 `include!`、不 `mod` 引 `crates/singlefs-harness/tests/` 下名字含 `layer0` 的文件；给这一轮一个状态总数上限（例：全部原型合计不超过约 10⁶），超过的交主 agent。③不宜改派发闸的名字表：在 `layer` 左边加词界会让它自己的自证 `runner-dispatch-guard.sh:555`（「跑 first_transaction_step_seven_layer0 的全量」要判 2，那里 `layer0` 左边同样是下划线）翻红；只能在 3b 旁写一句「派发提示里不点枚举函数名，写『照第 3b 步』」。

### H4：F13 放宽之后放过没有出处的断言

**被判的字面**：`ask-user-claim-guard.sh:57`——`/` 之后从 `{ASCII_PATH_CHARACTER}+` 换成 `{FILE_NAME_CHARACTER}+`（`[\w.-]`，认汉字），这一支不要求扩展名；文件头 `:23` 写的判法是「/ 前面紧挨着的那个字要是 ASCII」。于是「ASCII 串 / 汉字串 冒号 数」都算「文件:行号」。HEAD（73ba4a4）那一版与今天的一版并排喂同一句（`probe.py` 把 HEAD 那一版 `git show` 到临时目录，只读），原样：

```
Q1-head	ask-head	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
Q1-now	ask	主 agent	fg	exit=0	
Q2-head	ask-head	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
Q2-now	ask	主 agent	fg	exit=0	
Q3-head	ask-head	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
Q3-now	ask	主 agent	fg	exit=0	
Q4-head	ask-head	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
Q4-now	ask	主 agent	fg	exit=0	
Q5-head	ask-head	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
Q5-now	ask	主 agent	fg	exit=0	
Q6-head	ask-head	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
Q6-now	ask	主 agent	fg	exit=0	
Q7-head	ask-head	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
Q7-now	ask	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
Q8-head	ask-head	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
Q8-now	ask	主 agent	fg	exit=0	
Q9-head	ask-head	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
Q9-now	ask	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
```

句子（`cases-f13.json`）：Q1「O3/O8两格：2 格一定该升成打中」、Q2「54/55号：2 道一定要带前缀」、Q3「F1/F4两条：2 处一定都要改」、Q4「E142/第十六次跑：3 个字段一定是 0」、Q5「research/prompts下的判决：3 条一定都对」、Q6「抓到40/无效：0，这张表一定全抓了」；对照 Q7「第十六次跑：3 个字段一定是 0」（没有斜杠）、Q9「E1/E2 两格：3 条一定都打中」（斜杠后是 ASCII、中间有空格）两版都拒；Q8 是 F13 要放的中文文件名出处，改前拒、改后放。Q1–Q6 都是没有出处的推断，改前拒、改后放——F13 自证加的三格「必拒」里没有「ASCII / 汉字：数」这一形，所以没抓到。

**四句**：分辨臂——分辨（改法 G1 在这 6 句上照拒，见下）。系统看得到——看得到，判别只靠句子本身的字。字面分句：E1「F13 弹窗闸放宽之后会不会放过一个没有出处的断言」。改法在这几格上：F13 本身就是打中的那一处。

**改法 G1（只在我的模型上量过、被攻过零轮）**：第一支 `/` 之后改成「全 ASCII 的名字（HEAD 原样），或名字里可带汉字、但以登记过的扩展名收尾」：`(?:{ASCII_PATH_CHARACTER}+|{FILE_NAME_CHARACTER}*\.(?:{EXTENSION_ALTERNATION}))`。副本上（`f13-fix-g1.sh`，仓里的 hook 没动）原样：

```
copy selftest exit=0: 自检通过（查了 30 种） 
Q1-fixG1	exit=2
Q2-fixG1	exit=2
Q3-fixG1	exit=2
Q4-fixG1	exit=2
Q5-fixG1	exit=2
Q6-fixG1	exit=2
Q7-fixG1	exit=2
Q8-fixG1	exit=0
Q9-fixG1	exit=2
```

G1 同时要补一格自证「必拒：ASCII / 汉字跟冒号与数不是文件:行号」（Q1 那句），先在今天的判法上证红（Q1-now 放行就是那一格的红）。

### H5：F5 按名字里四个词认，漏一类、错点一大类

**被判的字面**：`experiment-runner.md:33`（字段名表示违例、不匹配、歧义、失败的计数，名字里带 `violation`、`mismatch`、`ambiguous`、`fail` 的，取值不是 0 的逐个点名；一个都没有就写「违例、不匹配、歧义、失败类计数都是 0」）。

照字面扫 `research/results/` 下全部判决行（`verdict-scan.py`），原样：

```
UNNAMED	cells_where_pad_taller_than_legacy=2	1 处	例 research/results/e146-livelist-entry-width-2026-09-16-tree-table-200.out:129
UNNAMED	grid_cells=21	1 处	例 research/results/e146-livelist-entry-width-2026-09-16-tree-table-200.out:129
UNNAMED	journal_differing_states=3	6 处	例 research/results/e142-first-txn-dry-run-2026-09-25-r17-layer0.out:608
UNNAMED	layout_total=413	1 处	例 research/results/e147-system-configuration-recompute-from-layout-2026-09-13.out:13
UNNAMED	legacy_entry_bytes=24	1 处	例 research/results/e146-livelist-entry-width-2026-09-16-tree-table-200.out:129
UNNAMED	legacy_leaf_fanout=676	1 处	例 research/results/e146-livelist-entry-width-2026-09-16-tree-table-200.out:129
UNNAMED	pad_entry_bytes=35	1 处	例 research/results/e146-livelist-entry-width-2026-09-16-tree-table-200.out:129
UNNAMED	pad_leaf_fanout=463	1 处	例 research/results/e146-livelist-entry-width-2026-09-16-tree-table-200.out:129
UNNAMED	prepaid_legacy=24	1 处	例 research/results/e146-livelist-entry-width-2026-09-16-tree-table-200.out:129
UNNAMED	prepaid_pad=35	1 处	例 research/results/e146-livelist-entry-width-2026-09-16-tree-table-200.out:129
UNNAMED	today_resolves_registered=1000	1 处	例 research/results/e145-self-describing-node-header-2026-09-16-tree-table-200.out:56
UNNAMED	variants=6	1 处	例 research/results/e147-system-configuration-recompute-from-layout-2026-09-13.out:13
UNNAMED	variants_fitting_512=6	1 处	例 research/results/e147-system-configuration-recompute-from-layout-2026-09-13.out:13
NAMED-COUNT	control_violations_ok=true	15 处	例 research/results/e142-r18-controls-2026-09-26.out:226
NAMED-COUNT	g6_violations_zero=true	10 处	例 research/results/e142-r18-controls-2026-09-26.out:226
NAMED-COUNT	layer0_violations=zero	1 处	例 research/results/e142-first-txn-dry-run-2026-09-25-r17-layer0.out:608
NAMED-COUNT	text_number_mismatches_zero=true	10 处	例 research/results/e142-r18-controls-2026-09-26.out:226
合计 NAMED-FALSE 92、NAMED-COUNT 36、UNNAMED 18
```

漏：`journal_differing_states=3`（「journal 读法不同」的状态数，`crash.rs` 里与 `violations`、`failed_states` 并列的计数）名字里没有那四个词，字面不要求点名；它是不是「坏」要看登记（推的）。错点：`control_violations_ok=true`、`g6_violations_zero=true`、`text_number_mismatches_zero=true` 是布尔、`layer0_violations=zero` 是字，取值都「不是 0」，字面要当成「违例计数非 0」逐个点名；E142 近期每份产物的判决行都带 `control_violations_ok`，「……计数都是 0」那一句对它们永远写不出。

**四句**：分辨臂——分辨（按取值类型判的写法两处都不中）。系统看得到——看得到，字段名与取值都在那一行上。字面分句：E1「F5」一行（判决第三节 F5）。改法在这几格上：F5 本身就是打中的那一句。

**改法（推的）**：只对取值是整数的字段套这一条；布尔与 `zero` 这类字走 `false` / `not_run` 那一条；四个词之外加 `differ`、`missed`、`false_alarm`、`wrong`，或者干脆「取值是非 0 整数、而跑前登记写着它应当是 0 的字段」。

### H6：F1 的 `.rc` 写法与 F2 的「分小批」叠起来，会读到上一批的值

**被判的字面**：`three-way-attack.md:38`（每件写 `<草稿目录>/<名字>.rc`，单独一个不带参数的 `wait`，按起的次序逐个读）、`three-way-attack.md:34`（分小批）、`agent-common.md:64`（④ 的出路同一种写法）。

**前台与 run_in_background 各是什么结局**：检出 hook 对这种写法两处都放行，循环里起、分批起也放行（原样）：

```
B1	detector	three-way-attack	rib	exit=0	
B2	detector	three-way-attack	rib	exit=0	
B3	detector	three-way-attack	fg	exit=0	
B4	detector	three-way-attack	rib	exit=0	
```

（B1：`for s in 1 2 3; do { …; echo "$?" > …/s$s.rc; } & done; wait; …`；B2：外层按批、内层按件，每批一个 `wait`；B3 是 B1 放前台；B4 是两件展开写。）run_in_background 里全部作业等齐才发完成通知，没有问题。前台受 Bash 单次上限（共用约束写前台 `timeout` 不超过 240000 毫秒）：超时之后这一条被工具停掉，还没写 `.rc` 的那几件没有新值——子进程会不会一起被停我没量过（推的）。

**打中的情形**：分小批时每批用同一组 `<名字>`（字面没要求每批换名字、开跑前删旧 `.rc`），第二批里一件卡住，照共用约束 `agent-common.md:49` 一次停一个自己起的进程，停掉的那一件没写 `.rc`；之后「按起的次序逐个读」读到的是第一批留下的值。演示（`rc-stale-demo.sh`）原样：

```
batch1: s1=0 s2=0
<bash: 作业被 KILL>                  { sleep 2; echo "$?" > "$work/s2.rc"; }
batch2: s1=1 s2=0   # s2 这一件没写，读到的是第一批的
```

command-safety 那一节自己有一条管完整性：「派出去多少项，就要收回来多少项。……收的时候数一遍，与派出去的条数对不上就整道红」（`command-safety.md:124`–`125`）。F1 没把这一条带进来；而且旧文件在场时「数一遍」也数得齐，只有「每批新名字或开跑前删掉」才挡得住。

**四句**：分辨臂——分辨（每批换名的写法不中）。系统看得到——看得到，腿自己起的、自己停的。字面分句：E1「F1 的并行收退出码写法在前台与 run_in_background 里各是什么结局」。改法在这一格上：F1 本身。

**改法（推的）**：F1 那一句后面加「每批开跑前删掉这一批要用的 `.rc`（或名字里带批号）；读的时候缺一个就算那一件失败，与起的件数对不上整批作废」。

### H8：F10 让实现员必跑的 74 号不在任何上限里

**被判的字面**：`implementation-writer.md:29`（交回前的验证加进「阶段归属表登记给你的门禁阶段（共用约束「门禁」一节，74 号在内）」）；74 号登记给实现员与崩溃验证员（`stage-owners.tsv:62`）。

hook 原样（T1、T2 是照共用约束「门禁」一节的写法直接跑，T3 是经包装）：

```
T1	heavy	implementation-writer	fg	exit=0	
T2	heavy	crash-verifier	fg	exit=0	
T3	heavy	implementation-writer	fg	exit=0	
```

74 号里面是 `cargo test --release -p singlefs-harness --test "$TEST_BINARY"`，跑编出来的代码；直接起这一道时它不在任何 scope 里。第一轮 O9 记的代价正是「`check.sh`、15、74 号起的 cargo 不在任何上限里」，F9 只在 `gate.sh` 那条路上收了它（而且带出 H1），F10 把实现员单跑 74 号写成了必做步；崩溃验证员第 1b 步只包 54、55、57（`crash-verifier.md:26`），它那一份 74 号同样不包。量级：整道 74 号经包装的峰值 4.41–4.70 GiB（H1 节峰值表那三行），不大，所以记小。

**四句**：分辨臂——分辨（「登记给你的阶段里跑编出来代码的也经包装」那条臂不中）。系统看得到——`heavy-test-guard.sh` 只认命令位置上的 cargo，看不进门禁阶段的正文（F14 清单那一句「门禁阶段……按名字判、不读正文」）。字面分句：E1「F10」。改法：F10 本身。

**改法（推的）**：实现员第 1b 步把「阶段归属表登记给你的、里面跑 cargo test 的阶段（74 号）」也列进经包装的范围，写法同崩溃验证员第 1b 步；崩溃验证员第 1b 步同样加上 74 号。

## E2

### H7：40、86 号同样在第 7 步之前判红（F16 只挪了 84 号）

**被判的字面**：`experiment-runner.md:35`（第 6 步「84 号除外：它放到第 7 步写完实验页之后跑」）。执行员还登记着 40 号（`stage-owners.tsv:26`，判据「`research/results/` 里的实验产物，必须在 `kb/experiments.md` 里被点名」，`40-results-cited.sh:4`）与 86 号（`stage-owners.tsv:50`，「`kb/experiments/` 里就必须有对应编号的正文文件」，`86-experiment-orphans.sh:5`）。

仓副本演示（`stage-demo.sh`：rsync 整仓、不带 `target` 与 `.git`；放进新实验号 E999 的装置源码与产物、还没写实验页；再补实验页与索引行），原样：

```
[baseline] 40-results-cited.sh exit=1 e999_lines=0
[baseline] 86-experiment-orphans.sh exit=0 e999_lines=0
[with-e999-no-page] 40-results-cited.sh exit=1 e999_lines=1
         research/results/e999-probe-2026-09-26.out
[with-e999-no-page] 86-experiment-orphans.sh exit=1 e999_lines=1
          E999  research/results/e999-probe-2026-09-26.out 
[after-step7-page] 40-results-cited.sh exit=1 e999_lines=0
[after-step7-page] 86-experiment-orphans.sh exit=0 e999_lines=0
```

40 号在副本里基线就红：副本不在 git 里，它判全部（主仓里此刻它也红，点名 `e156-alloc-basis-counts-2026-09-22-stage1.out` 等，不是这一轮的）；这里只看点没点名 E999。86 号 0 → 1 → 0，40 号多出 E999 那一行、写完页又消失：与第一轮 D2-e（84 号）同形。86 号的出路里还有一句「若那轮的结论不打算入库，就把 research 下那些文件删掉」，F16 写「这一次不写实验页的」情形时，执行员照这句出路会删自己刚跑的产物（推的）。

**四句**：分辨臂——分辨（40、86 与 84 一起挪到第 7 步之后的写法不中）。系统看得到——看得到，实验页还没写是执行员自己知道的。字面分句：E2 第一条线索。改法：F16 本身，它只挪了 84。

**改法（副本上量过「写完页之后两道不再点名 E999」那一半，其余推的）**：第 6 步的「84 号除外」改成「40、84、86 号除外」，三道一起放到第 7 步写完实验页之后；「这一次不写实验页的」那一支，40、86 的红照写、不照 86 号的出路删产物。

### H9（小，推的）：本地辩方的样本名与攻方「输入」里的前缀各说各的

F6 让辩方读攻方的「输入」一节，那一节要主 agent 给「样本文件名前缀」（`three-way-local-attack.md:19`），样本写成 `<前缀>-output-s<n>.md`（`three-way-local-attack.md:27`）；辩方的文件名形态把样本写死成 `research/prompts/<轮>-local-defense-output-s<n>.md`，换名规则只管「攻方定义里出现的 `-local-attack` 文件名」（`three-way-local-defense.md:19`），`<前缀>-output-s<n>.md` 里没有 `-local-attack`，这条规则够不着它。主 agent 照攻方「输入」给了一个别的前缀时，辩方照哪一边没有写；作废副本的名字跟着提示文件走（`ask-local.sh:67`），与样本名不一定同前缀。`grep` 门禁与脚本，除 66 号认提示文件名之外没有谁读样本名，所以后果只是名字对不上、主 agent 找文件，记小。改法（推的）：辩方文件名形态那一行加「输入里的样本文件名前缀就是 `research/prompts/<轮>-local-defense`」。

### F8 的 55 号下限（没打中，推的）

`crash-verifier.md:20` 的下限是 `MODES` 档数 × `VM_MEM` = 6 × 2048 MiB = 12 GiB。qemu 进程自己的开销与宿主侧确实没算；但客机内存不预占（`vm-bench.sh:133` 只有 `-m "$VM_MEM"`，没有 `-mem-prealloc` 或内存后端），cgroup 只记客机真碰过的页，六台一起碰满 2 GiB 的可能小，所以 12 GiB 更像松的上界而不是紧的下界。盘镜像在 `/tmp`，本机 `/tmp` 是 ext4（`df -hT /tmp`：`/dev/nvme0n1p2 ext4`），页缓存回收得掉，不是 tmpfs 的 shmem。峰值表里没有 54、55、57 号任何一行（`grep` 键名含 `54-layer0`、`55-qemu`、`57-lkmm` 的行：0），两个方向都没量过。真正缺的是 54、57 号：F8 只给了 55 号的算法，54、57 号的上限主 agent 没有依据可取（包装文件头的例子写 54 号 `--full` 用 16G，那是例子不是算法）——这一格不在 55 号的线索里，记在限度里。

### F10 的 8G（没打中）

`74-model-differential.sh:47` 建议的 8G 与 `replay.sh:24` 的默认值一致。整道 74 号（比建议命令多一段编译）经包装跑过三次：4.41 GiB（`capped.sh 12`、16G 上限）、4.59 GiB（`capped.sh 16`、24G 上限）、4.70 GiB（`capped.sh 10`、**8G 上限**，02:32 UTC 另一个会话跑的），原样见 H1 节峰值表最后三行。不设线程上限时（按 32）没量过：从 10 → 12 → 16 线程的 4.70 → 4.41 → 4.59 看不出随线程涨，推 8G 够用。

## 改法表（每一格：量过 = 副本或模型上贴了原样输出；推的 = 没实现没跑）

| 改法 | 修哪一格 | 在那一格上 | 被攻过几轮 |
|---|---|---|---|
| F9 甲：撤回整条包装 | H1 ①② | 推的（撤回之后没有外层 scope，①②的机制不存在；O9 那一格回到原状） | 零轮 |
| F9 乙：`capped.sh N` + 上限窗口算法 | H1 ①②③ | 推的（窗口算法没实现；N 取多少才让 `cargo test --all` 落进窗口没量） | 零轮 |
| F9 丙：分诊员改跑 `gate-staged.sh` | H1 ③的虚机那一半 | hook 放行量过（G3 exit=0）；复用是否成立推的 | 零轮 |
| F7 改：这一轮改过装置或变异表就留新文件 | H2 | 推的 | 零轮 |
| F4 改：原型历史自己造、不拷层 0 流，加总量上限 | H3 ①② | 推的（hook 看不见来源，只能靠字面） | 零轮 |
| 3b 旁注「派发提示里不点枚举函数名」 | H3 ③ | 量过（D4 exit=0） | 零轮 |
| G1：`/` 后的汉字名要带扩展名 | H4 | 量过（副本自证 30 种过、Q1–Q6 照拒、Q8 放） | 零轮 |
| F5 改：只对整数字段、加词或按登记的 0 | H5 | 推的 | 零轮 |
| F1 改：每批删旧 `.rc`、缺一个算失败、数件数 | H6 | 推的（演示只证了病，没证改法） | 零轮 |
| F16 改：40、84、86 一起挪到第 7 步之后 | H7 | 半量过（副本上写完页之后两道不再点名 E999）；其余推的 | 零轮 |
| 实现员、崩溃验证员第 1b 步把 74 号列进经包装 | H8 | 推的 | 零轮 |
| 辩方前缀写死 | H9 | 推的 | 零轮 |

## 没打中的形状

- **F1 的写法被 hook 拒**：循环起、分批起、展开写、前台与 run_in_background 共 4 种（B1–B4），全放行；第一轮修定义的 agent 另喂过 5 种（背景材料附录 `f1-out.txt`），全放行。
- **F3 的函数名与判法对不上**：`crash.rs` 里 `enumerate_layer0`、`enumerate_layer0_versions`、`closed_form_state_count`（`crash.rs:524`，闭式 `1 + Σ(2^|段| − 1)`）都在，枚举每个状态都跑 `check_pool_image`（`crash.rs:1321` 那一行调用）；`SparseBlockDevice`、`SparseDevice`、`MemoryPool` 都在。没找到字面与代码不符。
- **F11、F12 的描述与 hook 不符**：逐条对了 `runner-dispatch-guard.sh` 文件头第 2–3、16、44 行说的三条、`heavy-test-guard.sh:131` 的放行表、settings 注册表（修定义的 agent 报告第二节）；没找到不符。只有「主 agent 与子 agent 一样拒」对重型闸不完全成立，但条目里写了「主 agent 不带前缀也拒」，不记。
- **F15 的 59 号输出路径拿不到**：崩溃验证员超过单次上限的阶段后台跑、写 `<草稿目录>/<阶段>.log`，59 号整表一般都超过；没造出拿不到的情形。
- **F13 的第二支（不带 `/`、`[\w-]` 开头、要扩展名）**：试了带中文名加扩展名的写法，都是真文件名形态，没造出没有出处却被当成出处的自然句子。
- **F8 / F10 的上限数**：见 E2 两节，峰值表与代码都不支持「打中」。
- **F4 的内存**：层 0 流在 4G 上限下的实测峰值 0.18–2.75 GiB（峰值表里上限 4G、键含 `singlefs-harness` 与 `layer0` 的行，都是点名单个用例跑的），10⁶ 状态的原型不至于撞 8G，没打中。

取样：hook 探针 37 格（F13 18 格、hooks 13 格、派发闸 6 格——含对照格）；包装演示 2 个场景（嵌套 4 次、撞顶共跑 6 次，其中 5 次带隔 2 秒的第 4 步）；40 / 86 号副本演示 1 次（3 个时点）；判决行扫描覆盖 `research/results/` 下全部含 `name=verdict` 的行。

## 这条腿自己的限度

- H1 的机制（撞顶停整个 scope、外层占账挡里层）只在私有 slice 的缩小版上量过；真门禁里 `check.sh` 在 32 线程下要多少没量（按峰值表线性外推，推的），55 号六台虚机真碰多少页没量（QEMU 是重型，不跑）。「窗口可能是空的」是推的；「窗口在 24 GiB 以上、约 31–32 GiB 以下，且没有任何一处写它怎么取」是量过的下界加算出来的上界。
- 峰值表 `research/scripts/memory-peaks.tsv` 不进 git、别的会话随时在写，一个键只留最近一次（我读到的 74 号 4.22 GiB 那一行在读完之后被 02:32 UTC 的 4.70 GiB 顶掉了）；H1 节贴的那几行是 02:4x UTC 现跑的原样，复跑时数可能变。
- H2 没在副本上跑 69 号（它要 git，非 git 目录退 77；我不做 git 写操作，不在副本里 `git init`）。
- H5 里哪个计数「算坏」要看各实验的跑前登记，我没逐个去读，`journal_differing_states` 是不是坏计数是推的。
- H6「前台超时会不会连子进程一起停」没量。
- 两条攻方腿的分工：F14 清单逐类对 `classify` / `cargo_use` 是本地攻方的格，这条腿只加了 P1–P3（为 F4）；我没读本地攻方与云端辩方的任何文件。
- 按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」：上面「没打中」的每一条都只是这一次没造出来，不是「不存在」。
- 改法全部被攻过零轮；标「量过」的也只在我的模型或副本上量过，入库装置上没做。

## 没做什么

- 没改任何被判的文件；没有 git 写操作；没编译 Rust、没跑任何 `cargo test` 或重型测试、没跑 QEMU、herd7、门禁 54 / 55 / 57 / 59 / 87。
- 门禁阶段在仓副本里跑了 40、86 号（轻阶段）；主仓里跑过 69 号一次（02:5x UTC，核这份报告与模型目录没有把 `/tmp` 路径当依据：它点名的 25 处里没有这一条腿的文件；它红在 E142 的装置指纹与别的轮的背景材料上，不是这一条腿的），40 号一次看基线（红，点名的是 `e156-alloc-basis-counts-2026-09-22-stage1.out` 等，不是这一轮的）。
- 包装演示在自己开的 `singlefs_r2opus_nest_<pid>.slice`、`singlefs_r2opus_oom_<pid>.slice` 里跑，跑完 `stop` + `revert`；收尾 `systemctl --user list-units --all 'singlefs_r2opus*'` 数出 0 个。
- 草稿目录 `/tmp/claude-1000/defs-m2-closeout-r2-opus/` 里的仓副本（`copy/`、`repo/`）与 hook 副本（`f13fix/`、`probe-ask-head-*`）都已删掉；留着的是探针的检出记录、`quotes.list` / `quotes*.out`（`quotes.list` 已拷进模型目录）、`40-main.log`、`69.log`、`verdict-fields.txt`、`peaks-now.txt` 这类文本草稿，不进仓，主 agent 用不上可以删。
- E3（第一轮判决复核）不归这条腿；F14 逐格对表不归这条腿。
- 这条腿登记给自己的门禁阶段：`stage-owners.tsv` 里没有登记给 `three-way-attack` 的行，没跑。

## 附录 A：引文原行（行号取自被引文件本身）

命令（在仓根）：`while read -r f n; do printf '%s:%s\t' "$f" "$n"; awk -v n="$n" 'NR==n' "$f"; done < research/prompts/defs-m2-closeout-r2-opus-model/quotes.list`，62 行，原样（02:5x UTC 现跑；`quotes.list` 不进 `rerun.sh`，被引文件改了行号会移）：

```
.claude/agents/three-way-attack.md:31	   - 正式跑之前先跑一小段，按它估全量的挂钟；估出来超过 40 分钟，先缩历史、候选与几何的取样，报告里写明缩了什么、缩前缩后各多少、估时怎么算的。
.claude/agents/three-way-attack.md:33	   - 在自己的原型里这样调枚举函数跑小流不算重型：测试目标的名字不带 `layer0`（带了 `heavy-test-guard.sh` 按名字拒）；每次跑之前用同一份文件的 `closed_form_state_count` 算出这一次全量的状态数，不超过约 10⁶ 个（几段历史合起来超过的，分几次跑）；一段历史自己就超过的，那段不跑，写进报告交主 agent。
.claude/agents/three-way-attack.md:34	   - 随机跑批分小批：每批的段数按先跑一小段估出的时长定，不给批设限时（包装的 `RUN_WITH_MEMORY_CAP_TIME_LIMIT`、外面套 `timeout` 都不用）；每批的段数与估时写进报告。
.claude/agents/three-way-attack.md:38	   - 并行起的几件，每件把退出码写进自己的文件（`{ <命令>; echo "$?" > <草稿目录>/<名字>.rc; } &`），最后单独一个不带参数的 `wait` 等齐，再按起的次序逐个读 `.rc` 文件（共用约束「执行前拒绝的写法」那一条的 ④）；不用 `wait "$pid"` 收退出码。
.claude/agent-common.md:49	- 要停自己起的一条链（脚本、后台任务）时，先列出它的整棵子进程（`ps -o pid,ppid,args --ppid <pid>` 逐层往下），从最底层起逐个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`（`proc.py` 只停给定的那一个，不带子进程），停完再列一遍，确认一个不剩；不许留下正在跑的那一步「让它跑完」。
.claude/agent-common.md:53	- **崩溃点测试不衡量时间成本，也不为省时间缩范围**：要加崩溃点、要多枚举一档，就加。挂钟长不是收窄它的理由，也不用先算它值不值。
.claude/agent-common.md:64	    ④ run_in_background 里有作业以单独的 `&` 收尾，之后同一条命令里没有 `wait`。要并行就每件把退出码写进自己的文件（`{ <命令>; echo "$?" > <文件>; } &`），最后单独一个不带参数的 `wait` 等齐，再按派活的次序逐个读那几个文件；不用 `wait "$pid"` 收。
.claude/singlefs-ai-sop/rules/command-safety.md:124	**派出去多少项，就要收回来多少项。** 并行之后少跑一项是不报错的：那一项的文件不存在，
.claude/singlefs-ai-sop/rules/command-safety.md:125	循环少转一圈，末尾照样报绿。收的时候数一遍，与派出去的条数对不上就整道红
.claude/agents/experiment-runner.md:31	4. 跑产物进 `research/results/`。那里已有的产物一个都不删、不挪、不覆盖（共用约束「执行前拒绝的写法」那一条的 ⑤）：这一次的输出写一个新文件名（带今天的日期或 `rN`，写之前 `ls` 确认没有同名的），跑完按这个新文件名认本轮才有的完成标记。重跑已有实验时拿新文件与已有的那份比：逐字节一致就删掉这一次的新文件、不新存；对不上就两份都留，实验页两份都点名、写明对不上，交主 agent 定哪份承重。生成产物用的二进制编在 `research/target/` 下，用最后一次源码改动编出来；生成产物与第 5 步跑 `replay.sh` 时都不设 `CARGO_TARGET_DIR`；草稿目录里的 target 只拿来迭代。
.claude/agents/experiment-runner.md:33	4c. 交回之前逐行读判决行：`grep -n 'name=verdict' <产物>` 列出这一次每份产物里的全部判决行（逐字节一致没新存的，读 `research/results/` 里那一份；输出不截断），任何字段取值是 `false` 或 `not_run`，或字段名表示违例、不匹配、歧义、失败的计数（名字里带 `violation`、`mismatch`、`ambiguous`、`fail` 的，例 `layer0_violations`、`width_mismatches`、`today_ambiguous`）取值不是 0，在报告里逐个点名：产物文件与行号、字段名、取值、为什么是这个值、是不是登记预期内的；一个都没有，也写一句「判决行 N 条，没有 false / not_run 字段，违例、不匹配、歧义、失败类计数都是 0」。
.claude/agents/experiment-runner.md:35	6. 跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节），84 号除外：它放到第 7 步写完实验页之后跑；这一次不写实验页的，在这一步最后跑，红了照写。其中要编译或复跑整张表的，开跑前照共用约束看负载。15 号（编整个 research 工作区）与 87 号（复跑全部已入库实验）不归你，收尾由 `gate-triage` 统一跑；你只跑第 5 步自己那一个实验的 `replay.sh`。
.claude/gate.d/stage-owners.tsv:26	40-results-cited.sh	experiment-runner	实验产物写回
.claude/gate.d/stage-owners.tsv:50	86-experiment-orphans.sh	experiment-runner	实验号在 kb 里有正文
.claude/gate.d/stage-owners.tsv:58	69-evidence-in-repo.sh	experiment-runner,kb-scribe,gate-triage	两半各归一个：装置与变异表改了要有产物，是执行员跑完自己那一个实验就该核的（与 40、85、88 同一批）；kb 里把依据写成 /tmp 路径，是书记员写回时该核的；research/prompts/ 那一半（提示、报告与判决）由主 agent 写，提交前核
.claude/gate.d/stage-owners.tsv:62	74-model-differential.sh	implementation-writer,crash-verifier	模型对拍：改 crates/ 的实现员改完先跑（release 下单跑一个测试二进制，几秒）；崩溃验证员跑重阶段时一起跑
.claude/gate.d/69-evidence-in-repo.sh:16	#   而且它的 mtime 不早于那份源码。一个这样的产物都没有、或最新的那份比源码旧 ⇒ 红。
.claude/gate.d/69-evidence-in-repo.sh:259	    print(f"     → 怎么办：跑一次把产物存进 {results_dir}/，并在 research/scripts/replay.sh 把这个实验的登记行指到它。")
.claude/gate.d/86-experiment-orphans.sh:5	# `kb/experiments/` 里就必须有对应编号的正文文件。没有 = 干了活但没入库，
.claude/gate.d/40-results-cited.sh:4	# 判据：`research/results/` 里的实验产物，必须在 `kb/experiments.md` 里被点名。
.claude/agents/gate-triage.md:21	- `gate.sh` 整条经内存包装的上限（第 1b 步用）。
.claude/agents/gate-triage.md:27	1b. 自己单跑的编译出来的代码（`cargo test`、测试二进制）照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条经包装跑。`gate.sh` 整条经内存包装跑，上限取输入给的，退出码 250–254 照那一条办；里面 59 号、87 号逐条再经包装的照常跑（嵌套怎么排队见 `research/scripts/run-with-memory-cap.sh` 文件头「包装里再经包装跑的」那一句）。单跑的门禁阶段照第 2 步直接跑，外面不包一层（87 号经 `research/scripts/replay.sh` 逐条套了）。
.claude/agents/gate-triage.md:28	2. 只在提交时（或主 agent 转达用户要求时）跑：暂存过的跑 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh <上限> bash .claude/scripts/gate.sh --staged`（用户要求时 `=user-request`）；主 agent 明写全量的跑同一条、去掉 `--staged`（前缀照带）。54、55、57、59 这几道重阶段在 `gate.sh` 里照各自的复用判定走（54 号只核全绿标记），你不直接调它们，也不跑全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）；登记给你的其余阶段单跑时不带前缀，87 号单跑照带。超过 Bash 单次上限时后台跑、结束后读输出。
.claude/main-agent.md:48	派崩溃验证员时给 54、55、57 号各自的内存上限，55 号的不小于同时起的虚机数乘每台的内存（算法在 `.claude/agents/crash-verifier.md`「输入」一节）；派门禁分诊员时给 `gate.sh` 整条经内存包装的上限。
research/scripts/run-with-memory-cap.sh:38	#   这一条要的量：峰值表里这条命令（键见下）上一次实测的峰值，不超过它的上限；表里没有的按它的上限算——scope 的 MemoryMax 就是它最多能占的量，按它算不会少算。
research/scripts/run-with-memory-cap.sh:56	#   含页缓存（cargo 写编译产物的那些）与外壳 bash 自己，偏大不偏小。撞了这一条自己的上限记成上限；超时、被停、被总上限挤掉的不记（量到的是半截）。
research/scripts/run-with-memory-cap.sh:72	# 包装里再经包装跑的（整条 gate.sh 经它跑，里面 59 号的每条再经它）：里层另起一个 scope、挪出外层，挂在同一个 slice 里各排各的队；外层照它要的量占着账。
research/scripts/run-with-memory-cap.sh:649	  scope_properties=(-p MemoryMax="$cap" -p MemorySwapMax=0 -p OOMPolicy=stop)
research/scripts/stage-must-run.sh:28	#   传给整轮、跑绿之后前移同一棵。没有这个变量（有人直接跑 gate.sh）一律当成要跑。
research/scripts/admission.py:745	        return EXIT_GATE_MUST_RUN, "这一趟不是 gate-staged.sh 起的（没有 SINGLEFS_STAGED_TREE），被判的可能是工作区，不许复用"
.claude/singlefs-ai-sop/scripts/check.sh:47	cargo test --all || die "单测失败" \
.claude/singlefs-ai-sop/scripts/gate.sh:457	  run_stage "构建与单测" bash "$SCRIPTS/check.sh" "$ROOT"
.claude/gate.d/55-qemu-first-transaction.sh:39	MODES=(direct skip-first-transaction-barrier page-cache second-transaction second-instance raise-rollback-floor)
.claude/gate.d/55-qemu-first-transaction.sh:207	    ( VM_DISKS=2 VM_DISK_MB=4096 VM_BLKLOGWRITES_DIR="$work/$mode" bash research/scripts/vm-bench.sh "$BIN" "$mode" >"$work/$mode/out.txt" 2>&1
.claude/hooks/heavy-test-guard.sh:131	    "gate-triage": {"gate-sh", "gate-staged", "replay-all-stage"},
.claude/hooks/ask-user-claim-guard.sh:57	    rf"(?:{ASCII_PATH_CHARACTER}*[A-Za-z0-9_-]/{FILE_NAME_CHARACTER}+"
.claude/hooks/ask-user-claim-guard.sh:58	    rf"|[\w-]{FILE_NAME_CHARACTER}*\.(?:{EXTENSION_ALTERNATION}))[:：]\d+")
.claude/agents/implementation-writer.md:29	4. 变异证红按测试算、不按行算：每条新测试挑一行能让它红的变异证一次就够，其余行照样追加进 `crates/mutations.tsv` 末尾，整表复跑归最后的门禁 59 号；报告写明哪几行证过、哪几行留给 59 号。层 0 不跑：自己新加的层 0 流不在交回前跑，随提交时的层 0 全量验；已有流一条都不跑；新测试落在名字含 layer0 的测试二进制里的，第 3 步的证红也不跑，变异行照样追加、报告写明留给提交时的 59 号。层 0 全量每次提交跑一次（`main-agent.md`「什么时候派哪个 agent」里「暂存之后、提交之前跑门禁」那一步），不由实现员跑。交回前的验证只到这几样：动到的测试二进制（整个二进制，不按名字挑；名字含 layer0 的不跑）、第 3 步的证红、`cargo fmt --check`、`check.sh` 那一套 lint 下的 `cargo clippy`、`cargo build --offline --all-targets`、阶段归属表登记给你的门禁阶段（共用约束「门禁」一节，74 号在内），各贴末尾原样输出。其余门禁阶段、全量 `cargo test --all`、层 0 各流的快档与全量都不跑，留给提交时统一的那一次验证（`crash-verifier` 与整轮门禁）；派发提示另点名要跑的，不是重型测试的照点名跑，重型的照共用约束「不做」一节那一条交回主 agent。
.claude/gate.d/74-model-differential.sh:47	    echo "                bash research/scripts/run-with-memory-cap.sh 8G cargo test --release -p singlefs-harness --test $TEST_BINARY -- --nocapture"
research/scripts/replay.sh:24	REPLAY_MEMORY_CAP="${REPLAY_MEMORY_CAP:-8G}"
.claude/agents/crash-verifier.md:20	- 54、55、57 号各自的内存上限（第 1b 步用）；55 号的不小于同时起的虚机数乘每台的内存（`.claude/gate.d/55-qemu-first-transaction.sh` 的 `MODES` 档数 × `research/scripts/vm-bench.sh` 的 `VM_MEM`）。
research/scripts/vm-bench.sh:14	VM_MEM="${VM_MEM:-2048}"
research/scripts/vm-bench.sh:133	    -enable-kvm -m "$VM_MEM" -smp "$VM_CPUS" -no-reboot -nographic -serial mon:stdio -display none \
.claude/hooks/runner-dispatch-guard.sh:126	    ("层 0", re.compile(r"层\s*0(?!\d)|(?<![\d.])0\s*层|layer\s*0(?!\d)|(?<!\d)54\s*号|54-layer0", re.I)),
crates/singlefs-harness/src/crash.rs:524	pub fn closed_form_state_count(segments: &[Vec<usize>]) -> u64 {
crates/singlefs-harness/tests/second_transaction_step_three_acquisition_barrier_layer0.rs:195	    assert_eq!(tally.states, 1 + 15 + 1023);
crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool_layer0.rs:157	    assert_eq!(tally.states, 22, "1 + 六个 2 写段各 3 + 三个 1 写段各 1");
crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs:590	    assert_eq!(closed_form, 2_104_413, "闭式：1 + Σ(2^|段| − 1)，五十四段");
crates/singlefs-harness/tests/second_transaction_parallel_line_one_layer0.rs:318	    assert_eq!(closed_form, 5_505_123);
.claude/agents/three-way-local-attack.md:19	- 提示文件路径（形态 `research/prompts/<轮>-local-attack.md`）与样本文件名前缀。
.claude/agents/three-way-local-attack.md:27	3. 前台跑 `bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-s<n>.md`，不许 `setsid`、`&`、`disown`。
.claude/agents/three-way-local-defense.md:19	- 文件名形态（攻方定义里出现的 `-local-attack` 文件名，不论在哪一节，一律照这里换）：提示 `research/prompts/<轮>-local-defense.md`，核对表 `research/prompts/<轮>-local-defense-translation-audit.md`，样本 `research/prompts/<轮>-local-defense-output-s<n>.md`，运行记录 `research/prompts/<轮>-local-defense-runlog.md`。
research/scripts/ask-local.sh:67	    dir="$(dirname "$PROMPT_PATH")"; base="$(basename "$PROMPT_PATH" .md)"; base="${base%-prompt}"
.claude/gate.d/40-results-cited.sh:33	  case "$b" in *local*|*.r[0-9].out|*.round*|confirm*) continue;; esac
.claude/gate.d/86-experiment-orphans.sh:31	  echo "    若那轮的结论不打算入库，就把 research 下那些文件删掉——留着等于留一份没人能复核的证据。"
.claude/rules/implementation-workflow.md:59	只认命令位置；`bash -c`、`capped.sh`、`run-with-memory-cap.sh`、`nice`、`timeout`、`env` 这类包装里面的同样算。命令位置上执行的脚本（`bash x.sh`、`./x.sh`、`source x`）闸读进去，逐行照同一个 `classify` 判，里面有一样就整条算（例：`.claude/scripts/fetch-deps.sh --check` 第 81 行调 `herd7 -version`，整条算 herd7）；门禁阶段与 `lib_heavy_tests.py` 的 `KNOWN_SCRIPT_LOCATIONS` 登记的仓内脚本按名字判、不读正文。`.claude/gate.d/` 下 54、55、57、59、87 之外的阶段不是重型，谁都能跑。
.claude/agents/crash-verifier.md:26	1b. 54、55、57 号整条经内存包装跑（共用约束「不做」一节「跑编译出来的代码经内存包装」那一条；上限取输入给的那一道的，退出码 250–254 照那一条办）：第 2 步里起这三道的每条命令，在 `bash <阶段文件>` 前面加 `bash research/scripts/run-with-memory-cap.sh <上限>`，写成 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh <上限> bash .claude/gate.d/<文件>`；59 号在里面逐条套了，照第 2 步直接跑，外面不再包一层。
.claude/hooks/runner-dispatch-guard.sh:555	            case("重型:通用 agent 跑 layer0", None, "跑 first_transaction_step_seven_layer0 的全量", 2),
.claude/main-agent.md:59	| 跑变异表、看抓到 / 无效 / 没红 | `mutation-triage` → 要补取样点、补断言的：`crates/` 那侧 `implementation-writer`（输入给分诊报告），`research/` 那侧 `experiment-designer` 写重跑登记 → `experiment-runner` |
.claude/gate.d/59-crates-mutation-replay.sh:43	#   上限从三处取，先到先用：变异表里单起一行「# 每条变异的内存上限：<上限>」（判别力样本用它把上限压到 512M）、GATE_MUTATION_MEMORY_MAX、默认 4G。
crates/singlefs-harness/src/crash.rs:1321	    for (invariant, verdict) in check_pool_image(&image) {
.claude/hooks/ask-user-claim-guard.sh:23	#      / 前面紧挨着的那个字要是 ASCII）；research/results/ 下的文件名；
```
