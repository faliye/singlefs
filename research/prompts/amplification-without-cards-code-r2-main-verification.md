# amplification-without-cards-code-r2：判决（2026-11-18）

正文 `research/prompts/_amplification-without-cards-code-r2-body.md`，开工快照 `research/prompts/amplification-without-cards-code-r2-snapshot/`。这是这一题的第二轮；第一轮的判决是 `research/prompts/amplification-without-cards-code-r1-main-verification.md`。这一轮攻第一轮改法里已合进主工作区的修补，辩方腿复核第一轮判决。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方（three-way-attack） | `research/prompts/amplification-without-cards-code-r2-opus-output.md` | 「打中 7 条（其中 2 条的读数是推的、待主 agent 跑）；八格检查单里不成立 3 格、推不出 0 格。九问里第 4、5 问与第 9 问的 (a)(c) 构造不出，第 9 问 (b) 推不出。」 |
| 辩方（three-way-defense） | `research/prompts/amplification-without-cards-code-r2-sonnet-output.md` | 「第一轮「坐实的」七条里够得着 6 条（第 7 条够不着）；修补对第 1、2、3、5、6 条改到位 5 条（第 1、3 条各留残留）；八格里不成立 2 格（类型与名字、不兼容旧代码与旧格式），推不出 0 格。」 |

核查用的两条命令（原样）：

- `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/amplification-without-cards-code-r2-verdict -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py amplification-without-cards-code-r2 --out /home/fy5090/.local/share/singlefs/work/amplification-without-cards-code-r2-verdict/amplification-without-cards-code-r2-tree --repository /home/fy5090/code/singlefs`（任务 19497）：「快照 amplification-without-cards-code-r2：清单点名 12 份，倒推出 12 份，sha256 对上 12 份」。
- `python3 scripts/agents/cite-check.py research/prompts/amplification-without-cards-code-r2-sonnet-output.md research/prompts/amplification-without-cards-code-r2-opus-output.md --root /home/fy5090/.local/share/singlefs/work/amplification-without-cards-code-r2-verdict/amplification-without-cards-code-r2-tree --background research/prompts/_amplification-without-cards-code-r2-background.md`：「✓ 核了 123 处引文、26 处裸行号，对上 73 处，没判 76 处（其中裸行号没有引文可比 14 处；逐处列在上面）」，没有一处判成对不上。没判的是快照清单之外的路径（`gpu-verification-server/src/`、`scripts/`、`task-server/`、`.claude/` 下的文件，与腿草稿目录里的副本）；判决引到的那几处主 agent 在主工作区逐行现读，下面各条写了行号。

主 agent 复跑的读数：

- 攻方腿的两行探针变异（两道「有行才问」的守卫改成恒问）：任务 19500，两行都是「没红」（「点名 2 条：现判 2 条、复用 0 条、拒算 0 条；跑了 2 条，抓到 0 条」），点名的两条测试照绿。
- 辩方腿点的五条库单测：任务 19501「5 passed; 0 failed」。计划行那条挂 `gpu` 特性的集成用例：任务 19502「1 passed」（缩域到展开深度 2，缩过域、不算门禁通过）。
- 问服务一把不存在的键、两台各起一次：任务 19503（本机起）与 19504（第二台起）都退 78，那一行两台各一栏都是 `running/<键>`，没有 `unreachable`；连同任务 19423 是三次样本。
- 服务此刻有没有对不上版本的待判行：`python3 gpu-verification-server/control check` 两台都退 0（「近 4 小时这个进程没打 GPU_VERIFICATION_SERVER_VERSION_MISMATCH」）。服务日志里量到的旁证：早先两代进程各打了 268 行版本对不上（同一条流的 268 块，每起一个进程重打一遍），`~/.local/share/singlefs/work/gpu-service-overnight-effect/logs/local-summary.txt` 头两个 `PROCESS`。

## 二、判决

**坐实的（打中）**

1. **写了待判行之后服务换一把键重起，这批行成了没人判、没人删的行；此后每起一个服务进程的四小时里，配了 GPU 的每一趟开头就退 78**（攻方 H1）。主 agent 现读的行：判段入口无条件先问在不在跑，`crates/singlefs-verification/src/crash_amplification.rs:34360`「let control_command = exit_unless_the_gpu_verification_server_runs()?;」；等的那一圈问的也是它（第 34156–34158 行，交回 `ServerNotRunning`）；控制命令 `check` 读到此刻主进程打的版本对不上行就退 1，往回看四小时（`gpu-verification-server/control:239`「VERSION_MISMATCH_LOOKBACK_SECONDS = 4 * 3600」、第 934 行）；服务只认领键相同的行（`gpu-verification-server/src/library_pending_pool.rs:144`），对不上的每个进程打一次、行不删（第 601 行）。「每个进程重打一遍」有量到的旁证（上一节末条）。「四小时里每一趟都退 78」这一句是照这几行推的，没有在真环境里造过一次。出路绕圈那一半（末行叫 `control start`、`check` 叫重编，换上的新进程再打一遍）同样是推的。辩方腿从另一头追到同一处：① 段问到收之后服务换键，「结局是约 30 秒级退 78、库里留行」。两条腿不矛盾：这一趟自己退得快，坏在留下的行连累之后的每一趟。
2. **服务答收、认领之后配不出这条流时，判段照旧空等**（攻方 H2）。主 agent 现读：`crash_amplification.rs:33839` 的注释自己写着「块数不动就不打：服务答收这一族这把版本键却一直不判时日志不涨，交统一任务服务照卡住停」；服务一侧 `library_pending_pool.rs:747`「self.streams_not_configured.insert(pending.name);」之后这个进程不再取它。什么输入会让服务配不出流，腿给的那一种是推的，主 agent 没造出来；坐实的是「有这样一支、它的结局是空等」。等的那一圈自己读库花多少 CPU、会不会让卡住判法不触发（攻方第 9 问 (b)），没人量过，核不动，并进改法第 2 条一起量。
3. **控制命令退 2（用法错）、Python 异常退 1、起不来，都被当成「服务不收」退 78，这一侧自己的缺陷不红**（攻方 H3，辩方腿第八节同一处）。主 agent 现读：`crash_amplification.rs:33702` 的文档「退 78（不收）、别的退出码（不认 `serves` 的控制命令退 2 这类）」都当不收、第 33736 行「（控制命令不认 serves 的也当不收）」；控制命令自己的退出码表 `gpu-verification-server/control:44` 把 2 定成「用法错（含 serves 族名不认得、缺参数）」。族名两边各写一份字面量（`control:253` 与 `crash_amplification.rs` 的枚举），没有测试把它们绑在一起。同一棵树里的控制命令认 `serves`，「不认也当不收」只为旧控制命令留。
4. **`DOES_NOT_SERVE` 这个行头没有一个读者认**（攻方 H4，量的）。任务 19423 的结局是分项驱动打「修好设备，或改配置：日志里 GPU_CARD_NOT_USABLE 行点名的卡用不了」那一句泛的出路。崩溃枚举用例那一侧同一种情形由门禁照红判，与观察者盘那一路记本次未跑不一致（往严里错，不会记成过）。`said=` 里 `family=`、`expected=` 各出现两次并带进两台的地址，单测钉的样子没有 `host.` 那几栏，与真控制命令打的不一样。
5. **切段时段长向上取整，段数会少于拿到的条数**（攻方 H5）。主 agent 现读 `crash_amplification.rs:31758`「let piece_length = image_count.div_ceil(threads_taken.max(1)).max(1);」：18 张盘拿 17 条只起 9 段，另 8 条在单测构建里占着里层份额到判完。第一轮改法第 5 条「照拿到的条数切段」在不整除时没做到；攻方腿撤回了它第一轮「换成守卫的条数就够」那一句。按盘的次序交回成立。
6. **两道「有行才问」的守卫没有一条测试钉着**（攻方 H6，量的，任务 19500）：改成恒问，点名的两条测试照绿。变异表里这两处只有「跳过那一问」的方向。
7. **`GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort` 在产品路径上造不出来，消费它的两臂、一个布尔与一句注释指着已删的语义**（攻方 H7）。主 agent 现读：`crash_amplification.rs:4745`（唯一调用方当场改成 `Err`）、第 24984 行与 `crash_state_enumeration.rs:2757`（两处消费臂）、第 25002 行「let stop_was_requested = gpu_memory_was_short || stop_is_requested();」。辩方腿另追到第一轮第 10 条的来历：被删的借草稿区那一臂在 HEAD 里有第二个调用方 `print_the_placement`，是第一轮那几批删了调用方留下的壳；第一轮把它记成「死分支、不当打中」不对，这一轮升成打中，并进这一条。
8. **只在单测构建里编的那一层 `judge_my_blocks_through_the_pending_rows` 是为旧签名留的**（两条腿都判检查单「不兼容旧代码与旧格式」不成立）。主 agent 的正文已认、已登账；这一轮定成打中，同批删。

**对第一轮判决的复核（辩方腿）**

- 第一轮七条：辩方腿判 6 条够得着、没有一条判错。第 7 条是正文勘误、不是代码的打中，照辩方腿的话改记。第 2 条够到的比判决写的窄（驱动只看有没有那一行，不凭栏的值定跑不跑），实现员当时已核到同一处，判决那一句照窄的读。第 4、5 条打中的是 HEAD 里已有的写法，不是那三批的回归，记法不变。
- 第 8、9 条不升：辩方腿各给了今天能核的支点（任务服务对选卡变量提交时拒，`task-server/task_server_lib.py:2020`；判器那一族的认领键同样由服务一侧拼），主 agent 采纳。
- 第 10 条升成打中（上面第 7 条）。
- 「惰性取卡退出时不留半截行」：攻方腿两轮各追一次、辩方腿独立追一次，三次都没构造出，记「没打中」。
- 第一轮改法第 1 到 5 条标零轮而不做探针 A/B 的理由：辩方腿判「字面站不住」，主 agent 认。规则的两臂是「有这一改法」与「没有它」，第 1、2、5 条都量得出。实际发生的是实现员先写红测试、主 agent 用证红脚本判（任务 19395「点名 13 条……抓到 13 条」），等于单测尺度上的两臂，但读数在写进产品代码之后才有、任务号也没写回第一轮判决。这一轮照规则办：见「三、改法」开头。

**没打中的**：服务停而不换键、写到一半停、答收之后在跑那一行被换掉，三种都不留没人判的行（攻方第 1 问）；库里已有同键的行时这一趟照排、照问，排出空时不问也不等（第 2 问）；测试二进制里 `exit(78)` 不会让没跑完的记成过（第 4 问）；只排计划那一行的后两栏在事实表不在库里时到不了数栏、两对栏之间没有该守而没守的关系（第 5 问）；问服务的子进程在第二台上起时两台都问得到（第 9 问 (c)，三次样本）。这几条只有攻方腿一条腿覆盖，辩方腿的分工是复核第一轮；按「只有一条腿覆盖到的范围」的规矩，它们不当这一题收口的依据，第三轮不重攻、也不引它们撑结论。

**检查单八格**：走不到的理由二次查实——成立（两条腿各自追到 HEAD 里借草稿区那一支的调用方；实现员写的理由不是对的那一条，结论对）；类型与名字——不成立（第一轮第 4 条没做，另加族名两边各写一份）；不兼容旧代码与旧格式——不成立（第 3、8 条）；不新建判定库——成立；所有任务多机可用——成立（三次样本）；测试优先多线程——两条腿不一致（攻方不成立、辩方成立），按第 5 条现查的那一行判不成立；带 --item 的命令与项——成立；新崩溃流多罩了什么——成立。

## 三、改法

这一轮的改法都是腿提的或主 agent 照腿的话归并的，被攻过零轮。写进产品代码之前先做 A/B：每一条先在副本里写出「没有这一改法时红、有了它才绿」的测试，主 agent 经任务服务用证红脚本跑出两臂的读数，任务号补进这一节对应那一条，再合。第 6 条的「没有改法」那一臂已有读数（任务 19500，两行没红）。

A/B 读数（都在实现员的副本里、合进主工作区之前量的；下面各条末尾写「A/B 读数待量」的验证包那一半以这一段为准）：第 1 条验证包那一半与第 3、5、6、7 条，「没有改法」一臂是证红脚本把每处改法改回去的那 34 行变异，任务 19681「点名 34 条：现判 8 条、复用 26 条……抓到 34 条」；「有改法」一臂是点名的库测试，任务 19679「42 passed」、带 gpu 特性那一条任务 19680「1 passed」。第 1 条动服务的那一半（删没有服务收的待判行）与第 2 条没写代码，没有读数。第 3 条里「从控制命令的源码读族名表」那条单测没留在验证包里（测试读 `crates/` 之外的源码，算键时逃出包），改放服务一侧，登在 `records/2026-11-17-subagent账本自上而下清欠.md` 第二百五十四节第 5 条。

1. （第 1 条）等的那一圈与开头那一问改成按族、按这一趟的键问（`serves <族> <键>`），不再拿全局的 `check` 判在不在跑；开头那一问挪到排出了行之后，一行都不用写的一趟不问。库里没有任何在跑的服务收它那把键的待判行，给一条点名的删除路（服务或控制命令一侧，删之前打一行点名）：这一半动服务，并进服务一侧下一批——零轮，A/B 读数待量。
2. （第 2 条）先量再定：造一条服务认领得到而配不出的流，量等的那一圈的 CPU 与卡住判法触不触发；改法在「服务把配不出的原因写到判段读得到的地方」与「判段块数不动满一段就退 78」之间选，量完写成岔路——零轮，本轮不写代码。
3. （第 3 条）只把「退 78 且标准输出有 `served=no` 那一行」当不收；别的退出码、被信号杀、起不来交回 `Err`，照这一侧的错判红。删掉「不认 serves 也当不收」与它的单测。加一条单测从 `gpu-verification-server/control` 的源码读族名表与判段的两个族名比——零轮，A/B 读数待量。
4. （第 4 条）读 `DOES_NOT_SERVE` 行头的一方补齐：分项驱动、入口与共用库那一半已由治理类的一件写好新版（`~/.local/share/singlefs/work/batch99-amplification-entry-leftovers-and-switch-names/`），这一份判决写完之后换上；门禁里崩溃枚举用例那一路记红还是记本次未跑，定成与观察者盘那一路相同的本次未跑，改那份门禁并进治理类下一批。`said=` 那一栏的单测改钉真控制命令打的样子（带两台各一栏）——零轮。
5. （第 5 条）均分切段：段数取盘数与拿到的条数里小的那个，前「盘数除以段数的余数」段各多一张；单测钉 18 张 17 条时 17 段——零轮，A/B 读数待量。
6. （第 6 条）两段各加一条「全部复用、替身控制命令一问就答不收并留记号」的测试，断言这一趟判完、记号不在；变异表加「恒问」方向两行——「没有改法」一臂的读数是任务 19500。
7. （第 7、8 条）删 `NotRunTheGpuMemoryWasShort` 与两处消费臂、`gpu_memory_was_short`、那半句注释，改 `crates/mutations.tsv` 里钉第 25002 行的那一行；删只在单测构建里编的那一层，`crash_state_enumeration.rs` 那条单测改调 `ask_the_server_then_judge_my_blocks_through_the_pending_rows`——零轮（删的是造不出来的成员与旧签名，由点名测试与两种特性组合的 clippy 判）。

第 1 条的验证包那一半与第 3、5、6、7 条都落在 `crash_amplification.rs`（第 7 条另碰 `crash_state_enumeration.rs`），并成一簇交一个实现员；第一轮改法第 4 条（两张待判行表立 newtype）仍等服务一侧接核对那一族合入之后做。改完开第三轮，攻这一轮的修补；第三轮是这一题的最后一轮。

## 交用户的

没有要用户定的岔路。告知三件：

- 这一轮最重的一条（第 1 条）在今天的库上没有发作（两台 `check` 都退 0），发作的条件是「库里留着旧键的待判行时换上新键的服务」；提交时的门禁与崩溃放量跑起来之后每合一批改到判器组键的补丁都可能撞上，所以排在提交时那一趟之前改。
- 第 2 条本轮只量、不改，量完会有一条岔路交你定。
- 第一轮判决里「改法不做探针 A/B」那一段主 agent 认了写得不对，这一轮起照规则先量后合。

## 这一轮被判过的文件

- crates/singlefs-verification/src/crash_amplification.rs：判了写待判行之前问服务、等服务的那一圈、只排计划那一行、观察者盘切段、删借草稿区之后剩下的成员与只在单测构建里编的那一层；打中第 1、2、3、5、6、7、8 条。
- crates/singlefs-verification/src/crash_point_replay_provenance.rs：判了来历表新加的 `placement-facts-verifier` 一行对着流水线测试里的新用例，两条腿都判检查单「新崩溃流多罩了什么」成立。
- crates/singlefs-verification/src/crash_state_enumeration.rs：判了它调 `judge_my_blocks_on_gpu` 的那一处在问到不收时的结局（测试二进制退 78、门禁照红判，不会记成过）与显存不够那一条消费臂（第 7 条）。
- crates/singlefs-verification/tests/crash_amplification_pipeline_records_checks_and_reuses.rs：判了新用例钉的库况（第 ① 段结论都在、第 ② 段一块都没核过时后两栏等于整条流），任务 19502 在主工作区上过。
- crates/singlefs-verification/tests/crash_amplification_placement_only_run_takes_no_card.rs：判了它跟着改的七栏断言与计划行逐字那条单测一致（任务 19501 里那一条过）。
- crates/mutations.tsv：判了这一批新写与改写的 17 行（任务 19409 都抓到）与两道守卫缺「恒问」方向的行（第 6 条，任务 19500）。
- research/prompts/_amplification-without-cards-code-r2-body.md：判了正文八格的答案，「测试优先多线程」一格写的「照拿到的条数起线程」在不整除时不对（第 5 条）。
- research/prompts/amplification-without-cards-code-r1-main-verification.md：判了第一轮七条够不够得着、第 8 到 10 条该不该升、零轮不做探针 A/B 的理由；第 10 条升成打中，A/B 那一段认错。
- .claude/rules/verification.md：判了「卡只归 GPU 服务，任务不拿卡」在修补之后的两段上照旧成立，任务进程只起控制命令问服务。
- scripts/crash/crash-amplification-item.sh：判了它对服务不收某一族那一种退 78 打的是泛的出路（第 4 条）。
- gpu-verification-server/control：判了 `check` 读版本对不上行的判法与四小时回看、`serves` 至少一台收就收的合成、退出码表里 2 是用法错（第 1、3 条的支点）。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 15 | 不影响 | 2026-11-18 不受影响：这一轮判的是判段怎么问 GPU 服务、等不到时怎么退，一个验证包、一个门禁、两种跑法的定案没有被动到 |
| D13（验证路线） 已定项 17 | 不影响 | 2026-11-18 不受影响：第 1 条说的是键换了之后旧键的待判行没人删，键的组成与算法照旧只由函数单元、工具链与依赖版本定，这一轮没有改键的算法 |
