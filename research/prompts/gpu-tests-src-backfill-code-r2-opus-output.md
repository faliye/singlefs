# gpu-tests-src-backfill-code-r2：攻方腿（Opus，2026-11-18）

被判对象：开工快照 `research/prompts/gpu-tests-src-backfill-code-r2-snapshot/` 点名的七份文件。开工时在主工作区跑 `sha256sum -c` 两张表，九份全部 OK；仓副本建好后，在副本里再核七份，同样 OK。下文行号都取主工作区这一版。

探针与复跑：没有建模型目录。探针是仓副本里的两条单测，外加参照登记表换掉一个入口，命令写在末尾「要主 agent 跑的命令」。文件与 sha256 如下（草稿目录 `/home/fy5090/.local/share/singlefs/work/gpu-tests-src-backfill-code-r2-attack/`，下面简称草稿目录）：

| 文件 | sha256 |
|---|---|
| 草稿目录 `attack-probes.rs`（两条探针的源码，原样接进副本装置的 `mod tests` 末尾） | `52b25a7d998e84510b9d2dcb46bee42338121a496d759ab4256c5aff5ae33b76` |
| 草稿目录 `probes-in-the-apparatus.diff`（副本装置对主工作区的 diff） | `4df28e371ea9b88d12acef711de46dc4ead61e02e4678686528a0a4de0c27fd6` |
| 草稿目录 `reference-substitution.diff`（副本参照登记表对主工作区的 diff，只改 1 行） | `7be7ab2eb1b1f6a45a1fcadab0c5cedb5329245847c5b32da9d11769572bc5c8` |
| 副本 `crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs` | `ccb2029902b5bd8a170e2a65fccc6748b0ee7565806557be59da40f1348cb0c9` |
| 副本 `crates/singlefs-background-gpu-tests/src/reference_units_entries.rs` | `e4583d71193127e4c31567a830c2d157e5fe64a824ed560d6a38cb50d4f1b839` |

禁读：这一轮辩方腿的报告没读，别的题的腿报告也没读。第一轮我自己的攻方报告 `research/prompts/gpu-tests-src-backfill-code-r1-opus-output.md` 读了第一节与第九节。原因是攻击面 ③ 要逐处核那一份列的缺口，「关的是哪几条」第 3 条又要我核自己先前提的改法。派发的禁读清单写了「别的轮的腿报告不读」，这一处与上面两条要求冲突，我照攻击面的要求读了，在这里写明。另外，第一轮判决说「攻方报告第八节逐条列」，那张缺口表在我那份报告里其实是第九节，第八节是事务员线索。

## 各格判定一览

| 格 | 判定 | 一句话（依据在后面各节） |
|---|---|---|
| 正文第 1 条：续跑核卡的身份 | **打中** | 字段比得全：张数、开法、每张地址、型号、次序都比；并发跑也算进点名一张卡臂；夹了别的臂的行既不误停也不漏停。打中的是：只交来一张卡、又没给 `--cards` 时，全部卡臂报本次未跑，可照样记下一行 `cards_named=1`，之后交来两张时续跑被这一行挡住，而全部卡臂其实一条结果都没有（探针一，推的，待主 agent 跑）。弱打中：只点 CPU 臂的命令换台续跑一样都不核 |
| 正文第 2 条：一张都没交的那一趟不挡续跑 | **打中（弱）** | 判法是看「产物里有没有这条臂跑出的 `run` / `concurrent_run` 行」。三种出路（没交卡、`--cards` 多于交来的、领卡没成）都在记卡之前退出，也走的是同一条放行路，这一块没打中。打中的是：记了卡、还没打出一条 `run` 就断了的那一趟，照样挡住换卡之后的续跑。文件头那句话比代码宽（探针二，推的，待主 agent 跑） |
| 正文第 3 条：crates 摘要 | **打中** | 五处起点罩住了装置自己和 `#[path]` 引进来的两份模块。但产品 GPU 包与核心层的三个路径依赖（`singlefs-format`、`singlefs-replay-prefix`、`singlefs-mount-runtime`）在计时窗里跑，却不在摘要里，新注释「这一趟跑的代码就是这几处」不实。验证包里的按需领卡与判定库不必进摘要（理由见第二节） |
| 正文第 4 条：登记修订第 12 条 | **打中** | 修订写「并进第十一节停机条款」，可第十一节说「停机时产物照打、退 3」，而修订与代码都退 2。修订还写「产物合并时另看产物所在的机器」，但产物里没有一种行记机器（照字面名字打的 31 种行与记卡两种行都不记） |
| 正文第 5 条：参照登记表的测试、变异与按名取臂 | **打中** | 把一组的入口换成本目标里另一个存在、别组没用的 fn，新测试不红（探针三，推的，待主 agent 跑）。第一轮点名的三处位置取臂都改掉了；剩下的六行七处在搬迁前那一版里就有，都是旧代码 |
| 正文第 6 条：守卫罩不罩全开真库的用例 | 没打中 | 装置二进制里开全仓那份库的只有一处，已经包上守卫。守卫是读写锁的读一侧，几条读者可以并行，两个方向的先后都等得到 |
| 正文第 7 条：旧代码线索登记欠账 | **打中** | `.claude/kb/checks-owed.md` 里提到 E202 的行有 0 行。按函数名搜 `records/`，只在一份早于判决的审核记录里找到一条（T02，多判臂那一条） |
| 攻击面 ③：第一轮第九节列的缺口 | **打中** | 第九节列了 12 处，补了 6 处，还剩 6 处没有变异罩着。这一次新加的记卡接线（交替跑、并发跑两处，加上停下那一支）也没有变异罩着 |
| 检查单：走不到的理由 | 成立 | 补丁非测试段的加行里，`todo!`、`unreachable!`、`assert`、`panic!`、`expect`、`unwrap` 都是 0 处 |
| 检查单：类型与名字 | 成立（带注） | 卡的身份用裸 `String` 装在键值对里；写行与比对共用同一份 `Vec`，键名对不上的情形出不来。地址与型号在元组里的次序靠精确串的单测罩着 |
| 检查单：不兼容旧代码与旧格式 | 成立 | 唯一的放行支是「这条臂没有结果行」，不是为认旧产物开的；旧产物的行缺 `models` 字段，比对时取到 `None`，照停 |
| 检查单：不新建判定库 | 成立 | 补丁里命中 `.tsv` 的 3 处都是读构建产物与变异表；没有新建目录，没有 sqlite 或 RocksDB |
| 检查单：所有任务多机可用 | **不成立** | 同槽同型号的情形，加上只点 CPU 臂的命令，换台续跑都认不出；认得出的情形又会停下，而停下时给的出路「照产物里的那几张卡重起」用户做不到（推的） |
| 检查单：测试优先多线程 | 成立 | 守卫只让设一次性库的用例与真卡对拍互斥，读者之间不互斥 |
| 检查单：带 --item 的命令与项 | **推不出** | 带真卡特性的 2 行变异没判，它们抓不抓得到推不出；欠账表里也没有这两行（第六节） |
| 检查单：新崩溃流多罩了什么 | 成立（没有这一类） | 补丁里 `crash-case:` 与 `enumerate_crash_states` 都是 0 处 |
| 撤回 | **撤回三处** | 第一轮 1.2 节的两个改法我都撤回；1.1 节「加上这一台的地址」也撤回（第八节）。另补一条：1.1 节的改法带来一个没想到的代价，停下时给的出路做不到 |

## 一、续跑核卡（正文第 1、2 条）

### 1.1 比的字段全不全：全

全部卡臂那一行比四个字段：`cards_named`、`named_by`、`pci_bus_addresses`、`models`。点名一张卡臂那一行比两个：`pci_bus_address`、`model`。写行与比对用的是同一份 `fields_compared_on_resume`，取值都经 `field_value`，写出去再读回来逐字相等。地址按 PCI 次序拼接（`discrete_gpus` 在枚举时就排好了，见「discrete_gpus.sort_by(|left, right| left.pci_bus_address.cmp(&right.pci_bus_address));」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/arms_and_background_compute_settings.rs:128`）），所以次序也在比。`tier` 不比，这是有意不分档。

试了以下几种形状，都没打中：

- 交来的先后换了、卡还是那几张：一样。
- `--cards 2` 挑出的那两张换了：停。
- 产物里有几行记卡、只有后一行不同：停（有变异行「只核产物里头一行记卡的」罩着）。
- 产物里只有 CPU 臂的 `run`、这一趟头一次点全部卡臂：接着跑。
- 产物里有并发跑 `arm=gpu_one_card` 的 `concurrent_run`、却没有记卡行：停。
- 灵敏度那两点先交替跑、再并发跑，写进同一份产物：并发那一点核的是同一进程里交替跑刚写下的那一行，对得上。
- 产物里夹着 `arm=without_compaction` 或 `stage_three_cost` 一类别的行：不算进任何一条用卡的臂。

### 1.2 打中：全部卡臂报本次未跑的那一趟照样记卡，之后的续跑被它挡住

历史。每一步后面写的是许可它的那一句。

1. 起 `lockstep --unit-area 4GiB --product P`，不给 `--cards`、不给 `--arms`，六条臂全点。服务交的是那台此刻的空卡，这一刻只有 01 一张空着（文件头写「服务把那台此刻的空卡都交来」；`--cards` 没给，也就没有「多于交来的」那一支）。
2. 全部卡臂报本次未跑：「ComputeOfTheArm::GpuOnEveryCard if gpu_count == 1 => {」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/arms_and_background_compute_settings.rs:145`）。CPU 两条臂与点名一张卡两条臂照常跑，做完重复 0 之后被打断。
3. 可是记卡发生在判「跑不跑」之前。`cards_recorded_for_the_every_card_arm` 只看「点了这条臂」和「这一台交了卡」两样（「if !computes_requested.contains(&ComputeOfTheArm::GpuOnEveryCard) {」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/command_opening_and_construction_lines.rs:49`）与 `:6063`），于是 `lockstep_command` 在 `crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/concurrent_runs.rs:82-105` 打出 `name=every_card_arm_cards cards_named=1 named_by=every_handed_over_card pci_bus_addresses=0000:01:00.0 …`。到 `:6564` 起那个循环才判它不跑。
4. 同一条命令再起，这回 01、02 两张都空着。全部卡臂算出 `cards_named=2`，与产物里那一行不同，于是停下，退 2，理由写「产物里这条臂开的卡与这一趟不同」。

判据对照：

- 文件头说那一行记的是「全部卡臂开了几张、照什么开的、每张的 PCI 地址与型号打在 `name=every_card_arm_cards` 一行，」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:38`）。第 1 步那一趟全部卡臂一张都没开，行里却写开了 1 张。
- 被挡住的那条臂在产物里一条 `run` 都没有，没有可混的重复。代码自己的放行支（`:6101` 的文档、`:6115` 的判断）就是按「这条臂有没有跑出结果」逐臂判的，只是它只在「产物里没有这种行」时才问这一句。
- 反方向同样会停：上一趟交来两张，全部卡臂跑过；这一趟只交来一张，全部卡臂本来就不跑，却因为 `cards_named` 1 与 2 不同，连 CPU 臂与点名一张卡臂也一起停下（读码推的，没造探针）。

四句：

- 分不分辨臂：分辨。只挡全部卡臂两条。
- 系统当时看不看得到判别它的东西：看得到。`reason_the_arm_is_not_run` 在同一个函数里、记卡（`:6539`）之后的循环（`:6564`）里就算出来了，产物里也有那一趟写下的 `name=not_run … arm=gpu_every_card`。
- 满足判据字面的哪一个分句：文件头第 38 行「开了几张」那一分句写成了假话；「一条结果行都没跑出来的那一趟不挡以后的续跑」（`:41`）按字面说的是一整趟，这一趟别的臂有结果，所以那一句的字面没被违反，被违反的是代码自己逐臂放行的意图。
- 改法：没有跑前条款。

改法（只在读码时推过，被攻过零轮）：记卡只记这一趟真会跑的臂。先算 `reason_the_arm_is_not_run`，再算 `cards_recorded`；全部卡臂报本次未跑时不记、不核。另配一行变异：「全部卡臂只交来一张、报本次未跑时照样记卡」。

探针一（副本里）：`tests::attack_probe_an_every_card_arm_not_run_on_one_card_does_not_block_a_resume_on_two_cards`。它照上面的历史造产物，断言续跑不停。预期判红，同时打出 `ATTACK_PROBE every_card_arm_not_run_on_one_card not_run_reason=Some(…) recorded_while_not_run=Some("name=every_card_arm_cards tier=4GiB cards_named=1 …") stopped=Some("line=every_card_arm_cards …")`。推的，待主 agent 跑。

### 1.3 弱打中：只点 CPU 臂的命令，换台续跑一样都不核

`--arms` 只给 `cpu,cpu_rejudged`，是命令行许可的（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/unit_area_tiers_and_local_constants/tests.rs:26-34` 逐个按名字认臂，没有「至少一条 GPU 臂」的要求）。这时两个 `cards_recorded_for_*` 都交回 `None`，记卡与核卡一样都不做。到第二台续跑时，`repeats_already_done`（`:6426`）把头一台的 CPU 重复读回来，与这一台的合在一起判。并发跑 `--arms cpu,without_compaction` 也一样。

另一条历史：头一趟六条臂在第一台上跑，被打断；用户改成 `--arms cpu,cpu_rejudged` 在第二台上续跑。这时也一样都不核，头一台那几次重复里的 CPU 读数照样被读回。

判据对照：登记修订第 12 条写「什么观测会让它触发：同一份 `--product` 续跑时换了卡、换了张数或换了一台。」（`research/prompts/e202-preregistration.md:483`）。上面两条历史都是「换了一台」，而它不触发。第一轮我自己的四句里已经写过：「换台时 CPU 臂的读数也跟着变」。

四句：分辨臂（只漏 CPU 臂）；看不到，产物里没有一种行记机器；满足的是修订第 12 条「换了一台」那一分句；改法见第八节（记读数依赖的硬件，不记机器名）。可达性：要用户只点 CPU 臂，或者续跑时改 `--arms`，这两步都是用户动作。我把 `--arms` 放开扫了一遍：续跑时点到任何一条 GPU 臂，就会因为那条臂的卡不同而停下；只有全不点 GPU 臂这一种取值漏掉。

### 1.4 正文第 2 条：「一条结果行都没跑出来」怎么判

判法：产物里没有这一种记卡行时，再看产物里有没有这条臂跑出的 `name=run` 或 `name=concurrent_run`（照 `arm=` 认臂，多判臂算它那一种），见「if !the_product_has_runs_mounted_with(」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/command_opening_and_construction_lines.rs:106`）。三种出路写下的行，以及会不会被当成「有结果行而没有记卡行」：

| 出路 | 在哪一步退 | 写下的行 | 记卡没有 | 之后续跑 |
|---|---|---|---|---|
| 这一台一张都没交 | 记卡之后，`:6585` 打 `not_run` 退 77 | 开头几行、`gpus count=0`、`not_run`、`done` | 不记（`:6063` 交回 `None`） | 放行（单测 `a_session_that_ran_no_arm_does_not_block_a_later_resume_on_a_host_handed_cards`、变异 `crates/mutations.tsv:8171`） |
| `--cards` 多于交来的 | 记卡之前，`:6516-6533` 退 2 | 开头几行、`gpus`、`stopped clause=cards_option_exceeds_the_cards_handed_over` | 不记 | 放行 |
| 领卡没成 | 记卡之前，`cards_seen_by_the_command` 退 78 | 开头几行、`gpus enumerated=false`、`stopped clause=cards_not_taken` | 不记 | 放行（没有单测，读码推的） |

三种走的是同一条放行路：都没有记卡行，也都没有 `run` 行。`--cards` 多于交来的与领卡没成都在记卡之前退出，不走两条路。这一块没打中。

打中（弱）：记了卡、还没打出一条 `run` 就断了的那一趟。造法：`resume` 那一行之后、头一次重复里第一条臂跑完之前，进程被打断（被取消、撞了内存上限、机器重起）。这时产物里有记卡行，没有 `run`。之后换了卡续跑：记卡行在、字段不同，于是停。文件头原文是「一条结果行都没跑出来的那一趟（这一台一张都没交、`--cards` 多于交来的、领卡没成）不挡以后的续跑。」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs:49`），登记修订第 12 条是同一句话，后面没有括号（`research/prompts/e202-preregistration.md:483`）。主语「一条结果行都没跑出来的那一趟」罩住了这一种，代码却不放它。这一形没有可混的重复，挡住它的代价只是一份产物（出路是换一个 `--product`）。

四句：不分辨臂（两条用卡的臂一样）；看得到（产物里有记卡行，没有这条臂的 `run`）；满足的是第 41 行与登记那一句的主语，代码文档（`:6101`）只承诺「产物里没有这种行时」，这一层是自洽的；没有跑前条款。

改法（零轮）：记卡行加 `session=`，续跑只拿「这一趟有这条臂的 `run`」的那几趟写下的记卡行去比；或者把文件头与登记那一句收窄到括号里那三种。前一种多付一个字段，后一种不改代码。

探针二（副本里）：`tests::attack_probe_a_session_stopped_before_its_first_run_line_does_not_block_a_resume_on_other_cards`。它断言续跑不停。预期判红，同时打出 `ATTACK_PROBE session_without_a_run_line stopped=Some("line=every_card_arm_cards …")`。推的，待主 agent 跑。

### 1.5 停机那一行的字段

`name=stopped clause=cards_differ_from_the_product_being_resumed line=<种类> <键>_in_the_product=… <键>_now=… reason=…`（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:6206`，其余字段在 `:6104` 那个函数里拼）。逐字段带上了产物里的与这一趟的值，产物里没有的写 `none`，这一层有变异「产物那一边逐个字段都写成 none」罩着（`crates/mutations.tsv:8190`）。这一行比别的停机行少一个 `tier=`；不分档本来就是设计，不算错。

出路那一句有问题，见第七节第 2 点：理由写「照产物里的那几张卡重起或换一个--product」（`:6134`），前半句用户做不到。

## 二、crates 摘要（正文第 3 条）：打中

装置读的源文件罩全了。装置自己与它经 `#[path]` 编进来的两份模块（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:156-160`）都在五处起点里。那两份模块所在的目录里各只有一个 `mod.rs`（`ls` 看过），也没有再往外 `#[path]`。装置源码里的 `include_str!` 只有一处，在测试里，读的是它自己。参照登记表没有编进装置；编进来的是构建脚本写的 `OUT_DIR` 键表，只给真卡那一条内联测试用，不进交替跑与并发跑的计时。

没罩全的是「这一趟跑的代码」。新注释写「装置自己的源文件，与它经 `#[path]` 编进来的两份共用模块。这一趟跑的代码就是这几处，停机条款第 2 条（`crates/` 中途被改）要看得见它们每一处的改动。」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:4134`）。可是产品 GPU 包与核心层各有路径依赖，编进了同一个二进制：

- 「singlefs-mount-runtime = { path = "../singlefs-mount-runtime" }」（`crates/singlefs-background-gpu/Cargo.toml:15`）。产品 GPU 包开卡给挂载运行时用的就是它，见「use singlefs_mount_runtime::{」（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:33`）。
- 「singlefs-format = { path = "../singlefs-format" }」（`crates/singlefs-core/Cargo.toml:15`）。核心层 `src/` 里不在注释中的 `singlefs_format::` 有 150 处（`grep -rn … | grep -v '//' | wc -l`）。
- 「singlefs-replay-prefix = { path = "../singlefs-replay-prefix" }」（`crates/singlefs-core/Cargo.toml:17`）。前台写的路径上就在调它，见「singlefs_replay_prefix::file_range_proofs::units_for_sequential_write(」（`crates/singlefs-core/src/write_request_split.rs:61`）。

这三个包都在 `crates/` 下，最近还在改（`git log --oneline -5 -- crates/singlefs-replay-prefix crates/singlefs-format crates/singlefs-mount-runtime`：最近一笔是 74baab8e1）。

历史：第一段交替跑跑完 2288 MiB 档之后，别的会话改了 `crates/singlefs-replay-prefix/src/` 里前台写拆单元的函数，或者改了挂载运行时；之后起 4 GiB 档。`name=crates_version` 一字不变，停机条款第 2 条不触发，judge-trend 照样把两版代码量出的三档放在一起判趋势。

判据对照：停机条款第 2 条原文是「`name=crates_version` 的 sha256 在同一段的几条命令之间变了（`crates/` 中途被改）。」（`research/prompts/e202-preregistration.md:436`）。登记第 168 行把摘要定义成产品 GPU 包与核心层的 `src/`，按那一句的字面，这三个包本来就不在里面，所以这是旧缺口，不是这一次修补带进来的。但这一次修补新写了一句「这一趟跑的代码就是这几处」，这句话不实；而第一轮判决第六条打中的，恰好就是这一类缺口（括注「`crates/` 中途被改」看不见）。

四句：不分辨臂（续跑与 judge-trend 共用这一前提）；看得到（三个包的路径固定）；满足的是第 436 行的括注，以及新注释那一句陈述；跑前条款没有改法。

改法（零轮）：起点加 `crates/singlefs-format/src`、`crates/singlefs-replay-prefix/src`、`crates/singlefs-mount-runtime/src` 三处，再加一条单测：从三个包的 `Cargo.toml` 读出 `path = "../…"` 依赖的闭包，核它们的 `src` 都在起点里。以后再加一个路径依赖，这条单测就红。配一行变异：去掉其中一个起点。代价：`files=` 的数会变，这是产物格式变了，照「不兼容任何旧东西」处置。

验证包里装置用到的两个模块，我判不必进摘要：

- 按需领卡（`gpu_card_configuration`）只决定交来哪几张卡，交来的卡现在已经逐张记进产物、续跑时核。
- 判定库（`crash_library`）只给内联真卡测试用，交替跑与并发跑的计时窗里不走它。

推翻这个判断的观测：在交替跑、并发跑的调用链里找到验证包的一处调用，而且它在计时窗里（我只 `grep` 了装置源码里的 `singlefs_verification::`，共 10 处：文件头注释 1 处，顶层 `use` 2 处（`PairingJudged` 与按需领卡），其余 7 处都在 `mod tests` 里；没有追进验证包看那两样被 `use` 进来的东西往下调了什么）。

## 三、跑前登记修订第 12 条（正文第 4 条）：打中两处，另有一句要补记

把第 12 条（`research/prompts/e202-preregistration.md:478-485`）逐句拿去对装置今天的行为：

| 登记的句子（转述，原文在第 479–485 行） | 装置 | 对得上 |
|---|---|---|
| 卡只用服务交的；环境变量与领卡通道两样都没有，就报错退出 | 开头核来路，`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:6297` 退 78 | 对得上 |
| `--cards N` 只开交来的卡串里排在前头的 N 张；N 小于 2、不是数、多于交来的都退 2；一张都没交退 77 | `:5934-5962`、`:6516-6533`、`:6585-6598` | 对得上 |
| 没给 `--cards` 而只交来一张时，这条臂报本次未跑 | `:2819` | 对得上；但那一趟照样记卡，见 1.2 节 |
| 点名一张卡臂开的是哪一张，记进产物 | `:6044`（地址最小的那一张） | 对得上 |
| 产物多两种行，带的字段 | `:6037-6081` | 对得上 |
| 「续跑核卡（并进第十一节停机条款）」，退 2 | 退 `EXIT_USAGE`，见「Err(EXIT_USAGE)」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/run_resumed_by_hardware_group.rs:130`） | **对不上**，见下 |
| 「一条结果行都没跑出来的那一趟不挡以后的续跑」 | 只在「没有记卡行」时放行 | 主语比代码宽，见 1.4 节 |
| 「什么观测会让它触发：…换了一台」 | 只点 CPU 臂、或同槽同型号时不触发 | 部分对不上，见 1.3 节 |
| 「产物合并时另看产物所在的机器」 | 产物里没有一种行记机器 | **对不上**，见下 |

**打中一：停机条款的退出码。** 第 12 条写的是「续跑核卡（并进第十一节停机条款）」（`research/prompts/e202-preregistration.md:483`），同一句的末尾又写「退 2」。第十一节的规矩是「停机时产物照打、退 3，交主 agent。」（`research/prompts/e202-preregistration.md:442`）。文件头的退出码表也是「3 停机条款触发」。代码退 2，与第一轮判决第三节第 1 条的「对不上停、退 2」一致。所以对不上的是登记里「并进停机条款」这几个字。按第十一节的标题，停机讲的是「装置与 `crates/` 实现对不上」，续跑换了卡不属于这一类。

四句：不分辨臂；看得到；满足的是第 442 行「退 3」那一分句；改法：再起一条修订，把「并进第十一节停机条款」改成「命令行不对（退 2），不是停机条款」。前一条修订的原文不动。

**打中二：「产物合并时另看产物所在的机器」做不到。** 原文：「卡的身份取 PCI 地址加型号，认不出「两台同一个槽插同一种型号」那一形（…），产物合并时另看产物所在的机器。」（`research/prompts/e202-preregistration.md:485`）。我数了装置照字面名字打的行，共 31 种（`grep -A1 '\.emit($'` 取第一个参数），加上记卡的两种，没有一种记机器。judgement 那几行的名字是变量，没逐个看，它们报的是各个量。产物放在共享根下，两台读到的是同一个路径，所以「产物所在的机器」从产物本身看不出来。任务服务的日志也许记了每一趟在哪一台跑，但第 12 条没写去哪里看、拿什么对。

四句：分辨臂（只影响同槽同型号那一形）；看不到；满足的是第 485 行最后一分句；改法见第八节（产物里记读数依赖的硬件），或者删掉这半句，写明这一形认不出、交主 agent 合并时人工核。

**「服务交卡」这一句怎么记。** 正文说，任务服务此刻不给任务分卡，卡归 GPU 批处理验证服务。按今天的代码，要卡的子命令在开头核来路时就打 `stopped clause=no_cards_from_the_task_server`、退 78（`:6297-6305`），产物里只有开头几行。这符合「准入或运行条件不满足」，不是作废，也不是本次未跑（本次未跑是退 77）。我的建议是起一条修订，只记事实：「到 GPU 批处理验证服务交卡的接口定型之前，要卡的四个子命令一律在开头退 78；第 12 条里的『服务』指那时交卡的那一方」。不改第 12 条的原文，也不写成作废，因为一条产物都还没有。推翻条件：GPU 批处理验证服务交卡时用的不是 `GPU_CARDS_BY_PRIORITY`，也不是领卡通道。那样的话，`cards_handed_to_the_apparatus` 要跟着改，到时候第 12 条第一点就得另起修订。

## 四、参照登记表的测试与变异、按名字取臂（正文第 5 条）

### 4.1 打中：新测试红在「重名、重入口、没编进来」上，登记错成另一个存在的入口时不红

新测试文件头自己写了它补的是哪几种：「这里补的是构建脚本不停的那几种」（`crates/singlefs-background-gpu-tests/tests/reference_units_entries_name_functions_that_exist.rs:5`），也就是组拒算或没编进来、入口没进闭包、两组共用组名或入口。它判的是：组名两两不同，入口在参照这一张表里两两不同，每组算出了键，入口在闭包里恰好一个 fn，键两两不同。两行变异（重组名、重入口）都落在这几格里。

它没判的是：入口是不是这一组对拍真正调的那个参照函数。

历史：管 E202 的会话改参照登记表时，把 `apparatus_count_of_the_live_slots_per_segment_in_the_gpu_cpu_gc_comparison` 那一组的入口写成了同一个 bin 里的 `singlefs_background_gpu_tests::pairing_input_bytes_of_the_allocation_records_and_the_unit_area`。这个 fn 在 bin 本体里，构建脚本找得到，参照这张表里别的组没用它。新测试的每一格都过：入口唯一、在闭包里恰好一个、键与别组不同。可是对拍调的参照函数是另一个，`reference_bytes_of_the_apparatus_count_of_the_live_slots_per_segment`（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/judge_trend_subcommand.rs:70`，调用点在 `:12395`）。之后改了这个函数里自数的写法，参照一侧的键不跟着变，库里旧的参照与旧的对拍结论接着被复用。登记成 GPU 一侧那张表的入口（`judged_on_the_cards_that_…`）也一样不红，新测试只在参照这一张表里查重入口。

判据对照：第一轮判决第二节第 7 条打中的原话是「登记错一组入口不会有任何地方红」，第三节第 5 条的改法是「补一条逐组核入口的测试」。补上的测试罩住了「错成重的、错成不在的」，没罩住「错成另一个存在的」。

四句：不分辨臂；系统看得到：对拍入口的签名是 `reference_of_the_input: impl FnOnce(&[u8]) -> Vec<u8>`（`crates/singlefs-background-gpu-tests/tests/common_pairing_input_through_the_library/mod.rs:301`），拿得到传进来的那个函数的类型名；满足的是判决第 7 条那句话的字面；没有跑前条款。

改法（零轮，推的）：在 `pairing_concluded_on_the_cards_through_the_pairing_library` 开头，用 `std::any::type_name_of_val(&reference_of_the_input)` 取传进来的参照函数的路径，拿它去比组名 `reference_units_group` 在登记表里登记的入口。比法与新测试 `is_the_function_of_the_entry` 同一种：结尾是 `::<模块路径>::<fn>`。对不上就 panic。这样把「调的」与「登记的」绑在一起。闭包形式的参照（调用点传的是闭包，不是 fn 项）类型名里带 `{{closure}}`，这条比法认不出，要另想（我没逐个看各测试目标的调用点传的是什么）。

探针三（副本里，换 1 行，`reference-substitution.diff`）：把那一组的入口换成 `pairing_input_bytes_of_the_allocation_records_and_the_unit_area`，然后点名跑新测试。预期构建过、测试过（`test result: ok. 1 passed`），也就是这一行变异没被抓到。推的，待主 agent 跑。要是构建脚本停了，或者测试红了，这一条就撤回。

### 4.2 按名字取臂：没打中

第一轮判决点名的三行四处，在这一次补丁里都换成了 `computes_of_the_lockstep_arms_named(&["…"])`（`research/prompts/gpu-tests-src-backfill-code-r2-snapshot/changes.patch:699-700` 与 `:713-740`）。今天装置里 `LOCKSTEP_ARMS[` 还有六行七处：`:5378`、`:7986`（两处）、`:8038`、`:10665`、`:10942`、`:10943`。拿搬迁前那一版比（`git show 74baab8e1^:crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs`），对应在 5239、7243、7299、9660、9936、9937 行，六行一一对得上，都是旧代码。其中 `:5378`（交替跑里认 CPU 臂）与 `:7986`（灵敏度那一点点哪两条臂）在产品路径上：`LOCKSTEP_ARMS` 一旦改了次序，会悄悄换臂。这两处该登进旧代码线索的欠账（第六节），这一批不改。

## 五、并行下一次性库与真卡用例（正文第 6 条）：没打中

装置二进制里开库的地方我列全了（在装置源码里 `grep` `pairing_concluded_on_the_cards`、`CrashLibrary::open`、`DisposableLibrary`、`make_it_the_library_of_this_process`、`TheOneLibraryKeptForThisProcess`）：

- 开全仓那一份的只有真卡对拍那一条用例（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/pools_cards_and_sessions_for_tests.rs:205`），已经包在守卫里；它经 `#[path]` 模块开库的地方是 `crates/singlefs-background-gpu-tests/tests/common_pairing_input_through_the_library/mod.rs:310`，在守卫之内。
- 设一次性库的有两处：先录后判那一条（`:13414`）与守卫自证那一条（`:13874`）。
- `a_greedy_call…` 与别的真卡用例不开库。

守卫的实现：「pub fn keep_disposable_libraries_out() -> Self {」（`crates/singlefs-verification/src/crash_library.rs:911`）拿的是 `DISPOSABLE_LIBRARY_SETTING` 读写锁的读一侧，设一次性库拿的是写的一侧（`:852-866`）。两个方向的先后都会等：一次性库先设着，守卫等它放手；守卫先拿着，设库的那一边等守卫放手。几条读者之间不互斥，所以没有把本可以并行的用例串起来。Drop 时先把设置清成 `None`，再放写锁（字段在 `drop` 之后才析构），没有空窗。

试过的形状：两条真卡用例同时跑（读者对读者，可以并行，各自开全仓那一份）；先录后判那一条与真卡对拍那一条，两种先后都试了；守卫自证那一条在不带真卡特性时也跑，它起的是临时服务，不写真服务。

## 六、旧代码线索的欠账（正文第 7 条）与没有变异罩着的判红分支（攻击面 ③）

### 6.1 第 7 条：打中，没登记

第一轮判决第三节第 7 条的原文是「第 11 条那几处旧代码里的线索登记欠账，不在这一批改」。我找过的范围：

- `grep -c 'tests-on-a-real-gpu\|E202\|GPU与CPU两种模式GC对照组' .claude/kb/checks-owed.md` 输出 0，主工作区（含没提交的改动）里欠账表提到 E202 的有 0 行。
- 按函数名（`gc_wall_clock_of_the_run`、`rejudged_live_slots`、`full_segments_of_a_device`、`LOCKSTEP_ARMS[0]`）搜 `records/`、`.claude/kb/`，只在 `records/2026-11-13-singlefs-background-gpu与测试包严格审核.md` 里找到一条 T02（多判臂那一条，第一轮线索第 6 条）。那份记录早于第一轮判决。
- 第一轮线索第 3 条（zip 截断）、第 7 条（judge-trend 不核份数、同键覆盖、丢值）、第 10 条（除数为 0）、第 11 条（没有 `checked_sub`），以及第 9 条里产品路径上那两处位置取臂，都没找到登记。

批次记录里写过，带真卡特性的 2 行变异「登进欠账」，可欠账表里同样没有这两行（按两行的名字全仓搜 `.md`、`.tsv`、`.txt`，只在 `records/2026-11-16-三方豁免取消与存量补轮.md`、`crates/mutations.tsv` 与这一轮的正文、快照里命中）。

这一条只说「在这几处没找到」。另外核了拆分提案的现行账本：`.claude/gate.d/split-proposal-ledgers.tsv` 里 current 那一行点名的是 `records/2026-11-17-subagent账本自上而下清欠.md`，按 `E202`、装置文件名与两个函数名 `grep -c`，结果是 0。别处我没搜。

### 6.2 攻击面 ③：第一轮第九节列的缺口，补了一半

| 第一轮第九节的缺口 | 今天 | 依据 |
|---|---|---|
| 领卡没成那一支（`map_err`） | 补了 | `crates/mutations.tsv:8179` |
| concurrent 拒 `--cards` 没有变异行 | 补了 | `crates/mutations.tsv:8184` |
| 续跑核对改成「只比头一行」罩不住 | 补了 | `crates/mutations.tsv:8170` |
| 释放标记、盘号、跨槽数的编码 | 补了 | `crates/mutations.tsv:8181-8183` |
| `whether_of_the_number_written` 读到 0、1 以外 | 补了 | `crates/mutations.tsv:8180` |
| 持卡判「GPU 与装置自数不同就 panic」 | 补了行，没判（真卡特性，本次未跑） | `crates/mutations.tsv:8186` |
| `opening_of_the_command` 没有来路就退 78 那一支 | **没补** | `crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:6297-6306`；`crates/mutations.tsv:7933` 罩的是谓词本身，不是这里的接线 |
| `cards_taken_for_this_process` 与 `cards_seen_by_the_command` 退 78 | **没补** | `:5824`、`:5875-5897`；要真卡或 nvidia-smi |
| 先造池、后取卡的次序 | **没补** | `:6500-6515`、`:7813-7828` |
| `lowest_card` 与 `name=run` 型号串那两处调用点 | **没补** | `:5197-5199`、`:5593`；`crates/mutations.tsv:7952-7953` 罩的是 `discrete_gpus_of_the_arm` 本身 |
| 续跑核对的接线（第一轮说的是旧接线） | **没补，新接线同样没罩** | 见下 |
| 持卡判「没有来路、或拿到空卡就交回没跑成」 | **没补** | `:8922-8931` |

这一次新带进来、同样没有变异罩着的分支：

- 交替跑接上记卡与核卡（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/concurrent_runs.rs:82-105`）：整段 `if let Err(exit_code) = …` 删掉，或者数组里少了 `cards_recorded_for_the_one_card_arm(…)` 那一项，都没有一处会红。
- 并发跑那一段（`:7829-7849`）：`cards_recorded` 换成 `Vec::new()`，没有一处会红。
- 停下那一支（`emit_the_cards_recorded_unless_one_differs_from_the_product_being_resumed` 里 `Err` 那一臂，`:6203-6213`）：`Err(EXIT_USAGE)` 改成 `Ok(())`（打了停机行却接着跑），没有一处会红。
- 摘要读盘那一支 `what_the_path_is_on_the_disk`（`:4162-4177`）：单测走的是内存里的小树，读盘这一支没有单测。弄坏了会在开跑时报错退 78，看得见，不会悄悄过，所以只算轻的缺口。

核法：`awk 'NR>=9486' <装置> | grep -c 'emit_the_cards_recorded_unless_one_differs_from_the_product_being_resumed'` 输出 0；测试模块里提到 `lockstep_command(`、`concurrent_command(`、`opening_of_the_command(`、`cards_seen_by_the_command(`、`what_the_path_is_on_the_disk`、`source_files_and_digests()` 的有 0 处（只命中 `sensitivity_lockstep_command` 与 `command_fields_of_a_concurrent_command`，它们只造命令，不跑）。`crates/mutations.tsv` 里锚在接线上的行有 0 行。

改法（零轮）：把交替跑与并发跑「取卡之后到开跑之前」那一段抽成一个纯函数，入参是交来的卡、点的臂与产物里已有的行，返回要打的行与退出码；单测走这个函数，变异锚在它的调用点上。管 E202 的会话说过它要一并改服务交卡那几句，这一块可以随那一次做。

## 七、检查单八格

1. **走不到的理由：成立。** 补丁非测试段的加行（`changes.patch` 第 1–537 行里以 `+` 起头的）在 `grep 'todo!\|unreachable!\|assert\|panic!\|\.expect(\|\.unwrap()\|unimplemented!'` 下 0 处命中（`rc=1`）。实现员报的「无」核实了。
2. **所有任务多机可用：不成立。** 分三处说：
   - 认不出的两形：同槽同型号（实现员自己报的，登记第 485 行也写了），以及只点 CPU 臂（1.3 节）。卡的身份是本机独有的状态，读它是为了让续跑停下，不是为了改判定结局，本身不违反「不读会改变判定结局的本机独有状态」。缺的是另一半：CPU 臂的读数依赖哪台机器，这一点没有记。
   - 认得出的情形停下之后的出路。理由写「照产物里的那几张卡重起或换一个--product」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/command_opening_and_construction_lines.rs:125`）。用户重交之后，派到哪一台由任务服务定；共用约束不许往服务里加只派一台的名单，任务也不拿卡、不挑卡；`all` 交来的又是那一刻的空卡。所以前半句「照产物里的那几张卡重起」用户做不到，能做的只有换一个 `--product`。再看 `task-server/task_server_lib.py:238` 那一节：续跑声明只用来标注，被打断的命令要用户重交。推的：重交之后，只有当服务恰好派回原来那一台、空卡也恰好还是原来那几张，续跑才接得上；其余情形一律退 2。两台、卡常被别的任务占着的时候，跑前登记第十四节「续跑入口」能不能用，取决于运气。这一层不是修补的错，是第一轮我提的改法带来的代价，那时没想到（第八节）。
   - 新测试本身：参照登记表那条只读 `OUT_DIR` 与 `#[path]` 的源文件；摘要那条用的是内存里的小树；守卫那条起的是临时服务。三条都不读仓外路径，派到哪一台都一样。
3. **类型与名字：成立（带注）。** 卡的身份装在 `fields_compared_on_resume: Vec<(&'static str, String)>` 里，是裸字符串；地址与型号在旧代码 `DiscreteGpuSeenByThisProcess` 里本来也是 `String`。键名对不上的情形出不来：写行（`:6005`「fn fields_of_the_line(&self, tier: UnitAreaTier) -> String {」）与比对（`:6104` 那个函数）读的是同一份 `Vec`。地址与型号在 `pci_bus_addresses_and_models_of` 交回的元组里，次序换了编译器不拦，但精确串单测 `the_every_card_arm_cards_line_tells_how_many_cards_and_whether_the_cards_option_named_them` 会红。新加的 `expect` 都在测试里，消息写的是依赖的前提（「点了全部卡臂、这一台交了卡」）；只有守卫自证里 `unwrap_or_else(|refusal| panic!("{refusal}"))` 一处，消息只转了拒绝理由，没写依赖的前提（「起得了临时服务」）。另有一处命名可以商榷：`ArmWhoseCardsAreRecorded::TheCardWithTheLowestPciAddress` 这个枚举成员名说的是卡，不是臂；它跟 `ComputeOfTheArm` 的成员名一致，算约定一致，不算错。
4. **不兼容旧代码与旧格式：成立。** 放行只有「这条臂没有结果行」这一支（`:6115`）。r1 那一版的产物，记卡行里没有 `models`，`fields.get("models")` 取到 `None`，照样停。没有为旧行放宽的读法。
5. **不新建判定库：成立。** 补丁里 `crash-case:`、`enumerate_crash_states`、`create_dir`、`rusqlite`、`rocksdb`、`.tsv` 合计命中 3 处，都是读构建产物 `OUT_DIR/reference_units_keys.tsv` 与提到 `crates/mutations.tsv`。记卡的两种行追加进 `--product`，那份文件是产物。
6. **测试优先多线程：成立。** 见第五节。
7. **带 --item 的命令与项：推不出。** 26 行由任务 19641 判了，这是正文转述的读数，我没核那个任务。带真卡特性的 2 行（`crates/mutations.tsv:8186-8187`）没判，它们抓不抓得到推不出；它们的欠账也没有登记（6.1 节）。新测试目标由合入后验证的 `--item` 点到，这一点照正文写的接受，没核。
8. **新崩溃流多罩了什么：成立（没有这一类）。** 补丁里 `crash-case:` 与 `enumerate_crash_states` 都是 0 处。

## 八、撤回我在第一轮提的改法，与这一轮的改法表

### 8.1 撤回

1. **撤回第一轮 1.2 节的头一个改法**：「把「产物是新的」改成「产物里没有这一档的 `repeat_done` 与 `run` 行」」。这个判法不分臂，产物里只要有别的臂（比如 CPU 臂）的 `run`，就会把「这条臂有结果而没有记卡行」误判成真，于是误停。实现员改成了逐臂判（`the_product_has_runs_mounted_with`），单测 `only_the_runs_mounted_like_the_arm_make_its_missing_cards_line_stop_the_resume` 钉的正是我那个改法会判错的情形。实现的比我提的对。
2. **撤回第一轮 1.2 节的另一个改法**：「把 `every_card_arm_cards` 写在取卡之后、退出之前的每一条出路上」。照它做，什么都没跑的那一趟也会留下记卡行，之后换卡续跑就被挡住。1.2 节这一轮打中的（全部卡臂报未跑也照记），正是同一种错的一个小形状。实现员没照这一句做，是对的。
3. **撤回第一轮 1.1 节的「加上这一台的地址（取卡模块认这一台用的 `this_host_address_by_the_configuration_check`）」。** 理由有两条：
   - 产物要入库，进 `research/results/`。共用约束写「私有主机信息不进仓：本机的主机名与服务名，第二台的主机名与别名、IP、第二台上的路径……不写进 kb、records、research……」（`.claude/agent-common.md@写`，那一节第 44 行）。地址写进产物行，就进了仓。
   - 读数依赖的不是地址，是硬件。换掉的是什么：记读数依赖的东西，见下面改法表第 4 行。
4. **补一条第一轮没看到的代价**（不撤回改法本身，只补代价）：逐张比卡之后，续跑只有在派回同一台、交来同一组空卡时才接得上，而且停下时给的出路有一半做不到（第七节第 2 点）。第一轮我只看到「不该混」，没看到「混不了之后怎么接」。

### 8.2 改法表（各修哪一格；全是我提的，被攻过零轮）

| 改法 | 修哪一格 | 标注 |
|---|---|---|
| 先判「这条臂跑不跑」，再记卡；报本次未跑的臂不记不核 | 1.2 节（全部卡臂只交来一张） | 推的，待主 agent 跑（探针一在改前应当红；改后的样子没写、没跑） |
| 记卡行加 `session=`，只拿有这条臂 `run` 的那几趟写下的记卡行去比；或者把文件头与登记那一句收窄到括号里那三种 | 1.4 节（记了卡、没出 `run` 就断的那一趟） | 推的，待主 agent 跑（探针二在改前应当红） |
| 摘要起点加三个路径依赖的 `src`，配一条读 `Cargo.toml` 核闭包的单测与一行变异 | 第二节 | 推的，没实现 |
| 产物开头加一行 `name=measured_on`，记 CPU 型号（`/proc/cpuinfo` 的 `model name`）、逻辑 CPU 数、每张卡 wgpu 报的驱动（`AdapterInfo` 的 `driver`、`driver_info`）；续跑时与产物里的比，不论点了哪几条臂 | 1.3 节（只点 CPU 臂）、同槽同型号、登记第 485 行「另看产物所在的机器」 | 推的，没实现；装置今天只取了 `AdapterInfo` 的 `name` 与总线地址（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:2800-2803`），驱动字段在 wgpu 那一侧有没有、叫什么，我没现查 wgpu 30.0.1 的源码 |
| 停下的理由只给能做到的出路（换一个 `--product`），不写「照产物里的那几张卡重起」 | 第七节第 2 点 | 推的 |
| `pairing_concluded_on_the_cards_through_the_pairing_library` 开头拿 `type_name_of_val` 比登记的入口 | 4.1 节 | 推的；闭包形式的调用点认不出 |
| 「取卡之后到开跑之前」抽成纯函数，单测与变异锚在它上面 | 6.2 节（新旧接线） | 推的 |
| 第十二节再起一条修订：续跑核卡退 2 不是停机条款；「服务」在交卡接口定型前怎么读 | 第三节两处 | 推的；修订由主 agent 写 |

改法表里我在副本上量过的格：0 格。

## 没打中的形状

- 续跑核卡的字段与次序：交来先后换了、卡相同；`--cards 2` 挑出的那两张换了；几行记卡里只有后一行不同；同一地址换了型号；`named_by` 不同、张数相同。前一种接着跑，其余都停，都对。
- 夹着别的臂的行：产物里只有 CPU 臂的 `run`；只有 `concurrent_run arm=cpu` 或 `arm=without_compaction`；有 `stage_three_cost` 一类的行；灵敏度两点写进同一份产物。都不误停、不漏停（漏的只在 1.3 节那一种）。
- 三种「什么都没跑」的出路，写下的行与放行路（1.4 节表）。
- 停机行逐字段带两边的值、`none` 的写法。
- 守卫：两个方向的先后、读者之间能不能并行、Drop 次序。
- 摘要：`#[path]` 模块所在目录有没有子模块、装置有没有别的 `include_str!`、`Path` 的排序（`singlefs-background-gpu` 排在 `singlefs-background-gpu-tests` 前面，单测钉了）。
- 补丁非测试段有没有 `todo!`、`unreachable!`、`assert`、`expect`。
- 新开的判定存放处。

取样范围：只读码、`grep`、`git show`。装置读了 `:2700-2830`、`:4130-4300`、`:5860-6230`、`:6258-6800`、`:7788-8117`、`:8740-8800`、`:8905-8935`、`:9280-9390`，测试模块里几处 helper 与新单测；验证包 `crash_library.rs:800-949`；参照登记表与新测试全文；补丁全文；三份变异行文件全文。

## 这条腿自己的限度

- 三条探针都没跑。上文标「推的，待主 agent 跑」的，都要等「要主 agent 跑的命令」的结局。
- 第一轮判决与正文转述的任务读数（19622、19641、19655 等），我都没去核。
- 验证包里按需领卡往下调了什么，我没追进去（第二节的判断撑在这一点上）；wgpu 30.0.1 的 `AdapterInfo` 有哪些字段，我也没现查。
- 「这一形要靠运气才接得上」（第七节第 2 点）是推的：我没读任务服务派台的算法，只读了续跑声明那一节。
- 第六节第 1 点搜欠账的范围只有：`.claude/kb/checks-owed.md`、`records/`、`.claude/kb/` 下按函数名搜，加上拆分提案的现行账本。

## 要主 agent 跑的命令

1. `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu-tests 的测试目标 e202_gpu_cpu_gc_comparison 里的 2 条测试; 范围=2 个测试; 为什么=攻方副本里两条探针断装置文件头写下的续跑放行性质，红就是打中' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/gpu-tests-src-backfill-code-r2-attack/copy -- cargo test -p singlefs-background-gpu-tests --bin e202_gpu_cpu_gc_comparison -- tests::attack_probe_an_every_card_arm_not_run_on_one_card_does_not_block_a_resume_on_two_cards tests::attack_probe_a_session_stopped_before_its_first_run_line_does_not_block_a_resume_on_other_cards --exact`
   - 在哪起：副本 `/home/fy5090/.local/share/singlefs/work/gpu-tests-src-backfill-code-r2-attack/copy`（`.deliverables` 写成交接）。
   - 看哪几行：期望 `TASK_SERVER_RESULT` 的 `exit=101`，`task-server/task log <号> --all` 里有 `test result: FAILED. 0 passed; 2 failed`，并有两行 `ATTACK_PROBE every_card_arm_not_run_on_one_card … stopped=Some(` 与 `ATTACK_PROBE session_without_a_run_line stopped=Some(`。哪一条过了（`ok`），对应的 1.2 节或 1.4 节就撤回；编译不过不算结局，把日志交回、由攻方腿改。
2. `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu-tests 的测试目标 reference_units_entries_name_functions_that_exist 里的 1 条测试; 范围=1 个测试; 为什么=攻方副本里参照登记表一组换成同一目标里别组没用的入口，看新测试抓不抓得到' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/gpu-tests-src-backfill-code-r2-attack/copy -- cargo test -p singlefs-background-gpu-tests --test reference_units_entries_name_functions_that_exist -- every_reference_group_is_keyed_and_closes_over_each_of_its_own_entries_once --exact`
   - 在哪起：同上，同一个副本。
   - 看哪几行：期望 `exit=0`、`test result: ok. 1 passed`，也就是没抓到，4.1 节成立。要是 `exit=101` 并带 `✗`/`panicked`（测试红了），或者构建脚本停了（`登记表里组 … 的入口写错了`），4.1 节撤回。

两条用的是同一个副本，可以先后交，也可以同时交；两条互不读对方的产物。

## 没做什么

- 正推、辩方那几格不判；也不替主 agent 采纳哪条改法。
- 副本上的数不算入库装置上的数。
- 没往统一任务服务交任务。判决要引的三条探针，结局都等主 agent 跑上面两条命令。
- 没建模型目录，也没写 `rerun.sh`：探针是副本里的单测与一处换入口，不是独立模型。
- 第一轮判决第二节第 9 条点名的「取卡链与三条子命令的接线没有单测」，这一轮只核了变异罩没罩着，没逐条核单测有没有补上。

## 草稿目录里留下的

- 没删 `/home/fy5090/.local/share/singlefs/work/gpu-tests-src-backfill-code-r2-attack/copy`（57M）：这是仓副本，主 agent 要在里面跑上面两条命令，已在同一层的 `.deliverables` 写成交接，用完由主 agent 删。
- 其余都是普通文件：`attack-probes.rs`、两份 diff、`minimal-copy.txt`、`cite-check.txt`。没有编译目录。
