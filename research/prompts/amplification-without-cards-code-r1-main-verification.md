# amplification-without-cards-code-r1：判决（2026-11-18）

正文 `research/prompts/_amplification-without-cards-code-r1-body.md`，开工快照 `research/prompts/amplification-without-cards-code-r1-snapshot/`。这是这一题的第一轮，没有前几轮的判决。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（three-way-forward） | `research/prompts/amplification-without-cards-code-r1-sonnet-output.md` | 「问题 1（任务不拿卡）：不成立」「问题 2（0x1F 的键、值、守卫）：形状够，「服务真这么写」推不出」「服务在跑而不扫 0x1F（今天的现状）时这一段不返回，没有汇总行也没有退出码（代码读出来的推，没跑过）」；检查单八格：格二、格三、格五不成立 |
| 攻方（three-way-attack） | `research/prompts/amplification-without-cards-code-r1-opus-output.md` | 「打中 5 条，八格检查单里不成立 3 格、推不出 1 格。读数都没跑，全是推的。」；没打中的：核对待判行四种时序、流键三段、复用数三种库况、互等、锁次序的 panic、惰性取卡退出 |

核查用的两条命令（原样）：

- `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/amplification-without-cards-code-r1-verdict -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py amplification-without-cards-code-r1 --out /home/fy5090/.local/share/singlefs/work/amplification-without-cards-code-r1-verdict/amplification-without-cards-code-r1-tree --repository /home/fy5090/code/singlefs`（任务 19344）：「快照 amplification-without-cards-code-r1：清单点名 18 份，倒推出 18 份，sha256 对上 18 份」。
- `python3 scripts/agents/cite-check.py research/prompts/amplification-without-cards-code-r1-sonnet-output.md research/prompts/amplification-without-cards-code-r1-opus-output.md --root /home/fy5090/.local/share/singlefs/work/amplification-without-cards-code-r1-verdict/amplification-without-cards-code-r1-tree --background research/prompts/_amplification-without-cards-code-r1-background.md`：「✓ 核了 250 处引文、47 处裸行号，对上 222 处，没判 75 处（其中裸行号没有引文可比 31 处；逐处列在上面）」。没判的是快照清单之外的路径（`prune-server/src/`、`scripts/crash/` 下别的脚本）与裸行号；判决引到的几处主 agent 在主工作区逐处现读（下面各条写了行号）。

主 agent 自己量的：GPU 服务控制命令查一次花的 CPU（攻方腿报告「要主 agent 跑的命令」第 1 条，任务 19345）五次都是「user_seconds=0.10」到「0.11」、「system_seconds=0.01」到「0.02」，一次约 0.12 CPU 秒。

## 二、判决

两条腿的结论不一致的地方按下面逐条现查的为准；两条腿都没打中的范围记「没打中」。

**坐实的（打中）**

1. **配置开着 GPU 的崩溃放量第 ② 段写了核对待判行之后没有人判，这一趟一直等到被任务服务照卡住停。** 两条腿各自报了同一处。主 agent 现读：`crates/singlefs-verification/src/crash_amplification.rs:33405` 起 `gpu_verification_server_waiting_line_is_due` 的文档注释写明「块数不动就不打：服务一直不判（对不上版本）时日志不涨，交统一任务服务照卡住停」；`gpu-verification-server/src` 里 `facts_verifier_pending` 零命中。结局不是被当成绿：找不到那样的路（攻方腿追到分项驱动为止）。攻方腿担心的「控制命令把 CPU 顶过阈值、卡住判法不触发、永远挂」主 agent 量了不成立：一次约 0.12 CPU 秒、每 30 秒一次，600 秒里约 2.4 CPU 秒，低于它引的阈值 6 CPU 秒（等待圈自己的读库开销没量，这一句到此为止是推的）。坐实的是失败的样子：合入之前这类分项开头就退 78，合入之后先往全仓那一份库写一批没人判的行、再空等到被停，库里留下没人删的核对待判行。
2. **只排派活计划那一行只数第 ① 段。** 主 agent 现读 `crash_amplification.rs:34175`：常量 `PLACEMENT_LINE_PREFIX` 的注释写分项驱动「读 `pending_states` 定跑不跑」。第 ① 段都有结论、第 ② 段没核过时这一行报 `pending_states=0`，真跑却要给每一块写核对待判行。注释说的与驱动实际的判法是否相符，主 agent 没读驱动那一段，这一句记攻方腿的原话「不符」，交改法那一条的实现员先核。
3. **`CRASH_AMPLIFICATION_PLACEMENT` 那一行留着 `cards=` 与四个恒为 0 的字段，读它与造它的脚本还是旧样子。** 两条腿都报：分项驱动与入口的自证里造的假行带 `host=`（正推腿点 `scripts/crash/crash-amplification-item.sh:332、439、441、446`，`scripts/crash/crash-amplification.sh:997、1000、1001`）；驱动里「排计划借不到显存、退 77」那一支没有原意的输入了。检查单「不兼容旧代码与旧格式」一格不成立。
4. **块号、态序号区间、入池序号是裸 `u64`。** 主 agent 现读 `crates/singlefs-verification/src/verdict_store.rs:350` 起 `FactsVerifierPendingRow`：`state_ordinals: std::ops::Range<u64>`、`pool_entry_number: u64`。检查单「类型与名字」一格不成立。
5. **观察者交回的盘在 CPU 上判那一处，从里层份额拿的条数与实际起的线程数不是一个数。** 主 agent 现读 `crash_amplification.rs:31396`–`31400`（拿 `checking_threads().min(images.len())` 条）与第 31458–31462 行（`judge_the_images_after_the_observers_on_the_cpu` 里按 `checking_threads().max(1)` 切段起线程，不用拿到的条数）。这一处合入之前就有，这一轮看到了就记进改法。
6. **判段开头的注释与第 ② 段 GPU 路线拒算时的行为不一样**（攻方腿：第 24000 行说拒算「照常交回（不退 78）」，第 34331 行直接报错）；第 ① 段同形。主 agent 没逐字核这两行，记成注释与行为要对齐的一处，交实现员核后改注释或改行为。
7. **正文有一句写错了。** 检查单「所有任务多机可用」写「等服务那一段问的是判定库里服务的在跑标记」，两条腿都指出代码起的是 `python3 gpu-verification-server/control check` 子进程（本机 systemctl 与 journalctl，第二台经 ssh）。这一格主 agent 的答案撤回，改判见检查单那一段。

**记下、这一轮不当打中的**

8. 观察者交回的盘那一支有取卡的路（正推腿：「代码本身没有「任务里不取卡」的判断，环境里有卡串或通道就真开卡」）。这是惰性取卡那一成员本来的做法：任务服务交的任务环境里没有卡串与领卡通道（任务服务那一侧已删掉分卡），取不到就打一行、退 78。条款「任务不拿卡」在这一支上靠环境成立、不靠代码；等观察者盘那一族也交 GPU 服务判时这一支整个删掉，登在改法第 6 条。
9. 写回核对行的那一帧里没有认领键与持有者，而判定库服务的写结论帧要求认领键非空（正推腿引 `prune-server/src/protocol.rs:1170`，不在这一轮的快照里，主 agent 没现读）。这是 GPU 服务那一侧接这一族时要定的，不是这三批代码里的错；并进改法第 1 条。
10. 第 ① 段按卡池借草稿区的分支没删净（正推腿点第 4761、5100 行）：记成死分支，并进改法第 3 条由实现员核后删。

**没打中的**（两条腿都看了、都没构造出反例）：核对待判行四种时序（同一块两个进程同时写、写了之后任务崩、写回与读回之间行被删、流键两边算法不同）；只排计划那一趟的复用数在三种库况下的数法；里层线程份额下限 2 与锁次序会不会互等；「拿着改环境的锁去新占名额就 panic」有没有正常路径撞上；空枚举上的 `match *tables {}`。只有攻方腿看了的：惰性取卡退出时不留半截行；这一条只有一次抽样，不拿它当依据。

**检查单八格**：走不到的理由二次查实——成立；类型与名字——不成立（第 4 条）；不兼容旧代码与旧格式——不成立（第 3 条）；不新建判定库——成立；所有任务多机可用——推不出，算打中（第 7 条：等服务那一段起本机的控制命令，主 agent 的正文写错了依据，第二台上那一支没有量过）；测试优先多线程——成立，另记第 5 条那一处存量；带 --item 的命令与项——成立（任务 19325）；新崩溃流多罩了什么——成立。

## 三、改法

1. GPU 服务接核对待判行这一族（扫表、照核对器组键认领、卡上建核对器、带两个守卫一帧写回、写帧带认领键与持有者）。这是判决第 1 条的根：做完之后等的那一段才有人应。没做完之前，第 ② 段在写任何一行之前先判「在跑的服务收不收这一族」，不收就打一行、退 78，与观察者盘那一路取不到卡时的做法一样，不往库里留行。服务怎么让任务知道它收哪几族，由写这一条的实现员与工具员对着 `gpu-verification-server/src/` 的现状定出来再写——零轮。
2. 只排计划那一行把第 ② 段要核的块与状态也数进去（或另打一栏），`PLACEMENT_LINE_PREFIX` 的注释改成驱动实际的判法——零轮。
3. `CRASH_AMPLIFICATION_PLACEMENT` 那一行删掉 `cards=` 与四个恒为 0 的字段；分项驱动与入口不再读它们、自证里的假行换成新样子、「借不到显存退 77」那一支连同它的自证格与开关删掉；第 ① 段没删净的借草稿区分支删掉——零轮（删的是没人走的路，改完由各自的自证与点名测试判）。
4. 块号、态序号、入池序号各立一个 newtype，两张待判行表（待判行、核对待判行）一起换——零轮。两张表同一种写法，只换新加的那一张会留下一半，所以一起换。
5. 观察者盘 CPU 判那一处照拿到的条数切段——零轮。
6. 注释与行为对齐的那一处（判段开头说拒算照常交回）；观察者盘那一族交 GPU 服务判之后删掉任务里惰性取卡的那一支——后一句等服务那一侧接这一族，登在 `records/2026-11-17-崩溃判器与测试流的效率优化.md` 第九节之后服务侧那一件。

第 1 到 5 条标零轮而不先做探针 A/B 的理由：它们不是在被打中的那一支上换一种算法，是补上缺的一方（第 1 条）、删掉没人走的路（第 3 条）、换类型（第 4 条）与让两处用同一个数（第 2、5 条），没有两臂可比；每一条照 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」先做成会红的测试再改。改完再攻一轮（第二轮），派辩方腿复核这一份判决。

## 交用户的

没有要用户定的岔路。告知两件：崩溃放量分项在 GPU 服务接核对待判行之前跑不出结局（合入之前是开头退 78，合入之后是写了行空等到被停），提交时崩溃放量全部分项那一趟要等改法第 1 条做完；这一轮没有发现会把没判过的东西当成判过的路。

## 这一轮被判过的文件

- crates/singlefs-verification/src/crash_amplification.rs：判了崩溃放量一趟、只排计划、第 ② 段写核对待判行与等服务、观察者盘惰性取卡四处是不是条款说的做法；打中第 1、2、3、5、6 条。
- crates/singlefs-verification/src/crash_library.rs：判了测试进程里层线程份额下限与锁、名额的次序；两条腿都构造不出互等与正常路径上的 panic。
- crates/singlefs-verification/src/crash_state_enumeration.rs：判了崩溃枚举用例那一侧观察者盘的取卡路与锁次序的调用处；与 crash_amplification.rs 同一成员，归第 8 条。
- crates/singlefs-verification/src/verdict_store.rs：判了核对待判行那张表的键、值、不在才写、整表扫与一帧写回的两个守卫；形状够，类型打中第 4 条。
- crates/singlefs-verification/src/crash_verify_gpu.rs：判了这一批只改了一句注释、核对内核的行为与表宽常量照旧。
- crates/singlefs-verification/src/crash_point_replay_provenance.rs：判了来历表删掉已删用例那一行之后与合表、登记位对得上（任务 19321）。
- crates/singlefs-verification/src/bin/crash_library_reader.rs：判了按态的三种行只读库、三类态数之和等于录下的（任务 19339 的末行）。
- crates/singlefs-verification/tests/crash_amplification_pipeline_records_checks_and_reuses.rs：判了只排计划那条用例三趟的写法靠「两种判器同一种结论」成立、断言钉的复用数另算。
- crates/singlefs-verification/tests/crash_amplification_placement_only_run_takes_no_card.rs：判了「配置开着 GPU、只排计划不取卡」的断言在这一份里（正推腿点第 129 行）。
- crates/singlefs-verification/tests/crash_amplification_run_of_both_stages_records_the_flow_before_it_leases_a_card.rs：判了它跟着改的断言与卡的来路恒为不要卡一致。
- crates/singlefs-verification/tests/crash_amplification_observer_images_judged_on_the_gpu_name_the_card.rs：判了这一批只改一行注释；它的两条用例在任务里红在一次性库不收待判行，合入前后都红（任务 19313、19315）。
- crates/singlefs-verification/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs：判了卡池名单删两项之后与各份测试文件里的卡池调用对得上（任务 19320 那一趟这个测试目标绿）。
- crates/mutations.tsv：判了这一批追加 41 行、换 3 行、删 4 行，新写与改写的 44 行在快照上都抓到（任务 19325）。
- research/prompts/_amplification-without-cards-code-r1-body.md：判了正文「所有任务多机可用」一格的依据写错（第 7 条）。
- .claude/rules/verification.md：判了「卡只归 GPU 服务，任务不拿卡」在观察者盘那一支上靠任务环境成立（第 8 条）。
- records/2026-11-17-崩溃判器与测试流的效率优化.md：判了第九节说的服务侧三族里核对待判行那一族还没接，与第 1 条的现状一致。
- scripts/crash/crash-amplification-item.sh：判了它读 PLACEMENT 那一行的字段与自证里的假行还是旧样子（第 3 条）。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 15 | 不影响 | 2026-11-18 不受影响：这一轮判的是验证包里崩溃放量怎么把判段交给 GPU 服务，一个验证包、一个门禁、两种跑法的定案没有被动到 |
| D13（验证路线） 已定项 17 | 不影响 | 2026-11-18 不受影响：核对待判行的流键带核对器组键，键的组成照旧只由函数单元、工具链与依赖版本定，这一轮没有改键的算法 |
