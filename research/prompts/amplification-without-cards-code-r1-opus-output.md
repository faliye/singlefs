# 崩溃放量任务里不领卡：代码轮第一轮攻方腿报告

日期：2026-11-18。被判的是主工作区此刻的代码；开工时核过 `research/prompts/amplification-without-cards-code-r1-snapshot/` 两份哈希表，被判文件与快照逐个相同（`sha256sum -c` 没有一行不是 OK）。
这一轮没写模型（打中的都是流程与协议上的走法，靠读代码追到行，复现交给一次放量分项与几条该红的单测），所以没有模型目录、没有 `rerun.sh`；理由见末尾「这条腿自己的限度」。
下面凡没跑出来的读数都标「推的」。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 1 核对待判行四种时序 | 没打中 | 两进程同写、写后崩、写回与读回之间删行、流键三段，四种都走到「对」；只有「行被删而核对行不在」那一窄缝会让服务重核一块（重判，不漏） |
| 2 第 5 条现状（没人判核对待判行） | 打中 | 追到底没有「当成绿」的路；但结局是无声等待：进度行在块数不动时不打，交给任务服务的卡住判法，被停记 stalled（打回，233），或卡住判法被 30 秒一次的 `control check` 子进程顶过阈值而一直挂。同一种「服务还不收这一族」在观察者交回的盘那一路是当场退 78、点名出路，两处做法矛盾 |
| 3a 只排计划那一趟的复用数 | 没打中（三种库况数对）；另打中一处 | 别的版本键、只有观察者结论、只有一部分块有结论，三种都数对。打中的是：PLACEMENT 行只数第 ① 段，不数第 ② 段；「CPU 判完、开 GPU 再跑」那段历史上它报 `pending_states=0`，真跑却给每一块写核对待判行、挂住 |
| 3b PLACEMENT 去掉 `host=` 之后的读者 | 打中 | Rust 那一行留着 `cards=` 与四个恒为 0 的字段；分项驱动仍读其中的 `estimated_seconds_on_this_host` 算速度（恒 0）；驱动与入口的自证仍造旧形态的行；驱动「排计划借不到显存、退 77」那一支 Rust 已不会照原意打，只剩一种会被它误分类的行 |
| 4a 里层下限 2 与互等 | 没打中互等 | 份额 6 时两份合起来 9，超订属实；找不到拿着一样等另一样的环 |
| 4b 「拿着改环境的锁去新占名额就 panic」 | 没打中 | 只有那条故意反着拿的单测撞得上 |
| 4c 惰性取卡退出会不会留半截行 | 没打中 | 退出点不在写帧中间；配了 GPU 时这一路被第 ② 段的等待挡在前面，此刻走不到 |
| 4d 观察者盘 CPU 判那一处的线程份额 | 打中（存量） | 从份额拿 `min(核对线程数, 盘数)` 截到份额之后的条数，实际起的却是没截的那么多条 |
| 5 八格检查单 | 见第五节 | 走不到的理由：成立；类型与名字：不成立；不兼容旧东西：不成立；不新建判定库：成立；多机可用：推不出（理由写错）；测试多线程：不成立；带 --item：成立（变异行）；新崩溃流：成立 |
| 附 拒算时 GPU 路线报错 | 打中（文档与代码矛盾，与第 ① 段同形） | 判段开头说拒算的组「照常交回（不退 78）」，第 ② 段 GPU 路线拒算时直接报错，整趟非 0 |

「打中」的每一条都在下面给了文件:行、触发它的输入或时序、会看到什么、复现命令或该红的单测。四句（分不分辨臂、系统看不看得到判别它的东西、满足判据字面哪一句、跑前条款的改法）逐条答在各条末尾：这是代码轮，没有臂，「分不分辨臂」一律答「不涉臂」；正文写了「这一轮不判它们该怎么修」，跑前没有给改法，第四句一律答「跑前没给改法」，我自己提的改法都标「只在我的推演上、被攻过零轮」。

## 一、哪一条最脆弱：配了 GPU 的第 ② 段（派发第 2 条）

### 打中 A：核对待判行写下之后无声等待，结局交给任务服务的卡住判法

**走法（文件:行，量的是读代码，没跑）**

1. 本机多机配置开着 GPU（`grep -n '^ENABLE_GPU' multi-host.env` 打 `28:ENABLE_GPU=1`）。`computation_routes` 开了 GPU 就把第 ② 段也定成 GPU（`crates/singlefs-verification/src/crash_amplification.rs:1476-1479`）。
2. 第 ① 段判完、没被叫停，就走第 ② 段：「let facts_verifier_route = (!stop_was_requested).then_some(routes.facts_verifier);」（`crates/singlefs-verification/src/crash_amplification.rs:24754`），进 `verify_my_blocks_on_gpu`。
3. 先问一次服务在不在跑（`python3 <控制命令> check`，见第五节第 5 格），在跑就把这台机器领的、在 GPU 核对键下还没核过的块全写成核对待判行，然后进等待圈。等待圈的注释自己写了这一种结局：「服务在跑而一直不核（对不上版本）就一直等，块数不动不打进度行」（`crates/singlefs-verification/src/crash_amplification.rs:34398`）。
4. GPU 服务只扫判器那一族待判行：「.every_gpu_pending_row()?」（`gpu-verification-server/src/verification_package_side.rs:88`）；`grep -rn facts_verifier gpu-verification-server/src` 零命中。核对待判行永远没人删，块数永远不动。
5. 进度行只在块数变了才打：「&& blocks_remaining_now != blocks_remaining_at_the_last_line」（`crates/singlefs-verification/src/crash_amplification.rs:33415`）。所以从写完核对待判行起，这一趟的日志一行都不再涨。
6. 之后只剩两种外力：每 15 圈（30 秒）问一次服务，服务停了就退 78；任务服务的卡住判法。卡住判法要日志与 CPU 都不涨：「STALL_CPU_CORES = 0.01」（`task-server/task_server.py:836`），窗口 600 秒，宽限 600 秒。被判卡住的结局：「stalled｜打回｜宽限过后日志与 CPU 都满 STALL_SECONDS（10 分钟）不涨，服务停了它」（`task-server/task_server_lib.py:1818`），退出码 233。

**会看到什么（推的，待主 agent 跑）**

- 退出码：任务末行 `TASK_SERVER_RESULT` 是 stalled、233；或者（见下一条）一直不结束。没有一行是 `CRASH_AMPLIFICATION`，分项驱动与入口都没机会打汇总，所以**不会被当成判过、当成绿**。
- 卡住判法会不会触发是推不出的：等待圈每 30 秒起一次 `python3 gpu-verification-server/control check`，它要对点了卡的每一台判单元、二进制、日志里的版本对不上行，第二台经 ssh 问（`gpu-verification-server/control:34` 起那一段：「点名了卡的每一台逐样判二进制在不在、能不能执行」）。代码注释承认这一处是靠「30 秒一次」压在阈值下面：「问的那些 python 进程的 CPU 就让它判不出」（`crates/singlefs-verification/src/crash_amplification.rs:33308`）。窗口 600 秒里 20 次，阈值 6 CPU 秒，每次 check 加上它起的 ssh、systemctl、journalctl 摊到 0.3 CPU 秒以上就过阈值，任务服务判它在算、不停它；每 2 秒一圈的成批点读在块多的流上（几千块）也往上加。这时整趟挂到四小时被停（没带 `--full-gate`），带 `--full-gate` 的提交门禁就没有停的那一刻，只有两小时提醒。
- 库里留下的：每一块一行 `0x1F` 核对待判行，键里带录下的流程摘要，换一次流程（片挪了、树改了）就是另一条流、再写一批，没人删。

**被判的系统看不看得到判别它的东西**：看不到。`control check` 只答服务在不在跑、版本对不对，不答服务收不收核对待判行这一族；判段手里没有能区分「服务慢」与「服务根本不收这一族」的观测。这属于「判别子观测不到」那一形：只调等待的参数修不好，要服务那一侧报自己收哪几族。

**同一件事在别处的做法（内部矛盾）**：观察者交回的盘也是「服务还不收这一族」，那一路当场打一行、退 78、写出路：「卡归 GPU 批处理服务，观察者交回的盘这一类判活服务还不收」（`crates/singlefs-verification/src/crash_amplification.rs:31345`）。第 ② 段遇到同一种情形却写行、无声等。两族的处置不一致，正文第 5 条也没写这一趟会挂多久、被判成什么。

**这三批让本来判得了的变得判不了吗**：不是从「判得完」变成「判不完」——HEAD 那一份第 ② 段在任务进程里取卡（`git show HEAD:crates/singlefs-verification/src/crash_amplification.rs` 里 `verify_my_blocks_on_the_first_card`），任务服务不交卡（「CARD_CHOICE_VARIABLES = ("GPU_CARDS_BY_PRIORITY", "CUDA_VISIBLE_DEVICES")」，`task-server/task_server_lib.py:2020`，任务带了提交时拒），按读代码推那时开头取卡就退 78。变的是失败的样子：从「退 78、点名出路、入口停在这一项」换成「无声等待、交给卡住判法、出路写的是改了再重交」。另一个连带：配了 GPU 时观察者盘那一路排在第 ② 段之后（第 ② 段在 24754 行，观察者判段在 24872 行），第 ② 段挂住，观察者那一路的 78 永远走不到。

**满足判据字面哪一句**：派发第 2 条「会不会被入口脚本或门禁当成判过、当成绿」——不会；落在派发第 1 条验收的「挂住」那一种结局上。

**改法（只在我的推演上、被攻过零轮）**：写核对待判行之前照观察者盘那一路的样子退 78、打「服务还不收核对待判行」与出路（服务接上这一族之前）；服务接上之后，`control check` 或服务的在跑行报它收哪几族，判段不收就退 78；等待圈块数不动也按进度间隔打一行（日志涨，卡住判法不再是唯一的出口，人也看得出它在等什么）。

**复现**：见「要主 agent 跑的命令」第 1、2 条；该红的单测见第 3 条。

## 二、核对待判行四种时序（派发第 1 条）：没打中

逐种走到哪一行、结局是什么（读代码推的）：

| 时序 | 走到哪 | 结局 |
|---|---|---|
| 两个任务进程同写一块 | 先到的认领到键、写行（`crates/singlefs-verification/src/verdict_store.rs:4703-4706`）；后到的答别人认领着、隔一会再问（4722-4733），认领在写帧确认时放，再问答键写过、读回、交回库里那一份（4708-4715）。两边等的是同一把（流键，块号），值只差入池序号 | 对 |
| 任务写了行之后崩了 | 行留在库里；下一趟先查核对行，没核过就写、答已在、交回库里那一份接着等（`crates/singlefs-verification/src/crash_amplification.rs:34351-34387`）。认领着还没写就崩的，租期过了别人照写 | 对（服务接上之后下一趟接着等；此刻是第一节那一种挂） |
| 服务写回与任务读回之间行被删 | 服务那一帧原子地写核对行、删核对待判行（`verdict_store.rs:4820-4854` 编帧，带两个守卫）；任务读到行不在就读核对行，在就记核完（34410-34423） | 对 |
| 行被删而核对行不在 | 任务照不在才写补写（34424-34431），值是它第一次读到的那一份（旧入池序号）。两个判段同时补写时不经认领、后写的盖先写的，服务若刚好读了先写的那一份，那一帧的核对待判行守卫对不上、整帧拒，服务重读重核：重判一块，不漏 | 对（多一次重判） |
| 流键三段两边算法 | 判段：`facts_verifier_pending_stream_key(读回清单的流程摘要, 读回事实表的摘要, GPU 核对器组键的指纹)`（34336-34340）；服务编帧时用行里同样三样重算、对不上就拒（34691-34701）；核对行的键两边都只取指纹（`facts_verifier_row_key` 只读 `fingerprint()`，5556-5564；服务那一边用 `JudgingUnitsKey::of_the_parts(指纹, 空清单)`，34630-34637）。读回事实表按摘要核过（`crash_library.rs:2033-2042`） | 对，三段同算法 |

另试过的形状：同一份计划里两个点块键相同（「两个设点文件、两个名字下内容相同的点是同一把块键」）——两个块号各写一行、指同一把块键；服务核完前一块后第二行照「核完了只删」处理，第二块读回的核对行结论位条数与它的状态数相等（块键带块长），记账对。分片的段：片挪了之后录下的流程字节变、流程摘要变，是另一条流，块号不串。

## 三、只排派活计划那一趟（派发第 3 条）

### 3a 复用数：三种库况都数对（没打中）

`placement_line_of_the_pending_blocks` 照 `pending_blocks` 数（`crates/singlefs-verification/src/crash_amplification.rs:34193`），`pending_blocks` 只认判器那一种结论、这一把版本、判过带槽数而不红的块（4038-4060，经 2027-2038）。只有别的版本键的结论：全部算要判；只有观察者结论：种类不同，全部算要判；只有一部分块有结论：逐块数。`states_reused` 是同一个谓词在开跑前读一次的结果（23628-23655），两数之和等于这一份的块的状态数。两种判器同一把版本（「_judged_by_either_judge: JudgedBy,」，`crates/singlefs-verification/src/crash_amplification.rs:26734`），CPU 判完再开 GPU 排计划数成全部复用，与正文第 8 条一致。

### 打中 B：PLACEMENT 行不数第 ② 段，「CPU 判完、开 GPU 再跑」那段历史上它说没活、真跑挂住

- 文件:行：那一行自称「这台机器这一趟要判的块数与状态数」（`crates/singlefs-verification/src/crash_amplification.rs:34179`），只数第 ① 段；第 ② 段在 GPU 核对键下要核的块不进任何字段。头一行的文档又说驱动「读 `pending_states` 定跑不跑」（`crates/singlefs-verification/src/crash_amplification.rs:34175`），驱动实际只看有没有这一行（`scripts/crash/crash-amplification-item.sh:244-245`，`pending` 是空串才不跑，0 照跑）。
- 历史（用户动作放开扫过：先 CPU 判还是先 GPU 排计划、中间重录不重录，只固定「这条流在 CPU 配置下判完」这一前缀）：多机配置 `ENABLE_GPU=0` 跑一趟分项，判器结论与 CPU 核对行进库；改成 `ENABLE_GPU=1`、编进 gpu 特性再跑同一分项。排计划那一趟打 `pending_states=0 states_reused=<全部>`；本机那一趟第 ① 段整段复用，第 ② 段在 GPU 核对键下一块都没有（CPU 与 GPU 两组核对单元是两把键，24531-24536），给每一块写核对待判行，走进第一节的挂。前缀里先 GPU 排计划、或中间重录（录入键命中不重录），都不改这两个数。
- 会看到：placement.log 那一行说这台没有活，DECISION 行 `pending_states=0`，run.log 停在第 ① 段的汇总之后不动。
- 系统看得到判别它的东西吗：看得到。排计划那一趟手里有读回的事实表摘要与 GPU 核对器组键，照第 ② 段同样的读法就数得出（推的）。
- 改法（零轮）：PLACEMENT 行加第 ② 段在 GPU 核对键下没核过的块数与状态数两个字段，或把 34175 行的「定跑不跑」改成驱动真实的判法。

### 打中 C：去掉 `host=` 之后，Rust 与读者两边都还留着旧形态

- Rust 那一行仍打五个不带信息的字段：「cards= states_placed_on_this_host=0 states_placed_on_other_hosts=0 states_per_second_on_this_host=0 estimated_seconds_on_this_host=0」（`crates/singlefs-verification/src/crash_amplification.rs:34200`）。`.claude/agent-common.md` 那一条「里程碑 17 收尾之前不兼容任何旧东西」要的是「读者只认这一版」，这五格只是为旧读者留的列。
- 分项驱动仍按旧字段算速度：「seconds += int(fields.get("estimated_seconds_on_this_host", 0))」（`scripts/crash/crash-amplification-lib.sh:692`），新行恒 0，DECISION 行的 `states_per_second_on_this_host` 恒打 0；驱动文件头第 39 行还说它是「本机的卡试判量到的速度」。
- 两份自证仍造旧形态的行，测的是 Rust 已不打的样子：「echo "CRASH_AMPLIFICATION_PLACEMENT host=127.0.0.1 judge=gpu」（`scripts/crash/crash-amplification-item.sh:332`，同形还有 446 行），入口 `scripts/crash/crash-amplification.sh:997-1001` 的假驱动打 `host=local … cards=local:0:100:100`。速度那一格（驱动自证第 359-363 行，`pending_states=800 states_per_second_on_this_host=16`）靠的正是旧行里非 0 的 `estimated_seconds_on_this_host`。
- 驱动「排计划借不到显存、本次未跑、退 77」那一支（「echo "CRASH_AMPLIFICATION_DECISION where=not-run reason=no-card-of-the-pool-had-room-in-time"」，`scripts/crash/crash-amplification-item.sh:248`）认的是 `NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment` 打头、退 0 的排计划日志。排计划那一趟此刻不碰卡，打不出原意的那一行；还能打出这个头的只剩「配置开 GPU、二进制没编进 gpu 特性」那两行（`crates/singlefs-verification/src/crash_amplification.rs:1582-1592`，退 0、不打 PLACEMENT），驱动会把它记成「池里的卡没空出来」、退 77，而驱动文件头写这一种该退 78。走到要判段二进制不带 gpu 特性，驱动在 GPU 配置下自己带特性编，所以此刻走不到（推的）。
- 双机驱动 `scripts/crash/crash-amplification-two-hosts.sh` 不读 PLACEMENT 行（`grep -n PLACEMENT` 零命中），入口只在自证假驱动里打它、汇总不取。
- 改法（零轮）：Rust 只打 `judge= pending_blocks= pending_states= states_reused= tables_mebibytes=`；驱动删速度那一段与 77 那一支连同它们的自证格；两份自证的造行换成新形态。

## 四、测试进程线程份额与锁次序（派发第 4 条）

### 4a 里层下限 2 与互等：没打中互等

算术属实：份额 6 时临时服务一份「let temporary_servers = (heavy_threads / 2 / THREADS_OF_A_TEMPORARY_SERVER).max(1);」（`crates/singlefs-verification/src/crash_library.rs:567`）得 1 份记 7 条，里层取下限 2，合起来 9 条，超份额 3 条；份额小于 14 时临时服务都只有 1 份，库测试里用一次性库的用例一条接一条。

找环的形状与范围（都是读代码，验证包 `src/` 下的库测试；集成测试目标按非 test 构建编库，名额那一套 `#[cfg(test)]` 不在里面）：

- 里层线程的四处取法（`crates/singlefs-verification/src/crash_amplification.rs:2875`、`30588`、`31397`，`crash_identity.rs:909`）拿到之后都不再等别的：不再取里层、不开一次性库、不拿改环境的锁、不设一次性库。持有者总会还，等的一方总等得到。
- 名额：56 处 `DisposableLibrary::new` 全在测试模块里；同一个函数里先起线程、线程里再开一次性库的只有那条故意反着拿的单测（`crash_library.rs:5737`）。没有「主线程持名额、join 一条要新名额的子线程」。
- 改环境的锁与设一次性库那把读写锁：逐个函数比了两者的先后，`src/` 与 `tests/` 里同一函数两样都拿的 47 处都是先设库、后拿锁（脚本扫出一处先锁后设，是 `crates/singlefs-verification/tests/crash_injection_campaign.rs:119-127` 两个分支各拿一次、每支都是先库后锁，误报）。没找到先拿锁、后设库的用例，凑不出「甲持库等锁、乙持锁等库」。

结论：超订成立，互相等死没构造出来。

### 4b panic 那一处的正常路径：没打中

检查在 `crash_library.rs:700-704`，只在「这条线程此刻没起着一份、又拿着改环境的锁」时 panic。扫的形状：一条线程先开库、拿锁、丢掉那一份、再开一份（计数归 0 之后再开会 panic）——没找到这样写的用例；`crates/singlefs-verification/src/crash_state_enumeration.rs:3488` 那一处是持着一份再开第二份，不查。只有 `taking_a_disposable_library_place_while_holding_the_environment_lock_panics_at_once_naming_the_order` 撞得上，它就是证这一句的。

### 4c 惰性取卡的退出会不会留半截行：没打中

退出发生在 `judge` 里第一次要判盘、取卡那一刻（`crates/singlefs-verification/src/crash_amplification.rs:31320-31328`），那时这一批盘的结论还没编、没写；库是每次读写现取现放，退的那一刻不握库锁。写都经服务一帧原子落，不会有一半在库里。`process::exit` 不跑析构：客户端追加日志那份一次性目录留在任务的临时目录里（推的），任务服务收尾清它。配了 GPU 时这一路排在第 ② 段之后，此刻被第一节的挂挡住、走不到。

### 打中 E（存量）：观察者盘 CPU 判那一处起的线程多于从份额拿到的

- 文件:行：拿的是「checking_threads().min(images.len()),」（`crates/singlefs-verification/src/crash_amplification.rs:31398`），`InnerThreadsTaken::take` 把它截到里层份额；随后真正分块起线程用的是没截的数：「let threads = checking_threads().max(1);」（`crates/singlefs-verification/src/crash_amplification.rs:31460`），守卫的 `threads()` 没被用上。别的三处都照 `threads.threads()` 切。
- 输入：库测试进程，配置关 GPU，观察者交回的盘不少于 3 张，`SINGLEFS_THREAD_CAP=6`（合入后验证几个目标并一片的份额）：拿到 2 条，起 `min(盘数, 6)` 条；默认 31 时里层 17 条，盘数 31 以上起 31 条。
- 会看到：份额被这一处超用（超 4 到 14 条），同时跑的别的用例里层照份额算、以为没超。没有挂，没有红。
- HEAD 里同一处已是这样（`git show HEAD:…crash_amplification.rs` 第 30659 行），不是这三批新加的；它让第五节第 6 格「里层并行拿线程经 `InnerThreadsTaken::take`」的说法在这一处不成立。
- 该红的单测：照 `judge_the_images_after_the_observers_on_the_cpu` 起的线程数断言它不超过 `InnerThreadsTaken::take` 交回的条数（把分块数改成从守卫取就绿）。改法（零轮）：31460 行用守卫的 `threads()`。

### 打中 D：拒算时 GPU 路线报错，与判段开头的约定相反（与第 ① 段同形）

- 判段开头约定：「这条流的块全部计入拒算（refused_blocks=），照常交回（不退 78）」（`crates/singlefs-verification/src/crash_amplification.rs:24000`）。
- 第 ② 段 GPU 路线拒算时：「"这一把核对键照判不入库（GPU 核对器的判法单元拒算）：GPU 批处理服务写不进这个进程里的行，不写核对待判行"」（`crates/singlefs-verification/src/crash_amplification.rs:34331`），`?` 一路传出去，整趟交回错误（用例 `expect` 处 panic，推的退 101），分项驱动原样退、入口记红。CPU 路线同样拒算时照判不入库、留在进程里接着核。
- 输入：配了 GPU，编译时 GPU 核对器那一组闭包里有认不出的形态（`JudgingUnitsKeyRefused::RefusedAtTheBuild`）。
- 第 ① 段对拒算的结论版本也是这样报错（`crates/singlefs-verification/src/crash_amplification.rs:33572`），所以这一条不是这三批独有的毛病，是同一种做法扩到了第 ② 段；矛盾在那一句约定没跟着改。系统看得到判别它的东西（拒算那一刻就知道）。改法（零轮）：约定那一句写明「配了 GPU 时拒算的组报错、不照常交回」，或让 GPU 路线拒算时记进 `refused_blocks` 照常交回。

## 五、正文末尾八格检查单（派发第 5 条）

| 格 | 判定 | 依据 |
|---|---|---|
| 走不到的理由二次查实 | 成立 | 没编进 gpu 特性时那个类型是「pub enum PackedJudgeTablesOfTheRun {}」（`crates/singlefs-verification/src/crash_amplification.rs:7310`），只派生 `Debug`；库根「#![deny(unsafe_code)]」（`crates/singlefs-verification/src/lib.rs:18`），`grep -rn 'allow(unsafe_code)\|unsafe {' crates/singlefs-verification/src/` 只命中两份 `src/bin/` 实验程序（各自是另一个 crate 根，不提这个类型）。造不出值，`match *tables {}` 是无值类型上的穷举 |
| 类型与名字 | 不成立 | 块号、态序号、入池序号都是裸整数：「pub state_ordinals: std::ops::Range<u64>,」（`crates/singlefs-verification/src/verdict_store.rs:358`）、「pub pool_entry_number: u64,」（`crates/singlefs-verification/src/verdict_store.rs:362`），块号在 `put_the_facts_verifier_pending_row_unless_present` 等处是 `block_number: u64` 参数。逐个调用点看了实参次序，没找到串用的地方；格里问的「各用各的类型」字面不成立 |
| 不兼容旧代码与旧格式 | 不成立 | 打中 C。另：prune-server 三处文档仍写崩溃行表标签到 `0x1B`（`prune-server/src/storage.rs:4`、`prune-server/src/protocol.rs:201`、`prune-server/src/gpu_judging_rows.rs:5`），`0x1F` 加进来之后差得更远；不在被判文件里，`0x1C` 起就已过时 |
| 不新建判定库 | 成立 | 表标签 `0x1F` 在全仓那一份库里（`verdict_store.rs:1805`）；一次性库在写任何一行之前拒（`crates/singlefs-verification/src/crash_amplification.rs:34521-34523`）；读库程序经 `LibraryOpenInThisProcess::open_or_share` 开（`crash_library_reader.rs:706`），文件里没有写库的调用 |
| 所有任务多机可用 | 推不出 | 正文的理由（`research/prompts/_amplification-without-cards-code-r1-body.md@检查单：所有任务多机可用`）：「等服务那一段问的是判定库里服务的在跑标记，两台同一份库」。代码问的不是库里的标记，是起子进程：「let check = match std::process::Command::new("python3")」（`crates/singlefs-verification/src/crash_amplification.rs:33804`），跑多机配置所在目录下的 `gpu-verification-server/control check`，那一份照 systemd 单元判、经 ssh 问第二台。结论（两台都判得对）可能仍成立，要第二台起的任务 ssh 得回点了卡的那一台，这一点没核；写的理由不成立 |
| 测试优先多线程 | 不成立 | 打中 E：观察者盘 CPU 判那一处起的线程不照拿到的条数；份额 6 时两份合 9（正文已认） |
| 带 --item 的命令与项 | 成立（变异行）；放量分项那半句推不出 | `task-server/task log 19325` 末段：「✓ 点名的 44 条变异都抓到了（只判了这几行，不是整道结论；现判、复用、拒算各几条见上面「复用：」那一行）」。放量分项没交；照打中 A，交了在本机配置下是挂，不是「判不完」那么简单 |
| 新崩溃流多罩了什么 | 成立 | `git diff --stat HEAD -- .claude/gate.d/stage-inputs.tsv .claude/gate.d/crash-point-replay-segments.tsv scripts/crash/crash-amplification.sh` 空、退 0：没有新登记的用例、段与分项 |

## 没打中的形状

- 核对待判行：两进程同写同一块、认领着就崩、写完就崩、写回与读回之间删行、删了行而核对行不在时两个判段同时补写、同一计划两个点块键相同、分片挪片前后两条流同块号——都走到「对」或「多一次重判」（第二节表）。取样范围：只读 `verdict_store.rs` 那四个函数与 `crash_amplification.rs` 第 34307-34718 行，服务那一侧没有实现，按协议注释推。
- 复用数：别的版本键、只有观察者结论、只有一部分块有结论、两种判器之间复用——数对（3a）。
- 互等：里层线程嵌套取、主线程持名额而子线程要名额、先锁后设库、读写锁写饿——47 个同时拿两样的函数都没凑出环（4a）。
- panic 正常路径与惰性取卡留半截行（4b、4c）。
- 任务进程取卡：观察者盘那一路还留着照多机配置与服务交的卡在进程里起卡池的代码（`crates/singlefs-verification/src/crash_amplification.rs:33252-33260`），但任务带选卡的变量提交时就被拒，这一路在任务里只会退 78；不算打中，记一笔：正文第 1 条「不在任务进程里取卡，结果里卡的来路恒为 `NoCardNeeded`」要连着第 2 条读，单读第 1 条是宽话。
- 「当成绿」：叫停文件只给 `--stage flow-shape` 用、那一趟不判；入口的「点名项全部可复用放行」放行的是域上限、不跳过真跑；分项驱动 `pending_states=0` 照跑。没有一条路让第 ② 段没跑完而分项退 0 记绿。

## 这条腿自己的限度

- 没写模型、没跑任何东西：打中的是协议与流程上的走法，靠读代码追到行；第一节「卡住判法会不会触发」取决于 `control check` 一次的 CPU，我量不了，交主 agent 跑第 1 条命令。所有退出码与日志样子都是推的。
- 服务那一侧对 `0x1F` 还没有实现，第二节对服务的部分照代码注释与三条公开函数推，服务写出来之后要另判。
- 4a 的环只在验证包 `src/` 与 `tests/` 里按函数扫了两样锁的先后，跨函数传进来的守卫（调用方拿着、被调用方再拿）只抽看了几处。
- 没读 `gpu-verification-server/control` 的 check 全文，只读了文件头那几段。

## 要主 agent 跑的命令

1. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/amplification-without-cards-code-r1-attack -- bash /home/fy5090/.local/share/singlefs/work/amplification-without-cards-code-r1-attack/measure-control-check-cpu.sh`
   - 在哪起：草稿目录 `/home/fy5090/.local/share/singlefs/work/amplification-without-cards-code-r1-attack`（脚本 sha256 `dde30f9ac698e95d6db59758e42b6add6fd2533e3eb09cb5718140b0e5b2acb5`，里面五行字面的 `/usr/bin/time … python3 /home/fy5090/code/singlefs/gpu-verification-server/control check`）。
   - 看哪几行：`task-server/task log <号> --all` 里五行 `CONTROL_CHECK_CPU`。`exit=0`（服务在跑）的那几次，`user_seconds + system_seconds` 不小于 0.3 就说明等待圈单靠问服务就过了卡住判法的阈值，第一节的结局是「挂到四小时或一直挂」；都小于 0.3 时结局是 stalled（233）。`exit` 不是 0 的说明此刻服务没起，这一条要起了服务再跑。
2. `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=崩溃放量入口点名分项 fixed-script-through-rollback; 范围=1 项; 为什么=复现本机配置开 GPU 时第二段写下核对待判行之后无人核、日志不再涨的结局，缩域只看结局不算门禁' task-server/task submit --cwd /home/fy5090/code/singlefs -- bash scripts/crash/crash-amplification.sh --item fixed-script-through-rollback --crash-expand-up-to 8`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`。前提：GPU 批处理服务在跑（第 1 条 `exit=0`）。
   - 看哪几行：`task wait` 末行 `TASK_SERVER_RESULT` 的 `type=` 与 `exit=`（推的：stalled、233，或过了两小时提醒仍在跑）；`task-server/task log <号> --all` 里第 ① 段的 `CRASH_AMPLIFICATION_GPU_TOTAL` 之后再没有新行，没有一行 `CRASH_AMPLIFICATION_WAITING_FOR_THE_FACTS_VERIFIER_ON_THE_GPU_VERIFICATION_SERVER`，没有一行 `CRASH_AMPLIFICATION `。会占着 GPU 服务与一条任务位二十分钟以上，跑不跑由主 agent 定；跑完照 `task-server/task clean <号>`。
3. 该红而此刻不红的单测（写法，交实现员；不是命令）：在 `crash_amplification.rs` 的测试模块里调 `verify_my_blocks_through_the_facts_verifier_pending_rows`，库是一次性库、不起替身服务，`server_not_running_line` 恒答在跑，`between_two_reckonings` 数圈、到 1000 圈就 panic。断言它在有限圈内交回一种点名「服务不收核对待判行」的结局（或至少每隔 `progress_interval` 打一行进度）。此刻它会转到第 1000 圈 panic：红。
4. 交回之前我跑了 `python3 scripts/agents/cite-check.py research/prompts/amplification-without-cards-code-r1-opus-output.md`（文本工具，直接跑），末行贴在交回里。

## 没做什么

- 不判正推那一腿的格，没读禁读清单里的文件；不替主 agent 采纳。
- 没往统一任务服务交任务，第一节、第三节、第四节里的退出码与日志样子都是推的，判决要引的数由主 agent 跑上面第 1、2 条。
- 没建仓副本、没编译；草稿目录里留下的只有 `measure-control-check-cpu.sh`（第 1 条要用）。读 HEAD 那一份 `crash_amplification.rs` 时拷进草稿目录的单个文件与一份扫描清单已删。
- 自己提的改法（打中 A、B、C、D、E 末尾）都只在我的推演上，被攻过零轮。
