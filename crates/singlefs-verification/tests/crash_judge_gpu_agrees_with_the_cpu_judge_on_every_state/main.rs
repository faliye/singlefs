//! 验证包模块：crash、crash_state_enumeration、crash_injection、crash_identity、crash_library、crash_amplification、verdict_store、gpu_unit_checks、crash_facts、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch
// task-server-gpu-cards: all
//! GPU 判器与 CPU 判器逐状态对拍：六条流（新池新建文件；它改坏一次树表单元写两份副本之后的样子；它的 journal 记录提交标记改成 2、
//! 记录标志改成 0b11、系统配置槽距改成 512 三种重封过校验和的坏法；固定脚本到回退 D）上的每个
//! 崩溃状态，GPU 判出的红位（两遍 oracle 各七类、记录核对器两条）要与 CPU `judge_crash_image` 的 `red_items` 逐位相同，池级 checker
//! 每一条不变量的违例位（`IMPLEMENTED_INVARIANTS` 全部；只屏蔽内核不置位、对拍不比的那几条：`crash_judge_gpu::POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS`，
//! 第一版只有下标 92 的 I-1.13（C821），CPU 一侧照判定置位，比较处经 `pool_checker_bits_agree_in_the_cross_check` 两边都去掉再比）
//! 要与 CPU 池级 checker 逐位相同，一个状态都不许判不了。
//! 改坏那条流上 CPU 有红，GPU 要红在同样的状态、同样的项上——这是 GPU 判器分得出差别的自证；里程碑三二·1 那一批的每一条不变量
//! 另有一条只改坏它管的那个字段的流（`the_gpu_judge_agrees_with_the_cpu_when_…` 那一组）；I-3.5 的出现规则红与成立各一条，
//! 恢复施加一条记录取新根段的那几段、施加出来的那一版从记录走实例表与分配记录树、I-8.10 后三段，各有一条同样形态的流；
//! I-9.28 比到候选集之外的根与树表读不出的根、I-7.8 扫描方向的认领规则、I-7.9 两支只取锚在镜像最新根上的一圈窗口里的根，也各有流；
//! 树表 11 条与种类 9–11（谱系树的容器记录宽、谱系树根的树 ID、谱系树与 logged_ops 两条的发号次序）各有一条，树表条数不超过扫描内核的上限取卡之前就判。
//! 里程碑三·3 删除线（I-9.5、I-3.4、I-4.7、I-4.6、I-9.6 墓碑那一半）：删除中间态（unlink 意图与第一批）一条流钉四条判过而成立，
//! 每条各一条只改坏它管的那个字段的流钉 CPU 红、GPU 逐位跟上；走过 nlink = 0 那几版的流与别的流一样整份比（红位、池级 checker、两遍恢复的结局，
//! GPU 恢复内核与 CPU 冷走读一样把 nlink = 0 的第一个文件答成没有文件，C712（GPU 恢复内核不判 nlink = 0））；另一条流删到最后一批，
//! 那一版起 extent 树只剩上段零条目根，同样整份比（两边都答没有文件）；删除中间态那条流上展开上限
//! 不展开的段（新建文件、顺序写、意图、第一批那几次发布的单元写）另把「之前的段全落、这一段只差一个写」的每个状态各比一次。
//! I-9.16 ③ ④（种类重复、缺 deadlist、缺 `deleted_inodes`（I-3.20 已承诺预留那一句两边都记判不了）；另有一条十二种齐、只多一条重复的流只红 ③）、根槽 437..466（I-2.4）、带树一版根记录里的分配记录树根（I-5.4）各一条坏流整份比；指针头部的
//! 墓碑与谱系那九条：I-9.17、I-9.23 各一条改坏容器树根条目身份或类型段的流，I-9.18、I-9.19、I-9.20（谱系上的墓碑不重号且记录合法）两句各一条、I-9.29
//! （连同 I-9.10 墓碑那一个等号）各一条改坏墓碑或记录的流，I-9.24 并在队列 {1, 7} 那一条上，I-9.21 一条墓碑容器旋钮压到一片 1 条的流
//! （旋钮下合法、CPU 判器按旧入口判就红），I-9.4 第四句一条叶里记录对调的流，I-9.12 第三句一条两片叶的 inode 树根右片分隔 key 改成不大于左片最大号的流，删 → 回退 → 再删一条合法流钉九条不红；零条目的 extent 上段根
//! 与 inode 树根条目宽窄于字段表各一条，恢复两边都答没有文件。I-9.22 崩溃点重放恒不适用，没有流。
//! 条目类型段与孩子的真实类不符的四形（攻方 N4-a、N4-i、N4-f、N4-j：层级 1 的墓碑树根条目类型段 0 指码 3 容器，身份三段带着或清零；
//! 旋钮压出来的层级 2 墓碑树根条目类型段 1 指码 2 节点，根环转没转过一圈各一条）钉 CPU 红 I-9.23、不红走读失败连带的那几条，GPU 逐位跟上，
//! inode 树根条目类型段 0 指码 3 叶容器（I-9.2 同形）一条同钉；
//! I-9.29 判不了那一形（被崩溃恢复抛弃的根与现行线同号不同出生代、最新根的实例表两份翻坏）一条流整份比；
//! 同一段历史、最新根的实例表链指针记录之前多一条 kind 2（攻方 N5-a）一条流整份比；那一片解不开（I-3.14 红、
//! 主走读走不到末片、I-9.29 不比），同一段历史再加片中间多一条链指针记录、第一行 flags 非 0 两条流，三形各一条。
//! 同一段历史再加两条，都整份比——第一行 flags 非 0 且 T 抬到最新根的 txg（只读不判的读法拒收这一片，I-3.12 两边都不判它）；
//! 拆成两片、第 0 片链指针盘 0 那条位置条目指回第 0 片自己（攻方 H-b：「走过」按读回那一份的落点认，第 1 片照走、走到末片）。
//! 同一段历史、拆成两片（实例 2 那一行在第 1 片）、第 0 片链指针 MAC 头一个字节写 1（I-3.14 第五形，第 0 片解不开）一条流整份比；
//! 同一形、MAC 不动而链指针的 extent 偏移写 1（第五形 extent 偏移那一段）一条流整份比。
//! inode 树根条目类型段 0、身份三段清零，叶容器盘 0 那一份换成码 2 节点的陈旧字节（攻方 H1：头里的类恰是类型段那一类的副本也要自证）
//! 一条流钉 CPU 红 I-9.2、不红 I-7.2，GPU 逐位跟上。
//! inode 叶容器类标签改成不登记的码、不重封（孩子读不出），树根条目类型段 0 带身份、类型段 7 各一条流整份比；
//! 同一段被崩溃恢复抛弃的历史上，最新根的实例表链指针记录不合法、第 1 片读不出（GPU 关门那一侧）各一条流整份比；
//! 链 256 片一条整份比，链 257 片一条钉 GPU 池级 checker 记判不了（实例表链过长）、回落 CPU 之后结论相同（攻方 H-c）。
//! 孩子两份都读不出时照判「子指针头部的出生树 == 身份引用的出生树」（H-B）——inode 树根条目身份出生树加 1、墓碑树根条目子指针出生树加 1
//! 各一条流钉 CPU 红 I-9.2 / I-9.23；inode 叶容器盘 1 那一份换成码 2 节点的字节、位置条目改成对得上它（H-V：两份都看）一条流钉 CPU 红 I-9.2；
//! 同样换字节而位置条目不改（第二份副本自证不过）一条流钉 CPU 不红 I-9.2；GPU 都逐位跟上。
//! 走读不往下读的孩子每一份副本照判 I-2.1——inode 树根条目类型段 7、子指针两条位置条目的校验和都坏（H-J3a）一条流钉 CPU 红 I-2.1、
//! I-4.8、I-7.2、I-7.4，类型段 0、第一条位置条目的校验和坏（H-J3b）一条流钉 CPU 红 I-2.1，GPU 都逐位跟上；被崩溃恢复抛弃的根自己那张实例表链 257 片
//! （O-1）一条钉 GPU 记判不了（实例表链过长）、回落 CPU 之后结论相同；走读内核与扫描内核的链片数上限相同一条（不用卡）。
//! inode 叶容器两份第一条记录 blocks 加 1、盘 1 那一份再换成码 2 节点的字节（F-W：另一份按类型段那一类自证过，照类型段走它）
//! 一条流钉 CPU 红 I-9.2 与 I-9.15、不红 I-3.1、I-3.11、I-5.1；发布的根与它的记录里实例表指针头部诞生代号、实例、尾段各加 1（F-IT）三条流
//! 钉 CPU 只红 I-1.2；GPU 都逐位跟上。
//! 墓碑树根条目类型段 0、身份清零、两条位置条目校验和都坏（F-L0）、类型段 7 两条都坏、类型段 0 指容器第一条坏、
//! 层级 2 根条目类型段 1 指节点第一条坏（容器树四处 `w_judge_every_copy_of_a_child_the_walk_does_not_go_into`）各一条流；墓碑容器带记录宽违例、
//! 另一份自证成码 2 时类型段 1 与类型段 0 两形（容器树两处 F-W）各一条流，都整份比；inode 树根条目类型段 0、另一份自证成码 2 一条钉 GPU 记判不了
//! （类型段 0 的内部节点）、回落 CPU 之后同判；候选根自己那张实例表链 257 片、最新根的链途经它的一片一条钉 GPU 扫描内核记判不了（站点 37 / 38）、
//! 回落 CPU 之后同判。
//! 墓碑容器带记录宽违例、墓碑树根条目第 0 条位置条目指着先走过的 inode 树根（F-KEY：「走过」按读回那一份的落点认）、
//! 类型段 0 的条目两份副本一份只按前 16 KiB 对得上而头里是码 3、一份是码 2（F-ID-b：只认自证过的副本）、类型段 7 的条目第 1 条位置条目指着流里没写过的槽、
//! 校验和是全零 16 KiB 的 CRC（F-零槽：GPU 读者对没写过的槽读出全零）各一条流，都整份比。
//! 回退之后共享的叶容器诞生代号抬到回退那一版、盘 0 那一份写坏（一个单元的各把键按读回那一份的落点认，allrbmap）、
//! 类型段 7 的条目第 1 条位置条目的槽号加 2³² 与槽 2³² − 1（全零单元守卫按槽号装不装得进 32 位判）、指着没写过而下一槽写过的槽、校验和是 32 KiB
//! 全零的 CRC（F-α 比 32 KiB 时另按 32 KiB 读）、类型段 0 的条目两份都没自证过（当读不出）、删完的流带「没有文件的那几版」（恢复内核的 oracle）、
//! 由记录施加出来的版与下一次挂载先施加的那一版同在（C744 ①）各一条流，都整份比。
//! 由记录造出来的两类版本取带末条标志那条记录的新根段（三条记录的发布第一条写成上一版的根：下一次挂载那一版叠两条记录罩住同一个槽、
//! 挂载之后由记录施加出来的那一版各一条），树表指针整条全零照 core 跟过去（根与它的记录、由记录施加出来的那一版的记录各一条），都整份比。
//! 下一次挂载先施加的那一版照 I-4.1 那一份重放施加的第一次发布认（改法 R1）——三条记录的发布首条没落（断号）、
//! 前缀停在后面那次发布的点名单元上（第一次那一版照认）各一条流，CPU 一侧先在整条落盘的镜像上钉形，再整份比。
//! GPU 恢复内核照 CPU 的 F2（F2_EXACT 那一形）退回见证到的那一版——设计里那一退（R 走读失败、V 已确认，落 V）与攻方 A 形
//! （V 的根槽也读不出，报读不出、不退到 V⁻）各一条流；I-3.5 按全部版本走完之后的折叠表取单元（H-S1-fold A 臂）一条流；read_at 拼读那三形
//! （空段右端一槽按 32 KiB 读的数据指针记判不了回落 CPU、「16 KiB 节点加下一槽」照表判、F-α 的「空槽加下一已写槽前 16 KiB」拿拼出的 CRC 照比）各一条流；
//! 删完那条流钉 CPU 一个违例都没有，C744 ① 那条流钉两种版本同在的状态至少一个；借卡一律经 `lease_card_or_report_not_run`（此刻借不到就排队等这个进程
//! 别的用例还卡，没有可等的才报本次未跑）。
//! GPU 恢复内核的 F2 改成 F2_NEWEST 那一形（退到 [见证到的那一版, 所选根) 里最新、不被所选根实例表抛弃的根）——攻方 H4
//! （V 那次轮换的系统配置槽读回全零，落已确认的 V、不落 V⁻）、所选根的实例表抛弃 V（落 V⁻）、所选根的实例表读不出（只认恰好相等的 V⁻）、
//! c_见证 = 0 而环里有一条计数器 0 的伪造记录各一条流，都整份比；CPU 与 GPU 一起对 c_见证 = 0 设守卫，那一条流两边都不退、报读不出。
//! extent 偏移（I-2.4）在树表条目的树根指针、根记录的实例表指针、inode 内部条目子指针、extent 上段叶内联的数据指针与下段根指针、
//! extent 下段叶的数据指针、分配记录树内部条目子指针上各一条坏流整份比（core 读这几条指针都判、GPU 恢复内核照判）；这几处除实例表指针外
//! 另各有一条 MAC 第 0 字节非 0 的坏流（core 连加密、压缩那几段一并判，GPU 恢复内核同判）；extent 上段叶内联的数据指针另有一条算法类型写 1 的坏流
//! （池级 checker 两个走读内核判算法类型、压缩算法码、压后长度那几段）。墓碑树与谱系树里指向容器的子指针 extent 偏移各一条
//! （墓碑树那条在删除中间态的流上），树表里两条种类 40（≥ 32）一条，都整份比。
//! 要本机一张空闲显存够的 NVIDIA 独显：本机一张 NVIDIA 独显适配器都没有时用卡的用例打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、
//! 返回成功；有卡而用不上（一张都没起来，或 PCI 上有卡而 Vulkan 枚举不出）判红（`gpu_unit_checks::gpu_test_start`）。
//! 建判器、判的时候卡上 wgpu 报了 Out of Memory（显存账只记这个进程借的，别的进程同一刻借走了）同样打那一行报「本次未跑」、原因写显存不够，
//! 不判红（G1 二）。要 `gpu` 特性；
//! 卡经这个进程的卡池取（`gpu_test_start::gpu_card_pool_or_report_not_run`：每张卡一个设备、内核每张卡编一次、一张卡一本显存账，
//! 卡与额度照服务交的 `GPU_CARDS_BY_PRIORITY` 里这台机器那几项，没交卡串当没有卡、报「本次未跑」）：每条用例建好表之后按它的判器实际要的显存借，
//! 照卡的先后装满前一张才用下一张，放不下就等别的用例还，比每张卡的账都大判红（几十条用例并行不再各按空闲显存的 3/4 互抢）；
//! 这个测试二进制里的用例并行起，排队的是卡池那一份账
//! （`gpu_test_start::CardLeaseWaitingRoom`：借不到时等这个进程别的线程还卡或借完再借，别的进程占着的不等，借着卡的线程再借不等）；
//! 不按主机内存排队：测试线程数取登记的每台默认重活线程数（按内存限并发的入口只登记在 `.claude/memory-bounded-parallelism.tsv`，
//! 这个目标不在里面；`.claude/rules/verification.md`「一个门禁：两种跑法」里 GPU 测试线程那一条）；
//! 展开上限 `SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO`（没设取放量引擎的全量域上限 `crash_amplification::EXPAND_UP_TO_WRITES_PER_SEGMENT_OF_THE_AMPLIFICATION`，设了只往小缩）、显存额度 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES`（量速度那条用例当那张卡的额度；卡池里的读法在 `gpu_unit_checks::gpu_test_start::CARD_QUOTA_ENVIRONMENT_VARIABLE`）、
//! CPU 判器线程数 `SINGLEFS_CRASH_JUDGE_CPU_THREADS`（没设取每台默认重活线程数：`SINGLEFS_THREAD_CAP`，再没设取 `.claude/heavy-thread-default` 的登记值）：设了而认不出（线程数 0 也算）就 panic，不退回没设时的那一档。
//! 退役记录那四条 GPU 同判、不记判不了——删完的流（最后一批开出退役树、一条退役记录）上退役记录死亡 txg 加一（I-9.32、I-9.35）、
//! 同一条再追加一条（I-9.33）、死亡 txg 减一（只 I-9.35 漏记），左吸收右的流上退役记录改指幸存的左片（I-9.34、I-9.35）各一条流，都整份比。
//! 三棵容器树的旋钮值编进 GPU 扫描内核（`JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs`），
//! CPU 那一侧池级 checker 的违例位改按带同一组旋钮值的新入口判：墓碑、谱系、退役各一条旋钮下的合法流（I-9.22 / I-9.37 两边判过而成立），
//! 谱系多于旋钮值、谱系两片同号、退役两片同号、谱系跳号（1、3）、谱系从 2 起（2、3）、谱系最右多一片 0 条各一条坏流（CPU 红 I-9.37），都整份比。
//! J3：墓碑、谱系、退役各一条旋钮按 2、2、1、1 逐条换值写出 [2, 1, 1]（中间那片少一条）的流，两边都给那一棵旋钮 2，CPU 红 I-9.22 / I-9.37，整份比；
//! J4：墓碑产品支 [815, 1] 那条流的起点镜像是第 816 个 unlink 之前的整段录制流，写表只录那一次 unlink（从 mkfs 录起时枚举写表超过持久掩码的宽、建不出表）。
//! 崩溃点重放合并（判决 `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` 第三节 J6、J10，
//! `research/prompts/m3-crash-enumeration-merge-r2-main-verification.md` 第二节 L1-4、L1-6、L1-7、L6-2）：每个状态比的是对拍字
//! （`crash_judge_gpu::CrossCheckWords`：红项、两遍各自落到的根、结局、失败点、池级 checker 判到位、判不了）；甲二展开的一条流上
//! GPU 拆出的持久掩码（探针内核 `crash_judge_dispatch::persisted_masks_on_the_card`）与 CPU 判表、crash.rs 枚举逐序号相同、对拍字逐状态相同；
//! 33 次写的一段第 2³² + 5 个状态起 GPU 与 CPU 拆出同一个掩码；给定状态表上 GPU 照清单判、与 CPU 逐状态相同；判表核段长对不上的拒建。
//! E14：I-1.8 第一句在一块盘上有几个数据单元的流上（三个数据单元的文件只改最后一个单元一份副本的载荷）、第二句一条 inode 叶两份副本
//! 只差出生序号（同对象 key、同全序键的两组）的流，隔离豁免（I-3.11 的隔离的记录）豁免与不豁免各一条流，都整份比。
//! harness 场景交出来的镜像上 GPU 建不出表的两形：基线里只剩盘 1 的槽 1 自证得过（判表照 CPU 择系统配置的找法取落点几何）、
//! 池里少一块住着根环区域的盘（判表只在池里的盘上建落点）各一条流，取卡之前在 CPU 上核判表建得出，再整份比（少一块盘那一条上
//! 两边对 I-7.7 都报射程里一部分没判：根环槽读不全）。逐档读槽 1 读到带读者不收之值的槽（I-7.13）两形：槽距处本池的槽 1 照旧、
//! 再远一档那一份带不收之值，与逐档读到的第一个自证得过的槽 1 带不收之值、更远一档有一份读者收的槽 1，各一条流，取卡之前核判表给逐档读到的那几处
//! 建了系统配置落点，再整份比。
//! 里程碑四（D13（验证路线） 已定项 15；判决 m4-snapshot-r1 第三节 S1、S3、S4、S5，m4-gc-r1 第三节 G3）：建快照、快照下覆盖写（deadlist）、
//! 删快照的意图与走空的每一批一条流（CPU 判过 I-3.5、I-3.6、I-3.7 而成立），整理的一批搬第一个文件的数据单元（提示过期）、
//! 搬只被快照钉住的旧数据单元各一条流，新池新建文件之后盘 0 上罩住数据单元那一份的整段 discard（D13 已定项 4 三态）一条流，都整份比。
//! 里程碑四自建 GC 新判的那几条（判决 m4-gc-r1 第三节 G1、G3 的 N2–N6：I-3.15、I-5.6、I-4.4、I-3.16、I-1.14）上 GPU：签发整理意图、整理的一批、
//! 丢意图一条流（logged_ops 树走读、引用、恢复数映射条目，CPU 判过五条而成立），意图补零里塞增量（I-4.4）、c0 改成 0（I-5.6）、
//! 数据单元那条映射条目两条位置改指没人分配的槽（I-3.15）、盘 0 的 runs 那一行少一（I-3.16）、搬到目的地那一份改一个字节（I-1.14）各一条坏流，
//! 都整份比、钉 CPU 红那一条。
//! 里程碑四修复批二（判决 m4-segment-code-r1 第三节 F9 的 GPU 一侧：I-3.5 的用户可见树标签按版本记）：攻方种子下标 2022 缩出的 5 步历史
//! （无快照时回退 → 建快照 → 快照下覆盖写）从 mkfs 之后录起一条流，CPU 在整条落盘的镜像上判 I-3.5 成立，整份比。
//! 里程碑四修复批三（同一节 F9「种子下标 1032 那一形」的 GPU 一侧：快照独有的父单元下面与活头共享的孩子照样挂到它下面）：攻方种子下标 1032
//! 缩出的 16 步历史（建删快照、几次可写挂载、批间崩溃再挂载的整理）从 mkfs 之后录起一条流，CPU 在整条落盘的镜像上判 I-3.5 成立，整份比。
//! 截断那一批（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K5、K7 与第六节 S1、S2：I-4.6 改成每条记录都判、新立的 I-4.10 与 I-9.38、
//! I-4.4 收截断条目、上段叶条目标签 3、4）：三个单元的文件缩到单元中间、变长出洞（标签 3）、再缩、再变长出洞、缩到 0 的一条合法流（CPU 判过四条而成立），
//! 截断条目补零塞增量（I-4.4）、截断条目的对象出生代加一（I-4.10）、size 之内的单元声明长度多一（I-9.38）、下段叶一条记录的偏移跳过一个净荷（I-4.6 连号）、
//! 标签 3 改写成 1（I-4.6 单元数对标签）各一条坏流，钉 CPU 红那一条。这几条流都整份比（红位、池级 checker、两遍恢复的结局与落到的根）：
//! GPU 恢复内核照 core 读截断中间态与洞（判决 K2、K5 的读者那一半与第六节 S5：上段叶条目收标签 3、4，标签 1、2 的单元不少于 size 要的个数、
//! size 之外挂着的不读，标签 3、4 之后到 size 读 0、按内核现算的全零载荷区 CRC 与期望那一版逐单元比）；变长出洞那一版的期望内容在洞里塞一个
//! 非零字节一条流，钉 CPU 红「内容不同」、GPU 逐位跟上。
//! 截断（inode 记录偏移 120 起的截断字段、
//! I-9.7 拆段、新立的 I-9.39、incompat 位 2）：三个单元的文件那一版（或新池新建文件、截断意图那一版）第一个文件的记录按字节改、系统配置的
//! incompat 第一个字节按字节改，各一条流：I-9.7 三种坏法（截断字段之前、之后的预留段非零，T = 0 而 N_w ≠ 0）、I-9.39 的必红世界
//! （截断字段之外、size 之内的活单元出生早于 T），合法世界（单元恰在 txg T 写下、截到 0 留下单元 0、截断意图那一版 size 之外还挂着的单元、
//! 每一槽都是这一版布局的 0x06），系统配置 0x02、0x04、0x0E 三种不认得的第一个字节（位 2 是这一版布局身份的一部分）。
//! 池级 checker 那一半先比、钉 CPU 红哪几条，再比两遍恢复。
//! 里程碑三收尾（C663、C728）：系统配置槽 S 写 17、R 写 2 各一条流钉 CPU 红 I-7.13（系统配置池级字段在读者收的范围里）：R、S 越界整池拒，不再是这一槽不可择，
//! 系统配置槽末字节（偏移 4095）、根记录之后头一个字节（偏移 466）各写 1 的两条流钉 CPU 红 I-2.4（两段第一版恒 0），GPU 都逐位跟上；
//! 流里每一次系统配置槽写 incompat 第一个字节写 0x0E、格式版本写 2 的一条流钉格式版本先于 incompat 位判（C663「格式版本先」）：
//! 两遍恢复都按格式版本整池拒、没有一个状态报 incompat 位不认得，GPU 恢复内核逐态跟上。
//! 里程碑三收尾（F6 与两槽全废盘面观察者）：GPU 恢复内核判退看冷走读、只读挂载的打开、
//! 建上一版连同实例表三样读者的并集（C755）——两次覆盖写的流上 R 的数据单元没落而建上一版拒收 R 的记录、R 的 extent 树根没落而打开拒收 R 的 inode 记录
//! 各一条流，CPU 都不退、GPU 逐态跟上；走读内核交逐盘自证过的系统配置槽数，一段里盘 0 两槽撕裂清零的流上逐状态逐盘与池级 checker 的逐盘数相同。
//! 种违例位那条用例（一条一条跑时按名字排在用卡的用例里头一个，冷编判器内核落在它身上；并行起时没种的那一套落在先要它的用例上，
//! 同一张卡上同一套源码只编一次，别的用例等它编完）要两套内核源码（没种的与种过的），两套各建判器、判同一条流，
//! 按区间切片分给工作线程（`crash_injection::seed_slices`，同崩溃注入读故障用例的切法），两套的冷编叠在一起、不再一套编完才编下一套；
//! 工作线程数取 CPU 判器线程数那一个变量（没设取每台默认重活线程数），编内核一套一套编的变量写了 `1` 时只起 1 个；合并按片号放回两套的次序，
//! 判定与线程数无关（用例 `units_judged_in_slices_on_several_worker_threads_and_on_one_come_back_identical_and_in_unit_order` 对拍）。
//! 对拍按块：每条流按 `crash_library::STATES_PER_PAIRING_BLOCK` 切块，块键（这一次对拍的输入摘要拼上
//! 块的序号段）与两边判器组的单元键一起进 prune-server 那一份库里配对结论那一张表（`crash_library::pairing_conclusions_of_the_blocks_reused_or_judged`）；
//! 块键与单元键都相同的下一趟复用、不现判；没有的攒成一段两边都判、逐块比完存进库就丢，手里只压着一段的逐态结论，与整条流的状态数无关
//! 一条流的几段块在各自的线程上同时判（[`stretches_of_a_flow_judged_at_once_on_these_cards`]），借到哪张卡照卡池的账，这个进程分到几张卡就用上几张；
//! 文件第二行的 `task-server-gpu-cards` 声明要 `all`：张数由服务按此刻空卡现算，至少一张。
//! CPU 判完的逐态结论压在宿主内存里等借卡，判 CPU 那一段之前先按状态数排队（[`PairingStatesHeldOnTheHost`]），压着的状态数有上界
//! （[`most_pairing_states_held_on_the_host_of`]），不随并行的用例数涨。对拍判器的草稿区按卡的额度开（[`states_per_dispatch_of_a_pairing_judge`]），
//! 一次派活实际带几个由判器在卡上现量。几台分着判照 [`PairingBlocksShare`]（与崩溃放量同一对份额变量）：非整份的一趟只把自己领的块判完入库，不判用例钉的数；
//! 合并那一趟（整份）每块复用、照钉的数判。
//! 一条流状态多到一条用例判不完的，按状态序号区间拆成几条用例（[`compare_on_every_state_with_ordinals_in`]：区间两头落在对拍分组的边界上，
//! 块键与整条流跑时逐块相同），每条只领自己那一段序号的组（`PairingBlocksShare::only_the_groups_of_pairing_blocks_in`，与几台分份叠用），
//! 断言这一段每个状态两边逐位相同；只在含整条流某个状态才成立的断言放在含那个状态的那一条上。
//! （全量域上十亿态以上的流不再把逐态结论整份留在宿主内存里：CPU 那一侧按状态数预留一次约 300 GB 的分配会 SIGABRT）。
//! 要看逐状态结论的用例交逐状态另看的几样（`StateExamination`），对拍时每个状态上看、按块累加，结果随块一起入库复用。
//! 新池新建文件、固定脚本到回退与它们改坏一处的那十七条流一条流一个用例（按功能场景拆），
//! 每个用例测多少态由它那条流在全量域上自己定。
#![cfg(feature = "gpu")]

#[path = "../common/mod.rs"]
mod common;
#[path = "../common_crash_points/mod.rs"]
mod common_crash_points;
#[path = "../common_deletion_in_progress/mod.rs"]
mod common_deletion_in_progress;
#[path = "../common_pointer_head_corruption/mod.rs"]
mod common_pointer_head_corruption;
#[path = "../common_truncation/mod.rs"]
mod common_truncation;

use singlefs_verification::crash_amplification::{
    record_the_flow_and_read_it_back, EXPAND_UP_TO_WRITES_PER_SEGMENT_OF_THE_AMPLIFICATION,
};
use singlefs_verification::crash_injection::seed_slices;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use common::{
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool, file_content,
    geometry, overwrite_in_process_as_instance_one_and_advance_the_pool, parameters,
    publish_overwrite_in_process, BuiltPool, FIXED_WRITE_TIME_SECONDS,
};
use common_crash_points::{mount_writable_step, overwrite_step, record_pool_build, roll_back_step};
use common_deletion_in_progress::{
    content_of_three_data_units, index_node_entries_start, index_node_entry_count_offset,
    pool_with_a_three_unit_file, pool_with_new_inodes, publish_the_intent,
    publish_the_next_step_in_batches_of_one, roll_back_to,
    turn_the_root_ring_over_by_creating_inodes, unlink_to_the_end, ACCOUNTING_KEY_BYTES,
    ACCOUNTING_ROW_BYTES, ACCOUNTING_ROW_DEVICE_OFFSET, ACCOUNTING_ROW_VALUE_OFFSET,
    CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET, DELETED_INODES_KEY_BYTES, FIRST_FILE,
    INDEX_NODE_SMALLEST_KEY_OFFSET, INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES,
    INODE_RECORD_BLOCKS_OFFSET, INODE_RECORD_BYTES, INODE_RECORD_NLINK_OFFSET,
    INODE_RECORD_SIZE_OFFSET, INSTANCE_OF_THE_PUBLISHES, PACKED_UNIT_RECORDS_START,
    PACKED_UNIT_RECORD_COUNT_OFFSET, PACKED_UNIT_RECORD_WIDTH_OFFSET, RETIREMENT_RECORD_BYTES,
    RETIREMENT_RECORD_CONTAINER_OFFSET, RETIREMENT_RECORD_DEATH_TXG_OFFSET,
    STATISTIC_NO_DEVICE_DIMENSION, STATISTIC_PENDING_DELETION_OCCUPANCY, TOMBSTONE_RECORD_BYTES,
    TOMBSTONE_RECORD_DEATH_TXG_OFFSET, TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET,
    TREE_TABLE_ENTRY_KIND_OFFSET, UNIT_DECLARED_LENGTH_OFFSET,
    WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
};
use singlefs_checker::image::{InvariantVerdict, IMPLEMENTED_INVARIANTS};
use singlefs_checker::walk::{
    check_pool_image, check_pool_image_given_the_container_capacity_knobs, ContainerCapacityKnobs,
    LineageContainerCapacityKnob, RetirementContainerCapacityKnob, TombstoneContainerCapacityKnob,
};
use singlefs_core::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, InodeNumber, InstanceGeneration,
    SlotNumber, TreeIdentifier,
};
use singlefs_core::allocator::Placement;
use singlefs_core::compaction_plan::PlannedRelocation;
use singlefs_core::container_tree::{
    ContainerTreeCapacities, ContainerTreeNodeEntries, ContainerTreeNodeEntriesCappedForTests,
};
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::logged_operations::{CompactionIntent, CompactionRegion};
use singlefs_core::mount::RollbackTarget;
use singlefs_core::records::{
    SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION, TREE_KIND_DELETED_INODES, TREE_KIND_SNAPSHOT_INODE_ROOT,
};
use singlefs_core::recovery::RecoveryOutcome;
use singlefs_core::tombstone_tree::{
    TombstoneContainerCapacity, TombstoneRecordsPerContainerCappedForTests,
};
use singlefs_core::transaction::{
    central_mapping_locations_in_the_version, publish_compaction_step, publish_create_snapshot,
    publish_overwrite, publish_relocation_batch, publish_snapshot_destruction_intent,
    publish_unlink_intent, publish_unlink_steps_until_the_queue_is_empty,
    walk_the_deletion_queue_until_it_is_empty, CompactionIntentStep, FirstFile, MappingLookup,
    PoolWriter, RelocatedUnitHolder, TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::UNIT_CLASS_DATA;
use singlefs_core::unlink::UnlinkBatchSize;
use singlefs_verification::crash::{
    crash_point_replay_plan_state_count, CrashPointReplayOracleViolationKind,
    CrashPointReplaySegmentExpansion, CrashPointReplayVersionTable, CrashStateJudgement,
    CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE, CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER,
};
use singlefs_verification::crash_amplification::{
    gpu_recovery_words, gpu_verdict_items, judge_my_blocks_on_gpu_cards, plan_crash_points,
    record_crash_points, recovery_words_of, unjudged_blocks, CrashFlow, CrashPointRecorder,
    CrashPointSpan, RecordedGivenStatesTable, RecoveryRootAndOutcomesOfState,
    RECOVERY_ROOT_AND_OUTCOMES_WORDS,
};
use singlefs_verification::crash_identity::CoverageReport;
use singlefs_verification::crash_identity::SectorImage;
use singlefs_verification::crash_judge_conclusions::{
    devices_without_a_self_verified_system_configuration_slot_against_the_base_image_from_the_slot_counts,
    self_verified_system_configuration_slots_of_each_device,
};
use singlefs_verification::crash_judge_dispatch::{
    mebibytes_a_judge_needs, states_per_scratch_lane, GpuCrashJudge, PackedJudgeTables,
    PackedJudgeTablesReadBack, ACCOUNTING_SLICE_POSITIONS, CHECKER_ACCOUNTING_DECIDE_ENTRY_POINT,
    CHECKER_ACCOUNTING_FINAL_ENTRY_POINT, CHECKER_ACCOUNTING_HEAD_ENTRY_POINT,
    CHECKER_ACCOUNTING_SLICE_ENTRY_POINT,
    CHECKER_SCAN_ALLOCATION_GENERATIONS_THROUGH_THE_ROLLBACK_FLOOR_RAISES_ENTRY_POINT,
    CHECKER_SCAN_ALLOCATION_RECORDS_DISJOINT_ENTRY_POINT,
    GPU_KERNELS_ONE_AT_A_TIME_ENVIRONMENT_VARIABLE,
};
use singlefs_verification::crash_judge_gpu::{
    cross_check_words_agree_in_the_cross_check, cross_check_words_of_cpu_judgement,
    cross_check_words_of_gpu_verdict, pool_checker_bits_agree_in_the_cross_check,
    pool_checker_judged_bits, pool_checker_violated_and_judged_bits, pool_checker_violated_bits,
    self_verified_system_configuration_slots_of_each_device_of_the_gpu_verdict, verdict_bits,
    CrossCheckWords, JudgeKernelSources, CHECKER_REFERENCES_AND_SNAPSHOTS_ENTRY_POINT,
    CHECKER_SCAN_REBUILD_ENTRY_POINT, CONSULTED_PASS_FAILURE_POINT_WORD,
    IGNORED_PASS_FAILURE_POINT_WORD, KERNEL_ENTRY_POINT,
    POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS, POOL_CHECKER_JUDGED_FIRST_WORD,
    POOL_CHECKER_JUDGED_WORDS, POOL_CHECKER_REACHED_WORD, POOL_CHECKER_UNDECIDABLE_WORD,
    POOL_CHECKER_VIOLATED_BIT_CAPACITY, POOL_CHECKER_VIOLATED_FIRST_WORD,
    POOL_CHECKER_VIOLATED_WORDS, SCAN_REBUILD_OUTCOMES, SCAN_REBUILD_OUTCOME_WORD, VERDICT_WORDS,
};
use singlefs_verification::crash_judge_tables::{
    build_judge_tables, build_judge_tables_against_the_version_table, JudgeFlow, JudgeLocationKind,
    JudgeTables,
};
use singlefs_verification::crash_library::{
    groups_of_pairing_blocks_holding_the_states, judge_tables_through_the_pairing_library,
    pairing_conclusions_of_the_blocks_reused_or_judged, CrashLibrary, CrashLibraryClass,
    PairingAgreement, PairingJudged, StageOutcome, STATES_PER_PAIRING_BLOCK,
};
use singlefs_verification::crash_point_replay_provenance::DevicesWithoutASelfVerifiedSystemConfigurationSlotAgainstTheBaseImage;
use singlefs_verification::crash_state_enumeration::{
    sample_crash_states_on_the_cpu,
    sample_crash_states_on_the_cpu_with_threads_against_the_version_table, CrashPointReplayItem,
};
use singlefs_verification::gpu_memory_ledger::{
    reason_for_running_out_of_gpu_memory, GPU_NOT_RUN_MARKER,
};
use singlefs_verification::gpu_unit_checks::gpu_test_start::{
    cpu_reference_table_read_back_from_the_pairing_library, first_usable_card_or_report_not_run,
    gpu_card_pool_or_report_not_run,
    image_with_every_write_landed_read_back_from_the_pairing_library, not_run_line,
    usable_cards_or_report_not_run, GpuCardPoolStartedAtTheFirstLease,
    CARD_QUOTA_ENVIRONMENT_VARIABLE,
};
use singlefs_verification::gpu_unit_checks::GpuCard;
use singlefs_verification::heavy_threads::{
    own_variable_or_default_heavy_threads, registered_heavy_threads,
    THREAD_CAP_ENVIRONMENT_VARIABLE,
};
use singlefs_verification::hexadecimal::hexadecimal_text;
use singlefs_verification::history::{
    execute_history_with, CompactionChoice, CompactionRunEnding, ContentChoice, ContentLength,
    GeneratedHistory, HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed,
    HistoryStartingPoint, RollbackTargetChoice, SnapshotChoice,
    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_verification::memory_pool::{
    writes_and_segments_with_stream_indexes, CrashImage, MemoryPool, PublishedVersion,
    PublishedVersionWithoutAFile, RetainedWrite, WrittenContents,
};
use singlefs_verification::numeric_environment_variable::{
    number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
};
use singlefs_verification::segments::StepKind;
use singlefs_verification::sha256::{sha256_digest, DIGEST_BYTES};
use singlefs_verification::verdict_store::JudgingUnitsFingerprint;
use singlefs_verification::SharedStream;

// 对拍判器的宽度、份额各一行 `use`，不并进上面几行：比法那一组判法单元的闭包连着引进它用到的名字的 `use`，那几行的字变了比法的键跟着变，已存的对拍块全部重判
use singlefs_verification::crash_judge_dispatch::MAXIMUM_SCRATCH_LANES;
use singlefs_verification::crash_library::PairingBlocksShare;
use singlefs_verification::gpu_memory_ledger::most_states_whose_need_fits_the_capacity;

mod accounting_verdicts_across_slicings;
mod children_not_walked_into_and_their_second_copies;
mod comparison_on_every_state_and_room_on_the_host;
mod container_capacity_knobs_in_the_scan_kernel;
mod container_tree_children_not_walked_into;
mod deletion_in_progress;
mod environment_settings_and_cards_running_out_of_memory;
mod every_state_of_the_six_flows_and_their_resealed_fields;
mod expected_bits_and_examined_states;
mod fallback_on_the_union_of_the_readers;
mod first_publish_replayed_by_the_next_mount;
mod flows_breaking_one_field_per_invariant;
mod journal_groups_and_last_record_flags;
mod kernel_sources_and_planted_bits;
mod last_pairing_group_and_unrecognized_incompat_bytes;
mod pairing_blocks_judged_on_both_sides;
mod pairing_judges_on_the_cards;
mod pointer_head_extent_offsets_and_codecs;
mod recorded_flows_and_resealed_corruptions;
mod recovery_kernel_falling_back_to_a_witnessed_version;
mod reference_intervals_after_rollbacks_and_snapshots;
mod retirement_records_and_an_unlink_rolled_back;
mod root_ring_candidates_and_floor_raise_windows;
mod roots_abandoned_by_a_crash_recovery_and_instance_table_chains;
mod snapshots_relocations_discards_and_compactions;
mod system_configuration_slots_found_by_the_judge_tables;
mod tombstone_lineage_and_inode_tree_entries;
mod tree_kinds_and_root_record_bytes;
mod tree_table_entries_and_lineage_kinds;
mod truncations_holes_and_the_truncation_field;
mod unit_copies_and_quarantine_marks;
mod versions_applied_by_their_records;
mod walked_locations_verified_copies_and_unwritten_slots;

use children_not_walked_into_and_their_second_copies::*;
use comparison_on_every_state_and_room_on_the_host::*;
use container_capacity_knobs_in_the_scan_kernel::*;
use container_tree_children_not_walked_into::*;
use deletion_in_progress::*;
use environment_settings_and_cards_running_out_of_memory::*;
use expected_bits_and_examined_states::*;
use fallback_on_the_union_of_the_readers::*;
use flows_breaking_one_field_per_invariant::*;
use kernel_sources_and_planted_bits::*;
use pairing_blocks_judged_on_both_sides::*;
use pointer_head_extent_offsets_and_codecs::*;
use recorded_flows_and_resealed_corruptions::*;
use recovery_kernel_falling_back_to_a_witnessed_version::*;
use reference_intervals_after_rollbacks_and_snapshots::*;
use retirement_records_and_an_unlink_rolled_back::*;
use root_ring_candidates_and_floor_raise_windows::*;
use roots_abandoned_by_a_crash_recovery_and_instance_table_chains::*;
use snapshots_relocations_discards_and_compactions::*;
use system_configuration_slots_found_by_the_judge_tables::*;
use tombstone_lineage_and_inode_tree_entries::*;
use tree_kinds_and_root_record_bytes::*;
use tree_table_entries_and_lineage_kinds::*;
use truncations_holes_and_the_truncation_field::*;
use unit_copies_and_quarantine_marks::*;
use versions_applied_by_their_records::*;
use walked_locations_verified_copies_and_unwritten_slots::*;

/// 实例表链 257 片那三条流上恢复内核许交 CPU 的原因：恢复读到链 257 片的那条根的状态上，恢复内核沿链读满 256 片还有下一片，
/// 记 [`GPU_RECOVERY_UNDECIDABLE_TOO_DEEP`]、交 CPU；别的原因照旧算不一致。同造法、链 256 片的流不用它（每个状态两个内核都要判得了）。
const GPU_RECOVERY_REASONS_LEFT_TO_THE_CPU_ON_AN_INSTANCE_TABLE_CHAIN_OF_257_PAGES: &[u32] =
    &[GPU_RECOVERY_UNDECIDABLE_TOO_DEEP];
