# 产品 GPU 包的测试包 src 四份在途改动·代码轮第一轮·攻方腿

日期：2026-11-17。只读仓；仓里只写了这一份报告。禁读清单里的另一条腿报告没读。

开工核对：`sha256sum -c research/prompts/gpu-tests-src-backfill-code-r1-snapshot/crates-sha256.txt` 四份都是 OK，`kb-sha256.txt` 两份也是 OK（这一次跑出来的原样输出，见草稿目录之外没另存）。下文行号都按这一版取。

下文「装置」指 `crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs`。读数凡是写「量过」的，都是在仓副本上跑探针得来，命令在「要主 agent 跑的命令」一节；这一轮我自己没跑项目代码，所以下文没有一处是「量过」，全是读代码推的，探针交给主 agent 跑。

## 各格判定一览

| 攻击面或格 | 判定 | 一句话 |
|---|---|---|
| 续跑核对：`--cards` 与上一趟不同 | 没打中 | 张数或开法不同就停、退 2，四条单测与 116–119 号变异行都罩着 |
| 续跑核对：交来的卡换了（张数相同） | **打中** | 只比 `cards_named` 与 `named_by`，不比哪几张、哪一台；点名一张卡臂与并发跑根本不核。两组卡（或两台）的重复混进同一份产物，判定照混着的数算 |
| 续跑核对：没交卡或早停的那一趟之后再续跑 | **打中** | 注释说「不被它挡住」，代码却挡：只要产物里有过一行结果、又没有 `every_card_arm_cards` 行，就停下退 2。这份产物从此没法再续，只能换一个 `--product` |
| 搬迁带来：`crates_version` 摘要不再罩装置本身 | **打中** | 摘要只走产品 GPU 包与核心层两个 `src/`。装置原来住在产品 GPU 包的 `src/bin/` 里，算在摘要内；搬进测试包之后，装置自己与它 `#[path]` 引的两份取卡、过库模块都不在摘要里了，停机条款第 2 条与 judge-trend 的同版本核对对装置改动失明 |
| 卡对不上或一张没交时各子命令的出路 | 部分打中 | lockstep 与 concurrent 退 77 或 78，不退回 CPU。stage-three-cost 在这一台一张卡都没交时只跑 CPU 臂、退 0，跑前登记第 236 行要的是「退 77，不当跑过」。可达性要靠服务交出别台的卡，推的 |
| 取卡之前有没有臂用了卡 | 没打中 | 取卡前的挂载都是 `BackgroundComputeSettings::PRODUCT`（CPU） |
| 进程级开卡表在多线程下 | 没打中 | `OnceLock` 线程安全，四处开卡都取同一张表；产品自己的进程开卡器另有一张表，装置不经它开卡（推不出有没有第二张表在用） |
| 先录后判：输入会不会变、判定键罩没罩全 | 没打中；附一处弱打中 | 对拍输入编码了 `AllocationRecord` 的全部六个字段，两侧都解同一份字节。弱打中：「没跑成」的标记靠取卡模块在一个进程里只打一次的那一行，`a_greedy_call…` 那条用例拿不到卡时不声不响地 `return`。可达性推不出 |
| 判定行追加进 `--product`、续跑读回 | 不算另开判定库 | 依据见检查单第四格；但支撑「这是同一次实验的续」的两道闸（卡与台、装置版本）各缺一块，见上面两处打中 |
| 钟速单测前提不成立时返回、cargo 判绿 | 不在射程 | HEAD 旧代码，这次 diff 没改它；它本来就只打印、不判任何东西 |
| 参照一侧登记表没有测试与变异 | 属实 | 没有一条测试逐组核它，`crates/mutations.tsv` 里 0 行。某组入口登记错、指到另一个能编过的 fn 时，没有哪一处会红 |
| 事务员线索第 3、6、7、9、10、11 条 | 3、6、7、10、11 真；9 半真 | 九号线索列的下标用法全是 HEAD 旧代码，漏了这次新加的三处，见第七节 |
| 123 行变异罩不罩得住第 1 到 5 件的判红分支 | 不罩全 | 漏的分支列在第八节 |

### 检查单八格

| 格 | 判定 | 依据（细节在第九节） |
|---|---|---|
| 走不到的理由二次查实 | 成立 | diff 新加的有 3 处 `.expect`（8576、8593、8596）与 6 处 panic 或断言（8557、8583、8682、8695、8696、8702），逐处追了前提，都站得住 |
| 类型与名字 | 成立（带两处弱点） | 新加的 `LOCKSTEP_ARMS[下标]` 只在新测试里（12651、12654、12656）；`FirstHandedOverCards(NonZeroUsize)` 能装下 1，「至少两张」只在解析那一步守 |
| 不兼容旧代码与旧格式 | 成立 | 新代码没有按版本分支；「没有 `every_card_arm_cards` 行」走的是同一个 clause、同一条出路 |
| 不新建判定库 | 成立 | 依据是 `.claude/rules/verification.md` 那一节里列举「算新开判定库」的那一句，以及「产物、日志、临时现场归各自的目录」那一句 |
| 所有任务多机可用 | **不成立** | 续跑可以派到另一台，两台的重复混进一份产物而不停，违反「派到两台里任何一台都要跑得起、判得对」 |
| 测试优先多线程 | 成立 | 计时的跑串着跑，理由在跑前登记第十四节 |
| 带 --item 的命令与项 | 成立 | 123 行的 cargo 参数都换成了新包名；86 个不同的筛选词各自筛得到一个测试 fn；123 行的原文在被判文件里各出现恰好一次 |
| 新崩溃流多罩了什么 | 成立（不适用） | 没有新的崩溃流 |

开工核对的原样输出：

```
crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs: OK
crates/singlefs-background-gpu-tests/src/gpu_side_units_entries.rs: OK
crates/singlefs-background-gpu-tests/src/reference_units_entries.rs: OK
crates/singlefs-background-gpu-tests/src/lib.rs: OK
research/prompts/e202-preregistration.md: OK
.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md: OK
```

## 一、续跑核对

### 1.1 打中：张数相同、交来的卡换了（或换了一台），续跑接着跑，重复混在一起

被判对象只比两样东西：「.map(|(_name, fields)| (fields.get("cards_named"), fields.get("named_by")))」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:5964`）。`pci_bus_addresses` 写进了那一行，但从不拿来比。点名一张卡臂用的是交来的卡里 PCI 地址最小的那一张，这张卡没有任何续跑核对。concurrent 与 sensitivity 里的并发那一点没有全部卡臂，续跑时卡的一样都不核。

续跑读回来的数里没有卡：「struct RunNumbers {」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:5247`）只有臂、重复号、批数和三个纳秒数。`name=run` 行虽然带了 `gpu_models`，`repeats_already_done` 并不过滤它。

历史（每一步都是被判对象许可的）：

1. 某一台交来 01、02 两张卡，跑 `lockstep --unit-area 4GiB --product P`，不给 `--cards`。全部卡臂开这两张，点名一张卡臂开 01。做完重复 0 到 k，写下 `repeat_done`，然后被打断（文件头声明了 `task-server-resumable: lockstep`）。
2. 同一条命令再起（服务续跑，或主 agent 重交）。服务照 `all` 交来的是「那台此刻的空卡」，这一回是 02、03 两张；或者这一回派到了第二台，那台的卡是另一种型号。
3. `cards_named=2 named_by=every_handed_over_card` 与产物里那一行相同，所以 `every_card_arm_cards_differing_from_the_product_being_resumed` 交回 `None`，接着跑。点名一张卡臂换成了 02 号卡（或第二台的卡）。
4. `repeats_already_done` 把第一趟那 k+1 次重复的数读回来，与这一趟的数合在一起，交给 `judgement_of_a_lockstep_tier` 算量 W、C、S、P。

判据对照：文件头那句承诺「两种张数的重复不混在同一份产物里」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:39`）字面上只管张数，代码按字面做到了。可攻击面问的是「交来的卡换了时」，跑前登记第十四节又写着「成对相减要两条臂在同一台、同一组卡上跑。」（`research/prompts/e202-preregistration.md:573`）。一次重复之内，各臂确实在同一台、同一组卡上，这一句字面也成立；混的是重复与重复之间：一档 30 次重复的中位数与成对差，取自两组卡或两台机器。

四句：

- 分不分辨臂：分辨。GPU 两条臂的读数直接随卡的型号变；换台时 CPU 臂的读数也跟着变，量 S 的成对差在两组样本之间来回切。
- 被判的系统当时看不看得到判别它的东西：看得到。产物里有上一趟的 `every_card_arm_cards`（带 `pci_bus_addresses`）、`gpus`（带地址与型号）、`run`（带 `gpu_models`）三种行。
- 满足判据字面的哪一个分句：文件头「两种张数的重复不混」那一句，字面没被违反，打中的是它没罩到的那一半（同张数、不同卡）。跑前登记第 573 行在一次重复之内也成立。所以这一处更准确的说法是：续跑核对的射程比攻击面要的窄，并且撞上了检查单「所有任务多机可用」那一格（第九节）。不算违反了某一句登记判据。
- 跑前条款给的改法：没有现成的改法条款。

改法（只在我读代码时推过，被攻过零轮，没有在模型上量）：`every_card_arm_cards` 行连 `pci_bus_addresses` 一起比；另加一行记点名一张卡臂用的那张卡，加上这一台的地址（取卡模块认这一台用的 `this_host_address_by_the_configuration_check`），续跑时这几样有一样不同就照同一个 clause 停下；concurrent 也走同一道核对。

探针：`attack_probe_a_resume_handed_other_cards_of_the_same_count_stops`（在副本里，见「要主 agent 跑的命令」第 1 条）。它造一份上一趟交了 01、02 两张卡的产物，问这一趟交 02、03 两张时停不停。预期判红：`stopped=None`，同时 `numbers_read_back=1`。推的，待主 agent 跑。

### 1.2 打中：没交卡或早停的那一趟，把产物永久挡住（注释说不挡）

注释原文：「这一台一张都没交的不打不核：哪条 GPU 臂都不跑（退 77），产物里不留一行零张，以后在交了卡的那一台续跑时不被它挡住。」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:5935`）。

核对函数唯一放行「产物里没有 `every_card_arm_cards` 行」的条件是「if lines_already_in_the_product.is_empty() {」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:5956`），也就是产物里一行结果都没有。可是每一趟在取卡之前，就已经往产物里写了一串结果行：`opening_of_the_command` 第一件事就是写「"session_start",」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:6054`），接着写 constants、cpu_clock_check、crates_file、crates_version、tier_device、construction，到没交卡那一支又写 gpus、not_run、done。

历史一（注释点名的那一种）：服务交来的卡里没有这一台的。取卡模块交回 `None`，lockstep 写下 not_run，最后「return EXIT_NO_ARM_RAN;」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:6371`）。之后同一条命令在交了两张卡的那一台续跑：`the_every_card_arm_cards_are_recorded` 判真，产物非空，又没有 `every_card_arm_cards` 行，于是停下，写 `cards_named_in_the_product=none … 换一个--product`，退 2。

历史二（更常见的一种，`all` 正是让它变常见的那个设定）：`lockstep --cards 2 --product P` 起跑那一刻，这一台只有一张空卡。`discrete_gpus_named_for_the_every_card_arm` 判「多于交来的」，退 2。这时产物里已经有一趟的开头行和造池行（造池在取卡之前，顶档约 197 s，推的）。之后有两张空卡了再起同一条命令：同样停下，退 2，从此这份产物不能再用。领卡没成（退 78，`cards_not_taken`）那一支也一样。

与事务员记录的对照：`records/2026-11-11-GPU两段与按需领卡核查.md:417` 记了主 agent 收下的「没打这一行就停下的旧产物续跑会被挡」。那一句说的是旧产物。这里的两段历史都是新产物，是这一版代码自己造出来的。

四句：

- 分不分辨臂：不分辨。整份产物都停，病根在续跑核对共用的「产物非空」那个前提里。
- 系统看不看得到判别它的东西：看得到。只要看产物里有没有 `repeat_done` 或 `run` 行，就能分出「这一趟早停、什么都没量」和「量过、却没留卡数」两种情形。
- 满足的是哪一个分句：同一份代码里两段注释互相打架。5935 行说「不被它挡住」；同一函数的文档写「交回 `None` 是接着跑：产物是新的（一行结果都没有），或每一行都相同。」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:5947`）。代码照后一段做，前一段是假话。
- 改法：没有跑前条款。

改法（零轮）：把「产物是新的」改成「产物里没有这一档的 `repeat_done` 与 `run` 行」（`repeats_already_done(...).0` 为空，`numbers` 也为空）；或者在任何结果行写下之前，先做张数核对，并把 `every_card_arm_cards` 写在取卡之后、退出之前的每一条出路上。两种改法我都没量。

探针：`attack_probe_a_session_handed_no_card_does_not_block_a_later_resume_on_a_host_handed_cards`。它照 lockstep 在没交卡那一支写下的几种行造一份产物，再问交了两张卡的续跑停不停。预期判红：`stopped=Some(...)`。推的，待主 agent 跑。

### 1.3 没打中：`--cards` 与上一趟不同

`--cards 3` 之后再 `--cards 2`、`--cards 2` 之后不给 `--cards`、产物里有结果行而没有那一行：这三种都停下、退 2（单测 `resuming_a_product_whose_every_card_arm_opened_another_count_stops`、`resuming_a_product_with_the_same_count_named_another_way_stops`、`resuming_a_product_without_an_every_card_arm_cards_line_stops`，变异 117–119 行）。另试了产物里有两行 `every_card_arm_cards`、只有后一行不同的情形：`find` 会找到它，照样停下（读代码推的；单测里没有这种两行的产物，「只看头一行」那种变异罩不住，见第八节）。

## 二、搬迁带来的：crates 摘要不再罩装置本身（打中）

摘要走的两棵树是「PathBuf::from("crates/singlefs-background-gpu/src"),」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:4135`）与紧跟着的那一行 `crates/singlefs-core/src`。这两行 HEAD 里一字没改。HEAD 里装置住在 `crates/singlefs-background-gpu/src/bin/` 下（跑前登记「**入库装置**：`crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs`」，`research/prompts/e202-preregistration.md:234`），递归走 `crates/singlefs-background-gpu/src` 时会把它自己算进去。搬进测试包以后，下面这几样都不在摘要里了：

- 装置本身（`crates/singlefs-background-gpu-tests/src/bin/`）；
- 装置经 `#[path]` 引进来的取卡模块 `tests/common_discrete_gpus_of_this_host/mod.rs`（开跑回比的显存上限常量就是从这里取的），以及过库模块 `tests/common_pairing_input_through_the_library/mod.rs`；
- 装置新依赖的验证包 `gpu_card_configuration`（按需领卡）。

历史：第一趟 lockstep 跑完重复 0 到 14 后被打断。随后有人改了装置里的判法、计时窗口或造池（都在装置这一份文件里），产品 GPU 包和核心层没动。同一条命令续跑时，`the_crates_changed_since_the_earlier_commands` 拿到的摘要相同，于是不停，接着把两个版本的装置量出的重复合在一起判。judge-trend 那一处「crates_versions.len() != 1」（8889 行）同样放行三档出自不同装置版本的产物。

判据对照：停机条款第 2 条原文是「`name=crates_version` 的 sha256 在同一段的几条命令之间变了（`crates/` 中途被改）。」（`research/prompts/e202-preregistration.md:436`），而摘要本身的定义是「带产品 GPU 包与核心层 `src/` 下每个文件的 sha256」（`research/prompts/e202-preregistration.md:168`）。按摘要定义的字面，代码没违反；按停机条款括注里的「`crates/` 中途被改」，装置现在就住在 `crates/` 里，它被改了却看不见。搬迁之前，摘要定义的字面恰好把装置罩了进去；搬迁让两句之间出现了缝。

四句：

- 分不分辨臂：不分辨，这是续跑与 judge-trend 共用的前提。
- 系统看不看得到判别它的东西：看得到。装置要从仓根起，自己的源文件就在一个固定的相对路径上。
- 满足的是哪一个分句：括注「`crates/` 中途被改」；第 168 行的字面没被违反。
- 改法：跑前条款里没有。

改法（零轮）：摘要再走 `crates/singlefs-background-gpu-tests/src` 与 `crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host`、`tests/common_pairing_input_through_the_library`；验证包里装置用到的两份要不要也算，交主 agent 定。改了以后，`name=crates_version` 的 `files=` 数会变，这属于产物格式变了，照共用约束「不兼容任何旧东西」办。

探针：`attack_probe_the_crates_digest_covers_the_apparatus_source`。它从仓根调 `source_files_and_digests()`，数路径以 `crates/singlefs-background-gpu-tests/` 起头的有几个。预期判红：`of_the_apparatus_package=0`。推的，待主 agent 跑。

## 三、卡对不上、一张没交时各子命令的出路

逐条走过一遍（读代码）：

| 子命令 | 没有卡的来路 | 交来的卡对不上本机的独显（取卡模块 panic，被 `catch_unwind` 收成 `Err`） | 交了卡、但没有这一台的 | `--cards` 多于交来的 |
|---|---|---|---|---|
| geometry-check、construction-timing | 不要卡，打 `gpus enumerated=false` | 不适用 | 不适用 | 认不得 `--cards`，退 2 |
| lockstep | 开头退 78 `no_cards_from_the_task_server` | 造完池、取卡时退 78 `cards_not_taken` | 每条臂都打 not_run，退 77 | 退 2，产物被挡住（1.2 节） |
| concurrent | 退 78 | 退 78 | 退 77 | 认不得 `--cards`，退 2 |
| sensitivity | lockstep 那一点退非 0，并发那一点就不起 | 同左 | 同左 | 不适用 |
| stage-three-cost | 退 78 | 退 78 | **CPU 臂照跑、GPU 臂打 not_run，退 0** | 不适用 |
| 内联真卡对拍用例 | 打 `NOT_RUN_NO_GPU_ADAPTER test=<用例名>`，交回没跑成 | panic，被收成两边不一致，照这把键记红 | 交回没跑成；标记靠取卡模块打的那一行（见第五节） | 不适用 |

stage-three-cost 那一格：「if let Some(reason) = reason_the_arm_is_not_run_on_the_cards(arm, &cards) {」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:8767`）这一行是 diff 改过的。CPU 臂的理由恒为「ComputeOfTheArm::Cpu => None,」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:2810`），于是 CPU 臂跑完，`does_every_arm_agree` 保持 true，走到「if does_every_arm_agree {」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:8831`）退 0。跑前登记的要求是：要卡的子命令一张独显都看不到时「退 77，不当跑过」（`research/prompts/e202-preregistration.md:236`）。stage-three-cost 是文件头点名的四个要卡子命令之一。

- 会不会「悄悄退回 CPU」：不会把 GPU 臂偷偷换到 CPU 上跑，not_run 那一行照打。但整条命令退 0，服务与主 agent 看到的是成功，而丙段代价要比的 GPU 那一半根本没量。
- 可达性：服务只会把它派去的那一台的卡交出来（`task-server/task_server.py` 文件头 ② 的分卡说明；我没逐行核服务）。所以「交了卡却没有这一台的」要服务出岔子才走得到，推的。HEAD 里看不见独显时也是这样走（逻辑相同），这次 diff 只是把触发条件换了。
- 四句：不分辨臂（这一条命令本来只有两条臂）；看得到（`cards.discrete_gpus.is_empty()` 在手边）；满足的是跑前登记第 236 行「退 77」那一分句；没有改法条款。改法（零轮）：照 lockstep 的写法，空卡时每条臂打 not_run、退 77。

lockstep 与 concurrent 没打中：卡对不上和一张没交都不退回 CPU。没交这一台卡的那一支，连 CPU 臂也一起打 not_run、退 77，`arms_run` 不会只剩 CPU 臂去跑。

## 四、取卡之前有没有哪条臂用了卡（没打中）

试了 lockstep、concurrent、stage-three-cost 三条命令，从入口到 `cards_seen_by_the_command` 之间的每一步：`opening_of_the_command`（常量回比、本线程 CPU 钟自检、读登记线程数、算 crates 摘要，加上只看环境变量不领卡的来路核对）、`emit_whether_the_make_filesystem_accepts_the_tier`、`construct_the_starting_image`；stage-three-cost 另有 `allocation_records_and_unit_area_of_the_image`，它挂载时用的是 `BackgroundComputeSettings::PRODUCT`（CPU，8478 到 8483 行），以及 `live_slots_of_each_segment_counted_by_the_apparatus`。这些步骤都不建 Vulkan 实例、不开卡。内联真卡用例的只录那一段不调持卡判（单测 `the_record_stage_of_the_live_slot_pairing_takes_no_card_and_the_judge_stage_reads_back_the_apparatus_count` 钉了「持卡判一次都没调」）。`a_greedy_call…` 在只录那一段，先于取卡就返回。

## 五、进程级开卡表与进程级取卡在多线程下

开卡表：`OnceLock::get_or_init` 线程安全，四处开卡（4316、4330、8347、8449 行）都取同一张表（`is_the_same_table_as` 比的是 `Arc::ptr_eq`）。整份装置源文件里现建空表的只有一处（`grep -c with_none_not_returned` 结果是 1）。没打中。

推不出的一处：产品的进程开卡器 `GpuOpenerOfThisProcess` 自己另持一张表（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:308`）。装置不经它开卡（装置里 `GpuOpener` 0 命中）；但挂着时切换开卡（`put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`）走的是哪一张表，我没追进产品包，推不出。

取卡：`CARDS_SEEN_BY_THIS_PROCESS` 用 `get_or_init` 包着 `cards_taken_for_this_process`（5818 行），把取卡失败的那一句也一起缓存下来。弱打中（可达性推不出）：同一个测试进程里有两条真卡用例（12134 与 12153 两条，都挂特性）都调 `cards_of_this_process`。取卡模块在交了卡、却没有这一台的时候，只在第一次取卡那一刻打「println!("NOT_RUN_NO_GPU_ADAPTER test={test_name} reason={reason}");」（`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:698`），而且 `test=` 填的是装置名 `e202_gpu_cpu_gc_comparison`，不是用例名。后取卡的那条用例拿到缓存里的空卡：对拍用例交回没跑成（`pairing_concluded_on_the_cards_through_the_pairing_library` 交回 `()`，结局被丢掉）；`a_greedy_call…` 在「if cards.vulkan_instance.is_none() {」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:12172`）这里直接 `return`，一行标记都不打。门禁的按条记库是按时间认记号归谁的（`scripts/gate/lib_libtest_result_lines.py` 文件头「按条记库」）。后跑的那一条开始时，前一条打的那行已经过去了，它就会被记成绿，并且以后按单元键复用。走到这里要两个条件同时成立：服务交了别台的卡，或者卡串是空白。服务按需领卡时会摘掉 `GPU_CARDS_BY_PRIORITY`（`task-server/task_server.py:11205`），所以推的结论是：正常经服务交的任务走不到。改法（零轮）：`a_greedy_call…` 与 8683 行那一支自己打 `NOT_RUN_NO_GPU_ADAPTER test=<用例名>`。

## 六、先录后判：录与判之间输入会不会变、键罩没罩全（没打中）

试过的形状：

- 输入换了还被复用：录入键由录入单元键、对拍输入字节本身与参照一侧单元键拼成（`common_pairing_input_through_the_library/mod.rs` 313 到 318 行）。结论键由读回输入的摘要、参照一侧单元键与 GPU 一侧单元键拼成。编码器 `pairing_input_bytes_of_the_allocation_records_and_the_unit_area` 写在用例的闭包里，它一改，字节就变；解码器同时在参照一侧与 GPU 一侧两个闭包里，它一改，两把单元键都变。没造出输入变了而键不变的历史。
- 字段漏编：编码写了单元区的盘号、起点槽、槽数、记录条数，逐条记录写盘号、槽号、跨槽数、代、释放没有、隔离没有。解码用结构体字面量把 `AllocationRecord` 六个字段全部填上，少一个字段就编不过。单元区起点是照环长重算的，两侧解的是同一份字节，算出的起点一致。
- 判段现造输入：判段照样调 `input_bytes_of_the_test`（304 行）重造小池来算录入键。小池按种子造、单线程，读代码看是确定的，没找到会让两次造出来不一样的输入。没量。
- 判段读回的参照被换掉：单测在只判那一段把参照函数换成 panic，并核读回来的就是只录那一段装置自数的结果（变异第 99、100 行罩着）。

附：取卡失败被收成「两边不一致」存进库。8682 行的「判段要开卡，取卡没成」与 8695 行的「数不成」都在 `judgement_of_the_assertions_on_the_cards` 的 `catch_unwind` 里面。它的处理是「Err(payload) => PairingJudged {」（`crates/singlefs-background-gpu-tests/tests/common_pairing_input_through_the_library/mod.rs:159`），也就是记成两边不一致，照这把键记红。事务员表里标的「推的」现在核实了：确实如此。它不会造成假绿，因为记红的下一趟会现判；但库里那一条「两边不一致」，实际是取卡没成。

## 七、判定行追加进 --product、续跑读回：算不算另开判定存放处

判：不算，检查单「不新建判定库」那一格成立。依据的规则句：

- 「另开一个目录存判定文件、另写一张 TSV 或 sqlite、另开一份 RocksDB 都算。」（`.claude/rules/verification.md:56`）。这一句的主语是「新建自己的判定库」。库在这条规则里的定义是同一节第一条「库里只放剪枝按单元键复用时要读的键」。`--product` 文件不按单元键存，也不供剪枝复用：它是这一次实验的产物，按跑前登记写进 `research/results/` 下。
- 「产物、日志、临时现场归各自的目录」（同一节）。
- 共用约束写法：「不另开目录、TSV、sqlite 或另一份 RocksDB 存判定」（`.claude/agent-common.md@写`）。

续跑读回的是 `repeat_done` 与 `run` 两种行，用来跳过已经做完的重复，不读 `name=verdict` 来跳过判定。跑前登记第十四节把它写成续跑入口，并写明「判定库那一层做不到：计时是新观测，不复用」。

但这个判法撑在一个前提上：读回的确实是「同一次实验被打断的那一半」。这个前提在这一版里有两块是漏的：卡与台不核（1.1 节），装置版本不核（第二节）。前提不成立的时候，读回来的就成了别的实验条件下量的数。那时它就不只是产物了，实际起的是「跨条件复用」的作用，只是不经键。这一点交主 agent 判。我的读法是：只要把那两道闸补上，就仍然是产物。

②（对拍结论经验证包判定库）、③（单测用自己起的一次性库 `DisposableLibrary`，规则许「自证与压测一律对自己起的临时服务」）、④（构建脚本往 `OUT_DIR` 写的键表，是编译产物、不是判定）：都不算。

## 八、事务员线索第 3、6、7、9、10、11 条逐条

| 条 | 真伪 | 核的 |
|---|---|---|
| 3 `gc_wall_clock_of_the_run` 的 zip 截断 | 真；在 HEAD 旧代码里 | 5111 行 `windows` 是滤掉 `None` 之后的。有一窗读不到时，`every_window_read` 为假，整次作废（5159 行），截断不进判定。这次 diff 没改它 |
| 6 CPU 多判臂的活槽段只加条数、不比 | 真；在 HEAD 旧代码里 | 3161 到 3170 行只做 `entries_rejudged +=`，不比任何值；3149 到 3151 行读不到字节时只加 `mismatches`。这次 diff 没改它 |
| 7 judge-trend 不核份数，同键覆盖，解析不了的值被丢掉 | 真；在 HEAD 旧代码里 | 8850 行逐份读，8878 行 `insert` 覆盖，8872 行 `filter_map`。另加一条：8889 行的同版本核对同样受第二节那个缺口影响 |
| 9 `LOCKSTEP_ARMS[0]` 拿下标取臂 | 半真 | 线索列的 5327、7739、7791、10695 在 HEAD 里都有（旧文件 5239、7243、7299、9936 行），不是这一次新加的。这一次新加的是新测试里的三处：12651 行 `[2]`、12654 行 `[5]`、12656 行 `[0]` 与 `[1]`，线索漏了。新加的这三处在测试里：次序一改，断言就会对不上而判红，不会悄悄换臂。线索说「静默换臂」，只对产品段 5327 行那一处成立（HEAD 旧代码） |
| 10 `entries % full_segments_of_a_device` 除数为 0 | 真；在 HEAD 旧代码里 | 3661 行。三档的单元区都至少 2288 段，除数不会是 0（推的，照登记的档） |
| 11 `rejudged_live_slots` 没有 `checked_sub` | 真；在 HEAD 旧代码里 | 2959 行。debug 编译下段在单元区之前会 panic；release 编译下会绕回成一个很大的下标，`get` 交回 `None`，记一条对不上，不会假过 |

## 九、123 行变异罩不罩得住第 1 到 5 件的每个判红分支（不罩全）

核过的：123 行的原文在被判文件里各出现恰好一次（我在草稿目录用一段 `python3 -I -` 逐行数，输出 `checked 123`、没有一行报次数不等于 1）；e202 那 121 行用到的 86 个不同筛选词，在装置里各自匹配得到一个测试 fn（一个 `MISS` 都没打）；e202 那 121 行的 cargo 参数都是 `-p singlefs-background-gpu-tests --bin e202_gpu_cpu_gc_comparison -- …`（不是这个形状的行数为 0）。

逐分支对照（只列 diff 新加的判红或停下分支；「有」指表里有一行变异，指向的测试会在这个分支被弄坏时判红）：

| 件 | 分支 | 变异行 | 缺口 |
|---|---|---|---|
| 1 | 来路核对 `the_task_server_hands_over_cards` | 有（第 96 行） | — |
| 1 | `opening_of_the_command` 里没有来路就退 78 那一支（6074 行） | 没有 | 没有测试走 `opening_of_the_command`，删掉这个 `if` 也没有哪一处会红 |
| 1 | `cards_handed_to_the_apparatus` 两样都没有时报错 | 有（第 95 行） | 领卡没成那一支（`map_err`）没有 |
| 1 | `discrete_gpus_among_the_handed_over` 先筛、再核长度 | 有（第 97、98 行） | — |
| 1 | `cards_taken_for_this_process`：panic 收成 `Err`、`None` 收成空卡 | 没有 | 要真卡或 nvidia-smi，单测走不到 |
| 1 | `cards_seen_by_the_command` 取不成就退 78 | 没有 | 同上 |
| 1 | 先造池、后取卡的次序（lockstep、concurrent、stage-three-cost） | 没有 | 把取卡挪回造池之前，没有任何一处会红 |
| 2 | 设置点名交来的全部卡 | 有（第 11、101 行） | — |
| 2 | `discrete_gpus_named_for_the_every_card_arm` 的四支 | 有（第 102 到 106 行） | — |
| 2 | 按臂取卡 `discrete_gpus_of_the_arm` | 有（第 112、113 行） | `lowest_card` 与 `name=run` 型号串那两处调用点（5146、5542 行）没有：改回用 `setting.discrete_gpus` 不会红 |
| 2 | 解析 `--cards` | 有（第 107 到 110、114、115 行） | concurrent 拒 `--cards` 有测试，但没有变异行 |
| 2 | 续跑核对的函数 | 有（第 116 到 121 行） | 改成「只比头一行 `every_card_arm_cards`」罩不住（单测里每份产物都只有一行） |
| 2 | lockstep 里接上续跑核对、超出张数退 2 的接线（6293 到 6336 行） | 没有 | 删掉 `if let Some(fields) = …` 整段，没有哪一处会红 |
| 3 | 开卡表进程里只有一张 | 有（第 122、123 行），外加读源文件数次数的单测 | — |
| 5 | 只录那一段的参照字节，与隔离标记的编码 | 有（第 99、100 行） | 释放标记、盘号、跨槽数的编码没有变异行 |
| 5 | `whether_of_the_number_written` 读到 0、1 以外就 panic | 没有 | — |
| 5 | 持卡判里「GPU 与装置自数不同就 panic」（8702 行） | 没有 | 这一支只有真对拍出现差异时才走得到；改成恒交 `EveryComparisonHeld`，在产品正确时谁也抓不到。比较函数本身有第 68 行罩着，接线没有 |
| 5 | 持卡判里没有来路、或拿到空卡就交回没跑成 | 没有 | — |

第 4 件（核心层接口改名）靠编译器挡，不需要变异。

## 十、参照一侧登记表（属实，补一条后果）

它的测试与变异：`pairing_conclusion…` 那个目标经 `#[path]` 编它，却挂着「reason = "这里只读 GPU 一侧那张登记表，参照一侧那张与特性名不用"」（`crates/singlefs-background-gpu-tests/tests/pairing_conclusion_on_the_cards_is_reused_by_its_keys_and_judged_again_when_a_side_changes.rs:12`）；`crates/mutations.tsv` 里文件那一列指到它的有 0 行（`awk` 数的）。

后果（推的）：构建脚本只在入口路径认不出时 panic（`build.rs` 的 `keys_of_the_entry_groups`）。如果某一组登记成另一个存在、编得过的 fn（比如把 E202 那一组指到 `pairing_input_bytes_of_the_allocation_records_and_the_unit_area`），参照的代码改了，参照一侧的键却不变。于是录入键不变，库里那份旧参照照旧被读回、旧结论照旧被复用，没有哪一处会红。七组的组名与用例里交的参照函数我逐个对过：组名只在各自那一份用例里出现一次，参照函数都由那份用例自己交，今天的登记没有错。缺的只是一道把「登记的入口」与「用例真交的参照函数」绑在一起的检查。

## 十一、检查单八格的细节

1. 走不到的理由：8576、8593、8596 三处：写下的就是 `u32`、`u16` 的值，读回的字节就是这一把录入键下写进去的那一份，成立。8557、8583：编码器的输出进录入键，解码器在两个闭包里、进两把单元键，布局一改键就跟着变，「录的与读的布局对不上」走不到，成立。8682、8695：取卡或计数没成，被收成判红（第六节），成立。8696：循环正好 `STAGE_THREE_COST_REPEATS` 遍（8147 行），成立。8702：故意判红的出口，成立。
2. 类型与名字：`FirstHandedOverCards(NonZeroUsize)`（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:5865`）能装下 1，「至少两张」只在 `cards_of_the_every_card_arm_of_the_options` 里守（`machine-first` 的「类型让非法状态无法表示」是写法建议，不是判据）。「const NAME_OF_THE_APPARATUS_IN_THE_CARD_LINES: &str = "e202_gpu_cpu_gc_comparison";」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:5728`）被当作 `test=` 的值交给取卡模块，于是标记行里的用例名是装置名（第五节）。PCI 地址是裸 `String`，HEAD 里就这样。新加的名字里没找到序数或位置代称（新测试里的 `three[..2]`、`three[1..]` 是切片，不是名字）。判：成立。
3. 不兼容旧代码与旧格式：diff 里没有认旧行形态的分支。「没有 `every_card_arm_cards` 行」与「行不同」走同一个 clause、同一条出路（重起或换一个 `--product`），只是理由那一句不同，而且它也罩新产物（1.2 节），不是专为旧版本开的。判：成立。
4. 不新建判定库：见第七节。判：成立。
5. 所有任务多机可用：两台读同一份产物，续跑派到另一台时，两台的重复混进一份产物而不停（1.1 节）。依据是「派到两台里任何一台都要跑得起、判得对」（`.claude/rules/verification.md:65`）。`HOST_LOAD_CORES_AT_MOST = 16.0` 与节拍数 100 是登记里写死的常量，两台都一样，不会让判出的结局不同。判：不成立。
6. 测试优先多线程：计时的跑串着跑，理由「两条臂同时跑会互相抢 CPU、内存带宽与卡」写在跑前登记第十四节「串行」那一条；这次 diff 没改并行度。判：成立。
7. 带 --item 的命令与项：见第九节开头。判：成立。
8. 新崩溃流多罩了什么：装置里 `crash` 只命中 `crash_library` 的存取（`PairingJudged`、`RunStage`、`CrashLibrary`），不枚举崩溃状态。判：成立（不适用）。

## 十二、事实表 facts.md 写错或误导的地方

1. 第〇·一节「开卡表进程级一份」那一行（`research/prompts/gpu-tests-src-backfill-code-r1-snapshot/facts.md:42`）把 4332、8349、8451 列成「每处开卡多传这一张」。这三行其实是 `|_gpus_to_open| {},`，是产品开卡函数新加的那个回调参数。传开卡表的是 4316、4330、8347、8449 四行。
2. 第一节第 5 样第 9 条（`research/prompts/gpu-tests-src-backfill-code-r1-snapshot/facts.md:383`）列的四处下标用法都在 HEAD 里（旧文件 5239、7243、7299、9936 行），不是这次新加的；这次新加的三处（12651、12654、12656 行）它没列。主 agent 要判「是不是这一次新加的」时，这一条会把人引错。
3. 第一节第 3 样「所有任务多机可用」只把 4135、4136 两行记成「算 crates 摘要」读的相对路径，没注意到装置搬走之后，这两行罩不到装置自己了（第二节）。这是漏，不是写错。
4. 第一节第 3 样「走不到的理由」里 8682 那一行标着「推的，那份在测试共用模块，没逐行核」：现在核实为真（第六节）。可以去掉「推的」。

另外第二节第 3 样、第三节第 3 样「类型与名字」把入口前缀写成 `singlefs_background_gpu_tests::` 这件事列为疑点；构建脚本对认不出的入口会 panic（`build.rs` 的 `keys_of_the_entry_groups`），所以这种写法今天编得过，就说明单元图认它（推的，没起编译）。

## 要主 agent 跑的命令

1. 副本上的三条探针（副本在 `.deliverables` 里登记为「交接」；它们只在副本里，不进仓）：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu-tests 的测试目标 bin e202_gpu_cpu_gc_comparison 里筛选词 attack_probe 筛出的三条攻方探针; 范围=1 个测试目标里 3 条测试; 为什么=核攻方腿报的续跑核对两处与 crates 摘要一处打中，探针只在仓副本里' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/gpu-tests-src-backfill-code-r1-attack/copy -- cargo test -p singlefs-background-gpu-tests --bin e202_gpu_cpu_gc_comparison -- attack_probe --nocapture`
   - 在哪个目录起：`/home/fy5090/.local/share/singlefs/work/gpu-tests-src-backfill-code-r1-attack/copy`（仓副本，已加探针；加探针之后的装置 sha256 是 `6c933e5be3b74a166b71d261707db136863f79e435ec0c81e3ff83fb1d0f762e`，加之前的那一份备在草稿目录的 `e202-copy-before-probes.rs`，sha256 是 `395af7961855d51dab5f6662dd081c3184158fa2dfab43963aa97c9c1853c7a7`，与仓里那一份相同）
   - 跑完看什么：三条探针断言的都是被判对象自己写下的性质，**判红就是打中**。预期 `TASK_SERVER_RESULT … exit=101`，`task-server/task log <号> --all` 里 `test result: FAILED. 0 passed; 3 failed`，并且有这三行：`ATTACK_PROBE zero_card_then_resume stopped=Some(…cards_named_in_the_product=none…)`、`ATTACK_PROBE other_cards_same_count repeats_done=1 numbers_read_back=1 next_repeat=1` 加上 `stopped=None`、`ATTACK_PROBE crates_digest files=<N> of_the_apparatus_package=0`。哪一条探针过了，那一处打中就没坐实；编不过的，那一次不算数，交回改探针。
   - 这三处打中，判决要引的话，照规则要在入库装置上另做一次；副本上的数不算入库装置的数。

## 没打中的形状与取样范围

- 续跑核对：试了张数不同、开法不同、产物里有多行 `every_card_arm_cards` 只有后一行不同、产物里有结果行而没有那一行、新产物，五种都照文档停或照文档跑。
- 各子命令的出路：试了七个子命令与一条内联用例，分别在没有来路、卡对不上、交了卡却没有这一台的、超出张数四种情形下的走法（第三节表）。lockstep 与 concurrent 不退回 CPU。
- 取卡之前用卡：试了三条要卡命令，取卡之前的每一步（第四节）。
- 开卡表：试了四处开卡是不是同一张表、有没有现建的第二张，以及测试进程里几条线程同时取（第五节）。
- 先录后判：试了输入换了还被复用、字段漏编、判段现造输入不确定、读回的参照被换（第六节）。
- 判定库：试了 `--product` 读回之后是按键复用还是只是续跑；读回的行里有没有 `name=verdict`（没有）。

## 这条腿自己的限度

- 一处都没跑：三处打中都是读代码推的，探针交给主 agent 跑；第三节 stage-three-cost 退 0、第五节标记行只打一次这两处，可达性都依赖服务行为，我没逐行核服务代码。
- 产品 GPU 包里「挂着时切换开卡」走哪一张开卡表，我没追（第五节，推不出）。
- 单元图怎么解 `bin:` 目标与 `singlefs_background_gpu_tests::` 前缀，我没读 `singlefs-unit-graph` 的代码，只凭「认不出就 panic」推「今天认得出」。
- 跑前登记里「全部卡」那条臂的写法（第 49、297 行写「不给 `background_gpu_devices`」）与今天的代码不一样。用户定案与「冻结证据不改」都记在 `records/2026-11-11-GPU两段与按需领卡核查.md:334` 与 `records/2026-11-11-GPU两段与按需领卡核查.md:463`，我当已知的处置，没当打中来报。
- HEAD 旧代码不在射程里：线索 3、6、7、10、11 与钟速单测我都判了真伪，但没把它们算作这一轮的打中。

## 没做什么

- 没判正推、辩方那几格；没替主 agent 采纳；副本上量出的数不算入库装置的数；没往统一任务服务交任务，判决要引的数没复跑（命令在「要主 agent 跑的命令」一节）。
- 没写模型目录（这一题不是崩溃枚举，没造崩溃状态）。只在仓副本的装置末尾加了一个测试模块，三条探针。
- 没改仓里除这份报告之外的任何文件。

## 草稿目录里留下的

- `/home/fy5090/.local/share/singlefs/work/gpu-tests-src-backfill-code-r1-attack/copy`：仓副本，交接给主 agent 跑探针，主 agent 用完删。
- `/home/fy5090/.local/share/singlefs/work/gpu-tests-src-backfill-code-r1-attack/e202-copy-before-probes.rs`：加探针之前那一份的备份。
- `/home/fy5090/.local/share/singlefs/work/gpu-tests-src-backfill-code-r1-attack/e202-head-old.rs`：`git show HEAD:` 取出来的旧路径版本，对照行号用。
- 副本的大小（`du -sh`）是 55M，里面没有编译目录。没有删掉的建出物：只有这一份副本，它是交接给主 agent 的。

交回前跑的 `python3 scripts/agents/cite-check.py research/prompts/gpu-tests-src-backfill-code-r1-opus-output.md`，末行是「✓ 核了 23 处引文、9 处裸行号，对上 23 处，没判 9 处（其中裸行号没有引文可比 9 处；逐处列在上面）」。
