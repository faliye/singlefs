//! checker 档模块：crash、crash_state_enumeration、crash_identity、crash_library、crash_amplification、verdict_store、gpu_unit_checks、crash_facts、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch
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
//! I-9.16 ③ ④（种类重复、缺 deadlist）、根槽 428..457（I-2.4）、带树一版根记录里的分配记录树根（I-5.4）各一条坏流整份比；指针头部的
//! 墓碑与谱系那九条（第 23 批）：I-9.17、I-9.23 各一条改坏容器树根条目身份或类型段的流，I-9.18、I-9.19、I-9.20（两句各一条）、I-9.29
//! （连同 I-9.10 墓碑那一个等号）各一条改坏墓碑或记录的流，I-9.24 并在队列 {1, 7} 那一条上，I-9.21 一条墓碑容器旋钮压到一片 1 条的流
//! （旋钮下合法、CPU 判器按旧入口判就红），I-9.4 第四句一条叶里记录对调的流，删 → 回退 → 再删一条合法流钉九条不红；零条目的 extent 上段根
//! 与 inode 树根条目宽窄于字段表各一条，恢复两边都答没有文件。I-9.22 崩溃点重放恒不适用，没有流。
//! 第 24 批：条目类型段与孩子的真实类不符的四形（攻方 N4-a、N4-i、N4-f、N4-j：层级 1 的墓碑树根条目类型段 0 指码 3 容器，身份三段带着或清零；
//! 旋钮压出来的层级 2 墓碑树根条目类型段 1 指码 2 节点，根环转没转过一圈各一条）钉 CPU 红 I-9.23、不红走读失败连带的那几条，GPU 逐位跟上，
//! inode 树根条目类型段 0 指码 3 叶容器（I-9.2 同形）一条同钉；
//! I-9.29 判不了那一形（被崩溃恢复抛弃的根与现行线同号不同出生代、最新根的实例表两份翻坏）一条流整份比；
//! 第 25 批：同一段历史、最新根的实例表链指针记录之前多一条 kind 2（攻方 N5-a）一条流整份比；第 10k 批起那一片解不开（I-3.14 红、
//! 主走读走不到末片、I-9.29 不比），同一段历史再加片中间多一条链指针记录、第一行 flags 非 0 两条流，三形各一条。
//! 第 10n 批：同一段历史再加两条，都整份比——第一行 flags 非 0 且 T 抬到最新根的 txg（只读不判的读法拒收这一片，I-3.12 两边都不判它）；
//! 拆成两片、第 0 片链指针盘 0 那条位置条目指回第 0 片自己（攻方 H-b：「走过」按读回那一份的落点认，第 1 片照走、走到末片）。
//! 第 10u 批：同一段历史、拆成两片（实例 2 那一行在第 1 片）、第 0 片链指针 MAC 头一个字节写 1（I-3.14 第五形，第 0 片解不开）一条流整份比；
//! 第 10x 批：同一形、MAC 不动而链指针的 extent 偏移写 1（第五形 extent 偏移那一段，第 10t 批代码轮判决 K4-3）一条流整份比。
//! 第 26 批：inode 树根条目类型段 0、身份三段清零，叶容器盘 0 那一份换成码 2 节点的陈旧字节（攻方 H1：头里的类恰是类型段那一类的副本也要自证）
//! 一条流钉 CPU 红 I-9.2、不红 I-7.2，GPU 逐位跟上。
//! 第 27 批：inode 叶容器类标签改成不登记的码、不重封（孩子读不出），树根条目类型段 0 带身份、类型段 7 各一条流整份比；
//! 同一段被崩溃恢复抛弃的历史上，最新根的实例表链指针记录不合法、第 1 片读不出（GPU 关门那一侧）各一条流整份比；
//! 链 256 片一条整份比，链 257 片一条钉 GPU 池级 checker 记判不了（实例表链过长）、回落 CPU 之后结论相同（攻方 H-c）。
//! 第 28 批：孩子两份都读不出时照判「子指针头部的出生树 == 身份引用的出生树」（H-B）——inode 树根条目身份出生树加 1、墓碑树根条目子指针出生树加 1
//! 各一条流钉 CPU 红 I-9.2 / I-9.23；inode 叶容器盘 1 那一份换成码 2 节点的字节、位置条目改成对得上它（H-V：两份都看）一条流钉 CPU 红 I-9.2；
//! 同样换字节而位置条目不改（第二份副本自证不过）一条流钉 CPU 不红 I-9.2；GPU 都逐位跟上。
//! 第 29 批：走读不往下读的孩子每一份副本照判 I-2.1——inode 树根条目类型段 7、子指针两条位置条目的校验和都坏（H-J3a）一条流钉 CPU 红 I-2.1、
//! I-4.8、I-7.2、I-7.4，类型段 0、第一条位置条目的校验和坏（H-J3b）一条流钉 CPU 红 I-2.1，GPU 都逐位跟上；被崩溃恢复抛弃的根自己那张实例表链 257 片
//! （O-1）一条钉 GPU 记判不了（实例表链过长）、回落 CPU 之后结论相同；走读内核与扫描内核的链片数上限相同一条（不用卡）。
//! 第 30 批（删除线）：inode 叶容器两份第一条记录 blocks 加 1、盘 1 那一份再换成码 2 节点的字节（F-W：另一份按类型段那一类自证过，照类型段走它）
//! 一条流钉 CPU 红 I-9.2 与 I-9.15、不红 I-3.1、I-3.11、I-5.1；发布的根与它的记录里实例表指针头部诞生代号、实例、尾段各加 1（F-IT）三条流
//! 钉 CPU 只红 I-1.2；GPU 都逐位跟上。
//! 第 31 批（删除线）：墓碑树根条目类型段 0、身份清零、两条位置条目校验和都坏（F-L0）、类型段 7 两条都坏、类型段 0 指容器第一条坏、
//! 层级 2 根条目类型段 1 指节点第一条坏（容器树四处 `w_judge_every_copy_of_a_child_the_walk_does_not_go_into`）各一条流；墓碑容器带记录宽违例、
//! 另一份自证成码 2 时类型段 1 与类型段 0 两形（容器树两处 F-W）各一条流，都整份比；inode 树根条目类型段 0、另一份自证成码 2 一条钉 GPU 记判不了
//! （类型段 0 的内部节点）、回落 CPU 之后同判；候选根自己那张实例表链 257 片、最新根的链途经它的一片一条钉 GPU 扫描内核记判不了（站点 37 / 38）、
//! 回落 CPU 之后同判。
//! 第 32 批（删除线）：墓碑容器带记录宽违例、墓碑树根条目第 0 条位置条目指着先走过的 inode 树根（F-KEY：「走过」按读回那一份的落点认）、
//! 类型段 0 的条目两份副本一份只按前 16 KiB 对得上而头里是码 3、一份是码 2（F-ID-b：只认自证过的副本）、类型段 7 的条目第 1 条位置条目指着流里没写过的槽、
//! 校验和是全零 16 KiB 的 CRC（F-零槽：GPU 读者对没写过的槽读出全零）各一条流，都整份比。
//! 第 33 批（删除线）：回退之后共享的叶容器诞生代号抬到回退那一版、盘 0 那一份写坏（一个单元的各把键按读回那一份的落点认，allrbmap）、
//! 类型段 7 的条目第 1 条位置条目的槽号加 2³² 与槽 2³² − 1（全零单元守卫按槽号装不装得进 32 位判）、指着没写过而下一槽写过的槽、校验和是 32 KiB
//! 全零的 CRC（F-α 比 32 KiB 时另按 32 KiB 读）、类型段 0 的条目两份都没自证过（当读不出）、删完的流带「没有文件的那几版」（恢复内核的 oracle）、
//! 由记录施加出来的版与下一次挂载先施加的那一版同在（C744 ①）各一条流，都整份比。
//! 第 35 批：由记录造出来的两类版本取带末条标志那条记录的新根段（三条记录的发布第一条写成上一版的根：下一次挂载那一版叠两条记录罩住同一个槽、
//! 挂载之后由记录施加出来的那一版各一条），树表指针整条全零照 core 跟过去（根与它的记录、由记录施加出来的那一版的记录各一条），都整份比。
//! 第 36 批：下一次挂载先施加的那一版照 I-4.1 那一份重放施加的第一次发布认（改法 R1）——三条记录的发布首条没落（断号）、
//! 前缀停在后面那次发布的点名单元上（第一次那一版照认）各一条流，CPU 一侧先在整条落盘的镜像上钉形，再整份比。
//! 第 36 批（删除线）：GPU 恢复内核照 CPU 的 F2（F2_EXACT 那一形）退回见证到的那一版——设计里那一退（R 走读失败、V 已确认，落 V）与攻方 A 形
//! （V 的根槽也读不出，报读不出、不退到 V⁻）各一条流；I-3.5 按全部版本走完之后的折叠表取单元（H-S1-fold A 臂）一条流；read_at 拼读那三形
//! （空段右端一槽按 32 KiB 读的数据指针记判不了回落 CPU、「16 KiB 节点加下一槽」照表判、F-α 的「空槽加下一已写槽前 16 KiB」拿拼出的 CRC 照比）各一条流；
//! 删完那条流钉 CPU 一个违例都没有，C744 ① 那条流钉两种版本同在的状态至少一个；借卡一律经 `lease_card_or_report_not_run`（借不到报本次未跑）。
//! 第 39 批（删除线）：GPU 恢复内核的 F2 改成 F2_NEWEST 那一形（退到 [见证到的那一版, 所选根) 里最新、不被所选根实例表抛弃的根）——攻方 H4
//! （V 那次轮换的系统配置槽读回全零，落已确认的 V、不落 V⁻）、所选根的实例表抛弃 V（落 V⁻）、所选根的实例表读不出（只认恰好相等的 V⁻）、
//! c_见证 = 0 而环里有一条计数器 0 的伪造记录各一条流，都整份比；第 41 批（删除线）CPU 与 GPU 一起对 c_见证 = 0 设守卫，那一条流改成两边都不退、报读不出。
//! extent 偏移（I-2.4）在树表条目的树根指针、根记录的实例表指针、inode 内部条目子指针、extent 上段叶内联的数据指针与下段根指针、
//! extent 下段叶的数据指针、分配记录树内部条目子指针上各一条坏流整份比（core 读这几条指针都判、GPU 恢复内核照判）；这几处除实例表指针外
//! 另各有一条 MAC 第 0 字节非 0 的坏流（core 连加密、压缩那几段一并判，GPU 恢复内核同判）；extent 上段叶内联的数据指针另有一条算法类型写 1 的坏流
//! （池级 checker 两个走读内核判算法类型、压缩算法码、压后长度那几段）。墓碑树与谱系树里指向容器的子指针 extent 偏移各一条
//! （墓碑树那条在删除中间态的流上），树表里两条种类 40（≥ 32）一条，都整份比。
//! 要本机一张空闲显存够的 NVIDIA 独显：本机一张 NVIDIA 独显适配器都没有时用卡的用例打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、
//! 返回成功；有卡而用不上（一张都没起来，或 PCI 上有卡而 Vulkan 枚举不出）判红（`gpu_unit_checks::gpu_test_start`）。
//! 建判器、判的时候卡上 wgpu 报了 Out of Memory（显存账只记这个进程借的，别的进程同一刻借走了）同样打那一行报「本次未跑」、原因写显存不够，
//! 不判红（根环线第 10x 批，用户定的 G1 二）。要 `verdict-store,gpu` 特性；
//! 卡经这个进程的卡池取（`gpu_test_start::gpu_card_pool_or_report_not_run`：每张卡一个设备、内核每张卡编一次、一张卡一本显存账，
//! 卡与额度照 `GPU_CARDS_BY_PRIORITY` 里这台机器那几项，没配就是本机能用的卡）：每条用例建好表之后按它的判器实际要的显存借，
//! 照卡的先后装满前一张才用下一张，放不下就等别的用例还，比每张卡的账都大判红（几十条用例并行不再各按空闲显存的 3/4 互抢）；
//! 展开上限 `SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO`（默认 8）、显存额度 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES`（量速度那条用例当那张卡的额度；卡池里的读法在 `gpu_unit_checks::gpu_test_start::CARD_QUOTA_ENVIRONMENT_VARIABLE`）、
//! CPU 判器线程数 `SINGLEFS_CRASH_JUDGE_CPU_THREADS`（没设取每台默认重活线程数：`SINGLEFS_THREAD_CAP`，再没设取 `.claude/heavy-thread-default` 的登记值）：设了而认不出（线程数 0 也算）就 panic，不退回没设时的那一档。
//! 第 50 批（退役树）：退役记录那四条 GPU 同判、不记判不了——删完的流（最后一批开出退役树、一条退役记录）上退役记录死亡 txg 加一（I-9.32、I-9.35）、
//! 同一条再追加一条（I-9.33）、死亡 txg 减一（只 I-9.35 漏记），左吸收右的流上退役记录改指幸存的左片（I-9.34、I-9.35）各一条流，都整份比。
//! 第 56 批（G2、G3、G6）：三棵容器树的旋钮值编进 GPU 扫描内核（`JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs`），
//! CPU 那一侧池级 checker 的违例位改按带同一组旋钮值的新入口判：墓碑、谱系、退役各一条旋钮下的合法流（I-9.22 / I-9.37 两边判过而成立），
//! 谱系多于旋钮值、谱系两片同号、退役两片同号、谱系跳号（1、3）、谱系从 2 起（2、3）、谱系最右多一片 0 条各一条坏流（CPU 红 I-9.37），都整份比。
//! 第 58 批 J3：墓碑、谱系、退役各一条旋钮按 2、2、1、1 逐条换值写出 [2, 1, 1]（中间那片少一条）的流，两边都给那一棵旋钮 2，CPU 红 I-9.22 / I-9.37，整份比；
//! J4：墓碑产品支 [815, 1] 那条流的起点镜像是第 816 个 unlink 之前的整段录制流，写表只录那一次 unlink（从 mkfs 录起时枚举写表超过持久掩码的宽、建不出表）。
//! 崩溃点重放合并（判决 `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` 第三节 J6、J10，
//! `research/prompts/m3-crash-enumeration-merge-r2-main-verification.md` 第二节 L1-4、L1-6、L1-7、L6-2）：每个状态比的是对拍字
//! （`crash_judge_gpu::CrossCheckWords`：红项、两遍各自落到的根、结局、失败点、池级 checker 判到位、判不了）；甲二展开的一条流上
//! GPU 拆出的持久掩码（探针内核 `crash_judge_dispatch::persisted_masks_on_the_card`）与 CPU 判表、crash.rs 枚举逐序号相同、对拍字逐状态相同；
//! 33 次写的一段第 2³² + 5 个状态起 GPU 与 CPU 拆出同一个掩码；给定状态表上 GPU 照清单判、与 CPU 逐状态相同；判表核段长对不上的拒建。
//! 第59批 E14：I-1.8 第一句在一块盘上有几个数据单元的流上（三个数据单元的文件只改最后一个单元一份副本的载荷）、第二句一条 inode 叶两份副本
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
//! 截断第二轮（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R1、R6 与第六节用户定案：inode 记录偏移 120 起的截断字段、
//! I-9.7 拆段、新立的 I-9.39、incompat 位 2）：三个单元的文件那一版（或新池新建文件、截断意图那一版）第一个文件的记录按字节改、系统配置的
//! incompat 第一个字节按字节改，各一条流：I-9.7 三种坏法（截断字段之前、之后的预留段非零，带位 2 时 T = 0 而 N_w ≠ 0，不带位 2 而字段非零，
//! 几块盘里只有一块带位 2）、I-9.39 的必红世界（截断字段之外、size 之内的活单元出生早于 T），合法世界（单元恰在 txg T 写下、截到 0 留下单元 0、
//! 截断意图那一版 size 之外还挂着的单元、每一槽都带位 2），系统配置 0x04、0x0E、0x07 三种不认得的位。池级 checker 那一半先比、钉 CPU 红哪几条；
//! 两遍恢复的那一半要核心层的读者认截断字段与位 2（截庚）合进来才对得上（之前 core 拒收带截断字段的记录与 0x06，GPU 恢复内核已经照判决收）。
//! 里程碑三收尾（用户 2026-10-20 定 C663、C728）：系统配置槽 S 写 17、R 写 2 各一条流钉 CPU 红 I-7.13（R、S 越界整池拒，不再是这一槽不可择），
//! 系统配置槽末字节（偏移 4095）、根记录之后头一个字节（偏移 457）各写 1 的两条流钉 CPU 红 I-2.4（两段第一版恒 0），GPU 都逐位跟上；
//! 流里每一次系统配置槽写 incompat 第一个字节写 0x0E、格式版本写 2 的一条流钉格式版本先于 incompat 位判（C663「格式版本先」）：
//! 两遍恢复都按格式版本整池拒、没有一个状态报 incompat 位不认得，GPU 恢复内核逐态跟上。
#![cfg(all(feature = "verdict-store", feature = "gpu"))]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
mod common_crash_points;
#[path = "../../singlefs-harness/tests/common_deletion_in_progress/mod.rs"]
mod common_deletion_in_progress;
#[path = "../../singlefs-harness/tests/common_pointer_head_corruption/mod.rs"]
mod common_pointer_head_corruption;
#[path = "../../singlefs-harness/tests/common_truncation/mod.rs"]
mod common_truncation;

use singlefs_checker_tier::crash_amplification::record_the_flow_and_read_it_back;
use std::collections::BTreeMap;
use std::time::Instant;

use common::{
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool, file_content,
    geometry, overwrite_in_process_as_instance_one_and_advance_the_pool, parameters,
    publish_overwrite_in_process, BuiltPool, FIXED_WRITE_TIME_SECONDS,
};
use common_crash_points::{mount_writable_step, overwrite_step, record_pool_build, roll_back_step};
use common_deletion_in_progress::{
    content_of_three_data_units, pool_with_a_three_unit_file, pool_with_new_inodes,
    publish_the_intent, publish_the_next_step_in_batches_of_one, roll_back_to,
    turn_the_root_ring_over_by_creating_inodes, unlink_to_the_end, FIRST_FILE,
    INSTANCE_OF_THE_PUBLISHES, WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
};
use singlefs_checker::image::{InvariantVerdict, IMPLEMENTED_INVARIANTS};
use singlefs_checker::walk::{
    check_pool_image, check_pool_image_given_the_container_capacity_knobs, ContainerCapacityKnobs,
    LineageContainerCapacityKnob, RetirementContainerCapacityKnob, TombstoneContainerCapacityKnob,
};
use singlefs_checker_tier::crash::{
    crash_point_replay_plan_state_count, CrashPointReplayOracleViolationKind,
    CrashPointReplaySegmentExpansion, CrashPointReplayVersionTable, CrashStateJudgement,
    CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE, CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER,
};
use singlefs_checker_tier::crash_amplification::{
    gpu_recovery_words, gpu_verdict_items, judge_my_blocks_on_gpu_cards, plan_crash_points,
    record_crash_points, recovery_words_of, unjudged_blocks, CrashFlow, CrashPointRecorder,
    CrashPointSpan, RecordedGivenStatesTable,
};
use singlefs_checker_tier::crash_identity::CoverageReport;
use singlefs_checker_tier::crash_identity::{judging_units_key, JudgingUnits, SectorImage};
use singlefs_checker_tier::crash_judge_dispatch::{
    mebibytes_a_judge_needs, states_per_scratch_lane, GpuCrashJudge, PackedJudgeTables,
    PackedJudgeTablesReadBack,
};
use singlefs_checker_tier::crash_judge_gpu::{
    cross_check_words_agree_in_the_cross_check, cross_check_words_of_cpu_judgement,
    cross_check_words_of_gpu_verdict, pool_checker_bits_agree_in_the_cross_check,
    pool_checker_judged_bits, pool_checker_violated_and_judged_bits, pool_checker_violated_bits,
    verdict_bits, CrossCheckWords, JudgeKernelSources,
    CHECKER_REFERENCES_AND_SNAPSHOTS_ENTRY_POINT, CHECKER_SCAN_REBUILD_ENTRY_POINT,
    KERNEL_ENTRY_POINT, POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS,
    POOL_CHECKER_JUDGED_FIRST_WORD, POOL_CHECKER_JUDGED_WORDS, POOL_CHECKER_REACHED_WORD,
    POOL_CHECKER_UNDECIDABLE_WORD, POOL_CHECKER_VIOLATED_BIT_CAPACITY,
    POOL_CHECKER_VIOLATED_FIRST_WORD, POOL_CHECKER_VIOLATED_WORDS, SCAN_REBUILD_OUTCOMES,
    SCAN_REBUILD_OUTCOME_WORD, VERDICT_WORDS,
};
use singlefs_checker_tier::crash_judge_tables::{
    build_judge_tables, build_judge_tables_against_the_version_table, JudgeFlow, JudgeLocationKind,
    JudgeTables,
};
use singlefs_checker_tier::crash_library::{
    judge_tables_through_the_pairing_library, pairing_conclusion_reused_or_judged,
    toolchain_of_this_process, CrashLibrary, CrashLibraryClass, StageOutcome,
};
use singlefs_checker_tier::crash_state_enumeration::{
    sample_crash_states_on_the_cpu,
    sample_crash_states_on_the_cpu_with_threads_against_the_version_table, CrashPointReplayItem,
};
use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::{
    cpu_reference_table_read_back_from_the_pairing_library, first_usable_card_or_report_not_run,
    gpu_card_pool_or_report_not_run,
    image_with_every_write_landed_read_back_from_the_pairing_library, not_run_line,
    usable_cards_or_report_not_run, GpuCardPool, CARD_QUOTA_ENVIRONMENT_VARIABLE,
};
use singlefs_checker_tier::gpu_unit_checks::GpuCard;
use singlefs_checker_tier::verdict_store::{JudgingUnitsFingerprint, VerdictStore};
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
    SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION, TREE_KIND_SNAPSHOT_INODE_ROOT,
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
use singlefs_harness::gpu_memory_ledger::{
    reason_for_running_out_of_gpu_memory, GPU_NOT_RUN_MARKER,
};
use singlefs_harness::heavy_threads::{
    own_variable_or_default_heavy_threads, registered_heavy_threads,
    THREAD_CAP_ENVIRONMENT_VARIABLE,
};
use singlefs_harness::hexadecimal::hexadecimal_text;
use singlefs_harness::history::{
    execute_history_with, CompactionChoice, CompactionRunEnding, ContentChoice, ContentLength,
    GeneratedHistory, HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed,
    HistoryStartingPoint, RollbackTargetChoice, SnapshotChoice,
    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion,
    PublishedVersionWithoutAFile, RetainedWrite, WrittenContents,
};
use singlefs_harness::numeric_environment_variable::{
    number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::sha256::{sha256_digest, DIGEST_BYTES};
use singlefs_harness::SharedStream;

/// 崩溃点重放取样与崩溃点的设点文件：这份用例文件。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs";

/// 判表先入库（对拍那一类的库，项名是用例名）、再读库，GPU 判器只拿读回的那一份算；只录那一趟交 `None`（用例当场返回）。
fn packed_tables_through_the_library(
    item_name: &str,
    tables: &JudgeTables,
) -> Option<PackedJudgeTablesReadBack> {
    match judge_tables_through_the_pairing_library(item_name, &PackedJudgeTables::of(tables)) {
        StageOutcome::Judged(packed) => Some(packed),
        StageOutcome::RecordedOnly | StageOutcome::JudgedMyShareOnly { .. } => None,
    }
}

const EXPAND_UP_TO_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO";
const DEFAULT_EXPAND_UP_TO: usize = 8;
const CPU_THREADS_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_JUDGE_CPU_THREADS";
const SECOND_FILE_BYTES: usize = 4100;
const THIRD_FILE_BYTES: usize = 2500;

/// 展开上限变量的值（`None` 是没设）：没设取 [`DEFAULT_EXPAND_UP_TO`]；设了而不是非负整数就 panic、消息带变量名与原值，不退回默认
/// （与 `SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO` 同一判法：缩没缩规模要看得出来，写错的值不许悄悄变成默认那一档）。
/// 这一族三个变量（展开上限、显存额度、CPU 线程数）都经 [`number_from_the_variable_value_panicking_when_unreadable`] 判。
///
/// # Panics
/// 设了而不是非负整数。
fn expand_up_to_from_the_value_of_the_variable(value: Option<&str>) -> usize {
    number_from_the_variable_value_panicking_when_unreadable(
        EXPAND_UP_TO_ENVIRONMENT_VARIABLE,
        value,
    )
    .unwrap_or(DEFAULT_EXPAND_UP_TO)
}

/// # Panics
/// 展开上限变量设了而认不出（[`expand_up_to_from_the_value_of_the_variable`]），或它的值不是 UTF-8。
fn expand_up_to() -> usize {
    expand_up_to_from_the_value_of_the_variable(
        process_environment_variable(EXPAND_UP_TO_ENVIRONMENT_VARIABLE).as_deref(),
    )
}

#[test]
#[should_panic(expected = "SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=\"eight\" 认不出")]
fn an_unreadable_crash_judge_expand_up_to_value_panics_naming_the_variable_and_the_value() {
    let _limit = expand_up_to_from_the_value_of_the_variable(Some("eight"));
}

/// 显存额度变量的值（`None` 是没设）：没设是 `None`（不给这张卡定额度）；设了而不是非负整数就 panic、消息带变量名与原值，
/// 不悄悄当成没设（量速度时照的就不是配置的额度）。
///
/// # Panics
/// 设了而不是非负整数。
fn card_quota_mebibytes_from_the_value_of_the_variable(value: Option<&str>) -> Option<u64> {
    number_from_the_variable_value_panicking_when_unreadable(CARD_QUOTA_ENVIRONMENT_VARIABLE, value)
}

#[test]
#[should_panic(expected = "SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES=\"4GiB\" 认不出")]
fn an_unreadable_crash_judge_card_quota_value_panics_naming_the_variable_and_the_value() {
    let _quota = card_quota_mebibytes_from_the_value_of_the_variable(Some("4GiB"));
}

/// CPU 判器线程数变量的值（`None` 是没设）：设了照它，没设取每台默认重活线程数（`thread_cap_value` 是 `SINGLEFS_THREAD_CAP` 的值，
/// 设了照它，再没设取 `.claude/heavy-thread-default` 的登记值，判法在 `singlefs_harness::heavy_threads`）；设了而不是正整数（0 也算）
/// 就 panic、消息带变量名与原值，不悄悄换成至少 1 或默认。
///
/// # Panics
/// 设了而不是正整数；落到默认时 `SINGLEFS_THREAD_CAP` 设了而不是正整数。
fn cpu_threads_from_the_value_of_the_variable(
    value: Option<&str>,
    thread_cap_value: Option<&str>,
) -> std::num::NonZeroUsize {
    own_variable_or_default_heavy_threads(CPU_THREADS_ENVIRONMENT_VARIABLE, value, thread_cap_value)
}

#[test]
#[should_panic(expected = "SINGLEFS_CRASH_JUDGE_CPU_THREADS=\"0\" 认不出")]
fn a_zero_crash_judge_cpu_threads_value_panics_naming_the_variable_and_the_value() {
    let _threads = cpu_threads_from_the_value_of_the_variable(Some("0"), None);
}

/// CPU 判器线程数变量没设：取登记的每台默认重活线程数，不取本机核数；设了照它。
#[test]
fn crash_judge_cpu_threads_take_the_registered_heavy_thread_default_when_unset() {
    assert_eq!(
        cpu_threads_from_the_value_of_the_variable(None, None),
        registered_heavy_threads(),
        "SINGLEFS_CRASH_JUDGE_CPU_THREADS 与 SINGLEFS_THREAD_CAP 都没设取 .claude/heavy-thread-default 的登记值"
    );
    assert_eq!(
        cpu_threads_from_the_value_of_the_variable(None, Some("5")).get(),
        5
    );
    assert_eq!(
        cpu_threads_from_the_value_of_the_variable(Some("3"), Some("5")).get(),
        3
    );
}

/// 量速度的那条用例取一整张卡：空闲显存最大的那张，不经卡池记账（它量的就是一张卡独占时多快）。
/// `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES` 给了就当这张卡的显存额度（量速度时照配置的额度量；
/// 写错就 panic，[`card_quota_mebibytes_from_the_value_of_the_variable`]）。
/// 本机没有 NVIDIA 独显适配器时打「本次未跑」那一行、交回 `None`，用例当场返回；有适配器而一张卡都没起来判红。
fn the_whole_emptiest_card_or_report_not_run(test_name: &str) -> Option<GpuCard> {
    let quota = card_quota_mebibytes_from_the_value_of_the_variable(
        process_environment_variable(CARD_QUOTA_ENVIRONMENT_VARIABLE).as_deref(),
    );
    let mut card = first_usable_card_or_report_not_run(test_name)?;
    if quota.is_some() {
        card.memory_quota_mebibytes = quota;
    }
    Some(card)
}

/// 卡上一段活（建判器、判）的结局。
#[derive(Debug)]
enum GpuWorkOnTheCard<Outcome> {
    Finished(Outcome),
    /// 卡上 wgpu 报了 Out of Memory：这条用例报「本次未跑」，打的就是这一行。
    NotRunTheGpuMemoryWasShort {
        not_run_line: String,
    },
}

/// 卡上一段活的结局怎么判（不用卡）：没报 Out of Memory 照交 `outcome`（活里 panic 了照样往外抛）；报了（`out_of_memory_error` 是 wgpu
/// 那一句），活做没做完都是显存不够，报「本次未跑」、不判红。显存账只记这个进程借的，两个进程同一刻读到同一份空闲、一起借超了，
/// 由这里兜底（根环线第 10x 批，用户定的 G1 二「去掉跨进程账」，`singlefs_harness::gpu_memory_ledger` 文件头）；
/// 没卡、点名的卡找不到时怎么判不在这里，照旧经卡池（`gpu_test_start::gpu_card_pool_or_report_not_run`）。
fn gpu_work_on_the_card_with_out_of_memory_reported_as_not_run<Outcome>(
    test_name: &str,
    what_ran: &str,
    outcome: std::thread::Result<Outcome>,
    out_of_memory_error: Option<String>,
) -> GpuWorkOnTheCard<Outcome> {
    match (outcome, out_of_memory_error) {
        (Ok(outcome), None) => GpuWorkOnTheCard::Finished(outcome),
        (Ok(_) | Err(_), Some(error)) => GpuWorkOnTheCard::NotRunTheGpuMemoryWasShort {
            not_run_line: not_run_line(
                test_name,
                &format!(
                    "显存不够：{what_ran}时卡上 wgpu 报了 Out of Memory（{error}）。显存账只记这个进程借的，卡上别的进程同一刻借走了"
                ),
            ),
        },
        (Err(panic), None) => std::panic::resume_unwind(panic),
    }
}

/// 在 `card` 上做 `work`，同时接住 wgpu 报的 Out of Memory（[`gpu_work_on_the_card_with_out_of_memory_reported_as_not_run`]）：
/// 报了打「本次未跑」那一行、交 `None`，调用的用例当场返回；没报交 `work` 的结果。`work` 里因为缓冲没建成而 panic 的一并接住。
/// wgpu 的错误范围按线程分：`work` 要在调它的这条线程上建判器、派活。
fn on_the_card_or_report_not_run_when_out_of_memory<Outcome>(
    test_name: &str,
    card: &GpuCard,
    what_ran: &str,
    work: impl FnOnce() -> Outcome,
) -> Option<Outcome> {
    let out_of_memory_scope = card
        .device()
        .push_error_scope(wgpu::ErrorFilter::OutOfMemory);
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work));
    let out_of_memory_error =
        pollster::block_on(out_of_memory_scope.pop()).map(|error| error.to_string());
    match gpu_work_on_the_card_with_out_of_memory_reported_as_not_run(
        test_name,
        what_ran,
        outcome,
        out_of_memory_error,
    ) {
        GpuWorkOnTheCard::Finished(outcome) => Some(outcome),
        GpuWorkOnTheCard::NotRunTheGpuMemoryWasShort { not_run_line } => {
            println!("{not_run_line}");
            None
        }
    }
}

/// 放量路径交回的错是不是卡上 wgpu 报了 Out of Memory 那一句（`singlefs_harness::gpu_memory_ledger::reason_for_running_out_of_gpu_memory`，
/// 中间那一段是哪张卡、哪一段）：按那一句自己的头尾认，不另抄一份句子。
fn is_the_reason_that_the_card_ran_out_of_gpu_memory(reason: &str) -> bool {
    const PLACE_OF_WHAT_RAN_OUT: char = '\u{0}';
    let template = reason_for_running_out_of_gpu_memory(&PLACE_OF_WHAT_RAN_OUT.to_string());
    let (head, tail) = template
        .split_once(PLACE_OF_WHAT_RAN_OUT)
        .expect("那一句里有放「哪一段、哪张卡」的地方");
    reason.len() >= head.len() + tail.len() && reason.starts_with(head) && reason.ends_with(tail)
}

/// 放量路径在调用方起的卡上判一趟（`crash_amplification::judge_my_blocks_on_gpu_cards`）的结局：卡上 wgpu 报了 Out of Memory
/// （放量路径已打它自己的「本次未跑」那一行、交回的错是那一句）时这条用例也报「本次未跑」、交 `None`；别的错判红。
///
/// # Panics
/// 放量路径交回 Out of Memory 之外的错。
fn judged_on_the_cards_or_report_not_run_when_out_of_memory<Judged>(
    test_name: &str,
    judged: Result<Judged, String>,
) -> Option<Judged> {
    match judged {
        Ok(judged) => Some(judged),
        Err(reason) if is_the_reason_that_the_card_ran_out_of_gpu_memory(&reason) => {
            println!("{}", not_run_line(test_name, &reason));
            None
        }
        Err(reason) => panic!("{test_name}：放量路径没判完：{reason}"),
    }
}

/// 卡上报了 Out of Memory（判器建成、判的时候报的，与建判器时缓冲没建成、连带 panic 的两形）判成「本次未跑」：那一行带
/// `NOT_RUN_NO_GPU_ADAPTER` 记号、用例名与「显存不够」，不判红；没报照交结果。假的 wgpu 错误句子交给判那一支的函数，不用卡。
#[test]
fn an_out_of_memory_reported_on_the_card_is_reported_as_not_run_and_not_as_red() {
    const TEST_NAME: &str =
        "an_out_of_memory_reported_on_the_card_is_reported_as_not_run_and_not_as_red";
    const FAKE_OUT_OF_MEMORY_ERROR: &str = "Out of Memory";
    let finished = gpu_work_on_the_card_with_out_of_memory_reported_as_not_run(
        TEST_NAME,
        "判",
        Ok(7_u32),
        None,
    );
    assert!(
        matches!(finished, GpuWorkOnTheCard::Finished(7)),
        "没报 Out of Memory 照交结果：{finished:?}"
    );
    let reported_after_the_work = gpu_work_on_the_card_with_out_of_memory_reported_as_not_run(
        TEST_NAME,
        "判",
        Ok(7_u32),
        Some(FAKE_OUT_OF_MEMORY_ERROR.to_string()),
    );
    let reported_with_a_panic = gpu_work_on_the_card_with_out_of_memory_reported_as_not_run::<u32>(
        TEST_NAME,
        "建判器",
        Err(Box::new("缓冲没建成")),
        Some(FAKE_OUT_OF_MEMORY_ERROR.to_string()),
    );
    for reported in [reported_after_the_work, reported_with_a_panic] {
        let GpuWorkOnTheCard::NotRunTheGpuMemoryWasShort { not_run_line } = &reported else {
            panic!("卡上报了 Out of Memory 要报本次未跑、不判红：{reported:?}");
        };
        assert!(
            not_run_line.starts_with(&format!(
                "{GPU_NOT_RUN_MARKER} test={TEST_NAME} reason=显存不够："
            )),
            "那一行带记号、用例名，原因写明显存不够：{not_run_line}"
        );
        assert!(
            not_run_line.contains(FAKE_OUT_OF_MEMORY_ERROR),
            "那一行带 wgpu 报的那一句：{not_run_line}"
        );
    }
}

/// 没报 Out of Memory 而活里 panic 了（判错、内核没判完）：照样往外抛、判红，不当成显存不够。
#[test]
#[should_panic(expected = "内核没把状态判完")]
fn a_panic_on_the_card_without_an_out_of_memory_is_still_red() {
    let _not_reached = gpu_work_on_the_card_with_out_of_memory_reported_as_not_run::<u32>(
        "a_panic_on_the_card_without_an_out_of_memory_is_still_red",
        "判",
        Err(Box::new("内核没把状态判完")),
        None,
    );
}

/// 放量路径交回的错：卡上报了 Out of Memory 那一句认成显存不够（中间那一段是哪张卡、哪一段，照放量路径的写法造），
/// 别的错（建不出表、拒判）不认，照旧判红。
#[test]
fn only_the_out_of_memory_reason_of_the_product_path_is_taken_as_the_card_running_out_of_gpu_memory(
) {
    assert!(is_the_reason_that_the_card_ran_out_of_gpu_memory(
        &reason_for_running_out_of_gpu_memory("序号 0 的卡判一块时（Out of Memory）")
    ));
    for other_reason in [
        "建不出这条流的表：不支持清零写",
        "显存不够：此刻池里没有一张卡借得出 12 MiB",
        "",
    ] {
        assert!(
            !is_the_reason_that_the_card_ran_out_of_gpu_memory(other_reason),
            "「{other_reason}」不是卡上报了 Out of Memory"
        );
    }
}

/// 一条草稿缓冲装几个状态，按卡池里绑定上限最小的那张卡估（借卡之前还不知道借到哪张）。
fn states_of_one_scratch_lane_on_every_card_of(cards: &GpuCardPool) -> usize {
    states_per_scratch_lane(cards.smallest_max_storage_buffer_binding_size())
}

struct RecordedFlow {
    name: &'static str,
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
}

fn second_content() -> Vec<u8> {
    (0..SECOND_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

fn third_content() -> Vec<u8> {
    (0..THIRD_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 11) % 253).expect("小于 256"))
        .collect()
}

fn record_new_pool_file_creation(tag: &str, name: &'static str) -> RecordedFlow {
    let pool = build_pool(tag);
    let operations = pool.retained_operations();
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("发布有一条根槽写");
    RecordedFlow {
        name,
        base: pool.memory_pool_after_mkfs(),
        writes,
        segments,
        judged_root_index,
        versions: vec![PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
            content: file_content(),
        }],
    }
}

/// 改坏发布的根指着的树表单元（两份副本各一字节）：根落了的状态走读失败，CPU 与 GPU 都要红。
fn record_new_pool_with_a_broken_tree_table(tag: &str) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-broken-tree-table");
    let publish_root = flow.judged_root_index;
    break_both_copies_of_the_tree_table_named_by_the_root_write(&mut flow, publish_root);
    flow
}

/// 第 `root_write` 次写（根槽写）指着的树表单元，两份副本各改坏末字节、不重封：那条根落了的状态上它的树表读不出。
fn break_both_copies_of_the_tree_table_named_by_the_root_write(
    flow: &mut RecordedFlow,
    root_write: usize,
) {
    for copy in copies_of_the_tree_table_named_by_the_root_write(flow, root_write) {
        match &mut flow.writes[copy].contents {
            WrittenContents::Bytes(bytes) => {
                let last = bytes.len() - 1;
                bytes[last] ^= 0x01;
            }
            WrittenContents::Zeros { .. } => panic!("单元写是普通写"),
        }
    }
}

/// 第 `root_write` 次写（根槽写）指着的树表单元的两份副本（写下标，位置条目 0、1 的次序）。
fn copies_of_the_tree_table_named_by_the_root_write(
    flow: &RecordedFlow,
    root_write: usize,
) -> Vec<usize> {
    let root_bytes = flow.writes[root_write]
        .bytes()
        .expect("根槽写带字节")
        .to_vec();
    // 树表指针在根记录偏移 36：两条位置条目在 50 与 64（盘 4、槽 6、校验和 4）
    let locations: Vec<(u32, u64)> = [86usize, 100]
        .iter()
        .map(|offset| {
            let device =
                u32::from_le_bytes(root_bytes[*offset..*offset + 4].try_into().expect("4"));
            let mut slot = [0u8; 8];
            slot[..6].copy_from_slice(&root_bytes[*offset + 4..*offset + 10]);
            (device, u64::from_le_bytes(slot))
        })
        .collect();
    // 每条位置条目取根槽写之前落在那个槽上的最后一次单元写：那一份就是这条根指着的副本（同一个槽更早被别的单元用过时不碰那一次）
    let copies: Vec<usize> = locations
        .iter()
        .map(|(device, location_slot)| {
            flow.writes[..root_write]
                .iter()
                .rposition(|write| {
                    write.kind == StepKind::UnitWrite
                        && write.device.0 == *device
                        && write.offset.0 / 16384 == *location_slot
                })
                .expect("根槽写之前写过它指着的树表单元")
        })
        .collect();
    assert_ne!(copies[0], copies[1], "树表单元的两份副本是两次写");
    copies
}

/// 改一次写里的几个字节之后把那个结构的整槽校验和字段重封（CRC-32C 住字段前 4 字节、后 28 字节 0）：拦住它的只剩被改的字段本身。
fn rewrite_and_reseal(
    flow: &mut RecordedFlow,
    kind: StepKind,
    nth_from_the_end: usize,
    cover_end: usize,
    field_offset: usize,
    rewrite: &dyn Fn(&mut Vec<u8>),
) {
    let index = flow
        .writes
        .iter()
        .enumerate()
        .rev()
        .filter(|(_, write)| write.kind == kind)
        .nth(nth_from_the_end)
        .map(|(index, _)| index)
        .expect("流里有这一种写");
    match &mut flow.writes[index].contents {
        WrittenContents::Bytes(bytes) => {
            rewrite(bytes);
            let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(
                bytes,
                cover_end,
                field_offset,
            );
            bytes[field_offset..field_offset + 32].copy_from_slice(&digest);
        }
        WrittenContents::Zeros { .. } => panic!("结构写是普通写"),
    }
}

/// 发布那条 journal 记录的提交标记字节改成 2（重封头校验和）：core 的读者当记录损坏，checker 的 I-8.8 判红。
fn record_new_pool_with_a_commit_byte_of_two(tag: &str) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-journal-commit-byte-2");
    rewrite_and_reseal(&mut flow, StepKind::JournalRecord, 0, 4096, 46, &|bytes| {
        bytes[86] = 2;
    });
    flow
}

/// 发布那条 journal 记录的记录标志改成 0b11（重封）：core 拒收这条记录，checker 的 I-8.9 判「标志其余位」红。
fn record_new_pool_with_record_flags_of_three(tag: &str) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-journal-flags-3");
    rewrite_and_reseal(&mut flow, StepKind::JournalRecord, 0, 4096, 46, &|bytes| {
        bytes[7] = 3;
    });
    flow
}

/// 最后一次系统配置槽写里固定结构槽距改成 512（重封）：那一槽自证过而带读者不收的值，core 整池拒（两遍恢复都没择到根），
/// checker 的 I-7.13 判红、别的不变量不判。
fn record_new_pool_with_a_refused_slot_spacing(tag: &str) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-sysconfig-spacing-512");
    rewrite_and_reseal(
        &mut flow,
        StepKind::SystemConfigurationSlot,
        0,
        4096,
        155,
        &|bytes| {
            bytes[429..433].copy_from_slice(&512u32.to_le_bytes());
        },
    );
    flow
}

/// 发布那次根槽写之前，同一个根槽上垫几次写：垫满 32 次，发布的根落没落就只差组合掩码的高 32 位。
const FILLER_WRITES_BEFORE_THE_PUBLISH_ROOT: usize = 32;

/// 新池新建文件的流上，在暖机最后一次根槽写所在那一段之后另起一段：32 次根槽写，都写发布的根将要落的那个根槽、写的是它在基镜像里的
/// 字节（全零：那个槽在发布之前没有根）。这 32 次写不改任何一个崩溃态在盘上的字节（它们不带 journal 记录与单元，记录核对器也不看），
/// 只把发布那次根槽写挤成这个落点的第 33 次写：发布的根落了与没落的两个版本，组合掩码低 32 位相同（那 32 次都落了）、只差高位。
/// 判器查版本只比低 32 位时，把发布的根落了的状态读成全零的那一版，恢复落到暖机的根（C601（判器查版本不比组合掩码高 32 位时没有会红的流））。
fn record_new_pool_with_the_publish_root_slot_written_past_thirty_two_times(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-publish-root-slot-past-32-writes");
    let publish_root = flow.writes[flow.judged_root_index].clone();
    let slot_bytes_in_the_base = singlefs_core::recovery::PoolReader::read(
        &flow.base,
        publish_root.device,
        publish_root.offset,
        usize::try_from(publish_root.length_in_bytes()).expect("根槽写的长度装得进 usize"),
    )
    .expect("基镜像里读得到发布的根要落的槽");
    assert!(
        slot_bytes_in_the_base.iter().all(|byte| *byte == 0),
        "发布的根落在一个基镜像里空着的根槽上（垫的写写全零，不改任何崩溃态的盘上字节）"
    );
    assert!(
        flow.writes[..flow.judged_root_index]
            .iter()
            .all(|write| (write.device, write.offset) != (publish_root.device, publish_root.offset)),
        "发布之前流里没有别的写落在这个根槽上"
    );
    let warm_up_root = flow.writes[..flow.judged_root_index]
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("发布之前有暖机的根槽写");
    let segment_of_the_warm_up_root = flow
        .segments
        .iter()
        .position(|segment| segment.contains(&warm_up_root))
        .expect("暖机的根槽写在某一段里");
    let first_filler = *flow.segments[segment_of_the_warm_up_root]
        .last()
        .expect("段里至少一次写")
        + 1;
    let filler = RetainedWrite {
        device: publish_root.device,
        kind: StepKind::RootRecordFua,
        is_force_unit_access: publish_root.is_force_unit_access,
        offset: publish_root.offset,
        contents: WrittenContents::Bytes(slot_bytes_in_the_base),
    };
    flow.writes.splice(
        first_filler..first_filler,
        vec![filler; FILLER_WRITES_BEFORE_THE_PUBLISH_ROOT],
    );
    for segment in &mut flow.segments {
        for write_index in segment.iter_mut() {
            if *write_index >= first_filler {
                *write_index += FILLER_WRITES_BEFORE_THE_PUBLISH_ROOT;
            }
        }
    }
    flow.segments.insert(
        segment_of_the_warm_up_root + 1,
        (first_filler..first_filler + FILLER_WRITES_BEFORE_THE_PUBLISH_ROOT).collect(),
    );
    flow.judged_root_index += FILLER_WRITES_BEFORE_THE_PUBLISH_ROOT;
    flow
}

fn record_fixed_script_through_rollback(tag: &str) -> RecordedFlow {
    let mut pool = build_pool(tag);
    let mut recorder = CrashPointRecorder::new(pool.stream.clone(), pool.mkfs_operation_count);
    record_pool_build(&mut recorder, &pool);
    let mut versions = vec![PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(3),
        content: file_content(),
    }];
    let second = overwrite_step(
        &mut recorder,
        &mut pool,
        &second_content(),
        InstanceGeneration(1),
    );
    versions.push(PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(4),
        content: second_content(),
    });
    pool.output = second;
    let mounted = mount_writable_step(&mut recorder, &mut pool);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    for txg in 5..=7 {
        versions.push(PublishedVersion {
            instance: InstanceGeneration(2),
            checkpoint_txg: CheckpointTxg(txg),
            content: second_content(),
        });
    }
    let third = overwrite_step(
        &mut recorder,
        &mut pool,
        &third_content(),
        InstanceGeneration(2),
    );
    versions.push(PublishedVersion {
        instance: InstanceGeneration(2),
        checkpoint_txg: CheckpointTxg(8),
        content: third_content(),
    });
    pool.output = third;
    roll_back_step(
        &mut recorder,
        &mut pool,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
    );
    versions.push(PublishedVersion {
        instance: InstanceGeneration(2),
        checkpoint_txg: CheckpointTxg(9),
        content: file_content(),
    });
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("根槽写");
    RecordedFlow {
        name: "fixed-script-through-rollback",
        base,
        writes,
        segments,
        judged_root_index,
        versions,
    }
}

fn crc32c(bytes: &[u8]) -> u32 {
    singlefs_core::checksum::crc32_castagnoli(bytes)
}

/// 一个单元重封自己的两道校验和：载荷 CRC（按类的偏移，罩明文头之后到单元末）与头校验和（字段 10，罩明文头）。
fn reseal_unit(bytes: &mut [u8]) {
    let (header_end, payload_crc_offset) = match bytes[6] {
        1 => (105, 101),
        2 => {
            let key_width = usize::from(bytes[51]);
            (86 + 2 * key_width, 76 + 2 * key_width)
        }
        3 => (107, 89),
        other => panic!("单元类标签 {other} 不在 1 / 2 / 3 里"),
    };
    let payload_crc = crc32c(&bytes[header_end..]);
    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
    let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(bytes, header_end, 10);
    bytes[10..42].copy_from_slice(&digest);
}

/// journal 记录重封：载荷校验和（95，罩点名项）再整条头校验和（46，罩 4096）。
fn reseal_journal_record(bytes: &mut [u8]) {
    let named = usize::try_from(u32::from_le_bytes(bytes[12..16].try_into().expect("4")))
        .expect("点名项数");
    let payload_crc = crc32c(
        &bytes[JOURNAL_RECORD_HEADER_BYTES
            ..JOURNAL_RECORD_HEADER_BYTES + named * JOURNAL_NAMED_ENTRY_BYTES_IN_THE_RECORD],
    );
    bytes[95..99].copy_from_slice(&payload_crc.to_le_bytes());
    let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(bytes, 4096, 46);
    bytes[46..78].copy_from_slice(&digest);
}

/// 下一条记录该带的反向链：这一条 487 字节头、校验和字段按 0 参与的 CRC-32C。
fn journal_chain_value(bytes: &[u8]) -> u32 {
    let mut header = bytes[..JOURNAL_RECORD_HEADER_BYTES].to_vec();
    header[46..78].fill(0);
    crc32c(&header)
}

fn reseal_root_record(bytes: &mut [u8]) {
    let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(bytes, bytes.len(), 138);
    bytes[138..170].copy_from_slice(&digest);
}

/// 改坏一批单元写（同一个单元的两份副本，或只改其中一份）之后把校验和一路重封到根：被改的单元先重封自己的两道校验和，
/// 再把每一处指着它的位置条目（树节点、根记录、journal 点名项里的 盘 4 + 槽 6 + 整单元校验和 4）换成新的整单元 CRC，
/// 被改的那些结构再各自重封（单元继续往上传；journal 记录还要把同实例下一条的反向链改成它新的链值、一路传下去）。
/// 这样留下的坏法只剩被改的那个字段本身：CPU 的 checker 红在那个字段管的不变量上，GPU 要红在同样的状态、同样的项上。
fn corrupt_units_and_reseal_up_to_the_root(
    flow: &mut RecordedFlow,
    targets: &[usize],
    mutate: &dyn Fn(&mut [u8]),
) {
    corrupt_units_and_carry_their_whole_unit_checksums_up_to_the_root(
        flow,
        targets,
        mutate,
        OwnChecksumsOfTheChangedUnits::Resealed,
    );
}

/// 被改的单元自己那两道校验和（载荷 CRC、头校验和）重不重封。
#[derive(Clone, Copy, Debug)]
enum OwnChecksumsOfTheChangedUnits {
    /// 重封：被改的单元自证得过，坏法只剩改的那个字段。
    Resealed,
    /// 不重封：被改的单元按自己的类认不出、或自证不过（类标签改成不登记的码那一形），指着它的位置条目照样换成新的整单元 CRC。
    LeftAsBeforeTheChange,
}

/// 同 [`corrupt_units_and_reseal_up_to_the_root`]，被改的单元自己那两道校验和照 `own_checksums` 重封或不动；
/// 指着它的位置条目一律换成改后的整单元 CRC、沿引用链一路重封到根。
fn corrupt_units_and_carry_their_whole_unit_checksums_up_to_the_root(
    flow: &mut RecordedFlow,
    targets: &[usize],
    mutate: &dyn Fn(&mut [u8]),
    own_checksums: OwnChecksumsOfTheChangedUnits,
) {
    let mut pending: Vec<(usize, u32)> = Vec::new();
    for target in targets {
        let write = &mut flow.writes[*target];
        assert_eq!(write.kind, StepKind::UnitWrite, "改的是单元写");
        let WrittenContents::Bytes(bytes) = &mut write.contents else {
            panic!("单元写是普通写");
        };
        let old_crc = crc32c(bytes);
        mutate(bytes);
        match own_checksums {
            OwnChecksumsOfTheChangedUnits::Resealed => reseal_unit(bytes),
            OwnChecksumsOfTheChangedUnits::LeftAsBeforeTheChange => {}
        }
        pending.push((*target, old_crc));
    }
    while let Some((index, old_crc)) = pending.pop() {
        let (device, slot, new_crc) = {
            let write = &flow.writes[index];
            let bytes = write.bytes().expect("单元写带字节");
            (write.device.0, write.offset.0 / 16384, crc32c(bytes))
        };
        if old_crc == new_crc {
            continue;
        }
        let mut pattern = Vec::with_capacity(14);
        pattern.extend_from_slice(&device.to_le_bytes());
        pattern.extend_from_slice(&slot.to_le_bytes()[..6]);
        pattern.extend_from_slice(&old_crc.to_le_bytes());
        let mut journal_records_resealed: Vec<usize> = Vec::new();
        for referrer in 0..flow.writes.len() {
            let kind = flow.writes[referrer].kind;
            let WrittenContents::Bytes(bytes) = &mut flow.writes[referrer].contents else {
                continue;
            };
            let mut replaced = false;
            let mut position = 0;
            while position + 14 <= bytes.len() {
                if bytes[position..position + 14] == pattern[..] {
                    bytes[position + 10..position + 14].copy_from_slice(&new_crc.to_le_bytes());
                    replaced = true;
                    position += 14;
                } else {
                    position += 1;
                }
            }
            if !replaced {
                continue;
            }
            match kind {
                StepKind::UnitWrite => {
                    let referrer_old_crc = {
                        // 位置条目已换、校验和还没重封：整单元 CRC 要按重封之前的原样算，用写进去之前的字节复原
                        let mut before = bytes.clone();
                        let mut restore_position = 0;
                        while restore_position + 14 <= before.len() {
                            if before[restore_position..restore_position + 10] == pattern[..10]
                                && before[restore_position + 10..restore_position + 14]
                                    == new_crc.to_le_bytes()
                            {
                                before[restore_position + 10..restore_position + 14]
                                    .copy_from_slice(&old_crc.to_le_bytes());
                                restore_position += 14;
                            } else {
                                restore_position += 1;
                            }
                        }
                        crc32c(&before)
                    };
                    reseal_unit(bytes);
                    pending.push((referrer, referrer_old_crc));
                }
                StepKind::RootRecordFua => reseal_root_record(bytes),
                StepKind::JournalRecord => {
                    reseal_journal_record(bytes);
                    journal_records_resealed.push(referrer);
                }
                StepKind::ZeroFill
                | StepKind::Discard
                | StepKind::SystemConfigurationSlot
                | StepKind::Barrier => {
                    panic!("位置条目不该出现在 {kind:?} 里")
                }
            }
        }
        carry_the_back_chain_forward_from(flow, journal_records_resealed);
    }
}

/// 反向链一路传下去：这几条 journal 记录写重封过，同盘、同实例、计数器大 1 的那一条带的是前一条头的链值——不等就改成它、重封，再往下传。
fn carry_the_back_chain_forward_from(
    flow: &mut RecordedFlow,
    mut journal_records_resealed: Vec<usize>,
) {
    while let Some(record_index) = journal_records_resealed.pop() {
        let (record_device, instance, counter, chain) = {
            let write = &flow.writes[record_index];
            let bytes = write.bytes().expect("记录写带字节");
            let mut counter = [0u8; 8];
            counter[..6].copy_from_slice(&bytes[20..26]);
            (
                write.device,
                u32::from_le_bytes(bytes[16..20].try_into().expect("4")),
                u64::from_le_bytes(counter),
                journal_chain_value(bytes),
            )
        };
        for next in 0..flow.writes.len() {
            if flow.writes[next].kind != StepKind::JournalRecord
                || flow.writes[next].device != record_device
            {
                continue;
            }
            let WrittenContents::Bytes(bytes) = &mut flow.writes[next].contents else {
                continue;
            };
            let mut next_counter = [0u8; 8];
            next_counter[..6].copy_from_slice(&bytes[20..26]);
            if u32::from_le_bytes(bytes[16..20].try_into().expect("4")) != instance
                || u64::from_le_bytes(next_counter) != counter + 1
                || bytes[91..95] == chain.to_le_bytes()
            {
                continue;
            }
            bytes[91..95].copy_from_slice(&chain.to_le_bytes());
            reseal_journal_record(bytes);
            journal_records_resealed.push(next);
        }
    }
}

/// 流里最后一个单元（两份副本：最后两条命中 `select` 的单元写，同槽同字节）的写下标；`only_first_copy` 只取盘序小的那一份。
fn last_unit_copies(
    flow: &RecordedFlow,
    what: &str,
    select: &dyn Fn(&[u8]) -> bool,
    only_first_copy: bool,
) -> Vec<usize> {
    let matching: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::UnitWrite && write.bytes().is_some_and(select))
        .map(|(index, _)| index)
        .collect();
    assert!(
        matching.len() >= 2,
        "流 {} 里命中「{what}」的单元写只有 {} 条，要至少两条（一个单元的两份副本）；流里的单元写按类 / key 宽 / 条目宽 / 记录类型：{:?}",
        flow.name,
        matching.len(),
        flow.writes
            .iter()
            .filter(|write| write.kind == StepKind::UnitWrite)
            .filter_map(|write| write.bytes())
            .map(|bytes| match unit_class(bytes) {
                2 => format!("2/k{}/w{}", index_node_key_width(bytes), index_node_entry_width(bytes)),
                3 => format!("3/t{}", packed_record_type(bytes)),
                other => format!("{other}"),
            })
            .collect::<Vec<_>>()
    );
    let copies = &matching[matching.len() - 2..];
    assert_eq!(
        flow.writes[copies[0]].offset, flow.writes[copies[1]].offset,
        "最后两条命中的单元写是同一个单元的两份副本（同槽）"
    );
    assert_eq!(
        flow.writes[copies[0]].bytes(),
        flow.writes[copies[1]].bytes(),
        "两份副本字节相同"
    );
    if only_first_copy {
        let first = if flow.writes[copies[0]].device.0 < flow.writes[copies[1]].device.0 {
            copies[0]
        } else {
            copies[1]
        };
        vec![first]
    } else {
        copies.to_vec()
    }
}

fn unit_class(bytes: &[u8]) -> u8 {
    bytes[6]
}
fn index_node_key_width(bytes: &[u8]) -> u8 {
    bytes[51]
}
fn index_node_entry_width(bytes: &[u8]) -> u16 {
    let header_end = 86 + 2 * usize::from(bytes[51]);
    u16::from_le_bytes(bytes[header_end - 2..header_end].try_into().expect("2"))
}
fn packed_record_type(bytes: &[u8]) -> u16 {
    u16::from_le_bytes(bytes[51..53].try_into().expect("2"))
}

/// 新池新建文件的流上改坏最后一个 `select` 命中的单元（两份或只一份）的一个字段、重封到根：`name` 是流名，`invariant` 是该红的那条。
fn record_new_pool_with_a_resealed_field(
    tag: &str,
    name: &'static str,
    select: &dyn Fn(&[u8]) -> bool,
    only_first_copy: bool,
    mutate: &dyn Fn(&mut [u8]),
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, name);
    let targets = last_unit_copies(&flow, name, select, only_first_copy);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, mutate);
    flow
}

/// 固定脚本到回退 D 的流上同样改坏最后一个命中的单元。
fn record_rollback_with_a_resealed_field(
    tag: &str,
    name: &'static str,
    select: &dyn Fn(&[u8]) -> bool,
    mutate: &dyn Fn(&mut [u8]),
) -> RecordedFlow {
    let mut flow = record_fixed_script_through_rollback(tag);
    flow.name = name;
    let targets = last_unit_copies(&flow, name, select, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, mutate);
    flow
}

/// 各条必红的世界：被改的字段与它管的不变量。
fn resealed_field_flows() -> Vec<(RecordedFlow, &'static str)> {
    let inode_leaf = |bytes: &[u8]| unit_class(bytes) == 3 && packed_record_type(bytes) == 2;
    let instance_table = |bytes: &[u8]| unit_class(bytes) == 3 && packed_record_type(bytes) == 4;
    let data_unit = |bytes: &[u8]| unit_class(bytes) == 1;
    let allocation_leaf = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 10
            && index_node_entry_width(bytes) == 20
    };
    // 带至少一条已释放记录（跨度段位 15）的分配记录叶
    let allocation_leaf_with_a_released_record = |bytes: &[u8]| {
        if !allocation_leaf(bytes) {
            return false;
        }
        let header_end = 86 + 2 * usize::from(bytes[51]);
        let count = usize::from(u16::from_le_bytes(
            bytes[header_end - 4..header_end - 2].try_into().expect("2"),
        ));
        (0..count).any(|record| bytes[135 + record * 20 + 11] & 0x80 != 0)
    };
    let accounting_leaf = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 22
            && index_node_entry_width(bytes) == 34
    };
    let tree_table = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 8
            && index_node_entry_width(bytes) == 200
    };
    let mapping_leaf = |bytes: &[u8]| {
        unit_class(bytes) == 2 && index_node_key_width(bytes) == 27 && bytes[50] == 0
    };
    vec![
        (
            // inode 记录的 blocks 字段加 1：blocks ≠ ⌈size ÷ 512⌉
            record_new_pool_with_a_resealed_field(
                "crash-judge-gpu-i915",
                "new-pool-inode-blocks-plus-one",
                &inode_leaf,
                false,
                &|bytes| {
                    bytes[136 + 48] = bytes[136 + 48].wrapping_add(1);
                },
            ),
            "I-9.15",
        ),
        (
            // 数据单元五元组里的对象出生代加 1：与 inode 记录的对象出生代不符
            record_new_pool_with_a_resealed_field(
                "crash-judge-gpu-i910",
                "new-pool-data-unit-object-birth-plus-one",
                &data_unit,
                false,
                &|bytes| {
                    bytes[59] = bytes[59].wrapping_add(1);
                },
            ),
            "I-9.10",
        ),
        (
            // 分配记录叶第一条记录的分配代加 1：分配代 ≠ 它罩住的单元头里的诞生代号
            record_new_pool_with_a_resealed_field(
                "crash-judge-gpu-i310",
                "new-pool-allocation-generation-plus-one",
                &allocation_leaf,
                false,
                &|bytes| {
                    bytes[135 + 12] = bytes[135 + 12].wrapping_add(1);
                },
            ),
            "I-3.10",
        ),
        (
            // 分配记录叶第二条记录的槽号改成第一条的：两条记录罩住同一个槽
            record_new_pool_with_a_resealed_field(
                "crash-judge-gpu-i54",
                "new-pool-allocation-records-overlap",
                &allocation_leaf,
                false,
                &|bytes| {
                    let first = bytes[135..135 + 10].to_vec();
                    bytes[135 + 20..135 + 30].copy_from_slice(&first);
                },
            ),
            "I-5.4",
        ),
        (
            // 记账叶第一条的值加 1：已分配 / 空闲 / defer 的账对不上遍历
            record_new_pool_with_a_resealed_field(
                "crash-judge-gpu-i31",
                "new-pool-accounting-value-plus-one",
                &accounting_leaf,
                false,
                &|bytes| {
                    bytes[159 + 22] = bytes[159 + 22].wrapping_add(1);
                },
            ),
            "I-3.1",
        ),
        (
            // 树表前两条条目的树 ID 互换：不按树 ID 严格升序
            record_new_pool_with_a_resealed_field(
                "crash-judge-gpu-i916",
                "new-pool-tree-table-out-of-order",
                &tree_table,
                false,
                &|bytes| {
                    let first = bytes[131..131 + 8].to_vec();
                    let second = bytes[131 + 200..131 + 208].to_vec();
                    bytes[131..131 + 8].copy_from_slice(&second);
                    bytes[131 + 200..131 + 208].copy_from_slice(&first);
                },
            ),
            "I-9.16",
        ),
        (
            // 映射叶第一条 key 的出生 txg 加 1：映射 key 与单元头不符
            record_new_pool_with_a_resealed_field(
                "crash-judge-gpu-i111",
                "new-pool-mapping-key-birth-plus-one",
                &mapping_leaf,
                false,
                &|bytes| {
                    bytes[169 + 9] = bytes[169 + 9].wrapping_add(1);
                },
            ),
            "I-1.11",
        ),
        (
            // 回退流上最后一版实例表第一行的实例代号加 1：行不再低于挂载根的实例（新池流没有实例表写：取号那次写在 mkfs 那一段里）
            record_rollback_with_a_resealed_field(
                "crash-judge-gpu-i38",
                "rollback-instance-row-not-below-mount",
                &instance_table,
                &|bytes| {
                    bytes[136 + 1] = bytes[136 + 1].wrapping_add(1);
                },
            ),
            "I-3.8",
        ),
        (
            // 数据单元只改一份副本的载荷首字节：归并成一组的两份载荷不同
            record_new_pool_with_a_resealed_field(
                "crash-judge-gpu-i18",
                "new-pool-data-unit-copies-differ",
                &data_unit,
                true,
                &|bytes| {
                    bytes[134] ^= 0xFF;
                },
            ),
            "I-1.8",
        ),
        (
            // 回退流上最后一版树表里 inode 树条目的诞生 txg 加 1：跨根不一
            record_rollback_with_a_resealed_field(
                "crash-judge-gpu-i914",
                "rollback-tree-table-birth-txg-plus-one",
                &tree_table,
                &|bytes| {
                    for entry in 0..(bytes.len() - 131) / 200 {
                        let offset = 131 + entry * 200;
                        if u16::from_le_bytes(
                            bytes[offset + 10..offset + 12].try_into().expect("2"),
                        ) == 2
                        {
                            bytes[offset + 108] = bytes[offset + 108].wrapping_add(1);
                        }
                    }
                },
            ),
            "I-9.14",
        ),
        (
            // 回退流上最后一版分配记录叶里带已释放标志的第一条记录的释放代加 8：落到区间之外
            record_rollback_with_a_resealed_field(
                "crash-judge-gpu-i39",
                "rollback-release-generation-plus-eight",
                &allocation_leaf_with_a_released_record,
                &|bytes| {
                    let header_end = 86 + 2 * usize::from(bytes[51]);
                    let count = usize::from(u16::from_le_bytes(
                        bytes[header_end - 4..header_end - 2].try_into().expect("2"),
                    ));
                    for record in 0..count {
                        let offset = 135 + record * 20;
                        if bytes[offset + 11] & 0x80 != 0 {
                            bytes[offset + 12] = bytes[offset + 12].wrapping_add(8);
                            break;
                        }
                    }
                },
            ),
            "I-3.9",
        ),
    ]
}

// ───────────── 里程碑三二·1 那一批：每条不变量一条只改坏它管的字段的流（GPU 与 CPU 逐状态逐位对拍，CPU 要在这条流上红那一条）─────────────

/// 系统配置槽：整槽校验和字段在 155、罩 4096 字节。
const SYSTEM_CONFIGURATION_CHECKSUM_FIELD: usize = 155;
/// journal 记录头里新根段的起点（树表指针 86、映射根指针 86、树 ID 水位 8、回退下界 F 8）。
const JOURNAL_NEW_ROOT_SEGMENT_OFFSET: usize = 99;
/// 根记录里实例表指针的起点（86 字节的码 2 指针，出生树在指针偏移 34）。
const ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET: usize = 170;
/// 码 2 / 码 3 指针头里出生树的偏移。
const POINTER_BIRTH_TREE_OFFSET: usize = 34;
/// journal 记录头 487 字节（D23（journal 的角色与格式） 已定项 4），点名项从这里起、每项 56 字节（诞生代号在点名项偏移 37）。
const JOURNAL_RECORD_HEADER_BYTES: usize = 487;
/// 点名项宽（D23（journal 的角色与格式） 已定项 17）。
const JOURNAL_NAMED_ENTRY_BYTES_IN_THE_RECORD: usize = 56;

/// 发布的根（`judged_root_index` 那次根槽写）的 (实例代号, checkpoint_txg)：根记录偏移 24、28。
fn publish_root_identity(flow: &RecordedFlow) -> (u32, u64) {
    let bytes = flow.writes[flow.judged_root_index]
        .bytes()
        .expect("根槽写带字节");
    (
        u32::from_le_bytes(bytes[24..28].try_into().expect("4")),
        u64::from_le_bytes(bytes[28..36].try_into().expect("8")),
    )
}

/// 改一次根槽写的字节，再重封根记录的整槽校验和（罩整个槽宽）。
fn rewrite_root_record_and_reseal(
    flow: &mut RecordedFlow,
    index: usize,
    rewrite: &dyn Fn(&mut [u8]),
) {
    assert_eq!(
        flow.writes[index].kind,
        StepKind::RootRecordFua,
        "改的是根槽写"
    );
    let WrittenContents::Bytes(bytes) = &mut flow.writes[index].contents else {
        panic!("根槽写是普通写");
    };
    rewrite(bytes);
    reseal_root_record(bytes);
}

/// 改一次 journal 记录写的字节，再重封载荷校验和与头校验和；同盘同实例计数器大 1 的那一条的反向链跟着改、一路传下去。
fn rewrite_journal_record_and_reseal(
    flow: &mut RecordedFlow,
    index: usize,
    rewrite: &dyn Fn(&mut [u8]),
) {
    assert_eq!(
        flow.writes[index].kind,
        StepKind::JournalRecord,
        "改的是 journal 记录写"
    );
    let WrittenContents::Bytes(bytes) = &mut flow.writes[index].contents else {
        panic!("记录写是普通写");
    };
    rewrite(bytes);
    reseal_journal_record(bytes);
    carry_the_back_chain_forward_from(flow, vec![index]);
}

/// 发布那次的每一份 journal 记录（每块盘一份：实例与 checkpoint_txg 都等于发布的根）各改一次、重封：重放与逐盘判定读到的都是改过的那一份。
fn rewrite_every_journal_record_of_the_publish(
    flow: &mut RecordedFlow,
    rewrite: &dyn Fn(&mut [u8]),
) {
    let (instance, checkpoint_txg) = publish_root_identity(flow);
    let records: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| {
            write.kind == StepKind::JournalRecord
                && write.bytes().is_some_and(|bytes| {
                    u32::from_le_bytes(bytes[16..20].try_into().expect("4")) == instance
                        && u64::from_le_bytes(bytes[26..34].try_into().expect("8"))
                            == checkpoint_txg
                })
        })
        .map(|(index, _)| index)
        .collect();
    assert!(
        !records.is_empty(),
        "流 {} 里有发布那次（实例 {instance}、txg {checkpoint_txg}）的 journal 记录",
        flow.name
    );
    for index in records {
        rewrite_journal_record_and_reseal(flow, index, rewrite);
    }
}

/// 改流里最后一次系统配置槽写（它在那块盘上世代号最大，是那块盘择到的那一槽）的字节、重封整槽校验和。
fn rewrite_the_last_system_configuration_slot_and_reseal(
    flow: &mut RecordedFlow,
    rewrite: &dyn Fn(&mut Vec<u8>),
) {
    rewrite_and_reseal(
        flow,
        StepKind::SystemConfigurationSlot,
        0,
        4096,
        SYSTEM_CONFIGURATION_CHECKSUM_FIELD,
        rewrite,
    );
}

/// 在流尾另起一段，放这几次写（它们在最后全部持久的那个状态里都落了）。
fn append_a_segment_of_writes(flow: &mut RecordedFlow, writes: Vec<RetainedWrite>) {
    let first = flow.writes.len();
    let count = writes.len();
    flow.writes.extend(writes);
    flow.segments.push((first..first + count).collect());
}

/// I-1.12（指针出生树等于被指单元头）：发布的根记录里实例表指针的出生树加 1、重封根记录——实例表那一片单元头里的出生树是 0，指针不再与它相等。
fn record_new_pool_with_an_instance_table_pointer_naming_another_birth_tree(
    tag: &str,
) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-instance-table-pointer-birth-tree-plus-one");
    let publish_root = flow.judged_root_index;
    rewrite_root_record_and_reseal(&mut flow, publish_root, &|bytes| {
        let offset = ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + POINTER_BIRTH_TREE_OFFSET;
        bytes[offset] = bytes[offset].wrapping_add(1);
    });
    flow
}

/// I-1.12（指针出生树等于被指单元头）数据单元那一支：发布那片 extent 树上段叶第一条内联的数据指针（条目偏移 25，标签 2）出生树加 1、
/// 重封到根——数据单元五元组里的树（头偏移 43）不再等于它。
fn record_new_pool_with_a_data_pointer_naming_another_birth_tree(tag: &str) -> RecordedFlow {
    let extent_upper_leaf = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 24
            && index_node_entry_width(bytes) == 113
    };
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-data-pointer-birth-tree-plus-one",
        &extent_upper_leaf,
        false,
        &|bytes| {
            let first_entry = 115 + 2 * 24;
            assert_eq!(
                bytes[first_entry + 24],
                2,
                "上段叶第一条内联一个数据指针（标签 2）"
            );
            let birth_tree = first_entry + 25 + POINTER_BIRTH_TREE_OFFSET;
            bytes[birth_tree] = bytes[birth_tree].wrapping_add(1);
        },
    )
}

/// I-8.10（记录新根段与同代根相同）：发布那条记录新根段里的树 ID 水位加 1、重封——与同 (实例, txg) 的根里那 8 字节不再逐字节相同。
fn record_new_pool_with_a_last_record_carrying_another_watermark_than_its_root(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-last-record-watermark-plus-one");
    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
        let watermark = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + 86 + 86;
        bytes[watermark] = bytes[watermark].wrapping_add(1);
    });
    flow
}

/// I-8.11（点名项与被指单元头相同）：发布那条记录第一个点名项的诞生代号加 1、重封——它点名的单元整单元校验和照样对得上，头里的诞生代号不再等于它。
fn record_new_pool_with_a_named_entry_naming_another_birth_txg(tag: &str) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-named-entry-birth-txg-plus-one");
    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
        assert!(
            u32::from_le_bytes(bytes[12..16].try_into().expect("4")) >= 1,
            "发布那条记录至少点名一个单元"
        );
        let birth_txg = JOURNAL_RECORD_HEADER_BYTES + 37;
        bytes[birth_txg] = bytes[birth_txg].wrapping_add(1);
    });
    flow
}

/// I-4.9（非最新候选根引用的单元不晚于它）：暖机最后那条根的树表指针与映射根指针换成发布的根的、重封——发布的根落了的状态上，
/// 这条更早的候选根引用的树表与映射是发布那一代诞生的（经「别的版本先走过」那一支判到）。
fn record_new_pool_with_an_older_root_pointing_at_the_trees_of_the_publish(
    tag: &str,
) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-warm-up-root-points-at-the-publish-trees");
    let publish_root = flow.writes[flow.judged_root_index]
        .bytes()
        .expect("根槽写带字节")
        .to_vec();
    let warm_up_root = flow.writes[..flow.judged_root_index]
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("发布之前有暖机的根槽写");
    let (_, publish_txg) = publish_root_identity(&flow);
    rewrite_root_record_and_reseal(&mut flow, warm_up_root, &|bytes| {
        let warm_up_txg = u64::from_le_bytes(bytes[28..36].try_into().expect("8"));
        assert!(warm_up_txg < publish_txg, "暖机那条根比发布的根早");
        bytes[36..122].copy_from_slice(&publish_root[36..122]);
        bytes[256..342].copy_from_slice(&publish_root[256..342]);
    });
    flow
}

/// I-9.25（树表条目的 previous_snapshot_txg 取值）：发布那张树表里第一条不是 inode 树的条目，previous_snapshot_txg（条目偏移 100）写成 1、重封到根。
fn record_new_pool_with_a_previous_snapshot_txg_on_a_tree_other_than_the_inode_tree(
    tag: &str,
) -> RecordedFlow {
    let tree_table = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 8
            && index_node_entry_width(bytes) == 200
    };
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-previous-snapshot-txg-on-a-non-inode-tree",
        &tree_table,
        false,
        &|bytes| {
            let header_end = 86 + 2 * usize::from(bytes[51]);
            let count = usize::from(u16::from_le_bytes(
                bytes[header_end - 4..header_end - 2].try_into().expect("2"),
            ));
            let entry = (0..count)
                .map(|entry| 131 + entry * 200)
                .find(|offset| {
                    u16::from_le_bytes(bytes[offset + 10..offset + 12].try_into().expect("2")) != 2
                })
                .expect("树表里有不是 inode 树的条目");
            bytes[entry + 100] = 1;
        },
    )
}

/// I-9.27（树表条目的头 ID 取值）：发布那张树表里 extent 树那条（种类 1）的头 ID（条目偏移 116）写成 0、重封到根——
/// 它该等于同一张表里 inode 树条目的树 ID。
fn record_new_pool_with_the_head_identifier_of_the_extent_tree_entry_written_zero(
    tag: &str,
) -> RecordedFlow {
    let tree_table = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 8
            && index_node_entry_width(bytes) == 200
    };
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-extent-tree-entry-head-identifier-zero",
        &tree_table,
        false,
        &|bytes| {
            let header_end = 86 + 2 * usize::from(bytes[51]);
            let count = usize::from(u16::from_le_bytes(
                bytes[header_end - 4..header_end - 2].try_into().expect("2"),
            ));
            let entry = (0..count)
                .map(|entry| 131 + entry * 200)
                .find(|offset| {
                    u16::from_le_bytes(bytes[offset + 10..offset + 12].try_into().expect("2")) == 1
                })
                .expect("树表里有 extent 树那条");
            assert_ne!(
                bytes[entry + 116..entry + 124],
                [0u8; 8],
                "extent 树那条的头 ID 本来不是 0"
            );
            bytes[entry + 116..entry + 124].fill(0);
        },
    )
}

/// I-9.27 种类 2 那一支：发布那张树表里 inode 树那条（种类 2）的头 ID 从它自己的树 ID 改成树 ID + 1、重封到根。
fn record_new_pool_with_the_head_identifier_of_the_inode_tree_entry_off_its_own_tree_identifier(
    tag: &str,
) -> RecordedFlow {
    let tree_table = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 8
            && index_node_entry_width(bytes) == 200
    };
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-inode-tree-entry-head-identifier-off-its-own-tree-identifier",
        &tree_table,
        false,
        &|bytes| {
            let entry = only_tree_table_entry_of_kind(bytes, 2);
            let own_tree_identifier =
                u64::from_le_bytes(bytes[entry..entry + 8].try_into().expect("8"));
            assert_eq!(
                u64::from_le_bytes(bytes[entry + 116..entry + 124].try_into().expect("8")),
                own_tree_identifier,
                "inode 树那条的头 ID 本来等于它自己的树 ID"
            );
            bytes[entry + 116..entry + 124]
                .copy_from_slice(&(own_tree_identifier + 1).to_le_bytes());
        },
    )
}

/// I-9.27 写 0 那一支：发布那张树表里分配记录树那条（种类 3）的头 ID 从 0 改成 inode 树条目的树 ID、重封到根。
fn record_new_pool_with_a_non_zero_head_identifier_on_the_allocation_record_tree_entry(
    tag: &str,
) -> RecordedFlow {
    let tree_table = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 8
            && index_node_entry_width(bytes) == 200
    };
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-allocation-record-tree-entry-head-identifier-non-zero",
        &tree_table,
        false,
        &|bytes| {
            let entry = only_tree_table_entry_of_kind(bytes, 3);
            let inode_tree_entry = only_tree_table_entry_of_kind(bytes, 2);
            assert_eq!(
                bytes[entry + 116..entry + 124],
                [0u8; 8],
                "分配记录树那条的头 ID 本来是 0"
            );
            let inode_tree_identifier = bytes[inode_tree_entry..inode_tree_entry + 8].to_vec();
            bytes[entry + 116..entry + 124].copy_from_slice(&inode_tree_identifier);
        },
    )
}

/// 一张树表单元（码 2、条目宽 200）里种类是 `kind` 的那一条（恰好一条）在单元里的偏移。
fn only_tree_table_entry_of_kind(bytes: &[u8], kind: u16) -> usize {
    let header_end = 86 + 2 * usize::from(bytes[51]);
    let count = usize::from(u16::from_le_bytes(
        bytes[header_end - 4..header_end - 2].try_into().expect("2"),
    ));
    let entries_of_the_kind: Vec<usize> = (0..count)
        .map(|entry| 131 + entry * 200)
        .filter(|offset| {
            u16::from_le_bytes(bytes[offset + 10..offset + 12].try_into().expect("2")) == kind
        })
        .collect();
    let [entry] = entries_of_the_kind.as_slice() else {
        panic!("树表里种类 {kind} 的条目恰好一条：{entries_of_the_kind:?}")
    };
    *entry
}

/// I-5.5（从 mkfs 水位建树那一版的 t1 落点）：新池新建文件那一版唯一的数据单元（两份副本，单元区起始槽号 + 4）挪到再往后两槽
/// （下一个 32 KiB 对齐的槽对，这条流上空着）——两次单元写的偏移、流里每一处指着它的位置条目（extent 上段叶内联的数据指针、
/// 中央映射叶、journal 记录）的槽号一起改，单元重封到根、记录重封。树表与诞生 txg 一个字节不动，这一版照样被认出，
/// 每块盘上落点最小的数据单元在 U + 6、不是 U + 4。
fn record_new_pool_with_the_first_data_unit_two_slots_further(tag: &str) -> RecordedFlow {
    const SLOTS_MOVED: u64 = 2;
    let mut flow = record_new_pool_file_creation(tag, "new-pool-first-data-unit-two-slots-further");
    let data_unit_copies: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| {
            write.kind == StepKind::UnitWrite
                && write.bytes().is_some_and(|bytes| unit_class(bytes) == 1)
        })
        .map(|(index, _)| index)
        .collect();
    assert_eq!(
        data_unit_copies.len(),
        2,
        "新池新建文件只写一个数据单元，两块盘各一份"
    );
    // (旧的位置条目 14 字节, 新的槽号 6 字节)
    let renamed_locations: Vec<(Vec<u8>, [u8; 6])> = data_unit_copies
        .iter()
        .map(|index| {
            let write = &flow.writes[*index];
            let slot = write.offset.0 / 16384;
            let mut location = Vec::with_capacity(14);
            location.extend_from_slice(&write.device.0.to_le_bytes());
            location.extend_from_slice(&slot.to_le_bytes()[..6]);
            location.extend_from_slice(&crc32c(write.bytes().expect("单元写带字节")).to_le_bytes());
            let mut new_slot = [0u8; 6];
            new_slot.copy_from_slice(&(slot + SLOTS_MOVED).to_le_bytes()[..6]);
            (location, new_slot)
        })
        .collect();
    for index in &data_unit_copies {
        flow.writes[*index].offset.0 += SLOTS_MOVED * 16384;
    }
    let old_locations: Vec<Vec<u8>> = renamed_locations
        .iter()
        .map(|(location, _)| location.clone())
        .collect();
    let names_the_data_unit = |bytes: &[u8]| {
        old_locations.iter().any(|location| {
            bytes
                .windows(14)
                .any(|window| window == location.as_slice())
        })
    };
    let rename = move |bytes: &mut [u8]| {
        for (location, new_slot) in &renamed_locations {
            let mut position = 0;
            while position + 14 <= bytes.len() {
                if bytes[position..position + 14] == location[..] {
                    bytes[position + 4..position + 10].copy_from_slice(new_slot);
                    position += 14;
                } else {
                    position += 1;
                }
            }
        }
    };
    let unit_referrers: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| {
            write.kind == StepKind::UnitWrite
                && write
                    .bytes()
                    .is_some_and(|bytes| unit_class(bytes) != 1 && names_the_data_unit(bytes))
        })
        .map(|(index, _)| index)
        .collect();
    let journal_referrers: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| {
            write.kind == StepKind::JournalRecord && write.bytes().is_some_and(names_the_data_unit)
        })
        .map(|(index, _)| index)
        .collect();
    assert!(
        !unit_referrers.is_empty(),
        "流里有单元指着那个数据单元（extent 上段叶、中央映射叶）"
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &unit_referrers, &rename);
    for index in journal_referrers {
        rewrite_journal_record_and_reseal(&mut flow, index, &rename);
    }
    flow
}

/// I-3.13（记账行代段等于这一版的 txg）：发布那片记账叶第二条的代段（key 偏移 14）加 1、重封到根。取中间那条：
/// 首末两条的 key 是叶头里的区间，改了它们 I-1.1 先红。
fn record_new_pool_with_an_accounting_row_of_another_generation(tag: &str) -> RecordedFlow {
    let accounting_leaf = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 22
            && index_node_entry_width(bytes) == 34
    };
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-accounting-row-generation-plus-one",
        &accounting_leaf,
        false,
        &|bytes| {
            let header_end = 86 + 2 * usize::from(bytes[51]);
            let count = usize::from(u16::from_le_bytes(
                bytes[header_end - 4..header_end - 2].try_into().expect("2"),
            ));
            assert!(count >= 3, "记账叶至少三条，改中间那条（有 {count} 条）");
            let generation = 159 + 34 + 14;
            bytes[generation] = bytes[generation].wrapping_add(1);
        },
    )
}

/// I-9.26（inode 改动计数落在出生代与容器诞生代号之间）：发布那片 inode 叶第一条记录的改动计数（偏移 88）最高字节写成 0x7F、重封到根——
/// 远大于容器头的诞生代号。
fn record_new_pool_with_an_inode_change_count_past_its_container_birth(tag: &str) -> RecordedFlow {
    let inode_leaf = |bytes: &[u8]| unit_class(bytes) == 3 && packed_record_type(bytes) == 2;
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-inode-change-count-past-the-container-birth",
        &inode_leaf,
        false,
        &|bytes| {
            bytes[136 + 88 + 7] = 0x7F;
        },
    )
}

/// I-7.16（系统配置槽世代号与槽号的关系）：最后一次系统配置槽写的世代号（偏移 147）加 2、重封——奇偶照旧，与同盘另一槽相差 3。
fn record_new_pool_with_system_configuration_generations_three_apart(tag: &str) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-system-configuration-generation-plus-two");
    rewrite_the_last_system_configuration_slot_and_reseal(&mut flow, &|bytes| {
        bytes[147] = bytes[147].wrapping_add(2);
    });
    flow
}

/// I-7.17（根槽位置等于 txg 算出的位置）：发布的根原样再写一份到同一区域的下一个槽（流尾另起一段）——那个槽按 txg 算不该住它。
fn record_new_pool_with_a_root_copied_into_the_next_slot(tag: &str) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-publish-root-copied-into-the-next-slot");
    let publish_root = flow.writes[flow.judged_root_index].clone();
    let next_slot = singlefs_core::address::DeviceOffsetInBytes(
        publish_root.offset.0 + u64::from(geometry().fixed_structure_slot_spacing),
    );
    let next_slot_in_the_base = singlefs_core::recovery::PoolReader::read(
        &flow.base,
        publish_root.device,
        next_slot,
        usize::try_from(publish_root.length_in_bytes()).expect("根槽写的长度装得进 usize"),
    )
    .expect("基镜像里读得到下一个根槽");
    assert!(
        next_slot_in_the_base.iter().all(|byte| *byte == 0),
        "下一个根槽在基镜像里空着（发布的根在区域里不是最后一槽）"
    );
    assert!(
        flow.writes
            .iter()
            .all(|write| (write.device, write.offset) != (publish_root.device, next_slot)),
        "流里没有别的写落在下一个根槽上"
    );
    append_a_segment_of_writes(
        &mut flow,
        vec![RetainedWrite {
            device: publish_root.device,
            kind: publish_root.kind,
            is_force_unit_access: publish_root.is_force_unit_access,
            offset: next_slot,
            contents: publish_root.contents.clone(),
        }],
    );
    flow
}

/// I-7.15（各盘所选系统配置槽设备数相同）：最后一次系统配置槽写的设备数（偏移 143）改成 3、重封——那块盘择到的这一槽与别的盘不同。
fn record_new_pool_with_a_system_configuration_of_another_device_count(tag: &str) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-system-configuration-device-count-three");
    rewrite_the_last_system_configuration_slot_and_reseal(&mut flow, &|bytes| {
        bytes[143..147].copy_from_slice(&3u32.to_le_bytes());
    });
    flow
}

/// I-4.1（重放后树可遍历）：发布那条记录新根段里树表指针两条位置条目的整单元校验和各翻一遍、重封——点名单元照样验得过，
/// 发布的根没落而记录落了的状态上重放施加它，施加出来的那一版树表两份都对不上校验和、走不下去。
fn record_new_pool_with_a_replayed_version_whose_tree_table_cannot_be_read(
    tag: &str,
) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-replayed-tree-table-checksums-flipped");
    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
        for location_checksum in [
            JOURNAL_NEW_ROOT_SEGMENT_OFFSET + 60,
            JOURNAL_NEW_ROOT_SEGMENT_OFFSET + 74,
        ] {
            for byte in &mut bytes[location_checksum..location_checksum + 4] {
                *byte ^= 0xFF;
            }
        }
    });
    flow
}

/// I-8.1（环几何够大）三个字段那一支：最后一次系统配置槽写的安全系数 F（偏移 357）改成 2、重封——整池不作保，只红 I-8.1。
fn record_new_pool_with_a_safety_factor_of_two(tag: &str) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-system-configuration-safety-factor-2");
    rewrite_the_last_system_configuration_slot_and_reseal(&mut flow, &|bytes| {
        bytes[357..361].copy_from_slice(&2u32.to_le_bytes());
    });
    flow
}

/// 两支整池不作保都中：最后一次系统配置槽写的槽距改成 512（读者不收）、F 改成 2，重封——走 I-7.13 那一支，只红 I-7.13、I-8.1 报不适用。
fn record_new_pool_with_a_refused_slot_spacing_and_a_safety_factor_of_two(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(
        tag,
        "new-pool-system-configuration-spacing-512-and-safety-factor-2",
    );
    rewrite_the_last_system_configuration_slot_and_reseal(&mut flow, &|bytes| {
        bytes[429..433].copy_from_slice(&512u32.to_le_bytes());
        bytes[357..361].copy_from_slice(&2u32.to_le_bytes());
    });
    flow
}

/// 系统配置槽里区域数 R 与每区槽数 S 那两字节的偏移（`.claude/kb/layout/01-first-txn.md` 一：F 357 + 4 → R 361，S 紧随其后）。
const SYSTEM_CONFIGURATION_REGIONS_OFFSET: usize = 361;
const SYSTEM_CONFIGURATION_SLOTS_PER_REGION_OFFSET: usize = 362;

/// I-7.13（C663）：最后一次系统配置槽写的每区槽数 S 改成 17（区间 [4, 16] 之外）、重封——S 越界照 D22（单元原子性怎么合成） 已定项 2
/// 整池拒（用户 2026-10-20 定「S 照条款整池拒」），CPU 只红 I-7.13；GPU 走读内核读这一槽同判带不收之值。
fn record_new_pool_with_seventeen_slots_per_region(tag: &str) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-system-configuration-slots-per-region-17");
    rewrite_the_last_system_configuration_slot_and_reseal(&mut flow, &|bytes| {
        assert_eq!(
            bytes[SYSTEM_CONFIGURATION_SLOTS_PER_REGION_OFFSET], 8,
            "mkfs 写下的每区槽数是 8"
        );
        bytes[SYSTEM_CONFIGURATION_SLOTS_PER_REGION_OFFSET] = 17;
    });
    flow
}

/// I-7.13（C663）：最后一次系统配置槽写的区域数 R 改成 2（与第一版常量 3 不等）、重封——用户 2026-10-20 定「R 读并整池拒」，
/// CPU 只红 I-7.13；GPU 走读内核读这一槽同判带不收之值。
fn record_new_pool_with_two_regions(tag: &str) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-system-configuration-regions-2");
    rewrite_the_last_system_configuration_slot_and_reseal(&mut flow, &|bytes| {
        assert_eq!(
            bytes[SYSTEM_CONFIGURATION_REGIONS_OFFSET], 3,
            "mkfs 写下的区域数是 3"
        );
        bytes[SYSTEM_CONFIGURATION_REGIONS_OFFSET] = 2;
    });
    flow
}

/// I-2.4 系统配置槽尾（C728）：最后一次系统配置槽写的末字节（偏移 4095，字段表之后那一段 [489, 4096) 的最后一个）写 1、重封整槽校验和——
/// core 不读这一段，两遍恢复照常；CPU 池级 checker 对每个可择的槽判 [489, 4096) 全 0，那一槽落了的状态上红 I-2.4。
/// 取末字节：GPU 走读内核那一段先逐字节读到字边界、再按字读，末字节落在按字读那一截里（根槽尾那条流的偏移 457 落在逐字节那一截里）。
fn record_new_pool_with_a_non_zero_last_byte_in_a_system_configuration_slot(
    tag: &str,
) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-system-configuration-last-slot-byte-one");
    let last_byte_of_the_slot =
        usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES - 1).expect("4095");
    rewrite_the_last_system_configuration_slot_and_reseal(&mut flow, &|bytes| {
        assert_eq!(bytes[last_byte_of_the_slot], 0, "第一版字段表之后写 0");
        bytes[last_byte_of_the_slot] = 1;
    });
    flow
}

/// I-8.5（tail 槽有效集非空）：流尾另起一段，两块盘的两个系统配置槽各写一份全零——它们都落了的状态上一个自证过的槽都没有。
fn record_new_pool_with_every_system_configuration_slot_zeroed_at_the_end(
    tag: &str,
) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-every-system-configuration-slot-zeroed");
    let spacing = u64::from(geometry().fixed_structure_slot_spacing);
    let zeroed_slots: Vec<RetainedWrite> = [0u32, 1]
        .into_iter()
        .flat_map(|device| {
            [0, spacing].into_iter().map(move |offset| RetainedWrite {
                device: singlefs_core::address::DeviceIdentity(device),
                kind: StepKind::SystemConfigurationSlot,
                is_force_unit_access: false,
                offset: singlefs_core::address::DeviceOffsetInBytes(offset),
                contents: WrittenContents::Bytes(vec![0u8; 4096]),
            })
        })
        .collect();
    append_a_segment_of_writes(&mut flow, zeroed_slots);
    flow
}

/// I-7.5（根槽按判定宽度对齐）：最后一次系统配置槽写的 io_min（偏移 425）改成 12288、重封——判定宽度变成 12288，根环起点 1 MiB 不是它的整数倍。
fn record_new_pool_with_an_io_min_the_root_slots_are_not_aligned_to(tag: &str) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-system-configuration-io-min-12288");
    rewrite_the_last_system_configuration_slot_and_reseal(&mut flow, &|bytes| {
        bytes[425..429].copy_from_slice(&12288u32.to_le_bytes());
    });
    flow
}

/// I-3.12（实例表行 T 低于引用它的根的 txg）：回退流上最后一版实例表第一行的 T（行偏移 5，8 字节）最高字节加 1、重封到根——T 不小于引用它的根的 txg。
fn record_rollback_with_an_instance_table_row_not_below_its_root(tag: &str) -> RecordedFlow {
    let instance_table = |bytes: &[u8]| unit_class(bytes) == 3 && packed_record_type(bytes) == 4;
    record_rollback_with_a_resealed_field(
        tag,
        "rollback-instance-row-published-txg-past-its-root",
        &instance_table,
        &|bytes| {
            assert_eq!(bytes[136], 0, "实例表第一条是行（kind 0）");
            bytes[136 + 12] = bytes[136 + 12].wrapping_add(1);
        },
    )
}

/// I-8.12（同实例记录的 txg 不减）：回退流上最后一次 journal 记录写的 checkpoint_txg（偏移 26）改成 1、重封——同盘同实例更早的记录 txg 更大。
fn record_rollback_with_a_journal_record_txg_going_back(tag: &str) -> RecordedFlow {
    let mut flow = record_fixed_script_through_rollback(tag);
    flow.name = "rollback-last-journal-record-txg-one";
    let last_record = flow
        .writes
        .iter()
        .rposition(|write| write.kind == StepKind::JournalRecord)
        .expect("流里有 journal 记录写");
    let (device, instance) = {
        let write = &flow.writes[last_record];
        let bytes = write.bytes().expect("记录写带字节");
        (
            write.device,
            u32::from_le_bytes(bytes[16..20].try_into().expect("4")),
        )
    };
    assert!(
        flow.writes[..last_record].iter().any(|write| {
            write.kind == StepKind::JournalRecord
                && write.device == device
                && write.bytes().is_some_and(|bytes| {
                    u32::from_le_bytes(bytes[16..20].try_into().expect("4")) == instance
                        && u64::from_le_bytes(bytes[26..34].try_into().expect("8")) > 1
                })
        }),
        "同盘同实例有更早的、txg 大于 1 的记录"
    );
    rewrite_journal_record_and_reseal(&mut flow, last_record, &|bytes| {
        bytes[26..34].copy_from_slice(&1u64.to_le_bytes());
    });
    flow
}

// ───────────── 第 9 批：I-3.5 的出现规则、恢复施加一条记录取新根段的七段、施加出来的那一版从记录走实例表与分配记录树、I-8.10 后三段 ─────────────

/// journal 记录新根段里实例表指针的段内偏移（D23（journal 的角色与格式） 已定项 15：树表指针 86 + 映射根指针 86 + 水位 8 + F 8）。
const NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER: usize = 188;
/// 新根段里分配记录树根指针的段内偏移（实例表指针之后 86 字节）。
const NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER: usize = 274;
/// 新根段末尾卸载记号 4 字节的段内偏移（只有位 0，与根记录 flags 同一个编码）。
const NEW_ROOT_SEGMENT_UNMOUNT_MARKER: usize = 360;
/// 码 2 / 码 3 指针里两条位置条目各自的整单元校验和的偏移（位置条目 50 与 64 起，盘 4 + 槽 6 之后）。
const POINTER_LOCATION_CHECKSUM_OFFSETS: [usize; 2] = [60, 74];
/// 指针头里 extent 偏移 2 字节的偏移（MAC / nonce 28、算法类型 1、压缩码 1、压缩长度 2 之后）。
const POINTER_EXTENT_OFFSET_OFFSET: usize = 32;
/// 指针头里算法类型 1 字节的偏移（MAC / nonce 28 之后）。
const POINTER_ALGORITHM_TYPE_OFFSET: usize = 28;
/// 码 2 / 码 3 指针宽。
const NODE_POINTER_BYTES: usize = 86;
/// 数据单元头里诞生代号的偏移（码 1 写序五元组：树 43、inode 51、对象出生代 59、偏移 67、诞生代号 75）。
const DATA_UNIT_BIRTH_TXG_OFFSET: usize = 75;

/// I-3.5（引用区间的精确性）的出现规则：新池新建文件那条流上，发布那一版（txg 3）新写的数据单元头里的诞生代号 3 → 2、重封到根——
/// 发布那一版引用、暖机最后一版（txg 2）不引用的这个单元诞生代号不大于 2，它在 extent 树里，而 extent 树的根指针生在 txg 3、不是整棵继承下来的。
/// 指针、映射 key 与分配代还记 3，I-1.2、I-1.11、I-3.10 跟着红（与 `checker_known_bad_images.rs` 里 I-3.5 那一格同一个改法）。
fn record_new_pool_with_a_data_unit_born_no_later_than_the_previous_version(
    tag: &str,
) -> RecordedFlow {
    let data_unit = |bytes: &[u8]| unit_class(bytes) == 1;
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-data-unit-birth-txg-three-to-two",
        &data_unit,
        false,
        &|bytes| {
            let birth_txg = u64::from_le_bytes(
                bytes[DATA_UNIT_BIRTH_TXG_OFFSET..DATA_UNIT_BIRTH_TXG_OFFSET + 8]
                    .try_into()
                    .expect("8"),
            );
            assert_eq!(birth_txg, 3, "新建的文件那个数据单元生在发布那一版 txg 3");
            bytes[DATA_UNIT_BIRTH_TXG_OFFSET..DATA_UNIT_BIRTH_TXG_OFFSET + 8]
                .copy_from_slice(&2u64.to_le_bytes());
        },
    )
}

/// 发布那几条 journal 记录新根段里的卸载记号写成 0b10（位 0 之外有位为 1）、重封：core 的读者当记录损坏（重放不施加它），
/// CPU 池级 checker 照同一口径不收它（RR1）。
fn record_new_pool_with_a_last_record_whose_unmount_marker_sets_another_bit(
    tag: &str,
) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-last-record-unmount-marker-bit-one");
    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
        let marker = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + NEW_ROOT_SEGMENT_UNMOUNT_MARKER;
        assert_eq!(
            bytes[marker..marker + 4],
            [0u8; 4],
            "建文件那次发布不是卸载那一串，卸载记号是 0"
        );
        bytes[marker..marker + 4].copy_from_slice(&0b10u32.to_le_bytes());
    });
    flow
}

/// 发布那几条 journal 记录新根段里的卸载记号写成 1（只有位 0）、重封：记录照收、照施加，新根段与同代的根（flags 0）不逐字节相同，
/// CPU 池级 checker 红 I-8.10（第七段，卸载记号）。
fn record_new_pool_with_a_last_record_carrying_the_unmount_marker(tag: &str) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-last-record-unmount-marker-set");
    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
        let marker = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + NEW_ROOT_SEGMENT_UNMOUNT_MARKER;
        bytes[marker..marker + 4].copy_from_slice(&1u32.to_le_bytes());
    });
    flow
}

/// 发布那几条 journal 记录新根段里实例表指针的两条位置条目校验和各取反、重封：记录照收、照施加，施加出来那一版的实例表指针
/// 与所选根（暖机最后那条根）的不同、两份都读不对。core 的恢复从记录取实例表（已定项 15）走到文件之前就读不出；
/// CPU 池级 checker 走施加出来的那一版时实例表读不出（I-4.1），根也落了的状态上新根段与同代的根不逐字节相同（I-8.10 第五段）。
fn record_new_pool_with_a_last_record_whose_instance_table_pointer_reads_wrong(
    tag: &str,
) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-last-record-instance-table-checksums-flipped");
    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
        let pointer = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER;
        for location_checksum in POINTER_LOCATION_CHECKSUM_OFFSETS {
            for byte in &mut bytes[pointer + location_checksum..pointer + location_checksum + 4] {
                *byte ^= 0xFF;
            }
        }
    });
    flow
}

/// 发布那几条 journal 记录新根段里实例表指针头的 extent 偏移写成 1、重封：core 的读者当记录损坏（指针头不是第一版的取值，
/// `NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from`），重放不施加它；CPU 池级 checker 不判记录里的指针头、
/// 照收，根也落了的状态上红 I-8.10（第五段）。
fn record_new_pool_with_a_last_record_whose_instance_table_pointer_carries_an_extent_offset(
    tag: &str,
) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-last-record-instance-table-extent-offset-one");
    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
        let extent_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET
            + NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER
            + POINTER_EXTENT_OFFSET_OFFSET;
        bytes[extent_offset..extent_offset + 2].copy_from_slice(&1u16.to_le_bytes());
    });
    flow
}

/// 发布那几条 journal 记录新根段里分配记录树根指针（建文件那一版本来全零）换成外来的字节：同一条记录的实例表指针、
/// 两条位置条目校验和各取反，重封。施加出来那一版的分配记录树根与它施加在其上的那条根（全零）不同：CPU 池级 checker 走那一版时
/// 从记录取它、两份都读不对（I-4.1），根也落了的状态上新根段与同代的根不逐字节相同（I-8.10 第六段）。
fn record_new_pool_with_a_last_record_whose_allocation_record_tree_root_is_foreign(
    tag: &str,
) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-last-record-allocation-root-foreign-bytes");
    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
        let instance_table =
            JOURNAL_NEW_ROOT_SEGMENT_OFFSET + NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER;
        let allocation_root =
            JOURNAL_NEW_ROOT_SEGMENT_OFFSET + NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER;
        assert_eq!(
            bytes[allocation_root..allocation_root + NODE_POINTER_BYTES],
            [0u8; NODE_POINTER_BYTES],
            "带文件的一版分配记录树根恒全零"
        );
        let foreign = bytes[instance_table..instance_table + NODE_POINTER_BYTES].to_vec();
        bytes[allocation_root..allocation_root + NODE_POINTER_BYTES].copy_from_slice(&foreign);
        for location_checksum in POINTER_LOCATION_CHECKSUM_OFFSETS {
            for byte in &mut bytes
                [allocation_root + location_checksum..allocation_root + location_checksum + 4]
            {
                *byte ^= 0xFF;
            }
        }
    });
    flow
}

/// GPU 判器与 CPU 判器在这条流的每个状态上红位与违例位逐位相同、落到同一条根，CPU 池级 checker 在这条流上红 `invariant`。
/// 本机没有 NVIDIA 独显适配器时报「本次未跑」返回。
fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
    test_name: &str,
    flow: &RecordedFlow,
    invariant: &'static str,
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains(invariant),
        "流 {} 上 CPU 池级 checker 要红 {invariant}（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        comparison
            .mismatches
            .iter()
            .filter(|line| !line.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// 结局表（[`SCAN_REBUILD_OUTCOMES`]）里名字是 `constant_name` 的那一种的码。
fn scan_rebuild_outcome_code_named(constant_name: &str) -> u32 {
    SCAN_REBUILD_OUTCOMES
        .iter()
        .find(|outcome| outcome.constant_name == constant_name)
        .unwrap_or_else(|| panic!("结局表里有 {constant_name}"))
        .code
}

/// 这条流上有几个状态 GPU 交回的 I-1.13 结局码（结论的 [`SCAN_REBUILD_OUTCOME_WORD`]）是 `constant_name` 那一种。
/// 对拍逐状态整份比（对拍字带这一字），没有不一致时 CPU 在同几个状态上换出的也是这一种。
fn states_whose_scan_rebuild_ends_with(comparison: &Comparison, constant_name: &str) -> usize {
    let code = scan_rebuild_outcome_code_named(constant_name);
    comparison
        .gpu_words_of_each_state
        .iter()
        .filter(|words| words[SCAN_REBUILD_OUTCOME_WORD] == code)
        .count()
}

/// 同 [`assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening`]，再要这条流上至少一个状态两边判 I-1.13 都是池级 checker
/// 走到扫描重建之前整池交回的 `outcome_constant_name` 那一种（码 27 到 30：走读内核在那几处早退写的码，对拍逐位比它）。
fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_and_ending_the_scan_rebuild_with(
    test_name: &str,
    flow: &RecordedFlow,
    invariant: &'static str,
    outcome_constant_name: &str,
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains(invariant),
        "流 {} 上 CPU 池级 checker 要红 {invariant}（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
    assert!(
        states_whose_scan_rebuild_ends_with(&comparison, outcome_constant_name) > 0,
        "流 {} 上至少一个状态两边判 I-1.13 都是 {outcome_constant_name}",
        flow.name
    );
}

/// 一次恢复报出的那条根（(实例代号, checkpoint_txg)）：读出、没有文件两种结局带的那一条，失败时带的那一条（择不到根时没有）。
fn root_reported_by_the_recovery(outcome: &RecoveryOutcome) -> Option<(u32, u64)> {
    match outcome {
        RecoveryOutcome::FileRead { root, .. } | RecoveryOutcome::NoFile { root } => {
            Some((root.0 .0, root.1 .0))
        }
        RecoveryOutcome::Failed { root, .. } => root.map(|(instance, txg)| (instance.0, txg.0)),
    }
}

/// 恢复字（[`recovery_words_of`]）第 2、3 字的结局类别：没有文件、读出、失败。
const RECOVERY_WORD_CLASS_NO_FILE: u32 = 0;
const RECOVERY_WORD_CLASS_FILE_READ: u32 = 1;
const RECOVERY_WORD_CLASS_FAILED: u32 = 2;

fn oracle_index(kind: CrashPointReplayOracleViolationKind) -> u32 {
    u32::try_from(
        CrashPointReplayOracleViolationKind::EVERY_KIND
            .iter()
            .position(|candidate| *candidate == kind)
            .expect("七类之一"),
    )
    .expect("装得进 u32")
}

/// CPU 判定 → GPU 那一套红位。
fn expected_bits(judgement: &CrashStateJudgement) -> u32 {
    let mut bits = 0u32;
    if let Some(violation) = &judgement.consulted_oracle {
        bits |= 1 << (verdict_bits::CONSULTED_ORACLE_FIRST + oracle_index(violation.kind));
    }
    if let Some(violation) = &judgement.ignored_oracle {
        bits |= 1 << (verdict_bits::IGNORED_ORACLE_FIRST + oracle_index(violation.kind));
    }
    if judgement.records.root_without_record {
        bits |= 1 << verdict_bits::ROOT_WITHOUT_RECORD;
    }
    if judgement.records.claimed_state_missing_unit {
        bits |= 1 << verdict_bits::CLAIMED_STATE_MISSING_UNIT;
    }
    bits
}

/// CPU 池级 checker 的违例位：位 i ↔ `IMPLEMENTED_INVARIANTS[i]`；经 `pool_checker_violated_and_judged_bits` 取（与 CPU 一侧的对拍字同一个函数），
/// 内核不置位的那几条照判定置位，比较处再去掉。
fn expected_pool_checker_bits(judgement: &CrashStateJudgement) -> u128 {
    let (violated, _judged) = pool_checker_violated_and_judged_bits(&judgement.pool_checker);
    violated
}

/// CPU 池级 checker 在这个状态上判过而成立的不变量（不是报不适用）：一位一条，与违例位同一个下标；经同一个函数取，判到而没违例的那几位。
fn expected_pool_checker_held_bits(judgement: &CrashStateJudgement) -> u128 {
    let (violated, judged) = pool_checker_violated_and_judged_bits(&judgement.pool_checker);
    judged & !violated
}

fn invariant_names(bits: u128) -> Vec<&'static str> {
    IMPLEMENTED_INVARIANTS
        .iter()
        .enumerate()
        .filter(|(index, _)| bits & (1u128 << index) != 0)
        .map(|(_, name)| *name)
        .collect()
}

/// 一份红项（与 `CrashStateJudgement::red_items` 同名、同序）在对拍里比的那一份，排成 `{:?}`：去掉内核不置位的那几条池级 checker 红项
/// （`POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS`，第一版只有 I-1.13，C821），别的照原序留着。
/// 与违例位比较处的 `pool_checker_bits_agree_in_the_cross_check` 同一张表：照流水线那样判不了就回落 CPU 的用例两边都经它再比，
/// CPU 红 I-1.13 的状态上 GPU 交回的红项里没有它，不算不一致。
fn red_items_compared_by_the_cross_check(items: &[String]) -> String {
    let left_out: Vec<String> = POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS
        .iter()
        .map(|index| {
            format!(
                "{CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER}:{}",
                IMPLEMENTED_INVARIANTS[*index]
            )
        })
        .collect();
    let compared: Vec<&String> = items
        .iter()
        .filter(|item| !left_out.contains(item))
        .collect();
    format!("{compared:?}")
}

struct Comparison {
    states: u64,
    cpu_red: u64,
    gpu_red: u64,
    cpu_checker_red: u64,
    gpu_checker_red: u64,
    /// CPU 池级 checker 在这条流的哪几条不变量上红过。
    cpu_checker_red_invariants: std::collections::BTreeSet<&'static str>,
    /// CPU 池级 checker 在这条流的哪几条不变量上判过而成立过（至少一个状态上不是报不适用）。
    cpu_checker_held_invariants: std::collections::BTreeSet<&'static str>,
    /// CPU 池级 checker 在每个状态上的违例位（按状态序号）：要钉「同一个状态上红这一条、不红那一条」的用例读它。
    cpu_checker_bits_of_each_state: Vec<u128>,
    /// CPU 判器在每个状态上的红项（`CrashStateJudgement::red_items` 的 `{:?}`，按状态序号）。
    cpu_red_items_of_each_state: Vec<String>,
    /// 同一份红项去掉内核不置位的那几条池级 checker 红项之后的样子（[`red_items_compared_by_the_cross_check`]，按状态序号）：
    /// 照流水线那样判不了就回落 CPU 的用例拿它与 GPU 交回的红项比。
    cpu_red_items_compared_by_the_cross_check_of_each_state: Vec<String>,
    /// GPU 在每个状态上交回的结论字（按状态序号）：要按判不了的原因分开看、或照流水线那样判不了就回落 CPU 的用例读它。
    gpu_words_of_each_state: Vec<[u32; VERDICT_WORDS]>,
    /// CPU 看 journal 那一遍在每个状态上报的根（`RecoveryOutcome` 里的那一条，(实例代号, checkpoint_txg)；退回见证到的那一版时是退到的那一版，
    /// 择不到根时 `None`），按状态序号：要钉「落到哪一版、所选根是谁」的用例读它。
    cpu_consulted_reported_root_of_each_state: Vec<Option<(u32, u64)>>,
    /// CPU 在每个状态上的恢复字（[`recovery_words_of`]：看 journal 那一遍实际走的那条根的实例与 txg 低 32 位、两遍的结局类别 0 没有文件 / 1 读出 / 2 失败），
    /// 按状态序号。
    cpu_recovery_words_of_each_state: Vec<[u32; 4]>,
    /// 看 journal 那一遍落到的根或两遍的结局类别与 CPU 不同的状态数（算进不一致）。
    recovery_mismatches: u64,
    /// 对拍字（`CrossCheckWords`：红项、两遍各自落到的根、结局、失败点、池级 checker 判到位、判不了）与 CPU 不同的状态数；
    /// 不一致（`mismatches`）照它判（判决 m3-crash-enumeration-merge-r3 第三节 J6，用户 2026-10-10 定失败点这一版就比）。
    cross_check_mismatches: u64,
    mismatches: Vec<String>,
    /// 只看池级 checker 的不一致：违例位与 CPU 不同、或判不了的状态（前 12 个带说明，之后记空串，同 `mismatches`）。
    checker_mismatches: Vec<String>,
}

/// [`Comparison`] 写进对拍结论库的字节（小端；集合与表先写条数，串先写字节数）：逐字段按定义的次序。
impl Comparison {
    fn to_bytes(&self) -> Vec<u8> {
        fn put_u64(bytes: &mut Vec<u8>, value: u64) {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        fn put_length(bytes: &mut Vec<u8>, length: usize) {
            put_u64(bytes, u64::try_from(length).expect("条数装得进 u64"));
        }
        fn put_text(bytes: &mut Vec<u8>, text: &str) {
            put_length(bytes, text.len());
            bytes.extend_from_slice(text.as_bytes());
        }
        let mut bytes = Vec::new();
        for count in [
            self.states,
            self.cpu_red,
            self.gpu_red,
            self.cpu_checker_red,
            self.gpu_checker_red,
            self.recovery_mismatches,
            self.cross_check_mismatches,
        ] {
            put_u64(&mut bytes, count);
        }
        for invariants in [
            &self.cpu_checker_red_invariants,
            &self.cpu_checker_held_invariants,
        ] {
            put_length(&mut bytes, invariants.len());
            for invariant in invariants {
                put_text(&mut bytes, invariant);
            }
        }
        put_length(&mut bytes, self.cpu_checker_bits_of_each_state.len());
        for bits in &self.cpu_checker_bits_of_each_state {
            bytes.extend_from_slice(&bits.to_le_bytes());
        }
        for texts in [
            &self.cpu_red_items_of_each_state,
            &self.cpu_red_items_compared_by_the_cross_check_of_each_state,
            &self.mismatches,
            &self.checker_mismatches,
        ] {
            put_length(&mut bytes, texts.len());
            for text in texts {
                put_text(&mut bytes, text);
            }
        }
        put_length(&mut bytes, self.gpu_words_of_each_state.len());
        for words in &self.gpu_words_of_each_state {
            for word in words {
                bytes.extend_from_slice(&word.to_le_bytes());
            }
        }
        put_length(
            &mut bytes,
            self.cpu_consulted_reported_root_of_each_state.len(),
        );
        for root in &self.cpu_consulted_reported_root_of_each_state {
            match root {
                Some((instance, checkpoint_txg)) => {
                    bytes.push(1);
                    bytes.extend_from_slice(&instance.to_le_bytes());
                    put_u64(&mut bytes, *checkpoint_txg);
                }
                None => bytes.push(0),
            }
        }
        put_length(&mut bytes, self.cpu_recovery_words_of_each_state.len());
        for words in &self.cpu_recovery_words_of_each_state {
            for word in words {
                bytes.extend_from_slice(&word.to_le_bytes());
            }
        }
        bytes
    }

    /// [`Self::to_bytes`] 的逆；字节不够、不变量名不在清单里、串不是 UTF-8、后面还剩字节都交 `None`。
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        struct Reader<'bytes> {
            rest: &'bytes [u8],
        }
        impl<'bytes> Reader<'bytes> {
            fn take(&mut self, count: usize) -> Option<&'bytes [u8]> {
                if self.rest.len() < count {
                    return None;
                }
                let (taken, rest) = self.rest.split_at(count);
                self.rest = rest;
                Some(taken)
            }
            fn u32(&mut self) -> Option<u32> {
                Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
            }
            fn u64(&mut self) -> Option<u64> {
                Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))
            }
            fn u128(&mut self) -> Option<u128> {
                Some(u128::from_le_bytes(self.take(16)?.try_into().ok()?))
            }
            fn length(&mut self) -> Option<usize> {
                usize::try_from(self.u64()?).ok()
            }
            fn text(&mut self) -> Option<String> {
                let length = self.length()?;
                String::from_utf8(self.take(length)?.to_vec()).ok()
            }
            fn invariants(&mut self) -> Option<std::collections::BTreeSet<&'static str>> {
                (0..self.length()?)
                    .map(|_| {
                        let name = self.text()?;
                        IMPLEMENTED_INVARIANTS
                            .iter()
                            .find(|invariant| **invariant == name)
                            .copied()
                    })
                    .collect()
            }
            fn texts(&mut self) -> Option<Vec<String>> {
                (0..self.length()?).map(|_| self.text()).collect()
            }
        }
        let mut reader = Reader { rest: bytes };
        let states = reader.u64()?;
        let cpu_red = reader.u64()?;
        let gpu_red = reader.u64()?;
        let cpu_checker_red = reader.u64()?;
        let gpu_checker_red = reader.u64()?;
        let recovery_mismatches = reader.u64()?;
        let cross_check_mismatches = reader.u64()?;
        let cpu_checker_red_invariants = reader.invariants()?;
        let cpu_checker_held_invariants = reader.invariants()?;
        let cpu_checker_bits_of_each_state = (0..reader.length()?)
            .map(|_| reader.u128())
            .collect::<Option<Vec<u128>>>()?;
        let cpu_red_items_of_each_state = reader.texts()?;
        let cpu_red_items_compared_by_the_cross_check_of_each_state = reader.texts()?;
        let mismatches = reader.texts()?;
        let checker_mismatches = reader.texts()?;
        let gpu_words_of_each_state = (0..reader.length()?)
            .map(|_| {
                let mut words = [0u32; VERDICT_WORDS];
                for word in &mut words {
                    *word = reader.u32()?;
                }
                Some(words)
            })
            .collect::<Option<Vec<[u32; VERDICT_WORDS]>>>()?;
        let cpu_consulted_reported_root_of_each_state = (0..reader.length()?)
            .map(|_| match reader.take(1)?[0] {
                0 => Some(None),
                1 => Some(Some((reader.u32()?, reader.u64()?))),
                _ => None,
            })
            .collect::<Option<Vec<Option<(u32, u64)>>>>()?;
        let cpu_recovery_words_of_each_state = (0..reader.length()?)
            .map(|_| Some([reader.u32()?, reader.u32()?, reader.u32()?, reader.u32()?]))
            .collect::<Option<Vec<[u32; 4]>>>()?;
        if !reader.rest.is_empty() {
            return None;
        }
        Some(Self {
            states,
            cpu_red,
            gpu_red,
            cpu_checker_red,
            gpu_checker_red,
            cpu_checker_red_invariants,
            cpu_checker_held_invariants,
            cpu_checker_bits_of_each_state,
            cpu_red_items_of_each_state,
            cpu_red_items_compared_by_the_cross_check_of_each_state,
            gpu_words_of_each_state,
            cpu_consulted_reported_root_of_each_state,
            cpu_recovery_words_of_each_state,
            recovery_mismatches,
            cross_check_mismatches,
            mismatches,
            checker_mismatches,
        })
    }
}

fn compare_on_every_state(
    cards: &'static GpuCardPool,
    flow: &RecordedFlow,
    limit: usize,
) -> Option<Comparison> {
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= limit {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    compare_on_every_state_of_these_segments(cards, flow, &flow.segments, &expansion)
}

/// 同 [`compare_on_every_state`]，切段与每段怎么展开由调用方给（`segments` 与 `flow.segments` 罩同一组写、只是分法不同时，
/// 状态仍是这条流全量枚举里的状态）。
/// 表建好之后才向卡池借卡，借的量是这条流的判器实际要的（[`mebibytes_a_judge_needs`]，一次派活至多一条草稿缓冲那么宽）；
/// GPU 判完就还，CPU 那一侧判的时候不占着卡。建判器、判的时候卡上 wgpu 报了 Out of Memory：打「本次未跑」那一行（用例名那一格写流名）、
/// 交 `None`，调用的用例当场返回（[`on_the_card_or_report_not_run_when_out_of_memory`]）；这一族四个函数同此。
fn compare_on_every_state_of_these_segments(
    cards: &'static GpuCardPool,
    flow: &RecordedFlow,
    segments: &[Vec<usize>],
    expansion: &(dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion + Sync),
) -> Option<Comparison> {
    compare_on_every_state_of_these_segments_against_the_version_table(
        cards,
        flow,
        &[],
        segments,
        expansion,
    )
}

/// 同 [`compare_on_every_state_of_these_segments`]，两类库里的项名由调用方给：同一条流按几种切法各比一次时每种切法一个项名——
/// 一个项名下的清单只认一份，撞了名判段会读到别的切法那一份（`crash_library` 的两段协议当场拒）。
fn compare_on_every_state_of_these_segments_as_the_item(
    cards: &'static GpuCardPool,
    item_name: &str,
    flow: &RecordedFlow,
    segments: &[Vec<usize>],
    expansion: &(dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion + Sync),
) -> Option<Comparison> {
    compare_on_every_state_of_these_segments_against_the_version_table_given_the_container_capacity_knobs(
        cards,
        flow,
        item_name,
        &[],
        segments,
        expansion,
        ContainerCapacityKnobs::PRODUCT,
    )
}

/// 同 [`compare_on_every_state`]，版本表另带文件从那一版起没有了的那几版（删除流的意图那一版）：GPU 的期望版本表
/// （`crash_judge_tables::build_judge_tables_against_the_version_table`）与 CPU 的 oracle（`crash::CrashPointReplayVersionTable`）都认它们。
fn compare_on_every_state_against_the_version_table(
    cards: &'static GpuCardPool,
    flow: &RecordedFlow,
    versions_without_a_file: &[PublishedVersionWithoutAFile],
    limit: usize,
) -> Option<Comparison> {
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= limit {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    compare_on_every_state_of_these_segments_against_the_version_table(
        cards,
        flow,
        versions_without_a_file,
        &flow.segments,
        &expansion,
    )
}

/// [`compare_on_every_state_of_these_segments`] 的本体，版本表照 `versions_without_a_file` 另带「没有文件的那几版」；三棵容器树都没装旋钮。
fn compare_on_every_state_of_these_segments_against_the_version_table(
    cards: &'static GpuCardPool,
    flow: &RecordedFlow,
    versions_without_a_file: &[PublishedVersionWithoutAFile],
    segments: &[Vec<usize>],
    expansion: &(dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion + Sync),
) -> Option<Comparison> {
    compare_on_every_state_of_these_segments_against_the_version_table_given_the_container_capacity_knobs(
        cards,
        flow,
        flow.name,
        versions_without_a_file,
        segments,
        expansion,
        ContainerCapacityKnobs::PRODUCT,
    )
}

/// 同 [`compare_on_every_state`]，三棵容器树的旋钮值 `knobs` 编进 GPU 扫描内核（`JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs`，
/// 第 56 批 G6），CPU 那一侧的池级 checker 违例位改按带同一组旋钮值的新入口（`check_pool_image_given_the_container_capacity_knobs`）在同一张崩溃镜像上判；
/// 红位与恢复字照旧按 CPU 判器（`judge_crash_image`，池级 checker 走产品入口的那一份不进比较）。
fn compare_on_every_state_given_the_container_capacity_knobs(
    cards: &'static GpuCardPool,
    flow: &RecordedFlow,
    knobs: ContainerCapacityKnobs,
) -> Option<Comparison> {
    let limit = expand_up_to();
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= limit {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    compare_on_every_state_of_these_segments_against_the_version_table_given_the_container_capacity_knobs(
        cards,
        flow,
        flow.name,
        &[],
        &flow.segments,
        &expansion,
        knobs,
    )
}

/// 池级 checker 一份判定的违例位与判过而成立的位（位 i ↔ `IMPLEMENTED_INVARIANTS[i]`）：经 `pool_checker_violated_and_judged_bits` 取。
fn violated_and_held_bits_of(verdicts: &[(&'static str, InvariantVerdict)]) -> (u128, u128) {
    let (violated, judged) = pool_checker_violated_and_judged_bits(verdicts);
    (violated, judged & !violated)
}

/// [`compare_on_every_state_of_these_segments_against_the_version_table`] 的本体：`knobs` 是 [`ContainerCapacityKnobs::PRODUCT`] 时编判法摘要罩住的那三份内核、
/// CPU 池级 checker 取判器那一份（产品入口）；否则见 [`compare_on_every_state_given_the_container_capacity_knobs`]。
/// CPU 那一侧（带旋钮值重判池级 checker 的那一处在内）判的是从崩溃点重放枚举那一类的库里读回的流叠出来的镜像。
/// 对拍结论入库（用户 2026-10-15 定「对拍结论入库」，`crash_library::pairing_conclusion_reused_or_judged`）：键是这一次对拍的输入摘要
/// （[`pairing_inputs_digest`]）与两边判器那一组判法单元的键（GPU 判器、CPU 判器判崩溃镜像），库里有就复用、一个状态都不再判；没有就两边
/// 都判、比、整份比较结果（[`Comparison::to_bytes`]）按键存进库。两边哪一边的判法单元拒算：照判、不入库。
/// 卡池借不到卡、卡上显存不够、只录那一趟（判那一侧报「本次未跑」交 `None` 的几种）不存进库：判的那一步当场以 [`PairingNotRun`]
/// 收手，这里接住、照旧交 `None`。
// cpu-reference-for-gpu-comparison
fn compare_on_every_state_of_these_segments_against_the_version_table_given_the_container_capacity_knobs(
    cards: &'static GpuCardPool,
    flow: &RecordedFlow,
    item_name: &str,
    versions_without_a_file: &[PublishedVersionWithoutAFile],
    segments: &[Vec<usize>],
    expansion: &(dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion + Sync),
    knobs: ContainerCapacityKnobs,
) -> Option<Comparison> {
    let judge_flow = JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion,
    };
    let tables_started = Instant::now();
    let tables = build_judge_tables_against_the_version_table(&judge_flow, versions_without_a_file)
        .expect("建得出表");
    let state_count =
        crash_point_replay_plan_state_count(&flow.base, &flow.writes, segments, expansion);
    assert_eq!(tables.state_count, state_count);
    let words = tables.words();
    eprintln!(
        "CRASH_JUDGE_TABLES flow={} states={state_count} locations={} version_bytes_words={} built_in={:?}",
        flow.name,
        tables.locations.len(),
        words.version_bytes.len(),
        tables_started.elapsed()
    );
    let judging_units = |units: JudgingUnits| {
        judging_units_key(&units, toolchain_of_this_process()).map_err(|refused| refused.sentence())
    };
    // 比法那一组的键拼进输入摘要；它拒算时整份对拍照判、不入库（拒算交进一边）
    let comparison_units = judging_units(JudgingUnits::PairingComparison {
        group_name: "gpu_and_cpu_judge_pairing_comparison",
    });
    let mut inputs = pairing_inputs_digest(
        flow,
        versions_without_a_file,
        segments,
        expansion,
        knobs,
        &tables,
    )
    .to_vec();
    inputs.extend_from_slice(
        comparison_units
            .as_ref()
            .map_or(&[0u8; DIGEST_BYTES], |units| units.fingerprint()),
    );
    let inputs_digest = sha256_digest(&inputs);
    let library =
        CrashLibrary::open(CrashLibraryClass::JudgeTablesPairing).expect("对拍那一类的库开得了");
    let paired = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        pairing_conclusion_reused_or_judged(
            &library,
            &inputs_digest,
            &comparison_units.and_then(|_| judging_units(JudgingUnits::GpuJudge)),
            &judging_units(JudgingUnits::JudgementOfTheCrashImages),
            || {
                judge_on_the_cpu_and_the_gpu_and_compare(
                    cards,
                    flow,
                    item_name,
                    versions_without_a_file,
                    segments,
                    expansion,
                    knobs,
                    &tables,
                )
                .unwrap_or_else(|| std::panic::panic_any(PairingNotRun))
                .to_bytes()
            },
        )
    }));
    match paired {
        Ok(Ok((conclusion, _source))) => {
            Some(Comparison::from_bytes(&conclusion).expect("库里的对拍结论是这份测试写的，认得回"))
        }
        Ok(Err(reason)) => panic!("流 {} 的对拍结论读写库失败：{reason}", flow.name),
        Err(payload) if payload.is::<PairingNotRun>() => None,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

/// CPU 判器每个状态交回的那几样（红位、池级 checker 违例位、红项、恢复报告、恢复字、判过而成立的位、看 journal 那一遍报的根、对拍字、
/// 对拍比的那几条红项），按状态序号排好交给 [`compare_the_judgements_state_by_state`]。
type JudgedOnCpu = (
    u32,
    u128,
    String,
    String,
    [u32; 4],
    u128,
    Option<(u32, u64)>,
    CrossCheckWords,
    String,
);

/// 逐状态比 CPU 与 GPU 的判定（对拍的比法）：红位、池级 checker 违例位与判到位、两遍恢复落到的根与结局、对拍字；不一致的状态逐个打一行、
/// 前 12 个带说明。这一段的函数单元另成一组判法单元（`gpu_and_cpu_judge_pairing_comparison`），它的键拼进对拍结论的输入摘要：
/// 比法改了，库里早先的对拍结论不再复用。
fn compare_the_judgements_state_by_state(
    flow_name: &str,
    state_count: u64,
    cpu: &[JudgedOnCpu],
    gpu: &[[u32; VERDICT_WORDS]],
) -> Comparison {
    let mut comparison = Comparison {
        states: state_count,
        cpu_red: 0,
        gpu_red: 0,
        cpu_checker_red: 0,
        gpu_checker_red: 0,
        cpu_checker_red_invariants: std::collections::BTreeSet::new(),
        cpu_checker_held_invariants: std::collections::BTreeSet::new(),
        cpu_checker_bits_of_each_state: Vec::with_capacity(cpu.len()),
        cpu_red_items_of_each_state: Vec::with_capacity(cpu.len()),
        cpu_red_items_compared_by_the_cross_check_of_each_state: Vec::with_capacity(cpu.len()),
        gpu_words_of_each_state: gpu.to_vec(),
        cpu_consulted_reported_root_of_each_state: Vec::with_capacity(cpu.len()),
        cpu_recovery_words_of_each_state: Vec::with_capacity(cpu.len()),
        recovery_mismatches: 0,
        cross_check_mismatches: 0,
        mismatches: Vec::new(),
        checker_mismatches: Vec::new(),
    };
    for (
        ordinal,
        (
            cpu_bits,
            cpu_checker_bits,
            cpu_items,
            cpu_report,
            cpu_recovery,
            cpu_checker_held_bits,
            cpu_consulted_reported_root,
            cpu_cross_check_words,
            cpu_items_compared_by_the_cross_check,
        ),
    ) in cpu.iter().enumerate()
    {
        comparison
            .cpu_consulted_reported_root_of_each_state
            .push(*cpu_consulted_reported_root);
        comparison
            .cpu_recovery_words_of_each_state
            .push(*cpu_recovery);
        comparison
            .cpu_checker_held_invariants
            .extend(invariant_names(*cpu_checker_held_bits));
        comparison
            .cpu_checker_bits_of_each_state
            .push(*cpu_checker_bits);
        comparison
            .cpu_red_items_of_each_state
            .push(cpu_items.clone());
        comparison
            .cpu_red_items_compared_by_the_cross_check_of_each_state
            .push(cpu_items_compared_by_the_cross_check.clone());
        let gpu_words: [u32; VERDICT_WORDS] = gpu[ordinal];
        let gpu_bits = gpu_words[0] & !(1 << verdict_bits::UNDECIDABLE);
        let undecidable = gpu_words[0] & (1 << verdict_bits::UNDECIDABLE) != 0
            || gpu_words[POOL_CHECKER_UNDECIDABLE_WORD] != 0;
        let gpu_checker_bits = pool_checker_violated_bits(&gpu_words);
        if *cpu_bits != 0 {
            comparison.cpu_red += 1;
        }
        if gpu_bits != 0 {
            comparison.gpu_red += 1;
        }
        if *cpu_checker_bits != 0 {
            comparison.cpu_checker_red += 1;
            comparison
                .cpu_checker_red_invariants
                .extend(invariant_names(*cpu_checker_bits));
        }
        if gpu_checker_bits != 0 {
            comparison.gpu_checker_red += 1;
        }
        // 落到的根与两遍的结局也要相同：只比红位看不出「两遍恢复落到同一条根」这类漏洞（重放没扫环那一次就是这样漏的）
        let gpu_recovery = gpu_recovery_words(&gpu_words);
        if gpu_recovery != *cpu_recovery {
            comparison.recovery_mismatches += 1;
        }
        let gpu_cross_check_words = cross_check_words_of_gpu_verdict(&gpu_words);
        // 内核不置位的那几条（第一版是 I-1.13）对拍不比：比较处两边都去掉再比，CPU 一侧照判定置位
        let cross_check_words_agree = cross_check_words_agree_in_the_cross_check(
            &gpu_cross_check_words,
            cpu_cross_check_words,
        );
        let checker_bits_agree =
            pool_checker_bits_agree_in_the_cross_check(gpu_checker_bits, *cpu_checker_bits);
        if !cross_check_words_agree {
            comparison.cross_check_mismatches += 1;
        }
        if undecidable
            || !checker_bits_agree
            || !pool_checker_bits_agree_in_the_cross_check(
                gpu_cross_check_words.pool_checker_judged_bits,
                cpu_cross_check_words.pool_checker_judged_bits,
            )
        {
            if comparison.checker_mismatches.len() < 12 {
                comparison.checker_mismatches.push(format!(
                    "状态 {ordinal}：CPU checker={:?}；GPU checker={:?} 判不了的原因 {:#x}",
                    invariant_names(*cpu_checker_bits),
                    invariant_names(gpu_checker_bits),
                    gpu_words[POOL_CHECKER_UNDECIDABLE_WORD]
                ));
            } else {
                comparison.checker_mismatches.push(String::new());
            }
        }
        if undecidable
            || gpu_bits != *cpu_bits
            || !checker_bits_agree
            || gpu_recovery != *cpu_recovery
            || !cross_check_words_agree
        {
            // 每个不一致的状态打一行、不截（下面的说明只留前 12 个，CPU 那一半带读出的文件内容、GPU 的字排在后面）：GPU 结论字整份
            // （第 1 字是恢复内核判不了的原因、第 13 字是池级 checker 判不了的原因）、两侧恢复的四个字与对拍字，不带文件内容；只打、不进判定
            eprintln!(
            "CRASH_JUDGE_GPU_STATE flow={} state={ordinal} gpu_words={gpu_words:?} gpu_recovery={gpu_recovery:?} cpu_recovery={cpu_recovery:?} cpu_bits={cpu_bits:#x} gpu={gpu_cross_check_words:?} cpu={cpu_cross_check_words:?}",
            flow_name
        );
            if comparison.mismatches.len() < 12 {
                comparison.mismatches.push(format!(
                "状态 {ordinal}：CPU {cpu_bits:#x} {cpu_items} {cpu_report} checker={:?} 判到={:?} 对拍字={cpu_cross_check_words:?}；GPU {gpu_words:?} checker={:?} 判到={:?} 对拍字={gpu_cross_check_words:?}",
                invariant_names(*cpu_checker_bits),
                invariant_names(cpu_cross_check_words.pool_checker_judged_bits),
                invariant_names(gpu_checker_bits),
                invariant_names(gpu_cross_check_words.pool_checker_judged_bits)
            ));
            } else {
                comparison.mismatches.push(String::new());
            }
        }
    }
    comparison
}

/// 判那一侧报了「本次未跑」（卡池借不到卡、卡上显存不够、只录那一趟）：对拍结论不存进库，当场收手、由调用方照旧交 `None`。
struct PairingNotRun;

/// 一次对拍的输入摘要：流（基线镜像按扇区终值、每次写的盘、种类、偏移、FUA 与内容摘要、切段与每段的展开方式、被判的根、版本表与没有文件的那几版）、
/// 三棵容器树的旋钮值与判表的摘要。只按内容取，流名、项名不进。
fn pairing_inputs_digest(
    flow: &RecordedFlow,
    versions_without_a_file: &[PublishedVersionWithoutAFile],
    segments: &[Vec<usize>],
    expansion: &(dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion + Sync),
    knobs: ContainerCapacityKnobs,
    tables: &JudgeTables,
) -> [u8; DIGEST_BYTES] {
    let mut text = format!(
        "crash judge gpu and cpu pairing inputs\nbase={}\n",
        hexadecimal_text(&SectorImage::of_base(&flow.base).digest())
    );
    for write in &flow.writes {
        let contents = match &write.contents {
            WrittenContents::Bytes(bytes) => hexadecimal_text(&sha256_digest(bytes)),
            WrittenContents::Zeros { length } => format!("zeros:{length}"),
        };
        text.push_str(&format!(
            "write device={} kind={:?} offset={} force_unit_access={} contents={contents}\n",
            write.device.0, write.kind, write.offset.0, write.is_force_unit_access
        ));
    }
    for (segment_index, segment) in segments.iter().enumerate() {
        text.push_str(&format!(
            "segment {segment:?} expansion={}\n",
            expansion(segment_index, segment).name()
        ));
    }
    text.push_str(&format!(
        "judged_root_index={}\nversions={:?}\nversions_without_a_file={versions_without_a_file:?}\nknobs={knobs:?}\ntables={}\n",
        flow.judged_root_index,
        flow.versions,
        hexadecimal_text(&tables.digest())
    ));
    sha256_digest(text.as_bytes())
}

/// 对拍本身：CPU 判器从库里读回的流逐状态判、GPU 判器从库里读回的判表逐状态判，逐状态比。判那一侧报「本次未跑」时交 `None`。
#[allow(
    clippy::too_many_arguments,
    reason = "对拍的输入就是这几样：卡池、流、项名、版本表另带的没有文件的几版、切段、展开、旋钮、判表；收成一个类型只为让签名变短"
)]
// cpu-reference-for-gpu-comparison
fn judge_on_the_cpu_and_the_gpu_and_compare(
    cards: &'static GpuCardPool,
    flow: &RecordedFlow,
    item_name: &str,
    versions_without_a_file: &[PublishedVersionWithoutAFile],
    segments: &[Vec<usize>],
    expansion: &(dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion + Sync),
    knobs: ContainerCapacityKnobs,
    tables: &JudgeTables,
) -> Option<Comparison> {
    let kernel_sources = JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs(knobs);
    let state_count = tables.state_count;
    // 判表先入库、再读库（对拍那一类的库，项名是流名），GPU 只拿读回的那一份算；崩溃点重放的 CPU 判器同样先把流入库
    // （崩溃点重放枚举那一类的库，项名也是流名）、再从库里读回判。两样都录完才看只录那一趟要不要当场返回：录入段里两类库都要写到
    let packed = packed_tables_through_the_library(item_name, tables);
    // CPU 那一侧按线程分段判（`SINGLEFS_CRASH_JUDGE_CPU_THREADS`，没设取每台默认重活线程数），各线程按序号各报各的，合并按序号排
    let cpu_started = Instant::now();
    let threads = cpu_threads_from_the_value_of_the_variable(
        process_environment_variable(CPU_THREADS_ENVIRONMENT_VARIABLE).as_deref(),
        process_environment_variable(THREAD_CAP_ENVIRONMENT_VARIABLE).as_deref(),
    );
    let judged_by_ordinal: std::sync::Mutex<Vec<(u64, JudgedOnCpu)>> = std::sync::Mutex::new(
        Vec::with_capacity(usize::try_from(state_count).expect("装得进")),
    );
    let cpu_judged = sample_crash_states_on_the_cpu_with_threads_against_the_version_table(
        CrashPointReplayItem {
            name: item_name,
            setting_file: THIS_FILE,
        },
        &flow.base,
        &flow.writes,
        segments,
        flow.judged_root_index,
        CrashPointReplayVersionTable::new(&flow.versions, versions_without_a_file),
        expansion,
        0..state_count,
        threads,
        &|ordinal, image, judgement| {
            let mut cross_check_words = cross_check_words_of_cpu_judgement(judgement);
            let (pool_checker_bits, pool_checker_held_bits) =
                if knobs == ContainerCapacityKnobs::PRODUCT {
                    (
                        expected_pool_checker_bits(judgement),
                        expected_pool_checker_held_bits(judgement),
                    )
                } else {
                    // 旋钮值编进了 GPU 扫描内核：CPU 那一侧池级 checker 的违例位与判到位都按带同一组旋钮值的入口重判
                    let verdicts_given_the_knobs =
                        check_pool_image_given_the_container_capacity_knobs(image, knobs);
                    let (violated_given_the_knobs, judged_given_the_knobs) =
                        pool_checker_violated_and_judged_bits(&verdicts_given_the_knobs);
                    cross_check_words.pool_checker_violated_bits = violated_given_the_knobs;
                    cross_check_words.pool_checker_judged_bits = judged_given_the_knobs;
                    violated_and_held_bits_of(&verdicts_given_the_knobs)
                };
            judged_by_ordinal.lock().expect("判定线程不 panic").push((
                ordinal,
                (
                    expected_bits(judgement),
                    pool_checker_bits,
                    format!("{:?}", judgement.red_items()),
                    format!(
                        "consulted={:?} effective={:?} ignored={:?}",
                        judgement.consulted.outcome,
                        judgement.consulted.effective_root,
                        judgement.ignored.outcome
                    ),
                    recovery_words_of(judgement),
                    pool_checker_held_bits,
                    root_reported_by_the_recovery(&judgement.consulted.outcome),
                    cross_check_words,
                    red_items_compared_by_the_cross_check(&judgement.red_items()),
                ),
            ));
        },
    );
    let cpu_elapsed = cpu_started.elapsed();
    let (Some(packed), Some(cpu_judged)) = (packed, cpu_judged.judged()) else {
        return None;
    };
    assert_eq!(cpu_judged, state_count);
    let mut judged_by_ordinal = judged_by_ordinal.into_inner().expect("判定线程不 panic");
    judged_by_ordinal.sort_by_key(|(ordinal, _)| *ordinal);
    let mut cpu: Vec<JudgedOnCpu> = judged_by_ordinal
        .into_iter()
        .map(|(_, judged)| judged)
        .collect();
    let states_per_dispatch = states_of_one_scratch_lane_on_every_card_of(cards);
    let lease = cards.lease_card_or_report_not_run(
        flow.name,
        mebibytes_a_judge_needs(&packed, states_per_dispatch),
    )?;
    let gpu_started = Instant::now();
    let gpu = on_the_card_or_report_not_run_when_out_of_memory(
        flow.name,
        lease.card(),
        "建判器与判这条流",
        || {
            GpuCrashJudge::on_card_compiling_these_kernel_sources(
                lease.card(),
                &packed,
                states_per_dispatch,
                &kernel_sources,
            )
            .judge_states(0..state_count)
        },
    )?;
    let gpu_elapsed = gpu_started.elapsed();
    drop(lease);
    assert_eq!(gpu.len(), usize::try_from(state_count).expect("装得进"));
    assert_eq!(cpu.len(), gpu.len());
    cpu.truncate(gpu.len());
    let comparison = compare_the_judgements_state_by_state(flow.name, state_count, &cpu, &gpu);
    for line in comparison
        .mismatches
        .iter()
        .filter(|line| !line.is_empty())
        .take(3)
    {
        let shown: String = line.chars().take(700).collect();
        eprintln!("CRASH_JUDGE_GPU_MISMATCH flow={} {shown}", flow.name);
    }
    eprintln!(
        "CRASH_JUDGE_GPU flow={} states={} cpu_red={} gpu_red={} cpu_checker_red={} gpu_checker_red={} recovery_mismatches={} cross_check_mismatches={} mismatches={} checker_mismatches={} gpu_elapsed={:?} cpu_elapsed={:?}",
        flow.name,
        comparison.states,
        comparison.cpu_red,
        comparison.gpu_red,
        comparison.cpu_checker_red,
        comparison.gpu_checker_red,
        comparison.recovery_mismatches,
        comparison.cross_check_mismatches,
        comparison.mismatches.len(),
        comparison.checker_mismatches.len(),
        gpu_elapsed,
        cpu_elapsed
    );
    Some(comparison)
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_three_flows() {
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_three_flows",
    ) else {
        return;
    };
    let limit = expand_up_to();
    let flows = [
        record_new_pool_file_creation("crash-judge-gpu-healthy", "new-pool"),
        record_new_pool_with_a_broken_tree_table("crash-judge-gpu-broken"),
        record_new_pool_with_a_commit_byte_of_two("crash-judge-gpu-commit-2"),
        record_new_pool_with_record_flags_of_three("crash-judge-gpu-flags-3"),
        record_new_pool_with_a_refused_slot_spacing("crash-judge-gpu-spacing-512"),
        record_fixed_script_through_rollback("crash-judge-gpu-rollback"),
    ];
    let mut failures = Vec::new();
    let resealed = resealed_field_flows();
    let every_flow: Vec<(&RecordedFlow, Option<&'static str>)> = flows
        .iter()
        .map(|flow| (flow, None))
        .chain(
            resealed
                .iter()
                .map(|(flow, invariant)| (flow, Some(*invariant))),
        )
        .collect();
    for (flow, must_red_invariant) in every_flow {
        let Some(comparison) = compare_on_every_state(cards, flow, limit) else {
            return;
        };
        if let Some(invariant) = must_red_invariant {
            assert!(
                comparison.cpu_checker_red_invariants.contains(invariant),
                "流 {} 上 CPU 池级 checker 要红 {invariant}（红过的：{:?}）",
                flow.name,
                comparison.cpu_checker_red_invariants
            );
        }
        if flow.name == "new-pool-broken-tree-table" {
            assert!(comparison.cpu_red > 0, "改坏树表的流上 CPU 判器要有红");
        }
        if flow.name.starts_with("new-pool-journal") || flow.name.starts_with("new-pool-sysconfig")
        {
            assert!(
                comparison.cpu_checker_red > 0,
                "流 {} 上 CPU 池级 checker 要有红",
                flow.name
            );
        }
        if flow.name.starts_with("new-pool-sysconfig") {
            assert!(
                comparison.cpu_red > 0,
                "系统配置带拒收值的流上两遍恢复要报没择到根"
            );
        }
        if !comparison.mismatches.is_empty() {
            failures.push(format!(
                "流 {}：{} 个状态不一致（前几个）：\n{}",
                flow.name,
                comparison.mismatches.len(),
                comparison
                    .mismatches
                    .iter()
                    .filter(|line| !line.is_empty())
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// 一个根槽上发布的根是第 33 次写（`record_new_pool_with_the_publish_root_slot_written_past_thirty_two_times`）：发布的根落了与没落的
/// 两个版本组合掩码低 32 位相同、只差高位，字节也不同（一份是发布的根、一份全零）；GPU 判器在这条流的每个状态上与 CPU 判器红位逐位相同、
/// 落到同一条根。判器查版本只比低 32 位时，发布的根落了的状态被读成全零那一版、恢复落到暖机的根，这条红
/// （C601（判器查版本不比组合掩码高 32 位时没有会红的流））。表上的前提在取卡之前判，没卡的机器上也判到。
#[test]
fn the_gpu_judge_tells_apart_two_versions_of_a_root_slot_that_differ_only_past_the_thirty_second_write(
) {
    const EXPAND_UP_TO: usize = 8;
    const {
        assert!(
            FILLER_WRITES_BEFORE_THE_PUBLISH_ROOT > EXPAND_UP_TO,
            "垫的那一段不展开（展开就是 2^32 个状态）"
        );
    }
    let flow = record_new_pool_with_the_publish_root_slot_written_past_thirty_two_times(
        "crash-judge-gpu-past-32-writes",
    );
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= EXPAND_UP_TO {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let tables = build_judge_tables(&JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    })
    .expect("一个落点 64 次写以内，建得出表");
    let publish_root = &flow.writes[flow.judged_root_index];
    let root_slot = tables
        .locations
        .iter()
        .find(|location| {
            location.device == publish_root.device.0 && location.offset == publish_root.offset.0
        })
        .expect("发布的根槽是判器表里的一个落点");
    assert_eq!(
        root_slot.writes.len(),
        FILLER_WRITES_BEFORE_THE_PUBLISH_ROOT + 1,
        "这个根槽上是垫的 32 次写加发布的根那一次"
    );
    let told_apart_only_by_the_high_bits = root_slot.versions.iter().any(|first| {
        root_slot.versions.iter().any(|second| {
            first.combination != second.combination
                && first.combination & 0xFFFF_FFFF == second.combination & 0xFFFF_FFFF
                && first.bytes != second.bytes
        })
    });
    assert!(
        told_apart_only_by_the_high_bits,
        "这个根槽要有两个低 32 位相同、只差高位、字节不同的版本，不然只比低 32 位的内核也判得对：{:?}",
        root_slot
            .versions
            .iter()
            .map(|version| version.combination)
            .collect::<Vec<_>>()
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_tells_apart_two_versions_of_a_root_slot_that_differ_only_past_the_thirty_second_write",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, EXPAND_UP_TO) else {
        return;
    };
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        comparison
            .mismatches
            .iter()
            .filter(|line| !line.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// 重封到根之后，发布那条 journal 记录要自洽：头校验和、载荷校验和对得上，每个点名项的两份位置条目都与全部写落盘的镜像上那份单元的
/// 整单元 CRC 相同——这是 `corrupt_units_and_reseal_up_to_the_root` 的自证（它错了，必红流红的就不只是被改的那个字段）。
#[test]
fn resealed_publish_record_names_units_that_match_on_disk() {
    use singlefs_core::recovery::PoolReader;
    let flow = record_new_pool_with_a_resealed_field(
        "crash-judge-gpu-diagnose-reseal",
        "new-pool-data-unit-object-birth-plus-one",
        &|bytes| unit_class(bytes) == 1,
        false,
        &|bytes| {
            bytes[59] = bytes[59].wrapping_add(1);
        },
    );
    let mut pool = flow.base.clone();
    pool.apply_writes(&flow.writes);
    let record_index = flow
        .writes
        .iter()
        .rposition(|write| write.kind == StepKind::JournalRecord)
        .expect("有记录写");
    let record = flow.writes[record_index].bytes().expect("记录写带字节");
    let named = usize::try_from(u32::from_le_bytes(record[12..16].try_into().expect("4")))
        .expect("点名项数");
    assert!(named >= 1, "发布的记录点名了单元");
    assert!(
        singlefs_core::checksum::wide_checksum_field_holds(record, 4096, 46),
        "记录头校验和重封过"
    );
    let payload_crc = crc32c(
        &record[JOURNAL_RECORD_HEADER_BYTES
            ..JOURNAL_RECORD_HEADER_BYTES + named * JOURNAL_NAMED_ENTRY_BYTES_IN_THE_RECORD],
    );
    assert_eq!(
        payload_crc,
        u32::from_le_bytes(record[95..99].try_into().expect("4")),
        "记录载荷校验和重封过"
    );
    for entry in 0..named {
        let o = JOURNAL_RECORD_HEADER_BYTES + entry * JOURNAL_NAMED_ENTRY_BYTES_IN_THE_RECORD;
        let class = record[o + 28];
        let length = if class == 2 { 16384 } else { 32768 };
        for copy in 0..2 {
            let location = o + copy * 14;
            let device = u32::from_le_bytes(record[location..location + 4].try_into().expect("4"));
            let mut slot = [0u8; 8];
            slot[..6].copy_from_slice(&record[location + 4..location + 10]);
            let slot = u64::from_le_bytes(slot);
            let stored =
                u32::from_le_bytes(record[location + 10..location + 14].try_into().expect("4"));
            let on_disk = PoolReader::read(
                &pool,
                singlefs_core::address::DeviceIdentity(device),
                singlefs_core::address::DeviceOffsetInBytes(slot * 16384),
                length,
            )
            .map(|bytes| crc32c(&bytes));
            assert_eq!(
                on_disk,
                Some(stored),
                "点名项 {entry} 第 {copy} 份（类 {class}、盘 {device}、槽 {slot}）的校验和与盘上那份单元不符"
            );
        }
    }
}

/// 一次派活跨几条草稿缓冲（每条各派一次活、排在同一趟里）与一次派活只用一条缓冲，判出来的结论逐状态相同。
/// 流取固定脚本到回退 D、展开 16（131 195 态；这条流的段不是 6 次写以内就是 16 次以上，展开 14 只有 125 态、跨不过一条缓冲）；
/// 这张卡的显存额度装不下两条草稿缓冲就是环境不够，报出来，不悄悄只比一条。
/// 一条缓冲那一档与 CPU 判器的逐状态对拍在 `the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_three_flows`。
#[test]
fn verdicts_are_the_same_whether_one_dispatch_spans_one_scratch_lane_or_several() {
    const EXPAND_UP_TO: usize = 16;
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "verdicts_are_the_same_whether_one_dispatch_spans_one_scratch_lane_or_several",
    ) else {
        return;
    };
    let flow = record_fixed_script_through_rollback("crash-judge-gpu-lanes");
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= EXPAND_UP_TO {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let judge_flow = JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    };
    let tables = build_judge_tables(&judge_flow).expect("建得出表");
    let state_count = tables.state_count;
    // 判表先入库、再读库，GPU 只拿读回的那一份算
    let Some(packed) = packed_tables_through_the_library(
        "verdicts_are_the_same_whether_one_dispatch_spans_one_scratch_lane_or_several",
        &tables,
    ) else {
        return;
    };
    // 两个判器同时活着：借的是窄的那一档（256 个）加宽的那一档（三条缓冲）各自要的，两份表各算一份（约 6.45 GiB）
    let states_per_lane = states_of_one_scratch_lane_on_every_card_of(cards);
    let Some(lease) = cards.lease_card_or_report_not_run(
        "verdicts_are_the_same_whether_one_dispatch_spans_one_scratch_lane_or_several",
        mebibytes_a_judge_needs(&packed, 256)
            + mebibytes_a_judge_needs(&packed, 3 * states_per_lane),
    ) else {
        return;
    };
    let card = lease.card();
    assert_eq!(
        states_per_scratch_lane(card.device().limits().max_storage_buffer_binding_size),
        states_per_lane,
        "借到的卡一条草稿缓冲装的状态数与借之前按池里最小的绑定上限估的不同：借的量对不上这张卡"
    );
    let Some((narrow, wide)) = on_the_card_or_report_not_run_when_out_of_memory(
        "verdicts_are_the_same_whether_one_dispatch_spans_one_scratch_lane_or_several",
        card,
        "建两个判器与判这条流",
        || {
            let within_one_lane = GpuCrashJudge::on_card_with_packed_tables(card, &packed, 256);
            let across_lanes =
                GpuCrashJudge::on_card_with_packed_tables(card, &packed, 3 * states_per_lane);
            println!(
                "CRASH_JUDGE_LANES states={state_count} states_per_lane={states_per_lane} states_per_dispatch_within_one_lane={} states_per_dispatch_across_lanes={}",
                within_one_lane.states_per_dispatch(),
                across_lanes.states_per_dispatch()
            );
            assert!(
                within_one_lane.states_per_dispatch() <= states_per_lane,
                "窄的那一档一次派活只用一条草稿缓冲"
            );
            assert!(
                across_lanes.states_per_dispatch() > states_per_lane,
                "这张卡的显存额度装不下两条草稿缓冲（一次派活只带得了 {} 个状态，一条缓冲 {states_per_lane} 个）：腾出显存再跑",
                across_lanes.states_per_dispatch()
            );
            assert!(
                state_count > 2 * u64::try_from(states_per_lane).expect("装得进 u64"),
                "这条流只有 {state_count} 个状态，跨不过两条草稿缓冲"
            );
            (
                within_one_lane.judge_states(0..state_count),
                across_lanes.judge_states(0..state_count),
            )
        },
    ) else {
        return;
    };
    let first_difference = narrow
        .iter()
        .zip(&wide)
        .position(|(narrow_words, wide_words)| narrow_words != wide_words);
    assert_eq!(
        first_difference, None,
        "跨几条草稿缓冲判出来的结论与一条缓冲的不同，第一个不同的状态序号在这里"
    );
    assert_eq!(narrow.len(), wide.len());
}

/// 扫描内核源码里入口 `entry_point` 那个函数体里头一处 `anchor` 之后插一行 `violate(bit);`，交回插过的源码。
///
/// # Panics
/// 源码里 `fn <entry_point>(` 不是恰好一处，或那个函数体（到它之后第一个 `\n}\n` 为止）里没有 `anchor`。
fn plant_a_violated_bit_after_the_anchor_inside_the_entry_point(
    source: &str,
    entry_point: &str,
    anchor: &str,
    bit: usize,
) -> String {
    let signature = format!("fn {entry_point}(");
    assert_eq!(
        source.matches(signature.as_str()).count(),
        1,
        "扫描内核源码里入口 {entry_point} 的函数恰好一个"
    );
    let entry_start = source.find(signature.as_str()).expect("上面核过恰好一处");
    let body_end = entry_start
        + source[entry_start..]
            .find("\n}\n")
            .expect("入口函数以单独一行的 `}` 收尾");
    let anchor_start = entry_start
        + source[entry_start..body_end]
            .find(anchor)
            .unwrap_or_else(|| panic!("入口 {entry_point} 的函数体里有 {anchor:?}"));
    let anchor_end = anchor_start + anchor.len();
    format!(
        "{}    violate({bit}u);\n{}",
        &source[..anchor_end],
        &source[anchor_end..]
    )
}

/// 在走读内核清零违例位之后种一个假的违例位（下标取 64 与池级 checker 条数里大的那个，落在高 64 位、不撞真的不变量）、
/// 在扫描内核源码里三个入口读回违例位之后各种一位：主入口（扫描那一截）种第 128 位（下标 127），引用与快照那一截的入口
/// （[`CHECKER_REFERENCES_AND_SNAPSHOTS_ENTRY_POINT`]）种第 127 位（下标 126），扫描重建那一截的入口
/// （[`CHECKER_SCAN_REBUILD_ENTRY_POINT`]，C821 加的第三个入口，判 I-1.13 之前要读另三截写下的违例位）种第 126 位（下标 125）。
/// 两份种过的内核判出的每个状态，违例位等于没种的那一份判出的再并上种的位（第 65 位每个状态都有，扫描源码三个入口种的那三位只在
/// 交接段有效、扫描内核判完的状态上有：三个入口都只在交接段有效时读回违例位），
/// 判到位等于没种的那一份判出的再并上同样种的位（内核存回前把违例位并进判到位），其余的字逐字相同。
/// 违例位的高 64 位在走读内核写出、扫描主入口读回又写出、引用与快照那一截读回又写出、扫描重建那一截读回又写出、Rust 读出这五处任一处被截掉，
/// 这条红；三个入口任一个不读回前一趟写的违例位（只从零起记），在它之前的入口种的位就丢，这条也红。扫描内核源码里的入口
/// （`@compute` 那几处）不是这三个时先红：多出来的入口也读回违例位，就要在这里给它也种一位。
#[test]
fn a_planted_violated_bit_past_the_sixty_fourth_reaches_the_verdict_through_both_checker_kernels() {
    const FIRST_BIT_OF_THE_HIGH_HALF: usize = 64;
    const PLANTED_BY_THE_SCAN_REBUILD_ENTRY: usize = 125;
    const PLANTED_BY_THE_REFERENCES_AND_SNAPSHOTS_ENTRY: usize = 126;
    const PLANTED_BY_THE_SCAN: usize = 127;
    const SCAN_KERNEL_JUDGED_THE_STATE: u32 = 5;
    // 扫描内核源码里每个读回违例位的入口与它种的那一位（主入口、引用与快照那一截、扫描重建那一截）
    let planted_by_each_scan_source_entry = [
        (KERNEL_ENTRY_POINT, PLANTED_BY_THE_SCAN),
        (
            CHECKER_REFERENCES_AND_SNAPSHOTS_ENTRY_POINT,
            PLANTED_BY_THE_REFERENCES_AND_SNAPSHOTS_ENTRY,
        ),
        (
            CHECKER_SCAN_REBUILD_ENTRY_POINT,
            PLANTED_BY_THE_SCAN_REBUILD_ENTRY,
        ),
    ];
    let planted_by_the_walk = IMPLEMENTED_INVARIANTS.len().max(FIRST_BIT_OF_THE_HIGH_HALF);
    assert!(
        planted_by_the_walk < PLANTED_BY_THE_SCAN_REBUILD_ENTRY
            && PLANTED_BY_THE_SCAN_REBUILD_ENTRY < PLANTED_BY_THE_REFERENCES_AND_SNAPSHOTS_ENTRY
            && PLANTED_BY_THE_REFERENCES_AND_SNAPSHOTS_ENTRY < PLANTED_BY_THE_SCAN
            && PLANTED_BY_THE_SCAN < POOL_CHECKER_VIOLATED_BIT_CAPACITY,
        "种的四位都在违例位容量之内、两两不同、不撞真的不变量（池级 checker {} 条）",
        IMPLEMENTED_INVARIANTS.len()
    );
    let judge_sources = JudgeKernelSources::of_the_judge();
    let walk_anchor = "\n    clear_violated();\n";
    let scan_anchor = "\n    load_violated(out);\n";
    assert_eq!(
        judge_sources.checker_walk.matches(walk_anchor).count(),
        1,
        "走读内核里清零违例位的调用恰好一处"
    );
    assert_eq!(
        judge_sources.checker_scan.matches("@compute").count(),
        planted_by_each_scan_source_entry.len(),
        "扫描内核源码里的入口恰好是主入口、引用与快照那一截、扫描重建那一截三个（多了一个入口就在这里给它种一位）"
    );
    assert_eq!(
        judge_sources.checker_scan.matches(scan_anchor).count(),
        planted_by_each_scan_source_entry.len(),
        "扫描内核源码里读回违例位的调用恰好三处：主入口、引用与快照那一截、扫描重建那一截的入口各一处"
    );
    let checker_scan_planted_in_every_entry = planted_by_each_scan_source_entry.iter().fold(
        judge_sources.checker_scan.clone(),
        |source, (entry_point, bit)| {
            plant_a_violated_bit_after_the_anchor_inside_the_entry_point(
                &source,
                entry_point,
                scan_anchor,
                *bit,
            )
        },
    );
    let planted_sources = JudgeKernelSources {
        recovery: judge_sources.recovery.clone(),
        checker_walk: judge_sources.checker_walk.replacen(
            walk_anchor,
            &format!("{walk_anchor}    violate({planted_by_the_walk}u);\n"),
            1,
        ),
        checker_scan: checker_scan_planted_in_every_entry,
    };
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "a_planted_violated_bit_past_the_sixty_fourth_reaches_the_verdict_through_both_checker_kernels",
    ) else {
        return;
    };
    let limit = expand_up_to();
    let flow = record_new_pool_file_creation("crash-judge-gpu-planted-bit", "new-pool");
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= limit {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let judge_flow = JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    };
    let tables = build_judge_tables(&judge_flow).expect("建得出表");
    let state_count = tables.state_count;
    // 判表先入库、再读库，GPU 只拿读回的那一份算
    let Some(packed) = packed_tables_through_the_library("a_planted_violated_bit_past_the_sixty_fourth_reaches_the_verdict_through_both_checker_kernels", &tables) else {
        return;
    };
    // 两个判器先后建；前一个的缓冲什么时候还给驱动看 wgpu 收拾的时机，借两份
    let Some(lease) = cards.lease_card_or_report_not_run(
        "a_planted_violated_bit_past_the_sixty_fourth_reaches_the_verdict_through_both_checker_kernels",
        2 * mebibytes_a_judge_needs(&packed, usize::MAX),
    ) else {
        return;
    };
    let card = lease.card();
    let Some((unplanted, planted)) = on_the_card_or_report_not_run_when_out_of_memory(
        "a_planted_violated_bit_past_the_sixty_fourth_reaches_the_verdict_through_both_checker_kernels",
        card,
        "建没种与种过的两个判器与判这条流",
        || {
            let unplanted = GpuCrashJudge::on_card_with_packed_tables(card, &packed, usize::MAX)
                .judge_states(0..state_count);
            let planted = GpuCrashJudge::on_card_compiling_these_kernel_sources(
                card,
                &packed,
                usize::MAX,
                &planted_sources,
            )
            .judge_states(0..state_count);
            (unplanted, planted)
        },
    ) else {
        return;
    };
    assert_eq!(unplanted.len(), planted.len());
    let mut states_the_scan_judged = 0u64;
    let mut mismatches = Vec::new();
    for (ordinal, (unplanted_words, planted_words)) in unplanted.iter().zip(&planted).enumerate() {
        assert_eq!(
            unplanted_words[POOL_CHECKER_UNDECIDABLE_WORD], 0,
            "状态 {ordinal}：没种的那一份池级 checker 判得了"
        );
        let scan_judged =
            unplanted_words[POOL_CHECKER_REACHED_WORD] == SCAN_KERNEL_JUDGED_THE_STATE;
        if scan_judged {
            states_the_scan_judged += 1;
        }
        let planted_bits = (1u128 << planted_by_the_walk)
            | if scan_judged {
                planted_by_each_scan_source_entry
                    .iter()
                    .fold(0u128, |bits, (_, bit)| bits | (1u128 << bit))
            } else {
                0
            };
        let expected_violated_bits = pool_checker_violated_bits(unplanted_words) | planted_bits;
        // 内核存回结论之前把违例位并进判到位（违例的一定判到），种的两位因此也落进判到位
        let expected_judged_bits = pool_checker_judged_bits(unplanted_words) | planted_bits;
        let violated_words = POOL_CHECKER_VIOLATED_FIRST_WORD
            ..POOL_CHECKER_VIOLATED_FIRST_WORD + POOL_CHECKER_VIOLATED_WORDS;
        let judged_words = POOL_CHECKER_JUDGED_FIRST_WORD
            ..POOL_CHECKER_JUDGED_FIRST_WORD + POOL_CHECKER_JUDGED_WORDS;
        let other_words_agree = unplanted_words
            .iter()
            .zip(planted_words)
            .enumerate()
            .filter(|(word_index, _)| {
                !violated_words.contains(word_index) && !judged_words.contains(word_index)
            })
            .all(|(_, (unplanted_word, planted_word))| unplanted_word == planted_word);
        let read_violated_bits = pool_checker_violated_bits(planted_words);
        let read_judged_bits = pool_checker_judged_bits(planted_words);
        if read_violated_bits != expected_violated_bits
            || read_judged_bits != expected_judged_bits
            || !other_words_agree
        {
            mismatches.push(format!(
                "状态 {ordinal}：没种 {unplanted_words:?}；种过 {planted_words:?}；违例位要 {expected_violated_bits:#x}，读出 {read_violated_bits:#x}；判到位要 {expected_judged_bits:#x}，读出 {read_judged_bits:#x}"
            ));
        }
    }
    println!(
        "CRASH_JUDGE_PLANTED_BIT states={state_count} scan_judged={states_the_scan_judged} mismatches={}",
        mismatches.len()
    );
    assert!(
        states_the_scan_judged > 0,
        "这条流上要有扫描内核判完的状态，第 128、127、126 位那三路才走得到"
    );
    assert!(
        mismatches.is_empty(),
        "{} 个状态上种的位没走到结论（前几个）：\n{}",
        mismatches.len(),
        mismatches
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// 卡领块的时候被叫停：不再领新的块，已经领的判完写进库（块序连着）；下一趟只判剩下的，对账为空。
/// 配置开了 GPU、判器却建不出这条流的表（这里把一次写换成同样长的清零，判器的表不接清零写）：报错，
/// 不退回 CPU 判器，库里一块判定都不写（用户 2026-09-29 定：配了 GPU 就一定要 GPU 判，做不到就明确报错）。建表在上卡之前，不用卡。
#[test]
fn a_flow_whose_judge_tables_cannot_be_built_is_refused_and_not_judged_on_the_cpu() {
    let recorded = record_fixed_script_through_rollback("crash-judge-gpu-refused");
    let mut writes = recorded.writes.clone();
    let moved = *recorded.segments[0].first().expect("段里至少一次写");
    let length = writes[moved].length_in_bytes();
    writes[moved].contents = WrittenContents::Zeros { length };
    writes[moved].kind = StepKind::ZeroFill;
    let expansion = |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= 8 {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let flow = CrashFlow {
        base: &recorded.base,
        writes: &writes,
        segments: &recorded.segments,
        judged_root_index: recorded.judged_root_index,
        versions: &recorded.versions,
        versions_without_a_file: &[],
        expansion: &expansion,
        crash_points: vec![CrashPointSpan {
            name: "every_write".to_string(),
            code_file_in_repository: THIS_FILE.to_string(),
            writes: 0..writes.len(),
            ignored: false,
        }],
    };
    let plan = plan_crash_points(&flow);
    let version = JudgingUnitsFingerprint([9u8; 32]);
    let directory = singlefs_checker_tier::crash_library::ExclusiveTemporaryDirectory::create(
        "crash-judge-gpu-refused",
    )
    .expect("建得了一次性临时目录");
    // 库只在不存在的目录上建：建在这一份一次性目录底下
    let mut store = VerdictStore::create_empty(&directory.join("library")).expect("建得了判定库");
    record_crash_points(&mut store, &plan, &version, &mut CoverageReport::default())
        .expect("录得进");
    let before = unjudged_blocks(&store, &version).expect("对得了账").len();
    assert!(before > 0, "这个点至少一块要判");
    // 判法只吃库里读回的那一份：流先写进库再读回。
    let read_back_of_line_3177 =
        record_the_flow_and_read_it_back(&mut store, &flow).expect("流写得进库、读得回");
    let refused = judge_my_blocks_on_gpu_cards(
        &mut store,
        &read_back_of_line_3177.flow(),
        &plan,
        &|_block| true,
        &version,
        &[],
        &|| false,
    );
    let error = refused.expect_err("建不出表要报错，不退回 CPU 判器");
    assert!(
        error.contains("建不出这条流的表"),
        "报错说清是表建不出：{error}"
    );
    assert_eq!(
        unjudged_blocks(&store, &version).expect("对得了账").len(),
        before,
        "一块判定都没写进库"
    );
    drop(store);
    drop(directory);
}

/// 流取固定脚本到回退 D、展开 16，每段一个崩溃点（几十块）；只用一张卡，叫停之前恰好领五块。
#[test]
fn the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest() {
    const EXPAND_UP_TO: usize = 16;
    const BLOCKS_BEFORE_THE_STOP: usize = 5;
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest",
    ) else {
        return;
    };
    let recorded = record_fixed_script_through_rollback("crash-judge-gpu-stop");
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= EXPAND_UP_TO {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let flow = CrashFlow {
        base: &recorded.base,
        writes: &recorded.writes,
        segments: &recorded.segments,
        judged_root_index: recorded.judged_root_index,
        versions: &recorded.versions,
        versions_without_a_file: &[],
        expansion: &expansion,
        crash_points: recorded
            .segments
            .iter()
            .enumerate()
            .map(|(index, segment)| CrashPointSpan {
                name: format!("segment_{index}"),
                code_file_in_repository: THIS_FILE.to_string(),
                writes: *segment.first().expect("段里至少一次写")
                    ..*segment.last().expect("段里至少一次写") + 1,
                ignored: false,
            })
            .collect(),
    };
    // 借一个定量：一条草稿缓冲那么宽的判器加表（判别力在叫停与续判，不在宽度）；放量路径照借来的卡的额度自己定一次派活带几个。
    // 表先在这里建一次知道多大（放量路径里面照同一条流再建一次，这里没有点被忽略，两次建的是同一份）
    let tables_of_the_flow = PackedJudgeTables::of(
        &build_judge_tables(&JudgeFlow {
            base: flow.base,
            writes: flow.writes,
            segments: flow.segments,
            judged_root_index: flow.judged_root_index,
            versions: flow.versions,
            expansion: &expansion,
        })
        .expect("建得出表"),
    );
    let Some(lease) = cards.lease_card_or_report_not_run(
        "the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest",
        mebibytes_a_judge_needs(
            &tables_of_the_flow,
            states_of_one_scratch_lane_on_every_card_of(cards),
        ),
    ) else {
        return;
    };
    let card = std::sync::Arc::new(lease.card().clone());
    let plan = plan_crash_points(&flow);
    let version = JudgingUnitsFingerprint([7u8; 32]);
    let directory = singlefs_checker_tier::crash_library::ExclusiveTemporaryDirectory::create(
        "crash-judge-gpu-stop",
    )
    .expect("建得了一次性临时目录");
    // 库只在不存在的目录上建：建在这一份一次性目录底下
    let mut store = VerdictStore::create_empty(&directory.join("library")).expect("建得了判定库");
    record_crash_points(&mut store, &plan, &version, &mut CoverageReport::default())
        .expect("录得进");
    let total = unjudged_blocks(&store, &version).expect("对得了账").len();
    assert!(
        total > BLOCKS_BEFORE_THE_STOP + 1,
        "这条流只有 {total} 块，不够在中间叫停"
    );
    let asked = std::sync::atomic::AtomicUsize::new(0);
    let stop_after_five_blocks =
        || asked.fetch_add(1, std::sync::atomic::Ordering::SeqCst) >= BLOCKS_BEFORE_THE_STOP;
    // 判法只吃库里读回的那一份：流先写进库再读回。
    let read_back_of_line_3283 =
        record_the_flow_and_read_it_back(&mut store, &flow).expect("流写得进库、读得回");
    let Some((stopped_tally, stopped)) = judged_on_the_cards_or_report_not_run_when_out_of_memory(
        "the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest",
        judge_my_blocks_on_gpu_cards(
            &mut store,
            &read_back_of_line_3283.flow(),
            &plan,
            &|_block| true,
            &version,
            std::slice::from_ref(&card),
            &stop_after_five_blocks,
        ),
    ) else {
        drop(store);
        drop(directory);
        return;
    };
    println!(
        "CRASH_JUDGE_STOP total_blocks={total} blocks_before_the_stop={} states_before_the_stop={}",
        stopped.blocks_judged, stopped.states_judged
    );
    assert_eq!(
        stopped.blocks_judged,
        u64::try_from(BLOCKS_BEFORE_THE_STOP).expect("装得进 u64"),
        "叫停之前恰好领了五块"
    );
    assert_eq!(stopped_tally.blocks_checked, stopped.blocks_judged);
    assert_eq!(
        unjudged_blocks(&store, &version).expect("对得了账").len(),
        total - BLOCKS_BEFORE_THE_STOP,
        "领过的五块都在库里"
    );
    // 判法只吃库里读回的那一份：流先写进库再读回。
    let read_back_of_line_3314 =
        record_the_flow_and_read_it_back(&mut store, &flow).expect("流写得进库、读得回");
    let Some((resumed_tally, resumed)) = judged_on_the_cards_or_report_not_run_when_out_of_memory(
        "the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest",
        judge_my_blocks_on_gpu_cards(
            &mut store,
            &read_back_of_line_3314.flow(),
            &plan,
            &|_block| true,
            &version,
            std::slice::from_ref(&card),
            &|| false,
        ),
    ) else {
        drop(store);
        drop(directory);
        return;
    };
    assert_eq!(
        resumed.blocks_judged,
        u64::try_from(total - BLOCKS_BEFORE_THE_STOP).expect("装得进 u64"),
        "接着判的那一趟只判剩下的块"
    );
    assert_eq!(
        stopped.states_judged + resumed.states_judged,
        plan.points.last().expect("有点").ordinals.end,
        "两趟合起来每个状态恰好判一次"
    );
    assert_eq!(stopped_tally.red_states + resumed_tally.red_states, 0);
    assert!(unjudged_blocks(&store, &version)
        .expect("对得了账")
        .is_empty());
    drop(store);
    drop(directory);
}

/// 这个进程此刻在卡上占的显存（MiB，`nvidia-smi` 按进程报的；几张卡加起来）。读不到报 0。
fn memory_used_on_the_card_by_this_process() -> u64 {
    let Ok(output) = std::process::Command::new("nvidia-smi")
        .args([
            "--query-compute-apps=pid,used_memory",
            "--format=csv,noheader,nounits",
        ])
        .output()
    else {
        return 0;
    };
    let mine = std::process::id().to_string();
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let (process, used) = line.split_once(',')?;
            (process.trim() == mine)
                .then(|| used.trim().parse::<u64>().ok())
                .flatten()
        })
        .sum()
}

/// 规划显卡用：每张能用的卡按整张卡的额度（空闲显存）开草稿区，从一条草稿缓冲开到这张卡装得下的条数，每一档照那一档的宽度派满三次，
/// 打每次派活卡算了多久、每个状态几微秒、这个进程在卡上占多少显存。看草稿区开到多大时每个状态的用时开始猛涨（缓冲被驱动放出显存）。
#[test]
#[ignore = "量显存上限用：要清场之后空着的卡，每张卡开十几档草稿区"]
fn dispatch_time_at_each_scratch_size_with_the_whole_card_given() {
    let Some(usable_cards) = usable_cards_or_report_not_run(
        "dispatch_time_at_each_scratch_size_with_the_whole_card_given",
    ) else {
        return;
    };
    let every_card: Vec<(u32, u64)> = usable_cards
        .iter()
        .map(|card| (card.index, card.free_memory_mebibytes))
        .collect();
    // 量的是这个进程在卡上占多少显存：取卡时起的上下文先放掉，下面按额度重起的那一份才是量的对象
    drop(usable_cards);
    let flow = record_fixed_script_through_rollback("crash-judge-gpu-scratch-sizes");
    let limit = expand_up_to();
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= limit {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let judge_flow = JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    };
    let tables = build_judge_tables(&judge_flow).expect("建得出表");
    // 判表先入库、再读库，GPU 只拿读回的那一份算
    let Some(packed) = packed_tables_through_the_library(
        "dispatch_time_at_each_scratch_size_with_the_whole_card_given",
        &tables,
    ) else {
        return;
    };
    let state_count = tables.state_count;
    for card in singlefs_checker_tier::gpu_unit_checks::gpu_cards_by_priority(&every_card) {
        let limit_of_the_card = card.device().limits().max_storage_buffer_binding_size;
        let lane =
            singlefs_checker_tier::crash_judge_dispatch::states_per_scratch_lane(limit_of_the_card);
        let capacity = singlefs_checker_tier::crash_judge_dispatch::states_in_flight_within(
            limit_of_the_card,
            card.memory_budget_mebibytes(),
            &packed,
        );
        println!(
            "CRASH_JUDGE_SCRATCH_CARD card={:?} index={} free_mebibytes={} capacity_states={capacity} states_per_lane={lane} states={state_count}",
            card.name, card.index, card.free_memory_mebibytes
        );
        let mut lanes = 1usize;
        loop {
            let width = (lanes * lane).min(capacity);
            let measured_this_width = on_the_card_or_report_not_run_when_out_of_memory(
                "dispatch_time_at_each_scratch_size_with_the_whole_card_given",
                &card,
                "按这一档草稿区建判器与派活",
                || {
                    let judge = GpuCrashJudge::on_card_with_packed_tables(&card, &packed, width);
                    let used = memory_used_on_the_card_by_this_process();
                    // 从流的中间取，避开开头那些很快判完的小状态
                    let first =
                        (state_count / 2).min(state_count - u64::try_from(width).expect("装得进"));
                    for round in 0..3 {
                        let before = judge.time_spent().seconds_waiting_for_the_card;
                        let verdicts = judge.judge_states_in_one_dispatch(
                            first..first + u64::try_from(width).expect("装得进"),
                        );
                        let seconds = judge.time_spent().seconds_waiting_for_the_card - before;
                        println!(
                            "CRASH_JUDGE_SCRATCH card_index={} lanes={lanes} states={width} round={round} seconds={seconds:.3} microseconds_per_state={:.2} scratch_mebibytes={} process_memory_on_the_card_mebibytes={used}",
                            card.index,
                            seconds * 1e6 / verdicts.len() as f64,
                            judge.scratch_mebibytes()
                        );
                        if seconds > 5.0 {
                            break;
                        }
                    }
                },
            );
            if measured_this_width.is_none() {
                return;
            }
            if width == capacity {
                break;
            }
            lanes += 1;
        }
    }
}

/// 规划显卡用：同一条流（固定脚本到回退 D）在一张卡上按几种「一次派活的状态数」各判一遍，打每秒判多少态、草稿区占多少显存；
/// 换派活的粒度不许改判定：每一档的结论逐状态与第一档相同。到这张卡的上限（绑定上限、空闲显存）那一档为止。
#[test]
#[ignore = "量速度用：要一张空闲显存够的独显，几种派活粒度各把整条流判一遍"]
fn gpu_judge_throughput_by_states_per_dispatch() {
    let Some(card) =
        the_whole_emptiest_card_or_report_not_run("gpu_judge_throughput_by_states_per_dispatch")
    else {
        return;
    };
    let flow = record_fixed_script_through_rollback("crash-judge-gpu-throughput");
    let limit = expand_up_to();
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= limit {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let judge_flow = JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    };
    let tables = build_judge_tables(&judge_flow).expect("建得出表");
    let state_count = tables.state_count;
    println!(
        "CRASH_JUDGE_THROUGHPUT_CARD card={:?} free_mebibytes={} max_storage_buffer_binding_size={} max_buffer_size={} states={state_count}",
        card.name,
        card.free_memory_mebibytes,
        card.device().limits().max_storage_buffer_binding_size,
        card.device().limits().max_buffer_size
    );
    let mut first_verdicts: Option<Vec<[u32; VERDICT_WORDS]>> = None;
    // 判表先入库、再读库，GPU 只拿读回的那一份算
    let Some(packed) =
        packed_tables_through_the_library("gpu_judge_throughput_by_states_per_dispatch", &tables)
    else {
        return;
    };
    let mut reached_the_limit_of_the_card = false;
    for requested in [256usize, 2170, 4340, 8680, 13_020, 17_360, 26_040, 34_720] {
        if reached_the_limit_of_the_card {
            break;
        }
        let Some((judge, verdicts, build_seconds, seconds)) =
            on_the_card_or_report_not_run_when_out_of_memory(
                "gpu_judge_throughput_by_states_per_dispatch",
                &card,
                "按这一档派活粒度建判器与判这条流",
                || {
                    let build_started = Instant::now();
                    let judge =
                        GpuCrashJudge::on_card_with_packed_tables(&card, &packed, requested);
                    let build_seconds = build_started.elapsed().as_secs_f64();
                    let started = Instant::now();
                    let verdicts = judge.judge_states(0..state_count);
                    let seconds = started.elapsed().as_secs_f64();
                    (judge, verdicts, build_seconds, seconds)
                },
            )
        else {
            return;
        };
        let spent = judge.time_spent();
        println!(
            "CRASH_JUDGE_THROUGHPUT requested={requested} states_per_dispatch={} scratch_mebibytes={} judge_seconds={seconds:.2} states_per_second={:.0} build_seconds={build_seconds:.1} dispatches={} seconds_preparing={:.2} seconds_waiting_for_the_card={:.2} seconds_reading_back={:.2} card_memory_used_mebibytes={}",
            judge.states_per_dispatch(),
            judge.scratch_mebibytes(),
            state_count as f64 / seconds,
            spent.dispatches,
            spent.seconds_preparing,
            spent.seconds_waiting_for_the_card,
            spent.seconds_reading_back,
            memory_used_on_the_card_by_this_process()
        );
        match &first_verdicts {
            None => first_verdicts = Some(verdicts),
            Some(first) => assert!(
                first == &verdicts,
                "一次派活 {requested} 个状态判出来的结论与第一档不同"
            ),
        }
        if judge.states_per_dispatch() < requested {
            reached_the_limit_of_the_card = true;
        }
    }
}

/// I-1.12：发布的根记录里实例表指针的出生树与实例表那一片单元头不符。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_birth_tree() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_birth_tree",
        &record_new_pool_with_an_instance_table_pointer_naming_another_birth_tree("crash-judge-gpu-i112"),
        "I-1.12",
    );
}

/// I-1.12 数据单元那一支：extent 树里内联的数据指针的出生树与数据单元头不符。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_data_pointer_names_another_birth_tree_than_its_unit() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_data_pointer_names_another_birth_tree_than_its_unit",
        &record_new_pool_with_a_data_pointer_naming_another_birth_tree("crash-judge-gpu-i112-data"),
        "I-1.12",
    );
}

/// I-8.10：发布那条记录的新根段与同代的根不逐字节相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_last_record_of_a_publish_carries_another_watermark_than_its_root(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_last_record_of_a_publish_carries_another_watermark_than_its_root",
        &record_new_pool_with_a_last_record_carrying_another_watermark_than_its_root("crash-judge-gpu-i810"),
        "I-8.10",
    );
}

/// I-8.11：点名项的诞生代号与它点名的单元头不符。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_named_entry_names_another_birth_txg_than_its_unit() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_named_entry_names_another_birth_txg_than_its_unit",
        &record_new_pool_with_a_named_entry_naming_another_birth_txg("crash-judge-gpu-i811"),
        "I-8.11",
    );
}

/// I-4.9：更早的候选根引用的单元晚于它诞生。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_older_candidate_root_references_units_born_after_it() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_older_candidate_root_references_units_born_after_it",
        &record_new_pool_with_an_older_root_pointing_at_the_trees_of_the_publish("crash-judge-gpu-i49"),
        "I-4.9",
    );
}

/// I-9.25：不是 inode 树的树表条目带了 previous_snapshot_txg。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tree_table_entry_other_than_the_inode_tree_carries_a_previous_snapshot_txg(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tree_table_entry_other_than_the_inode_tree_carries_a_previous_snapshot_txg",
        &record_new_pool_with_a_previous_snapshot_txg_on_a_tree_other_than_the_inode_tree("crash-judge-gpu-i925"),
        "I-9.25",
    );
}

/// I-9.27：extent 树那条树表条目的头 ID 写成 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_extent_tree_entry_carries_a_head_identifier_other_than_the_inode_tree(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_extent_tree_entry_carries_a_head_identifier_other_than_the_inode_tree",
        &record_new_pool_with_the_head_identifier_of_the_extent_tree_entry_written_zero("crash-judge-gpu-i927"),
        "I-9.27",
    );
}

/// I-9.27 种类 2 那一支：inode 树那条树表条目的头 ID 不等于它自己的树 ID。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_inode_tree_entry_carries_a_head_identifier_other_than_its_own_tree_identifier(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_inode_tree_entry_carries_a_head_identifier_other_than_its_own_tree_identifier",
        &record_new_pool_with_the_head_identifier_of_the_inode_tree_entry_off_its_own_tree_identifier("crash-judge-gpu-i927-inode"),
        "I-9.27",
    );
}

/// I-9.27 写 0 那一支：分配记录树那条树表条目的头 ID 不是 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_allocation_record_tree_entry_carries_a_non_zero_head_identifier(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_allocation_record_tree_entry_carries_a_non_zero_head_identifier",
        &record_new_pool_with_a_non_zero_head_identifier_on_the_allocation_record_tree_entry("crash-judge-gpu-i927-allocation"),
        "I-9.27",
    );
}

/// I-5.5：从 mkfs 水位建树的那一版，数据单元不落在单元区起始槽号 + 4。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_first_tree_building_version_places_its_data_unit_past_the_unit_area_start_plus_four(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_first_tree_building_version_places_its_data_unit_past_the_unit_area_start_plus_four",
        &record_new_pool_with_the_first_data_unit_two_slots_further("crash-judge-gpu-i55"),
        "I-5.5",
    );
}

/// I-3.13：记账行的代段不是这一版的 txg。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_accounting_row_carries_another_generation_than_its_version(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_accounting_row_carries_another_generation_than_its_version",
        &record_new_pool_with_an_accounting_row_of_another_generation("crash-judge-gpu-i313"),
        "I-3.13",
    );
}

/// I-9.26：inode 记录的改动计数越过所在容器的诞生代号。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_change_count_runs_past_its_container_birth() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_change_count_runs_past_its_container_birth",
        &record_new_pool_with_an_inode_change_count_past_its_container_birth("crash-judge-gpu-i926"),
        "I-9.26",
    );
}

/// I-3.12：实例表的一行的 T 不小于引用这张表的根的 txg（回退流）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_instance_table_row_is_not_below_the_root_referencing_it(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_instance_table_row_is_not_below_the_root_referencing_it",
        &record_rollback_with_an_instance_table_row_not_below_its_root("crash-judge-gpu-i312"),
        "I-3.12",
    );
}

/// I-7.16：同一块盘两个系统配置槽的世代号相差 3。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_two_system_configuration_slots_are_three_generations_apart(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_two_system_configuration_slots_are_three_generations_apart",
        &record_new_pool_with_system_configuration_generations_three_apart("crash-judge-gpu-i716"),
        "I-7.16",
    );
}

/// I-7.17：一条根住在按它的 txg 算不该住的槽里。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_root_lives_in_another_slot_than_its_txg_names() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_root_lives_in_another_slot_than_its_txg_names",
        &record_new_pool_with_a_root_copied_into_the_next_slot("crash-judge-gpu-i717"),
        "I-7.17",
    );
}

/// I-8.12：同一块盘同一实例的 journal 记录 txg 往回走（回退流）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_journal_record_txg_goes_back_within_an_instance() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_journal_record_txg_goes_back_within_an_instance",
        &record_rollback_with_a_journal_record_txg_going_back("crash-judge-gpu-i812"),
        "I-8.12",
    );
}

// ─── 走读内核逐盘 journal 组判按段走（E11）之后的两支：I-8.8「没提交的组之后本实例还有记录」、I-8.10「恰一条带末条标志才比根」 ───

/// journal 记录头里事务号（8 字节）、提交标记（1 字节）、记录标志（1 字节）的偏移（D23（journal 的角色与格式） 已定项 17 的字段表；
/// 走读内核 `order_the_records_of_device_by_group` 取事务号 78，`judge_commit_markers_of_device` 读提交标记 86，I-8.9 与 I-8.10 读记录标志 7）。
const JOURNAL_RECORD_TRANSACTION_OFFSET: usize = 78;
const JOURNAL_RECORD_COMMIT_MARKER_OFFSET: usize = 86;
const JOURNAL_RECORD_FLAGS_OFFSET: usize = 7;
/// 提交标记字节取 1 是带提交标记、取 0 是不带（D23（journal 的角色与格式） 已定项 17；只许这两个值）。
const JOURNAL_RECORD_COMMIT_MARKER_PRESENT: u8 = 1;
const JOURNAL_RECORD_COMMIT_MARKER_ABSENT: u8 = 0;
/// 记录标志位 0：本次发布末条（D23（journal 的角色与格式） 已定项 17）。
const JOURNAL_RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH: u8 = 0b0000_0001;
/// 新根段里树 ID 水位 8 字节的段内偏移（树表指针 86、映射根指针 86 之后）。
const NEW_ROOT_SEGMENT_TREE_IDENTIFIER_WATERMARK: usize = 172;

/// 三个数据单元那条流（[`record_new_pool_through_a_three_unit_file`]）里新建文件那次发布（实例 1、txg 3）与顺序写那次发布（实例 1、txg 4）。
const CHECKPOINT_TXG_OF_THE_FILE_CREATION: u64 = 3;
const CHECKPOINT_TXG_OF_THE_THREE_RECORD_PUBLISH: u64 = 4;

/// I-8.8 判据④（一组一条提交标记都不带时，本实例之后不许再有记录）：三个数据单元那条流上，新建文件那次发布（实例 1、txg 3，一条记录）
/// 两块盘各一份的提交标记写成 0、重封、同实例后面的记录重链。这一组一条提交标记都不带，同一块盘上同一实例之后还有顺序写那次发布（txg 4）的三条记录，
/// CPU 池级 checker 红 I-8.8 判据④（walk.rs `judge_commit_markers_per_transaction`）。
/// 走读内核按 (实例, 事务号) 排好次序之后，这一判比的是「本实例在表里最靠后那一条的下标 > 组末的下标」（`crash_judge_checker.wgsl`
/// `judge_commit_markers_of_device`）：比较写反或永不成立时，txg 3 那条记录与 txg 4 的记录都落了的状态上 GPU 不红而 CPU 红，对拍字不同。
fn record_three_unit_file_whose_file_creation_record_carries_no_commit_marker(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_through_a_three_unit_file(
        tag,
        "three-unit-file-creation-record-without-commit-marker",
    );
    let records_of_the_file_creation =
        journal_record_writes_of_the_publish(&flow, CHECKPOINT_TXG_OF_THE_FILE_CREATION);
    assert_eq!(
        records_of_the_file_creation.len(),
        2,
        "流 {}：新建文件那次发布一条记录、两块盘各一份",
        flow.name
    );
    let records_of_the_three_record_publish =
        journal_record_writes_of_the_publish(&flow, CHECKPOINT_TXG_OF_THE_THREE_RECORD_PUBLISH);
    for (index, counter) in &records_of_the_file_creation {
        let write = &flow.writes[*index];
        let bytes = write.bytes().expect("记录写带字节");
        assert_ne!(
            read_u64_at(bytes, JOURNAL_RECORD_TRANSACTION_OFFSET),
            0,
            "流 {}：新建文件那次发布的事务号不是 0（事务号 0 的空发布不进 I-8.8 的组）",
            flow.name
        );
        assert_eq!(
            bytes[JOURNAL_RECORD_COMMIT_MARKER_OFFSET], JOURNAL_RECORD_COMMIT_MARKER_PRESENT,
            "流 {}：改之前新建文件那条记录带提交标记",
            flow.name
        );
        assert!(
            records_of_the_three_record_publish
                .iter()
                .any(|(later_index, later_counter)| {
                    flow.writes[*later_index].device == write.device && later_counter > counter
                }),
            "流 {}：同一块盘上同一实例在新建文件那条记录之后还有顺序写那次发布的记录",
            flow.name
        );
    }
    for (index, _) in records_of_the_file_creation {
        rewrite_journal_record_and_reseal(&mut flow, index, &|bytes| {
            bytes[JOURNAL_RECORD_COMMIT_MARKER_OFFSET] = JOURNAL_RECORD_COMMIT_MARKER_ABSENT;
        });
    }
    flow
}

/// I-8.8 判据④：[`record_three_unit_file_whose_file_creation_record_carries_no_commit_marker`]。CPU 一侧先钉流造对了：整条落盘的镜像上
/// I-8.8 只红在判据④（判据③与提交标记字节不红）；再整份比，GPU 逐状态与 CPU 相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_uncommitted_transaction_is_followed_by_more_records_of_its_instance(
) {
    let flow = record_three_unit_file_whose_file_creation_record_carries_no_commit_marker(
        "crash-judge-gpu-i88-uncommitted-transaction-then-more-records",
    );
    let mut image_with_every_write_landed = flow.base.clone();
    image_with_every_write_landed.apply_writes(&flow.writes);
    match pool_checker_verdict_on_the_image(&image_with_every_write_landed, "I-8.8") {
        InvariantVerdict::Violated(detail) => assert!(
            detail.starts_with("判据④：") && !detail.contains("；"),
            "流 {}：整条落盘的镜像上 I-8.8 只红在判据④，实际：{detail}",
            flow.name
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => panic!(
            "流 {}：整条落盘的镜像上 I-8.8 要红在判据④，实际 {other_verdict:?}",
            flow.name
        ),
    }
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_uncommitted_transaction_is_followed_by_more_records_of_its_instance",
        &flow,
        "I-8.8",
    );
}

/// I-8.10 只比带末条标志恰一条的组：三个数据单元那条流上，顺序写那次发布（实例 1、txg 4，三条记录）两块盘各一份的第一条记录也带上末条标志、
/// 最后一条记录新根段里的树 ID 水位加 1，都重封、同实例后面的记录重链。那次发布带末条标志的有两条：CPU 池级 checker 红 I-8.9（多于一条末条），
/// I-8.10 不比这一组（walk.rs `judge_new_root_segment_of_the_last_record_against_the_roots`：末条标志不唯一的组归 I-8.9）。
/// 走读内核按 (实例, checkpoint_txg) 排好次序之后数一遍这一组带末条标志的，恰一条才比（`crash_judge_checker.wgsl`
/// `judge_new_root_segment_of_the_last_record_against_the_roots_of_device`）：放成「至少一条就比」时，它拿段里最后那条带标志的
/// （水位加了 1 的末条）去比 (1, 4) 的根，三条记录与那条根都落了的状态上 GPU 红 I-8.10 而 CPU 不红，对拍字不同。
fn record_three_unit_file_whose_three_record_publish_carries_two_last_record_flags(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_through_a_three_unit_file(
        tag,
        "three-unit-file-three-record-publish-with-two-last-record-flags",
    );
    let records =
        journal_record_writes_of_the_publish(&flow, CHECKPOINT_TXG_OF_THE_THREE_RECORD_PUBLISH);
    assert_eq!(
        records.len(),
        6,
        "流 {}：顺序写那次发布切成三条记录、两块盘各一份",
        flow.name
    );
    let first_counter = records
        .iter()
        .map(|(_, counter)| *counter)
        .min()
        .expect("顺序写那次发布有记录");
    let last_counter = records
        .iter()
        .map(|(_, counter)| *counter)
        .max()
        .expect("顺序写那次发布有记录");
    assert_eq!(
        last_counter - first_counter,
        2,
        "流 {}：三条记录的计数器连着",
        flow.name
    );
    for (index, counter) in &records {
        let flags = flow.writes[*index].bytes().expect("记录写带字节")[JOURNAL_RECORD_FLAGS_OFFSET];
        assert_eq!(
            flags,
            if *counter == last_counter {
                JOURNAL_RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH
            } else {
                0
            },
            "流 {}：改之前只有末条（计数器 {last_counter}）带末条标志，计数器 {counter} 那条的记录标志是 {flags:#010b}",
            flow.name
        );
    }
    for (index, counter) in records {
        if counter == first_counter {
            rewrite_journal_record_and_reseal(&mut flow, index, &|bytes| {
                bytes[JOURNAL_RECORD_FLAGS_OFFSET] |=
                    JOURNAL_RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH;
            });
        } else if counter == last_counter {
            rewrite_journal_record_and_reseal(&mut flow, index, &|bytes| {
                let watermark =
                    JOURNAL_NEW_ROOT_SEGMENT_OFFSET + NEW_ROOT_SEGMENT_TREE_IDENTIFIER_WATERMARK;
                bytes[watermark] = bytes[watermark].wrapping_add(1);
            });
        }
    }
    flow
}

/// I-8.10 末条标志不唯一的组不比：[`record_three_unit_file_whose_three_record_publish_carries_two_last_record_flags`]。CPU 一侧先钉流造对了：
/// 整条落盘的镜像上 I-8.9 红在「多于一条末条」、I-8.10 不红（水位加了 1 的末条所在那一组不比）；再整份比，GPU 逐状态与 CPU 相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_with_two_last_record_flags_is_not_compared_with_its_root(
) {
    let flow = record_three_unit_file_whose_three_record_publish_carries_two_last_record_flags(
        "crash-judge-gpu-i810-two-last-record-flags",
    );
    let mut image_with_every_write_landed = flow.base.clone();
    image_with_every_write_landed.apply_writes(&flow.writes);
    match pool_checker_verdict_on_the_image(&image_with_every_write_landed, "I-8.9") {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains("多于一条末条："),
            "流 {}：整条落盘的镜像上 I-8.9 红在多于一条末条，实际：{detail}",
            flow.name
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => panic!(
            "流 {}：整条落盘的镜像上 I-8.9 要红在多于一条末条，实际 {other_verdict:?}",
            flow.name
        ),
    }
    let verdict_of_the_new_root_segment =
        pool_checker_verdict_on_the_image(&image_with_every_write_landed, "I-8.10");
    assert!(
        !matches!(verdict_of_the_new_root_segment, InvariantVerdict::Violated(_)),
        "流 {}：整条落盘的镜像上 I-8.10 不比带两条末条标志的那一组、不红，实际 {verdict_of_the_new_root_segment:?}",
        flow.name
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_with_two_last_record_flags_is_not_compared_with_its_root",
        &flow,
        "I-8.9",
    );
}

/// I-7.15：两块盘择到的系统配置设备数不同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_chosen_system_configurations_disagree_on_the_device_count(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_chosen_system_configurations_disagree_on_the_device_count",
        &record_new_pool_with_a_system_configuration_of_another_device_count("crash-judge-gpu-i715"),
        "I-7.15",
    );
}

/// I-4.1：重放施加出来的那一版走不下去。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_replayed_version_cannot_be_walked() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_replayed_version_cannot_be_walked",
        &record_new_pool_with_a_replayed_version_whose_tree_table_cannot_be_read(
            "crash-judge-gpu-i41",
        ),
        "I-4.1",
    );
}

/// I-7.13（C663）：系统配置槽自述 S = 17，整池不作保；GPU 走读内核把这一槽读成带不收之值、不是「这一槽不可择」，逐位跟上 CPU。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_system_configuration_slot_declares_seventeen_slots_per_region(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_system_configuration_slot_declares_seventeen_slots_per_region",
        &record_new_pool_with_seventeen_slots_per_region("crash-judge-gpu-i713-slots-per-region"),
        "I-7.13",
    );
}

/// I-7.13（C663）：系统配置槽自述 R = 2，整池不作保；GPU 走读内核同判。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_system_configuration_slot_declares_two_regions() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_system_configuration_slot_declares_two_regions",
        &record_new_pool_with_two_regions("crash-judge-gpu-i713-regions"),
        "I-7.13",
    );
}

/// I-2.4（C728）：系统配置槽字段表之后有一个非 0 字节；GPU 走读内核对每个可择的槽判 [489, 4096) 全 0，逐位跟上 CPU。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_system_configuration_slot_carries_a_non_zero_byte_past_its_field_table(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_system_configuration_slot_carries_a_non_zero_byte_past_its_field_table",
        &record_new_pool_with_a_non_zero_last_byte_in_a_system_configuration_slot(
            "crash-judge-gpu-i24-system-configuration-tail",
        ),
        "I-2.4",
    );
}

/// I-8.1 三个字段那一支：系统配置槽里的 F 不是格式常量，整池不作保；I-1.13 两边都报环几何三格不等整池拒那一句（码 28）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_system_configuration_slot_records_a_safety_factor_other_than_the_format_constant(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_and_ending_the_scan_rebuild_with(
        "the_gpu_judge_agrees_with_the_cpu_when_a_system_configuration_slot_records_a_safety_factor_other_than_the_format_constant",
        &record_new_pool_with_a_safety_factor_of_two("crash-judge-gpu-i81-fields"),
        "I-8.1",
        "SCAN_REBUILD_OUTCOME_POOL_REFUSED_FOR_UNEQUAL_JOURNAL_RING_GEOMETRY_FIELDS",
    );
}

/// 两支整池不作保都中（槽距读者不收、F 不是格式常量）：走 I-7.13 那一支，CPU 只红 I-7.13、一个状态都不红 I-8.1，GPU 逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_refused_value_wins_over_a_wrong_safety_factor_in_the_same_slot(
) {
    let flow = record_new_pool_with_a_refused_slot_spacing_and_a_safety_factor_of_two(
        "crash-judge-gpu-i713-over-i81",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_that_a_refused_value_wins_over_a_wrong_safety_factor_in_the_same_slot",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-7.13")
            && !comparison.cpu_checker_red_invariants.contains("I-8.1"),
        "流 {} 上 CPU 池级 checker 红 I-7.13、不红 I-8.1（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    // I-1.13 跟着走带越界值整池拒那一支（码 27），一个状态都不报环几何三格不等那一句（码 28）；对拍逐位比这一字
    assert!(
        states_whose_scan_rebuild_ends_with(
            &comparison,
            "SCAN_REBUILD_OUTCOME_POOL_REFUSED_FOR_A_SYSTEM_CONFIGURATION_VALUE_THE_READER_DOES_NOT_ACCEPT"
        ) > 0
            && states_whose_scan_rebuild_ends_with(
                &comparison,
                "SCAN_REBUILD_OUTCOME_POOL_REFUSED_FOR_UNEQUAL_JOURNAL_RING_GEOMETRY_FIELDS"
            ) == 0,
        "流 {} 上 I-1.13 报带越界值整池拒那一句、不报环几何三格不等那一句",
        flow.name
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        comparison
            .mismatches
            .iter()
            .filter(|line| !line.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// I-8.5：全池一个自证过的系统配置槽都没有；I-1.13 两边都报一块盘都交不出有效系统配置那一句（码 29）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_no_system_configuration_slot_verifies_itself() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_and_ending_the_scan_rebuild_with(
        "the_gpu_judge_agrees_with_the_cpu_when_no_system_configuration_slot_verifies_itself",
        &record_new_pool_with_every_system_configuration_slot_zeroed_at_the_end(
            "crash-judge-gpu-i85",
        ),
        "I-8.5",
        "SCAN_REBUILD_OUTCOME_NO_DEVICE_YIELDS_A_VALID_SYSTEM_CONFIGURATION",
    );
}

/// I-7.5：根槽不是声明的 io_min 的整数倍。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_root_slots_are_not_aligned_to_the_declared_io_min() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_root_slots_are_not_aligned_to_the_declared_io_min",
        &record_new_pool_with_an_io_min_the_root_slots_are_not_aligned_to("crash-judge-gpu-i75"),
        "I-7.5",
    );
}

/// 基线里盘 0 的两个系统配置槽与盘 1 的槽 0 都抹成 0，只剩盘 1 的槽 1 自证得过（CPU 恢复与池级 checker 都择它；流里之后的系统配置槽写
/// 再把轮到的那一槽写回来）：判表的落点几何要从盘 1 的槽 1 取，只读第一块盘槽 0 的取法在这条流上建不出表。
fn record_new_pool_with_only_the_slot_one_of_the_second_device_self_verified_in_the_base(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(
        tag,
        "new-pool-only-slot-one-of-device-one-self-verified-in-the-base",
    );
    let spacing = u64::from(geometry().fixed_structure_slot_spacing);
    for (device, offset) in [(0, 0), (0, spacing), (1, 0)] {
        flow.base
            .devices
            .get_mut(&DeviceIdentity(device))
            .expect("基线里有这块盘")
            .write(
                singlefs_core::address::DeviceOffsetInBytes(offset),
                &[0u8; 4096],
            );
    }
    flow
}

/// 新池新建文件那条流只留盘 0：基线里没有盘 1（系统配置记的根环区域 1 在盘 1 上），写表只录盘 0 上的写（发布那次 txg 3 的根落在区域 0、盘 0 上）。
/// 判表只在池里的盘上建落点；给区域盘一律建落点的取法从基线读盘 1 读不出、建表时 panic。
fn record_new_pool_file_creation_on_device_zero_alone(tag: &str) -> RecordedFlow {
    let pool = build_pool(tag);
    let operations_on_device_zero: Vec<_> = pool.retained_operations()[pool.mkfs_operation_count..]
        .iter()
        .filter(|retained| retained.operation.device == DeviceIdentity(0))
        .cloned()
        .collect();
    let (writes, segments, _stream_indexes) =
        writes_and_segments_with_stream_indexes(&operations_on_device_zero, &geometry());
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("发布那次的根槽写在盘 0 上（txg 3 落在区域 0）");
    let mut base = pool.memory_pool_after_mkfs();
    base.devices
        .remove(&DeviceIdentity(1))
        .expect("mkfs 之后基线里有盘 1");
    RecordedFlow {
        name: "new-pool-file-creation-on-device-zero-alone",
        base,
        writes,
        segments,
        judged_root_index,
        versions: vec![PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
            content: file_content(),
        }],
    }
}

/// 一条流的判表（展开上限照 [`expand_up_to`]，与 [`compare_on_every_state`] 同一种展开）：取卡之前在 CPU 上建，没有卡的机器上这一半照样判。
fn judge_tables_of_the_flow(flow: &RecordedFlow) -> JudgeTables {
    let limit = expand_up_to();
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= limit {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    build_judge_tables(&JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    })
    .unwrap_or_else(|refusal| panic!("流 {} 建得出判表：{refusal:?}", flow.name))
}

/// 一条流上 GPU 与 CPU 逐状态比，一个状态都不许不一致、判不了。
fn assert_the_gpu_judge_agrees_with_the_cpu_on_every_state_of_the_flow(
    test_name: &str,
    flow: &RecordedFlow,
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, flow, expand_up_to()) else {
        return;
    };
    assert!(comparison.states > 0, "流 {} 有状态可判", flow.name);
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// 基线里只剩盘 1 的槽 1 自证得过：判表照 CPU 择系统配置的找法从那一槽取落点几何、建得出（取卡之前判），每个崩溃状态 GPU 与 CPU 逐状态相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_only_the_slot_one_of_the_second_device_self_verifies_in_the_base(
) {
    let flow =
        record_new_pool_with_only_the_slot_one_of_the_second_device_self_verified_in_the_base(
            "crash-judge-gpu-only-slot-one-of-device-one",
        );
    let tables = judge_tables_of_the_flow(&flow);
    let spacing = u64::from(geometry().fixed_structure_slot_spacing);
    let system_configuration_locations: Vec<(u32, u64)> = tables
        .locations
        .iter()
        .filter(|location| location.kind == JudgeLocationKind::SystemConfiguration)
        .map(|location| (location.device, location.offset))
        .collect();
    assert_eq!(
        system_configuration_locations,
        vec![(0, 0), (0, spacing), (1, 0), (1, spacing)],
        "流 {}：落点几何取自盘 1 的槽 1，两块盘各有两个系统配置槽的落点",
        flow.name
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_every_state_of_the_flow(
        "the_gpu_judge_agrees_with_the_cpu_when_only_the_slot_one_of_the_second_device_self_verifies_in_the_base",
        &flow,
    );
}

/// 池里少一块盘（只剩盘 0，系统配置记的根环区域 1 在丢掉的盘 1 上）：判表建得出（取卡之前判）、盘 1 上一个落点都不建，
/// 每个崩溃状态 GPU 与 CPU 逐状态相同（CPU 读盘 1 交 None、GPU 在盘 1 上找不到落点，两边都当那几个根槽读不出）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_a_pool_that_lost_a_device_holding_a_root_ring_region() {
    let flow = record_new_pool_file_creation_on_device_zero_alone(
        "crash-judge-gpu-new-pool-on-device-zero-alone",
    );
    let tables = judge_tables_of_the_flow(&flow);
    assert_eq!(tables.devices, vec![0], "流 {}：池里只有盘 0", flow.name);
    assert!(
        tables.locations.iter().all(|location| location.device == 0),
        "流 {}：落点都在盘 0 上",
        flow.name
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_every_state_of_the_flow(
        "the_gpu_judge_agrees_with_the_cpu_on_a_pool_that_lost_a_device_holding_a_root_ring_region",
        &flow,
    );
}

/// journal 环起点字段写成第一版常量 1024 之外的一个槽号：自证得过、读者不收（与 harness 场景
/// `system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs` 的同名常量同一个值）。
const JOURNAL_RING_START_SLOT_THE_READER_REFUSES: u64 = 1025;

/// 基线里 `device` 上偏移 `offset` 那一个系统配置槽（自证得过）。
fn system_configuration_slot_in_the_base(
    flow: &RecordedFlow,
    device: DeviceIdentity,
    offset: u64,
) -> singlefs_core::system_configuration::SystemConfiguration {
    let slot_bytes =
        usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    singlefs_core::system_configuration::SystemConfiguration::parse_slot(
        &singlefs_core::recovery::PoolReader::read(
            &flow.base,
            device,
            singlefs_core::address::DeviceOffsetInBytes(offset),
            slot_bytes,
        )
        .expect("基线里这一槽读得到"),
    )
    .expect("基线里这一槽自证得过")
}

/// 基线里 `device` 上偏移 `offset` 起写 `bytes`。
fn write_into_the_base(flow: &mut RecordedFlow, device: DeviceIdentity, offset: u64, bytes: &[u8]) {
    flow.base
        .devices
        .get_mut(&device)
        .expect("基线里有这块盘")
        .write(singlefs_core::address::DeviceOffsetInBytes(offset), bytes);
}

/// 两块盘的槽 0 都抹成 0（全池没有一个可择的槽 0：恢复与池级 checker 逐档读槽 1，直到这条流把槽 0 写回）。
fn wipe_the_slot_zeros_in_the_base(flow: &mut RecordedFlow) {
    let slot_bytes =
        usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        write_into_the_base(flow, device, 0, &vec![0u8; slot_bytes]);
    }
}

/// 新池新建文件那条流的基线：两块盘的槽 0 都抹掉，槽距处本池的槽 1 照旧，再远一档（两倍槽距）放一份本池槽 1 的拷贝、journal 环起点写成 1025
/// （自证得过、读者不收，记的槽距仍是本池的，不等于它所在的偏移；harness 场景「8192 处那份槽带不收之值」的造法）。
fn record_new_pool_with_a_refused_slot_one_farther_than_the_pool_slot_one_in_the_base(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(
        tag,
        "new-pool-refused-slot-one-farther-than-the-pool-slot-one-in-the-base",
    );
    let spacing = u64::from(geometry().fixed_structure_slot_spacing);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let mut refused = system_configuration_slot_in_the_base(&flow, device, spacing);
        refused.immutable.journal_ring_start_slot =
            singlefs_core::address::SlotNumber(JOURNAL_RING_START_SLOT_THE_READER_REFUSES);
        write_into_the_base(&mut flow, device, 2 * spacing, &refused.to_slot());
    }
    wipe_the_slot_zeros_in_the_base(&mut flow);
    flow
}

/// 新池新建文件那条流的基线：两块盘的槽 0 都抹掉；逐档读到的第一个自证得过的槽 1（槽距处、记的槽距就是它所在的偏移）带读者不收之值
/// （journal 环起点 1025），更远一档（两倍槽距）有一份读者收的槽 1（本池槽 1 的拷贝、记的槽距改成两倍槽距、等于它所在的偏移）。
fn record_new_pool_whose_first_self_verified_slot_one_carries_a_refused_value_and_a_farther_one_is_accepted(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(
        tag,
        "new-pool-first-self-verified-slot-one-refused-and-a-farther-one-accepted-in-the-base",
    );
    let spacing = u64::from(geometry().fixed_structure_slot_spacing);
    let farther_spacing = 2 * spacing;
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let pool_slot_one = system_configuration_slot_in_the_base(&flow, device, spacing);
        let mut refused = pool_slot_one;
        refused.immutable.journal_ring_start_slot =
            singlefs_core::address::SlotNumber(JOURNAL_RING_START_SLOT_THE_READER_REFUSES);
        let mut farther = pool_slot_one;
        farther.immutable.sizes.fixed_structure_slot_spacing =
            u32::try_from(farther_spacing).expect("两倍槽距装得进 4 字节");
        write_into_the_base(&mut flow, device, spacing, &refused.to_slot());
        write_into_the_base(&mut flow, device, farther_spacing, &farther.to_slot());
    }
    wipe_the_slot_zeros_in_the_base(&mut flow);
    flow
}

/// 一条流的判表里系统配置槽那一类落点在哪几处（盘、偏移），升序。
fn system_configuration_locations_of(tables: &JudgeTables) -> Vec<(u32, u64)> {
    tables
        .locations
        .iter()
        .filter(|location| location.kind == JudgeLocationKind::SystemConfiguration)
        .map(|location| (location.device, location.offset))
        .collect()
}

/// 两块盘的槽 0 都废、槽距处本池的槽 1 照旧、再远一档有一份带读者不收之值的槽（hand_out 那张「8192 处那份槽带不收之值」的形，搬到新池新建文件这条流上）：
/// 判表给每块盘的两倍槽距处建系统配置落点（取卡之前判，没有卡的机器上这一半照样判）；这条流把槽 0 写回之前的状态 CPU 逐档读到那一份、判 I-7.13，
/// GPU 逐档读同一处，每个状态逐位相同。只建偏移 0 与几何槽距两处落点的取法在取卡之前那一判就红（落点表少了两倍槽距那两处），
/// 上卡时 GPU 读那一处读成不自证、不红 I-7.13。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_a_refused_slot_one_found_farther_than_the_pool_slot_one_while_reading_every_spacing(
) {
    let flow = record_new_pool_with_a_refused_slot_one_farther_than_the_pool_slot_one_in_the_base(
        "crash-judge-gpu-refused-slot-one-farther",
    );
    let tables = judge_tables_of_the_flow(&flow);
    let spacing = u64::from(geometry().fixed_structure_slot_spacing);
    assert_eq!(
        system_configuration_locations_of(&tables),
        vec![
            (0, 0),
            (0, spacing),
            (0, 2 * spacing),
            (1, 0),
            (1, spacing),
            (1, 2 * spacing)
        ],
        "流 {}：两块盘各有偏移 0、槽距与逐档读到的那一处三个系统配置槽的落点",
        flow.name
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_on_a_refused_slot_one_found_farther_than_the_pool_slot_one_while_reading_every_spacing",
        &flow,
        "I-7.13",
    );
}

/// 两块盘的槽 0 都废、逐档读到的第一个自证得过的槽 1 带读者不收之值、更远一档槽距上有一份读者收的槽 1：落点几何取第一个那一份
/// （记的槽距就是这条流的槽距，根槽写都对得上落点；不看读者收不收），更远那一份另建落点（取卡之前判）；这条流把槽 0 写回之前的状态
/// CPU 逐档读到第一个那一份就判 I-7.13，GPU 逐位相同。GPU 走读内核把 journal 环起点不是 1024 的槽读成不自证时，GPU 逐档读到更远那一份、
/// 照常判，不红 I-7.13，这条红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_first_self_verified_slot_one_carries_a_refused_value_and_a_farther_one_is_accepted(
) {
    let flow = record_new_pool_whose_first_self_verified_slot_one_carries_a_refused_value_and_a_farther_one_is_accepted(
        "crash-judge-gpu-first-slot-one-refused-farther-accepted",
    );
    let tables = judge_tables_of_the_flow(&flow);
    let spacing = u64::from(geometry().fixed_structure_slot_spacing);
    assert_eq!(
        system_configuration_locations_of(&tables),
        vec![
            (0, 0),
            (0, spacing),
            (0, 2 * spacing),
            (1, 0),
            (1, spacing),
            (1, 2 * spacing)
        ],
        "流 {}：几何取槽距处那一份，两块盘各有偏移 0、槽距与更远那一份三个系统配置槽的落点",
        flow.name
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_first_self_verified_slot_one_carries_a_refused_value_and_a_farther_one_is_accepted",
        &flow,
        "I-7.13",
    );
}

/// 列出这条流上不一致的状态（前几个），没有就是空串。
fn mismatch_listing(comparison: &Comparison) -> String {
    comparison
        .mismatches
        .iter()
        .filter(|line| !line.is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join("\n")
}

/// I-3.5：一版引用、前一版不引用的单元诞生代号不大于前一版的 txg、也不在整棵继承下来的用户可见树里——CPU 红 I-3.5，GPU 在同样的状态上红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_version_references_a_unit_born_no_later_than_the_previous_version(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_version_references_a_unit_born_no_later_than_the_previous_version",
        &record_new_pool_with_a_data_unit_born_no_later_than_the_previous_version(
            "crash-judge-gpu-i35",
        ),
        "I-3.5",
    );
}

/// 攻方种子区间的起点（`research/prompts/m4-segment-code-r1-opus-model/src/attack_common/weights.rs` 的 `ATTACK_SEED_OFFSET`）
/// 加种子下标 2022：只当这段历史的名字，执行不读它。同 harness 档
/// `pool_checker_appearance_rule_labels_each_version_by_its_own_user_visible_trees.rs` 的同名常量（两档不共用代码）。
const SEED_OF_THE_ATTACK_HISTORY_OF_SEED_INDEX_2022: u64 =
    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + 7_000_000 + 2022;

/// 攻方种子下标 2022 缩出的 5 步（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 T4、F9）：挂载、一个数据单元以内的覆盖写、
/// 回退到根环里第三新的那条根（覆盖写之前那一版）、建快照、跨 291 个数据单元的顺序写。取值照抄 harness 档那一份。
fn rollback_without_snapshots_then_a_snapshot_then_an_overwrite_under_it() -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(SEED_OF_THE_ATTACK_HISTORY_OF_SEED_INDEX_2022),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::InsideOneDataUnit {
                    selector: 7_879_751_949_923_199_194,
                },
                fill_seed: 17_840_163_520_990_143_491,
            }),
            HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot {
                index_from_newest: 2,
            }),
            HistoryOperation::CreateSnapshot,
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                    data_units: 291,
                    selector: 7_664_500_909_322_504_686,
                },
                fill_seed: 4_676_315_349_287_084_229,
            }),
        ],
    }
}

/// 攻方种子下标 2022 那段历史从 mkfs 之后录起的整条流（[`record_the_flow_of_a_history_from_the_make_filesystem`]）。
fn record_rollback_without_snapshots_then_a_snapshot_and_an_overwrite_under_it(
) -> (RecordedFlow, Vec<PublishedVersionWithoutAFile>) {
    record_the_flow_of_a_history_from_the_make_filesystem(
        &rollback_without_snapshots_then_a_snapshot_then_an_overwrite_under_it(),
        "rollback-without-snapshots-then-a-snapshot-and-an-overwrite-under-it",
    )
}

/// 一段历史从 mkfs 之后录起的整条流，流名是 `flow_name`（起点镜像是空盘上叠 mkfs 那几步，mkfs 不是事务、不枚举；被判的根是流里最后一条根槽写）。
/// 版本表取理想模型每一步之后根环里的每一版（`IdealModel::committed_versions`，连同历史停下时那一份）：有文件的进 `versions`，
/// 树表 0 条或没有文件的进交回的那几版。
///
/// # Panics
/// 历史没有每一步都做成（或按前提不适用）、模型对不上；流里一条根槽写都没有；实例代号装不进 u32。
fn record_the_flow_of_a_history_from_the_make_filesystem(
    history: &GeneratedHistory,
    flow_name: &'static str,
) -> (RecordedFlow, Vec<PublishedVersionWithoutAFile>) {
    let execution = HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES;
    let stream = SharedStream::retaining_contents();
    let mut contents_of_each_committed_version: BTreeMap<(u64, u64), Option<Vec<u8>>> =
        BTreeMap::new();
    let run = execute_history_with(history, execution, &stream, &mut |observation| {
        for (key, file) in observation.model.committed_versions() {
            contents_of_each_committed_version
                .entry((key.instance.0, key.checkpoint_txg.0))
                .or_insert_with(|| file.map(|content| content.to_vec()));
        }
    });
    assert_eq!(
        run.ending,
        HistoryEnding::Completed,
        "流 {flow_name} 那段历史每一步都做成、模型对得上：{:?}",
        run.outcomes
    );
    for (key, file) in &run.committed_versions_at_the_end {
        contents_of_each_committed_version
            .entry((key.instance.0, key.checkpoint_txg.0))
            .or_insert_with(|| file.clone());
    }
    let operations = stream.retained_operations();
    let mkfs_operations = run.operations_written_by_make_filesystem;
    let mut base = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        execution.device_width.device_bytes(),
    );
    base.apply(&operations[..mkfs_operations]);
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[mkfs_operations..],
        &execution.device_width.fixed_geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .unwrap_or_else(|| panic!("流 {flow_name}：历史里最后那次发布有一条根槽写"));
    let mut versions = Vec::new();
    let mut versions_without_a_file = Vec::new();
    for ((instance, checkpoint_txg), file) in contents_of_each_committed_version {
        let instance = InstanceGeneration(
            u32::try_from(instance)
                .unwrap_or_else(|_| panic!("流 {flow_name}：这段历史的实例代号装得进 u32")),
        );
        let checkpoint_txg = CheckpointTxg(checkpoint_txg);
        match file {
            Some(content) => versions.push(PublishedVersion {
                instance,
                checkpoint_txg,
                content,
            }),
            None => versions_without_a_file.push(PublishedVersionWithoutAFile {
                instance,
                checkpoint_txg,
            }),
        }
    }
    (
        RecordedFlow {
            name: flow_name,
            base,
            writes,
            segments,
            judged_root_index,
            versions,
        },
        versions_without_a_file,
    )
}

/// I-3.5 的用户可见树标签按版本记（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F9 的 GPU 一侧）：攻方种子下标 2022
/// 那段历史里，回退那一版复活的单元在它自己的活头 extent 树里；最新那一版先走、经快照那两棵树走到它（快照树不是用户可见树），回退那一版共享这片子树、
/// 不重走。CPU 一侧先钉流造对了——整条落盘的镜像上 I-3.5 判成立（不是不适用）。GPU 走读内核按版本记标签（并进来的共享子树落点按这一版此刻
/// 在哪棵树里打）之后每个状态逐位相同；内核照旧跨版本按落点取标签时，回退那一版的 (b) 款落空、GPU 判红 I-3.5 而 CPU 不红，对不上。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_the_reference_intervals_after_a_rollback_without_snapshots_then_a_snapshot_and_an_overwrite_under_it(
) {
    const TEST_NAME: &str = "the_gpu_judge_agrees_with_the_cpu_on_the_reference_intervals_after_a_rollback_without_snapshots_then_a_snapshot_and_an_overwrite_under_it";
    let (flow, versions_without_a_file) =
        record_rollback_without_snapshots_then_a_snapshot_and_an_overwrite_under_it();
    let mut image_after_the_whole_flow = flow.base.clone();
    image_after_the_whole_flow.apply_writes(&flow.writes);
    assert_eq!(
        pool_checker_verdict_on_the_image(&image_after_the_whole_flow, "I-3.5"),
        InvariantVerdict::Holds,
        "整条落盘的镜像上 I-3.5 判到了单元、成立：回退那一版复活的单元按这一版自己的标签落在它的活头 extent 树里"
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(comparison) = compare_on_every_state_against_the_version_table(
        cards,
        &flow,
        &versions_without_a_file,
        expand_up_to(),
    ) else {
        return;
    };
    assert!(
        comparison.cpu_checker_held_invariants.contains("I-3.5"),
        "流 {} 上 CPU 池级 checker 至少在一个状态上判过 I-3.5 而成立（成立过的：{:?}；红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_held_invariants,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// 攻方种子区间的起点（`research/prompts/m4-segment-code-r1-opus-model/src/attack_common/weights.rs` 的 `ATTACK_SEED_OFFSET`）
/// 加种子下标 1032：只当这段历史的名字，执行不读它。同 harness 档
/// `pool_checker_shared_subtree_below_a_snapshot_only_unit_keeps_the_children_it_shares_with_the_head.rs` 的 `SEED_OF_THE_ATTACK_HISTORY`（两档不共用代码）。
const SEED_OF_THE_ATTACK_HISTORY_OF_SEED_INDEX_1032: u64 =
    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + 7_000_000 + 1032;

/// 攻方种子下标 1032 缩出的 16 步（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F9「种子下标 1032 那一形」，攻方 T4）：
/// 去掉回退那一组比重生成 40 步，留第 0、1、4、5、8、9、24、25、26、27、28、29、35、36、37、38 步。没有回退；有建快照、unlink、几次可写挂载、
/// 批间崩溃再挂载的整理、删快照。取值照抄 harness 档那一份。
fn snapshots_created_and_destroyed_around_writable_mounts_and_crashed_compactions(
) -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(SEED_OF_THE_ATTACK_HISTORY_OF_SEED_INDEX_1032),
        starting_point: HistoryStartingPoint::AfterMakeFilesystem,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishFirstFile(ContentChoice {
                length: ContentLength::InsideOneDataUnit {
                    selector: 4_908_652_545_014_035_923,
                },
                fill_seed: 1_175_750_450_067_266_269,
            }),
            HistoryOperation::PublishOverwrite(ContentChoice {
                length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                    data_units: 152,
                    selector: 1_112_582_400_669_913_327,
                },
                fill_seed: 4_144_227_704_568_475_653,
            }),
            HistoryOperation::CreateSnapshot,
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::Unlink,
            HistoryOperation::CompactWhileMounted(CompactionChoice {
                batch_units: 1,
                region_selector: 9_802_755_752_694_277_730,
                ending: CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 3 },
            }),
            HistoryOperation::CreateSnapshot,
            HistoryOperation::CompactWhileMounted(CompactionChoice {
                batch_units: 3,
                region_selector: 16_953_606_325_372_812_583,
                ending: CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches: 2 },
            }),
            HistoryOperation::DestroySnapshot(SnapshotChoice {
                index_among_the_snapshots: 1_939_626_785_022_617_838,
            }),
            HistoryOperation::CreateInode,
            HistoryOperation::DestroySnapshot(SnapshotChoice {
                index_among_the_snapshots: 10_045_164_462_792_776_581,
            }),
            HistoryOperation::CreateSnapshot,
            HistoryOperation::CreateSnapshot,
            HistoryOperation::DestroySnapshot(SnapshotChoice {
                index_among_the_snapshots: 849_968_957_743_593_146,
            }),
            HistoryOperation::CreateInode,
        ],
    }
}

/// 快照独有的父单元下面与活头共享的孩子照样挂到它下面（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F9
/// 「种子下标 1032 那一形」的 GPU 一侧，池级 checker 那一侧是 `walk.rs` 的 `note_references_of_a_pointer_followed_by_the_walk`）：
/// 攻方种子下标 1032 那段历史里，建快照之后活头改写了一片 extent 叶，改写之前那片叶只从快照那两棵树走得到、它下面的孩子活头也引用；
/// 根环转过一圈之后改写那一版先走、建快照之前那一版后走，后走那一版不重走那片叶、只按「单元 → 它下面的落点」那张表并进它的孩子。
/// CPU 一侧先钉流造对了——整条落盘的镜像上 I-3.5 判成立（不是不适用）。GPU 走读内核把孩子一律挂到父单元下面之后每个状态逐位相同；
/// 内核照旧对这一版已经从活头记过的孩子不挂时（`crates/mutations.tsv` 里「m4 修复批三丁三」那一行），后走那一版少一个落点、
/// GPU 判红 I-3.5 而 CPU 不红，对不上。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_the_reference_intervals_when_a_snapshot_only_unit_shares_its_children_with_the_head(
) {
    const TEST_NAME: &str = "the_gpu_judge_agrees_with_the_cpu_on_the_reference_intervals_when_a_snapshot_only_unit_shares_its_children_with_the_head";
    let (flow, versions_without_a_file) = record_the_flow_of_a_history_from_the_make_filesystem(
        &snapshots_created_and_destroyed_around_writable_mounts_and_crashed_compactions(),
        "snapshots-created-and-destroyed-around-writable-mounts-and-crashed-compactions",
    );
    let mut image_after_the_whole_flow = flow.base.clone();
    image_after_the_whole_flow.apply_writes(&flow.writes);
    assert_eq!(
        pool_checker_verdict_on_the_image(&image_after_the_whole_flow, "I-3.5"),
        InvariantVerdict::Holds,
        "整条落盘的镜像上 I-3.5 判到了单元、成立：快照独有的那片叶下面与活头共享的孩子并进建快照之前那一版，那一版的引用集合不少落点"
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(comparison) = compare_on_every_state_against_the_version_table(
        cards,
        &flow,
        &versions_without_a_file,
        expand_up_to(),
    ) else {
        return;
    };
    assert!(
        comparison.cpu_checker_held_invariants.contains("I-3.5"),
        "流 {} 上 CPU 池级 checker 至少在一个状态上判过 I-3.5 而成立（成立过的：{:?}；红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_held_invariants,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// I-3.5 判成立的那一支：固定脚本到回退 D 的流上，回退那次发布引用回退目标那一版（txg 3）的单元，诞生代号都不大于前一版的 txg，
/// 它们落在根指针生在 txg 3 的 extent / inode 树里、整棵继承下来（(b)）。CPU 在至少一个状态上判过 I-3.5 而成立、一个状态都不红它，
/// GPU 每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_rollback_publish_inheriting_its_user_visible_trees_whole_keeps_the_appearance_rule(
) {
    let flow = record_fixed_script_through_rollback("crash-judge-gpu-i35-holds");
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_that_a_rollback_publish_inheriting_its_user_visible_trees_whole_keeps_the_appearance_rule",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_held_invariants.contains("I-3.5")
            && !comparison.cpu_checker_red_invariants.contains("I-3.5"),
        "流 {} 上 CPU 池级 checker 判过 I-3.5 而成立、一个状态都不红它（成立过的：{:?}；红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_held_invariants,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// 恢复内核与 core 同一口径读记录（D23（journal 的角色与格式） 已定项 15，`root_record.rs` 的 `of_unmount_marker_field_of_a_new_root_segment`）：
/// 新根段卸载记号位 0 之外有位为 1 的记录当损坏、不施加。发布那条记录落了而根没落的状态上 CPU 恢复停在暖机那条根，
/// GPU 收了这条记录就落到发布那一版、两遍恢复的结论对不上；池级 checker 那一边 GPU 也照 RR1 不收它。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_record_whose_unmount_marker_sets_another_bit_is_not_applied(
) {
    let flow = record_new_pool_with_a_last_record_whose_unmount_marker_sets_another_bit(
        "crash-judge-gpu-rr1",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_that_a_record_whose_unmount_marker_sets_another_bit_is_not_applied",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// 恢复施加一条记录取它新根段里的实例表指针（已定项 15，`recovery.rs` 的 `replay_journal`），不照抄所选根的：记录里的实例表指针读不对时
/// CPU 恢复走到文件之前就读不出，GPU 照抄所选根就读得出；池级 checker 走施加出来那一版也从记录取（I-4.1），根落了的状态上红 I-8.10 第五段。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_applied_record_names_an_instance_table_other_than_the_chosen_root(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_applied_record_names_an_instance_table_other_than_the_chosen_root",
        &record_new_pool_with_a_last_record_whose_instance_table_pointer_reads_wrong(
            "crash-judge-gpu-applied-instance-table",
        ),
        "I-8.10",
    );
}

/// 恢复内核判记录新根段四条指针的头与 core 同一口径：实例表指针头的 extent 偏移非 0 的记录当损坏、不施加；
/// CPU 池级 checker 不判记录里的指针头、照收，根落了的状态上红 I-8.10 第五段。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_record_whose_instance_table_pointer_carries_an_extent_offset_is_not_applied(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_that_a_record_whose_instance_table_pointer_carries_an_extent_offset_is_not_applied",
        &record_new_pool_with_a_last_record_whose_instance_table_pointer_carries_an_extent_offset(
            "crash-judge-gpu-record-pointer-head",
        ),
        "I-8.10",
    );
}

/// 走读内核走由记录施加出来的那一版时，分配记录树根从记录新根段取（walk.rs 的 `walk_version_applied_only_by_records`），
/// 不取它施加在其上的那条根的：记录里的分配记录树根是外来的、读不对时 CPU 判 I-4.1 红，GPU 取那条根（全零、不走）就不红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_applied_version_names_an_allocation_record_tree_root_other_than_the_root_it_is_applied_on(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_applied_version_names_an_allocation_record_tree_root_other_than_the_root_it_is_applied_on",
        &record_new_pool_with_a_last_record_whose_allocation_record_tree_root_is_foreign(
            "crash-judge-gpu-applied-allocation-root",
        ),
        "I-4.1",
    );
}

/// I-8.10 第七段：发布那条记录的卸载记号与同代的根不同（记录 1、根 0）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_last_record_carries_another_unmount_marker_than_its_root(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_last_record_carries_another_unmount_marker_than_its_root",
        &record_new_pool_with_a_last_record_carrying_the_unmount_marker(
            "crash-judge-gpu-i810-unmount",
        ),
        "I-8.10",
    );
}

/// 两个池级 checker 内核把违例位的每个字都写出来：结论缓冲派活前整个填全 1，没被写的字读出来是全 1。
/// 新池新建文件那条流的每个状态（不种假位）上，违例位在池级 checker 条数之外的那些位全是 0、判得了——高 64 位那两个字只要有一个没写，这条红。
#[test]
fn the_checker_kernels_write_every_violated_word_and_leave_no_bit_past_the_invariant_count() {
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_checker_kernels_write_every_violated_word_and_leave_no_bit_past_the_invariant_count",
    ) else {
        return;
    };
    let limit = expand_up_to();
    let flow = record_new_pool_file_creation("crash-judge-gpu-violated-words-written", "new-pool");
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= limit {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let tables = build_judge_tables(&JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    })
    .expect("建得出表");
    let state_count = tables.state_count;
    // 判表先入库、再读库，GPU 只拿读回的那一份算
    let Some(packed) = packed_tables_through_the_library(
        "the_checker_kernels_write_every_violated_word_and_leave_no_bit_past_the_invariant_count",
        &tables,
    ) else {
        return;
    };
    let Some(lease) = cards.lease_card_or_report_not_run(
        "the_checker_kernels_write_every_violated_word_and_leave_no_bit_past_the_invariant_count",
        mebibytes_a_judge_needs(&packed, usize::MAX),
    ) else {
        return;
    };
    let Some(verdicts) = on_the_card_or_report_not_run_when_out_of_memory(
        "the_checker_kernels_write_every_violated_word_and_leave_no_bit_past_the_invariant_count",
        lease.card(),
        "建判器与判这条流",
        || {
            GpuCrashJudge::on_card_with_packed_tables(lease.card(), &packed, usize::MAX)
                .judge_states(0..state_count)
        },
    ) else {
        return;
    };
    drop(lease);
    let invariant_count = IMPLEMENTED_INVARIANTS.len();
    let states_with_bits_past_the_invariant_count: Vec<String> = verdicts
        .iter()
        .enumerate()
        .filter(|(_, words)| {
            words[POOL_CHECKER_UNDECIDABLE_WORD] != 0
                || pool_checker_violated_bits(words) >> invariant_count != 0
        })
        .map(|(ordinal, words)| format!("状态 {ordinal}：{words:?}"))
        .collect();
    assert!(
        states_with_bits_past_the_invariant_count.is_empty(),
        "{} 个状态的违例位越过池级 checker 的 {invariant_count} 条或判不了（前几个）：\n{}",
        states_with_bits_past_the_invariant_count.len(),
        states_with_bits_past_the_invariant_count
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

// ───────────── 第 9 批：I-9.28 比到候选集之外的根与树表读不出的根、I-7.8 扫描方向的认领规则、I-7.9 两支只取两个一圈窗口里的根 ─────────────

/// 根记录里 flags（卸载记号在位 0）、checkpoint_txg、回退下界 F 的偏移（D22（单元原子性怎么合成） 已定项 7 的偏移表）。
const ROOT_RECORD_FLAGS_OFFSET: usize = 20;
const ROOT_RECORD_CHECKPOINT_TXG_OFFSET: usize = 28;
const ROOT_RECORD_ROLLBACK_FLOOR_OFFSET: usize = 130;

/// 池级 checker 违例位里这一条的那一位（位 i ↔ `IMPLEMENTED_INVARIANTS[i]`）。
fn invariant_bit(invariant: &str) -> u128 {
    let index = IMPLEMENTED_INVARIANTS
        .iter()
        .position(|candidate| *candidate == invariant)
        .expect("这一条在池级 checker 的清单里");
    1u128 << index
}

/// 流尾另起一段写的那条根带不带卸载记号（根记录 flags 位 0）。
#[derive(Clone, Copy, Debug)]
enum UnmountMarkerOfTheAppendedRoot {
    Written,
    NotWritten,
}

/// 流尾另起一段写一条根：抄第 `source_root_write` 次写（根槽写）的字节，checkpoint_txg 改成 `checkpoint_txg`、回退下界 F 改成
/// `rollback_floor`、flags 位 0 照 `unmount_marker`，实例代号与各条指针照抄，重封整槽校验和，写到这个 txg 按发布规则算出的那个根槽上
/// （`singlefs_core::root_ring::target_for_publish`，I-7.17 不因为它红）。那个槽里原来的根被它盖掉。
fn append_a_root_at_a_later_txg(
    flow: &mut RecordedFlow,
    source_root_write: usize,
    checkpoint_txg: u64,
    rollback_floor: u64,
    unmount_marker: UnmountMarkerOfTheAppendedRoot,
) {
    let source = flow.writes[source_root_write].clone();
    assert_eq!(source.kind, StepKind::RootRecordFua, "抄的是根槽写");
    let WrittenContents::Bytes(mut bytes) = source.contents else {
        panic!("根槽写是普通写");
    };
    bytes[ROOT_RECORD_CHECKPOINT_TXG_OFFSET..ROOT_RECORD_CHECKPOINT_TXG_OFFSET + 8]
        .copy_from_slice(&checkpoint_txg.to_le_bytes());
    bytes[ROOT_RECORD_ROLLBACK_FLOOR_OFFSET..ROOT_RECORD_ROLLBACK_FLOOR_OFFSET + 8]
        .copy_from_slice(&rollback_floor.to_le_bytes());
    let flags: u32 = match unmount_marker {
        UnmountMarkerOfTheAppendedRoot::Written => 1,
        UnmountMarkerOfTheAppendedRoot::NotWritten => 0,
    };
    bytes[ROOT_RECORD_FLAGS_OFFSET..ROOT_RECORD_FLAGS_OFFSET + 4]
        .copy_from_slice(&flags.to_le_bytes());
    reseal_root_record(&mut bytes);
    let target = singlefs_core::root_ring::target_for_publish(
        CheckpointTxg(checkpoint_txg),
        geometry().root_ring_slots_per_region,
    );
    let device = parameters().region_devices[usize::try_from(target.region).expect("区域号")];
    let offset =
        singlefs_core::root_ring::slot_offset(target, geometry().fixed_structure_slot_spacing);
    append_a_segment_of_writes(
        flow,
        vec![RetainedWrite {
            device,
            kind: StepKind::RootRecordFua,
            is_force_unit_access: source.is_force_unit_access,
            offset,
            contents: WrittenContents::Bytes(bytes),
        }],
    );
}

/// 根环槽数 3S（三个区域、每区 S 槽）。
fn root_ring_slot_count() -> u64 {
    singlefs_format::ROOT_RING_REGIONS * geometry().root_ring_slots_per_region.count()
}

/// 树表单元：码 2、key 宽 8、条目宽 200。
fn is_a_tree_table_unit(bytes: &[u8]) -> bool {
    unit_class(bytes) == 2
        && index_node_key_width(bytes) == 8
        && index_node_entry_width(bytes) == 200
}

/// 树表单元里 inode 树（种类 2）那几条条目的诞生 txg（条目偏移 108）加 1。
fn add_one_to_the_birth_txg_of_the_inode_tree_entries(bytes: &mut [u8]) {
    for entry in 0..(bytes.len() - 131) / 200 {
        let offset = 131 + entry * 200;
        if u16::from_le_bytes(bytes[offset + 10..offset + 12].try_into().expect("2")) == 2 {
            bytes[offset + 108] = bytes[offset + 108].wrapping_add(1);
        }
    }
}

/// 回退流上回退那次发布 D（txg 9）的树表里 inode 树条目的诞生 txg 加 1、重封到根（同 `rollback-tree-table-birth-txg-plus-one`），
/// 流尾再在 txg 30 写一条 D 的根、回退下界 F 抬到 9（落在 txg 6 那个槽上）。这一条落了的状态上候选集只剩 D 与它（别的根都在 F 之下），
/// 两条共用 D 那片树表，I-9.14 比不出来；I-9.28 照比根环里每一条读得出的根，A、B、C 那几片记的诞生 txg 与 D 的不同，红。
fn record_rollback_with_another_birth_txg_recorded_only_outside_the_candidate_set(
    tag: &str,
) -> RecordedFlow {
    const TXG_OF_THE_ROOT_RAISING_THE_FLOOR_TO_THE_ROLLBACK: u64 = 30;
    const FLOOR_AT_THE_ROLLBACK: u64 = 9;
    let mut flow = record_rollback_with_a_resealed_field(
        tag,
        "rollback-birth-txg-differs-only-outside-the-candidate-set",
        &is_a_tree_table_unit,
        &add_one_to_the_birth_txg_of_the_inode_tree_entries,
    );
    assert_eq!(
        publish_root_identity(&flow),
        (2, FLOOR_AT_THE_ROLLBACK),
        "被判的根是回退那次发布 D（实例 2、txg 9）"
    );
    let rollback_root = flow.judged_root_index;
    append_a_root_at_a_later_txg(
        &mut flow,
        rollback_root,
        TXG_OF_THE_ROOT_RAISING_THE_FLOOR_TO_THE_ROLLBACK,
        FLOOR_AT_THE_ROLLBACK,
        UnmountMarkerOfTheAppendedRoot::NotWritten,
    );
    flow.versions.push(PublishedVersion {
        instance: InstanceGeneration(2),
        checkpoint_txg: CheckpointTxg(TXG_OF_THE_ROOT_RAISING_THE_FLOOR_TO_THE_ROLLBACK),
        content: file_content(),
    });
    flow
}

/// I-9.28 比的是根环里每一条读得出、自证过的根，不只是回退候选集（I-9.14 只比候选集）：有一个状态上 CPU 红 I-9.28 而 I-9.14 不红，
/// GPU 每个状态逐位相同。GPU 那一条只比候选根时，这个状态上它不红、与 CPU 不一致。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_only_roots_outside_the_candidate_set_record_another_birth_txg(
) {
    let flow = record_rollback_with_another_birth_txg_recorded_only_outside_the_candidate_set(
        "crash-judge-gpu-i928-outside-candidates",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_when_only_roots_outside_the_candidate_set_record_another_birth_txg",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    let every_readable_root = invariant_bit("I-9.28");
    let candidate_set = invariant_bit("I-9.14");
    assert!(
        comparison
            .cpu_checker_bits_of_each_state
            .iter()
            .any(|bits| bits & every_readable_root != 0 && bits & candidate_set == 0),
        "流 {} 上要有一个状态 CPU 红 I-9.28 而 I-9.14 不红（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// 回退流上覆盖写 C（txg 8）那条根指着的树表两份副本各改坏末字节：C 那条根落了的状态上它读得出、自证过，树表指针非零而树表读不出，
/// I-9.28 少它那一份证据（CPU 报「部分没判」，GPU 与不适用一样不置违例位）；读得出的那几片记的诞生 txg 都相同，不红。
fn record_rollback_with_the_tree_table_of_the_overwrite_before_it_unreadable(
    tag: &str,
) -> RecordedFlow {
    const TXG_OF_THE_OVERWRITE_BEFORE_THE_ROLLBACK: u64 = 8;
    let mut flow = record_fixed_script_through_rollback(tag);
    flow.name = "rollback-tree-table-of-c-unreadable";
    let root_of_the_overwrite = flow
        .writes
        .iter()
        .rposition(|write| {
            write.kind == StepKind::RootRecordFua
                && write.bytes().is_some_and(|bytes| {
                    u64::from_le_bytes(
                        bytes[ROOT_RECORD_CHECKPOINT_TXG_OFFSET
                            ..ROOT_RECORD_CHECKPOINT_TXG_OFFSET + 8]
                            .try_into()
                            .expect("8"),
                    ) == TXG_OF_THE_OVERWRITE_BEFORE_THE_ROLLBACK
                })
        })
        .expect("回退流上有覆盖写 C（txg 8）的根槽写");
    break_both_copies_of_the_tree_table_named_by_the_root_write(&mut flow, root_of_the_overwrite);
    flow
}

/// I-9.28 的四种判定在 GPU 上只有违例置位：有一条读得出、自证过的根树表读不出，而读得出的那几片都对得上时，CPU 一个状态都不红它，
/// GPU 每个状态逐位相同。GPU 把读不出树表的根当成违例时这条红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_root_whose_tree_table_cannot_be_read_does_not_redden_the_every_readable_root_birth_invariant(
) {
    let flow = record_rollback_with_the_tree_table_of_the_overwrite_before_it_unreadable(
        "crash-judge-gpu-i928-unreadable-tree-table",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_that_a_root_whose_tree_table_cannot_be_read_does_not_redden_the_every_readable_root_birth_invariant",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        !comparison.cpu_checker_red_invariants.contains("I-9.28"),
        "流 {} 上 CPU 一个状态都不红 I-9.28（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// 覆盖写那一版的文件长度：一个数据单元装得下（32768 − 134），内容跨过单元的第二槽起点（单元偏移 16384）。
const FORGED_HEADER_FILE_BYTES: usize = 20_000;
/// 伪头里的树 ID：2⁴⁰，远高于这条流的水位。
const FORGED_HEADER_TREE_IDENTIFIER: u64 = 1 << 40;
/// 伪头的 key 宽（码 2 头偏移 51）。
const FORGED_HEADER_KEY_WIDTH: u8 = 8;
/// 伪头里的诞生代号：1，不超过根环里最大的 checkpoint_txg（扫描方向第一道排除挡不住它）。
const FORGED_HEADER_BIRTH_TXG: u64 = 1;
/// 伪头带的「别的池」的 fsid 低 8 字节：写序实例按 fsid 认不出，扫描方向第二道排除也挡不住它。
const FORGED_HEADER_FOREIGN_FILESYSTEM_IDENTIFIER_LOW: u64 = 0x0123_4567_89ab_cdef;

/// 覆盖写的内容：按位置取的字节，内容偏移 16384 − 134 起放一个头校验和自洽的码 2 伪头（D18（块里携带什么信息） 已定项 18 的偏移：
/// 树 ID 42、key 宽 51、诞生代号 52 + 2k、fsid 60 + 2k、明文头末尾 86 + 2k，头校验和在 10）。
fn content_forging_an_index_node_header_on_the_second_slot_of_its_data_unit() -> Vec<u8> {
    let mut content: Vec<u8> = (0..FORGED_HEADER_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 5) % 251).expect("小于 251"))
        .collect();
    let key_span = 2 * usize::from(FORGED_HEADER_KEY_WIDTH);
    let header_end = 86 + key_span;
    let mut header = vec![0u8; header_end];
    header[..4].copy_from_slice(&singlefs_core::unit::UNIT_MAGIC);
    header[6] = singlefs_core::unit::UNIT_CLASS_INDEX_NODE;
    header[42..50].copy_from_slice(&FORGED_HEADER_TREE_IDENTIFIER.to_le_bytes());
    header[51] = FORGED_HEADER_KEY_WIDTH;
    header[52 + key_span..60 + key_span].copy_from_slice(&FORGED_HEADER_BIRTH_TXG.to_le_bytes());
    header[60 + key_span..68 + key_span]
        .copy_from_slice(&FORGED_HEADER_FOREIGN_FILESYSTEM_IDENTIFIER_LOW.to_le_bytes());
    let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(&header, header_end, 10);
    header[10..42].copy_from_slice(&digest);
    let forged_header_content_offset =
        usize::try_from(singlefs_format::SLOT_BYTES - singlefs_format::DATA_UNIT_PAYLOAD_OFFSET)
            .expect("小于一个槽");
    content[forged_header_content_offset..forged_header_content_offset + header_end]
        .copy_from_slice(&header);
    content
}

/// 新池新建文件之后覆盖写一次，内容在它那个数据单元第二槽的起点上带一个码 2 伪头（[`content_forging_an_index_node_header_on_the_second_slot_of_its_data_unit`]）。
fn record_new_pool_with_an_overwrite_forging_an_index_node_header_on_the_second_slot_of_its_data_unit(
    tag: &str,
) -> RecordedFlow {
    let mut pool = build_pool(tag);
    let mut recorder = CrashPointRecorder::new(pool.stream.clone(), pool.mkfs_operation_count);
    record_pool_build(&mut recorder, &pool);
    let content = content_forging_an_index_node_header_on_the_second_slot_of_its_data_unit();
    pool.output = overwrite_step(&mut recorder, &mut pool, &content, InstanceGeneration(1));
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let second_slot = usize::try_from(singlefs_format::SLOT_BYTES).expect("16384");
    assert!(
        writes.iter().any(|write| {
            write.kind == StepKind::UnitWrite
                && write.bytes().is_some_and(|bytes| {
                    unit_class(bytes) == 1
                        && bytes.len() > second_slot + 52
                        && bytes[second_slot..second_slot + 4] == singlefs_core::unit::UNIT_MAGIC
                        && bytes[second_slot + 6] == singlefs_core::unit::UNIT_CLASS_INDEX_NODE
                })
        }),
        "覆盖写那一版的文件装在一个数据单元里，伪头落在它第二槽的起点上"
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("根槽写");
    RecordedFlow {
        name: "new-pool-overwrite-forging-an-index-node-header-on-the-second-slot",
        base,
        writes,
        segments,
        judged_root_index,
        versions: vec![
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(3),
                content: file_content(),
            },
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(4),
                content,
            },
        ],
    }
}

/// I-7.8 扫描方向的认领规则（D18（块里携带什么信息） 已定项 7，walk.rs 的 `scanned_tree_identifiers`）：覆盖写的数据单元第二槽起点上
/// 带一个头校验和自洽、树 ID 2⁴⁰ 的码 2 伪头。GPU 的落点表里有这个第二槽（覆盖写那次写碰过的每个 16 KiB 槽各是一个落点），
/// CPU 的崩溃镜像只把那次写的起点槽当候选；数据单元落了的状态上，GPU 照认领规则让起点槽上的数据单元认领两槽、不读第二槽的头，
/// 与 CPU 一样不数它，CPU 一个状态都不红 I-7.8、GPU 每个状态逐位相同。GPU 不按整单元跳时把伪头数进「出现过的」树 ID、红 I-7.8。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_data_unit_claims_its_second_slot_and_a_header_forged_there_is_not_counted(
) {
    let flow = record_new_pool_with_an_overwrite_forging_an_index_node_header_on_the_second_slot_of_its_data_unit(
        "crash-judge-gpu-i78-claim",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_that_a_data_unit_claims_its_second_slot_and_a_header_forged_there_is_not_counted",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        !comparison.cpu_checker_red_invariants.contains("I-7.8"),
        "流 {} 上 CPU 一个状态都不红 I-7.8（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// 新池新建文件那条流上，流尾在 txg 27 = 3 + 3S 写一条抄发布那条根 (1, 3)、回退下界 F 从 0 抬到 5 的根（落在 txg 3 那个槽上、把它盖掉）。
/// 之后根环里是 txg 0、1、2 的根与这一条，它是镜像最新的根：锚在它上的一圈窗口（txg > 3）里 txg 比它小的根一条都没有，
/// 两支都不判它（准入那一支上限无从算起；带卸载记号那一支没有「之前的根」）。不取窗口时 txg 1、2 的根进来，F 5 高过它们，红。
fn record_new_pool_with_a_floor_raise_one_revolution_past_every_earlier_root(
    tag: &str,
    name: &'static str,
    unmount_marker: UnmountMarkerOfTheAppendedRoot,
) -> RecordedFlow {
    const FLOOR_RAISED_ONE_REVOLUTION_PAST_THE_PUBLISH: u64 = 5;
    let mut flow = record_new_pool_file_creation(tag, name);
    let (instance, publish_txg) = publish_root_identity(&flow);
    assert_eq!(
        (instance, publish_txg),
        (1, 3),
        "被判的根是新建文件那次发布"
    );
    let txg_one_revolution_past_the_publish = publish_txg + root_ring_slot_count();
    let publish_root = flow.judged_root_index;
    append_a_root_at_a_later_txg(
        &mut flow,
        publish_root,
        txg_one_revolution_past_the_publish,
        FLOOR_RAISED_ONE_REVOLUTION_PAST_THE_PUBLISH,
        unmount_marker,
    );
    flow.versions.push(PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(txg_one_revolution_past_the_publish),
        content: file_content(),
    });
    flow
}

/// I-7.9 准入抬 F 那一支只取两个一圈窗口里的根（用户 2026-10-04 定「乙，两支都取两个窗口」）：窗口里一条之前的根都没有，这条根不判，
/// CPU 一个状态都不判 I-7.9（不红、也不报成立），GPU 每个状态逐位相同。GPU 那一支不取窗口时红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_floor_raise_with_no_earlier_root_inside_the_one_revolution_windows_is_not_judged(
) {
    let flow = record_new_pool_with_a_floor_raise_one_revolution_past_every_earlier_root(
        "crash-judge-gpu-i79-admission-window",
        "new-pool-floor-raise-one-revolution-past-every-earlier-root",
        UnmountMarkerOfTheAppendedRoot::NotWritten,
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_that_a_floor_raise_with_no_earlier_root_inside_the_one_revolution_windows_is_not_judged",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        !comparison.cpu_checker_red_invariants.contains("I-7.9")
            && !comparison.cpu_checker_held_invariants.contains("I-7.9"),
        "流 {} 上 CPU 一个状态都不判 I-7.9（红过的：{:?}；成立过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants,
        comparison.cpu_checker_held_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// I-7.9 带卸载记号那一支同样只取两个一圈窗口里的根：同一张根环、那条根带卸载记号，窗口里 txg 比它小的根一条都没有，CPU 一个状态都不红 I-7.9，
/// GPU 每个状态逐位相同。GPU 那一支不取窗口时拿 txg 2 那条根比、F 5 高过它，红。CPU 那一支的窗口由同一批的兄弟实现员补，
/// 合入之前这条在 CPU 一侧照旧红（不取窗口）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_an_unmount_marked_floor_raise_with_no_earlier_root_inside_the_one_revolution_windows_is_not_reddened(
) {
    let flow = record_new_pool_with_a_floor_raise_one_revolution_past_every_earlier_root(
        "crash-judge-gpu-i79-unmount-window",
        "new-pool-unmount-marked-floor-raise-one-revolution-past-every-earlier-root",
        UnmountMarkerOfTheAppendedRoot::Written,
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_that_an_unmount_marked_floor_raise_with_no_earlier_root_inside_the_one_revolution_windows_is_not_reddened",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        !comparison.cpu_checker_red_invariants.contains("I-7.9"),
        "流 {} 上 CPU 一个状态都不红 I-7.9（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// 回退流（根环里 txg 0..9，S = 8、3S = 24）上流尾先写一段：txg 29 一条抄回退那次发布 D 的根、F 从 0 抬到 6（落在 txg 5 那个槽上）；
/// 再写一段：txg 27 一条同样的根、F 6（落在 txg 3 那个槽上），这时抬 F 的是 txg 27 那一条（txg 29 那一条的前一条变成它、F 不再高）。
/// 两段都落了的状态上，锚在被判那条根（27）上的窗口是 txg > 3、锚在镜像最新那条根（29）上的是 txg > 5：B 那条根 (1, 4) 只在前一个里。
/// 只取两个窗口的交时之前的有效根是 6、7、8、9，上限 6，F 6 成立；把 (1, 4) 算进来，最旧有效根变成 4、上限 4，红。
/// 只有前一段落了的状态上抬 F 的是 txg 29 那一条、它就是最新的根，两个窗口相同（txg > 5），上限 6，成立。
fn record_rollback_with_a_floor_raise_whose_window_anchored_at_the_newest_root_drops_an_older_root(
    tag: &str,
) -> RecordedFlow {
    const TXG_OF_THE_NEWEST_ROOT: u64 = 29;
    const TXG_OF_THE_ROOT_RAISING_THE_FLOOR: u64 = 27;
    const FLOOR_RAISED_PAST_THE_ROLLBACK: u64 = 6;
    let mut flow = record_fixed_script_through_rollback(tag);
    flow.name = "rollback-floor-raise-whose-image-window-drops-an-older-root";
    assert_eq!(
        publish_root_identity(&flow),
        (2, 9),
        "被判的根是回退那次发布 D"
    );
    assert_eq!(
        TXG_OF_THE_NEWEST_ROOT - root_ring_slot_count(),
        5,
        "锚在最新那条根上的窗口是 txg > 5"
    );
    let rollback_root = flow.judged_root_index;
    for checkpoint_txg in [TXG_OF_THE_NEWEST_ROOT, TXG_OF_THE_ROOT_RAISING_THE_FLOOR] {
        append_a_root_at_a_later_txg(
            &mut flow,
            rollback_root,
            checkpoint_txg,
            FLOOR_RAISED_PAST_THE_ROLLBACK,
            UnmountMarkerOfTheAppendedRoot::NotWritten,
        );
        flow.versions.push(PublishedVersion {
            instance: InstanceGeneration(2),
            checkpoint_txg: CheckpointTxg(checkpoint_txg),
            content: file_content(),
        });
    }
    flow
}

/// I-7.9 准入那一支的「之前的根」还要落在锚在镜像最新那条根上的一圈窗口里，不只是锚在被判那条根上的那一个：CPU 判过 I-7.9 而成立、
/// 一个状态都不红它，GPU 每个状态逐位相同。GPU 只取锚在被判那条根上的窗口时把 B 那条根算进来、红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_floor_raise_takes_no_root_outside_the_window_anchored_at_the_newest_root_of_the_image(
) {
    let flow = record_rollback_with_a_floor_raise_whose_window_anchored_at_the_newest_root_drops_an_older_root(
        "crash-judge-gpu-i79-image-window",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_that_a_floor_raise_takes_no_root_outside_the_window_anchored_at_the_newest_root_of_the_image",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_held_invariants.contains("I-7.9")
            && !comparison.cpu_checker_red_invariants.contains("I-7.9"),
        "流 {} 上 CPU 判过 I-7.9 而成立、一个状态都不红它（成立过的：{:?}；红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_held_invariants,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

// ───────────── 树表 11 条与种类 9–11（删除与并发那条线批 6、批 8 的盘面）：GPU 判器跟上之后各有一条分得出的流 ─────────────

/// 一个树表单元（码 2、key 宽 8、条目宽 200）的条目数（索引节点头里 key 区间之后那两字节）。
fn tree_table_entry_count(bytes: &[u8]) -> usize {
    let header_end = 86 + 2 * usize::from(index_node_key_width(bytes));
    usize::from(u16::from_le_bytes(
        bytes[header_end - 4..header_end - 2].try_into().expect("2"),
    ))
}

/// 扫描内核一张树表最多记几棵树：池级 checker 共用那一半里 `const C_MAX_TREES_PER_TABLE: u32 = Nu;` 那一行的 N。
fn tree_count_the_gpu_scan_kernel_records_per_tree_table() -> usize {
    const DECLARATION_PREFIX: &str = "const C_MAX_TREES_PER_TABLE: u32 = ";
    let source = singlefs_checker_tier::crash_judge_gpu::GPU_JUDGE_CHECKER_COMMON_SOURCE;
    let declarations: Vec<&str> = source
        .lines()
        .filter_map(|line| line.strip_prefix(DECLARATION_PREFIX))
        .collect();
    assert_eq!(
        declarations.len(),
        1,
        "池级 checker 共用那一半里恰好一行 `{DECLARATION_PREFIX}…`"
    );
    declarations[0]
        .strip_suffix("u;")
        .expect("声明以 `u;` 结尾")
        .parse()
        .expect("上限是十进制数")
}

/// 新池新建文件与固定脚本到回退 D 两条流写下的每一张树表，条目数都不超过 GPU 扫描内核一张树表能记的树数：核心层的树表再长
/// （删除与并发那条线批 6 把第一个文件版本的树表从 7 条加到 11 条，扫描内核那时只记 8 棵，第 9 棵起判不了），这条在取卡之前就红，
/// 不等对拍跑到新池那条流的状态 36。没卡的机器上也判到。
#[test]
fn every_tree_table_the_new_pool_and_the_rollback_flows_write_fits_the_tree_count_the_gpu_scan_kernel_records(
) {
    let capacity = tree_count_the_gpu_scan_kernel_records_per_tree_table();
    let is_a_tree_table = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 8
            && index_node_entry_width(bytes) == 200
    };
    for flow in [
        record_new_pool_file_creation("crash-judge-gpu-tree-count-new-pool", "new-pool"),
        record_fixed_script_through_rollback("crash-judge-gpu-tree-count-rollback"),
    ] {
        let entry_counts: Vec<usize> = flow
            .writes
            .iter()
            .filter(|write| write.kind == StepKind::UnitWrite)
            .filter_map(|write| write.bytes())
            .filter(|bytes| is_a_tree_table(bytes))
            .map(tree_table_entry_count)
            .collect();
        assert!(!entry_counts.is_empty(), "流 {} 写了树表单元", flow.name);
        let largest = entry_counts.iter().copied().max().expect("至少一张");
        assert!(
            largest <= capacity,
            "流 {} 写下的树表最多 {largest} 条，GPU 扫描内核一张树表只记 {capacity} 棵树：先把 C_MAX_TREES_PER_TABLE 跟上（草稿区顺移、SCRATCH_WORDS 重算）",
            flow.name
        );
    }
}

/// 第一版能出现的最大树表那条流：新池新建文件（txg 3）之后接连建 `SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION`（10）个快照，每一版都是第一个文件的内容。
/// 最后那一版的树表是进发号次序的 12 条加 10 对码 14 / 15 条目，共 32 条（判决 `research/prompts/m4-snapshot-r1-main-verification.md`
/// 第七节岔路 1 用户定的上限）。
///
/// # Panics
/// 最后那一版的树表里码 14 条目不是 10 条：那这条流就没走到第一版的上限。
fn record_new_pool_through_the_most_snapshots_of_the_first_version(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let mut pool = build_pool(tag);
    let mut versions = vec![published_version_of(&pool.output, file_content())];
    for _snapshot in 0..SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION {
        let created = create_a_snapshot_and_advance(&mut pool);
        versions.push(published_version_of(&created, file_content()));
    }
    assert_eq!(
        pool.output
            .tree_table_entries
            .iter()
            .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
            .count(),
        SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION,
        "最后那一版的树表里码 14 条目是第一版的快照上限那么多条"
    );
    recorded_flow_of_the_pool(&pool, name, versions)
}

/// 第一版能出现的最大树表（10 个快照：12 + 2 × 10 = 32 条）装得进 GPU 扫描内核一张树表能记的树数，而且恰好装满（上限少一棵这条就红），
/// CPU 参照一侧照这条流建得出判表（`build_judge_tables`）：快照六条流在 GPU 上判不了（站点 28：扫描内核一张树表只记 12 棵）那一次，
/// 上限只按发号次序的 12 种取、没把快照条目算进去。取卡之前就判，没卡的机器上也判到。
#[test]
fn the_tree_table_of_the_most_snapshots_of_the_first_version_fills_the_tree_count_the_gpu_scan_kernel_records_and_its_judge_tables_build(
) {
    const TREE_KINDS_IN_THE_ISSUING_ORDER: usize = 12;
    let capacity = tree_count_the_gpu_scan_kernel_records_per_tree_table();
    let flow = record_new_pool_through_the_most_snapshots_of_the_first_version(
        "crash-judge-gpu-tree-count-most-snapshots",
        "most-snapshots-of-the-first-version",
    );
    let is_a_tree_table = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 8
            && index_node_entry_width(bytes) == 200
    };
    let largest = flow
        .writes
        .iter()
        .filter(|write| write.kind == StepKind::UnitWrite)
        .filter_map(|write| write.bytes())
        .filter(|bytes| is_a_tree_table(bytes))
        .map(tree_table_entry_count)
        .max()
        .expect("这条流写了树表单元");
    assert_eq!(
        largest,
        TREE_KINDS_IN_THE_ISSUING_ORDER + 2 * SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION,
        "流 {} 最大的那张树表是发号次序的 12 条加每个快照一对",
        flow.name
    );
    assert_eq!(
        capacity, largest,
        "GPU 扫描内核一张树表能记的树数（C_MAX_TREES_PER_TABLE）就是第一版能出现的最大树表的条数：少了快照流判不了（站点 28），多了是没有来源的数"
    );
    let expansion =
        |_segment_index: usize, _segment: &[usize]| CrashPointReplaySegmentExpansion::NotExpanded;
    let judge_flow = JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    };
    let tables = build_judge_tables(&judge_flow);
    assert!(
        tables.is_ok(),
        "流 {} 上 CPU 参照一侧建得出判表：{:?}",
        flow.name,
        tables.err()
    );
}

/// 回退流上回退那次发布 D 往谱系树追加的那一片容器（码 3、记录类型 5）：记录宽从 24 改成 25、重封到根。
/// CPU 池级 checker 走谱系树走到这片容器、按 I-1.7 判它的记录宽不是登记的 24 → 红；恢复不读容器，两遍恢复照常读到文件。
/// GPU 走读内核不走谱系树、或不认谱系记录的登记宽时这一位对不上。
fn record_rollback_with_the_lineage_container_record_width_off_its_registered_width(
    tag: &str,
) -> RecordedFlow {
    const LINEAGE_RECORD_TYPE: u16 = 5;
    const PACKED_RECORD_WIDTH_OFFSET: usize = 71;
    let lineage_container =
        |bytes: &[u8]| unit_class(bytes) == 3 && packed_record_type(bytes) == LINEAGE_RECORD_TYPE;
    record_rollback_with_a_resealed_field(
        tag,
        "rollback-lineage-container-record-width-seventeen",
        &lineage_container,
        &|bytes| {
            let width = u16::from_le_bytes(
                bytes[PACKED_RECORD_WIDTH_OFFSET..PACKED_RECORD_WIDTH_OFFSET + 2]
                    .try_into()
                    .expect("2"),
            );
            assert_eq!(width, 24, "谱系记录宽 24");
            bytes[PACKED_RECORD_WIDTH_OFFSET..PACKED_RECORD_WIDTH_OFFSET + 2]
                .copy_from_slice(&(width + 1).to_le_bytes());
        },
    )
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_lineage_container_of_a_rollback_publish_carries_another_record_width(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_lineage_container_of_a_rollback_publish_carries_another_record_width",
        &record_rollback_with_the_lineage_container_record_width_off_its_registered_width(
            "crash-judge-gpu-lineage-container",
        ),
        "I-1.7",
    );
}

/// 回退流上回退那次发布 D 往谱系树追加的那一片容器（码 3、记录类型 5、记录宽 24）：第一条谱系记录的 H_cur（偏移 16，C 的 inode 号水位 2）
/// 改成 3、重封到根。落在 D 上的状态 CPU 池级 checker 判 I-9.30 ④：D 记账里 T_rb − 1 以前最大的台阶是 A 留的 (3, 2)，不等于 3 → 红；
/// 恢复不读谱系记录的内容，两遍恢复照常读到文件。GPU 扫描内核不读 H_cur、不判 I-9.30 ④，或谱系记录宽仍按 16 读时这一位对不上。
fn record_rollback_with_the_lineage_record_carrying_another_watermark_rolled_back_from(
    tag: &str,
) -> RecordedFlow {
    const LINEAGE_RECORD_TYPE: u16 = 5;
    const PACKED_UNIT_FIRST_RECORD_OFFSET: usize = 136;
    const LINEAGE_RECORD_INODE_NUMBER_WATERMARK_OF_THE_VERSION_ROLLED_BACK_FROM_OFFSET: usize = 16;
    let lineage_container =
        |bytes: &[u8]| unit_class(bytes) == 3 && packed_record_type(bytes) == LINEAGE_RECORD_TYPE;
    record_rollback_with_a_resealed_field(
        tag,
        "rollback-lineage-record-watermark-rolled-back-from-three",
        &lineage_container,
        &|bytes| {
            let offset = PACKED_UNIT_FIRST_RECORD_OFFSET
                + LINEAGE_RECORD_INODE_NUMBER_WATERMARK_OF_THE_VERSION_ROLLED_BACK_FROM_OFFSET;
            let watermark_rolled_back_from =
                u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8"));
            assert_eq!(
                watermark_rolled_back_from, 2,
                "D 的谱系记录带 C 的 inode 号水位 2"
            );
            bytes[offset..offset + 8].copy_from_slice(&3u64.to_le_bytes());
        },
    )
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_lineage_record_of_a_rollback_publish_carries_another_watermark_rolled_back_from(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_lineage_record_of_a_rollback_publish_carries_another_watermark_rolled_back_from",
        &record_rollback_with_the_lineage_record_carrying_another_watermark_rolled_back_from(
            "crash-judge-gpu-lineage-watermark-rolled-back-from",
        ),
        "I-9.30",
    );
}

/// 回退流上谱系树的根（码 2、key 宽 8、条目宽 120、第一条条目的类型段是谱系记录 5）：头里的树 ID 加 1、重封到根。
/// 落在 D 上的状态，CPU 恢复读谱系树根时核自描述（树 ID 不是树表条目里那棵）→ 走读失败、读不到文件；CPU 池级 checker 读这个根判 I-1.3 → 红。
/// GPU 恢复内核不读谱系树根、或读了不核树 ID，GPU 走读内核不读谱系树根时，恢复或这一位对不上。
fn record_rollback_with_the_lineage_root_naming_another_tree(tag: &str) -> RecordedFlow {
    const LINEAGE_RECORD_TYPE: u16 = 5;
    const INDEX_NODE_TREE_IDENTIFIER_OFFSET: usize = 42;
    const CONTAINER_TREE_ENTRY_TYPE_OFFSET: usize = 16;
    let lineage_root = |bytes: &[u8]| {
        if unit_class(bytes) != 2
            || index_node_key_width(bytes) != 8
            || index_node_entry_width(bytes) != 120
        {
            return false;
        }
        let first_entry = 86 + 2 * 8 + 29;
        u16::from_le_bytes(
            bytes[first_entry + CONTAINER_TREE_ENTRY_TYPE_OFFSET
                ..first_entry + CONTAINER_TREE_ENTRY_TYPE_OFFSET + 2]
                .try_into()
                .expect("2"),
        ) == LINEAGE_RECORD_TYPE
    };
    record_rollback_with_a_resealed_field(
        tag,
        "rollback-lineage-root-names-another-tree",
        &lineage_root,
        &|bytes| {
            bytes[INDEX_NODE_TREE_IDENTIFIER_OFFSET] =
                bytes[INDEX_NODE_TREE_IDENTIFIER_OFFSET].wrapping_add(1);
        },
    )
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_lineage_root_of_a_rollback_publish_names_another_tree(
) {
    let flow =
        record_rollback_with_the_lineage_root_naming_another_tree("crash-judge-gpu-lineage-root");
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_when_the_lineage_root_of_a_rollback_publish_names_another_tree",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-1.3"),
        "流 {} 上 CPU 池级 checker 要红 I-1.3（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.cpu_red > 0,
        "流 {} 上落在 D 上的状态 CPU 恢复读谱系树根要失败、判器要有红",
        flow.name
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// 新池新建文件那张树表里谱系树（种类 11）与 logged_ops（种类 12）两条互换种类码、重封到根：盘上次序仍按树 ID 升序（I-9.16 ① 成立），
/// 按发号次序谱系树在 logged_ops 之前、树 ID 却更大（② 不成立）。两条都是零根、头 ID 都该是 0，别的判定不受影响。
/// 根落了的状态 CPU 恢复在读任何一棵树之前拒（I-9.16）、判器有红，CPU 池级 checker 红 I-9.16；GPU 两处 ② 只数到种类 8 时都对不上。
fn record_new_pool_with_the_lineage_and_logged_operations_kinds_swapped(tag: &str) -> RecordedFlow {
    const LINEAGE_KIND: u16 = 11;
    const LOGGED_OPERATIONS_KIND: u16 = 12;
    const TREE_TABLE_FIRST_ENTRY_OFFSET: usize = 86 + 2 * 8 + 29;
    const TREE_TABLE_ENTRY_BYTES: usize = 200;
    const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 10;
    let tree_table = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 8
            && index_node_entry_width(bytes) == 200
    };
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-tree-table-lineage-and-logged-operations-kinds-swapped",
        &tree_table,
        false,
        &|bytes| {
            let kind_offset_of = |kind: u16| {
                let offsets: Vec<usize> = (0..tree_table_entry_count(bytes))
                    .map(|entry| {
                        TREE_TABLE_FIRST_ENTRY_OFFSET
                            + entry * TREE_TABLE_ENTRY_BYTES
                            + TREE_TABLE_ENTRY_KIND_OFFSET
                    })
                    .filter(|offset| {
                        u16::from_le_bytes(bytes[*offset..*offset + 2].try_into().expect("2"))
                            == kind
                    })
                    .collect();
                assert_eq!(offsets.len(), 1, "树表里种类 {kind} 恰好一条");
                offsets[0]
            };
            let lineage = kind_offset_of(LINEAGE_KIND);
            let logged_operations = kind_offset_of(LOGGED_OPERATIONS_KIND);
            assert!(
                lineage < logged_operations,
                "谱系树那条在 logged_ops 那条之前（树 ID 小）"
            );
            bytes[lineage..lineage + 2].copy_from_slice(&LOGGED_OPERATIONS_KIND.to_le_bytes());
            bytes[logged_operations..logged_operations + 2]
                .copy_from_slice(&LINEAGE_KIND.to_le_bytes());
        },
    )
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_lineage_and_logged_operations_entries_break_the_issuing_order(
) {
    let flow = record_new_pool_with_the_lineage_and_logged_operations_kinds_swapped(
        "crash-judge-gpu-i916-issuing-order",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_when_the_lineage_and_logged_operations_entries_break_the_issuing_order",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-9.16"),
        "流 {} 上 CPU 池级 checker 要红 I-9.16（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.cpu_red > 0,
        "流 {} 上根落了的状态 CPU 恢复要在读树之前拒、判器要有红",
        flow.name
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

// ───────────── 第 26 批 26-5：I-9.16 ③ ④、根槽 428..457、带树一版的分配记录树根、码 2 节点指针头部的 extent 偏移 ─────────────

/// 树表条目里种类那 2 字节的偏移（D8（核心索引结构） 已定项 8 的字段表：树 ID 8、条目长度 2 之后）。
const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 10;
/// 树表条目里树根指针（86 字节的码 2 指针）的偏移（种类 2、flags 2 之后）。
const TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET: usize = 14;
/// 稀疏旁表与 deadlist 的种类码（D18（块里携带什么信息） 已定项 24 第 6 条那一组）：第一版两棵树都不建根，树表条目的根指针全零，走读与恢复都不读。
const TREE_KIND_SPARSE_SIDE_TABLE: u16 = 7;
const TREE_KIND_DEADLIST: u16 = 8;
/// 一个没登记的种类码：走读与恢复都不认它、不读它的根（13 第 50 批登记给了退役树、14 / 15 里程碑四登记给了只读快照的两种条目
/// （快照 inode 根、快照 extent 根），改用 16，与 harness 档坏镜像里的 `TREE_KIND_NOT_REGISTERED` 同一个码）。
const UNREGISTERED_TREE_KIND: u16 = 16;
/// 分配记录树的种类码（D8（核心索引结构） 已定项 9 的登记表）。
const TREE_KIND_ALLOCATION_RECORD: u16 = 3;
/// 根记录里树表指针的第 0 条位置条目（盘 4 + 槽 6）的偏移：树表指针在 36，位置条目在指针偏移 50。
const ROOT_RECORD_TREE_TABLE_POINTER_FIRST_LOCATION_OFFSET: usize = 86;
/// 根记录里「树表 0 条那一版的分配记录树根」指针的偏移（D22（单元原子性怎么合成） 已定项 7 的字段表）。
const ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT_OFFSET: usize = 342;
/// 根记录里算法类型 1 + nonce 12 + MAC 16 那 29 字节的最后一个字节（428..457 的末字节）。
const ROOT_RECORD_LAST_MAC_BYTE_OFFSET: usize = 456;

/// 发布那张树表里种类是 `from_kind` 的那条（恰好一条、根指针全零）的种类改成 `to_kind`、重封到根。树 ID 与条目次序不动。
fn record_new_pool_with_a_tree_table_entry_kind_rewritten(
    tag: &str,
    name: &'static str,
    from_kind: u16,
    to_kind: u16,
) -> RecordedFlow {
    record_new_pool_with_a_resealed_field(tag, name, &is_a_tree_table_unit, false, &|bytes| {
        let entry = only_tree_table_entry_of_kind(bytes, from_kind);
        let root_pointer = entry + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET;
        assert_eq!(
            bytes[root_pointer..root_pointer + NODE_POINTER_BYTES],
            [0u8; NODE_POINTER_BYTES],
            "种类 {from_kind} 那条的根指针本来全零（这棵树第一版不建根，走读与恢复都不读它）"
        );
        let kind = entry + TREE_TABLE_ENTRY_KIND_OFFSET;
        bytes[kind..kind + 2].copy_from_slice(&to_kind.to_le_bytes());
    })
}

/// I-9.16 ③（每种至多一条）：deadlist 那条的种类改成稀疏旁表的 7——种类 7 两条、种类 8 一条都没有。① ② 照旧成立（② 按每种第一次出现的那条）。
/// core 的只读恢复在读任何一棵树之前拒（`tree_table_entries_each_kind_at_most_once`），GPU 恢复内核种类重复那一支同样拒。
fn record_new_pool_with_two_tree_table_entries_of_the_sparse_side_table_kind(
    tag: &str,
) -> RecordedFlow {
    record_new_pool_with_a_tree_table_entry_kind_rewritten(
        tag,
        "new-pool-tree-table-two-entries-of-the-sparse-side-table-kind",
        TREE_KIND_DEADLIST,
        TREE_KIND_SPARSE_SIDE_TABLE,
    )
}

/// I-9.16 ④（树表非空时十一种各恰好一条）：deadlist 那条的种类改成没登记的码（[`UNREGISTERED_TREE_KIND`]）——没有一种重复，deadlist 一条都没有。
/// core 的只读恢复照常读到文件（只要种类 1–4 四棵有根的树），可写挂载重建这一版时拒「树表里没有那棵树」；GPU 恢复内核也照常读到。
fn record_new_pool_with_no_tree_table_entry_of_the_deadlist_kind(tag: &str) -> RecordedFlow {
    record_new_pool_with_a_tree_table_entry_kind_rewritten(
        tag,
        "new-pool-tree-table-without-the-deadlist-kind",
        TREE_KIND_DEADLIST,
        UNREGISTERED_TREE_KIND,
    )
}

/// I-3.16 那道守（最新根那一版一棵分配记录树都没有，按记录口径的全空聚簇段数数不出，记射程里没判、没有违例时整条报不适用）走得到的形：发布那张树表里分配记录树那一条
/// （种类 3，带根）的种类改成没登记的码（[`UNREGISTERED_TREE_KIND`]）、重封到根。树 ID、根指针与条目次序不动，树表就缺了种类 3（I-9.16 ④ 判它）。
/// 分配记录树的节点照旧有中央映射条目，走读经映射条目把它们记进引用（`walk.rs` 的 `walk_central_mapping_entries`），I-3.1 那一份占着的槽不变，
/// I-3.16 走得过「占着的槽数乘 16 KiB 等于记账的已分配」那一格、走到按记录口径数全空段那一步，而这一版读到的分配记录树是 0 棵。
/// 不是 [`record_new_pool_with_a_tree_table_entry_kind_rewritten`]：那一个要被改的那一条根指针本来全零，分配记录树有根。
fn record_new_pool_with_no_tree_table_entry_of_the_allocation_record_kind(
    tag: &str,
) -> RecordedFlow {
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-tree-table-without-the-allocation-record-kind",
        &is_a_tree_table_unit,
        false,
        &|bytes| {
            let entry = only_tree_table_entry_of_kind(bytes, TREE_KIND_ALLOCATION_RECORD);
            let root_pointer = entry + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET;
            assert_ne!(
                bytes[root_pointer..root_pointer + NODE_POINTER_BYTES],
                [0u8; NODE_POINTER_BYTES],
                "发布那一版的分配记录树有根"
            );
            let kind = entry + TREE_TABLE_ENTRY_KIND_OFFSET;
            bytes[kind..kind + 2].copy_from_slice(&UNREGISTERED_TREE_KIND.to_le_bytes());
        },
    )
}

/// I-2.4 根记录那一处：发布的根记录里 MAC 末字节（偏移 456）写 1、重封根记录。core 不读这 29 字节（`root_record.rs` 的 `parse_slot` 跳过），
/// 两遍恢复照常；CPU 池级 checker 对根环里每条自证过的根判这 29 字节全 0，发布的根落了的状态上红 I-2.4。
fn record_new_pool_with_a_non_zero_byte_in_the_root_record_mac(tag: &str) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-root-record-mac-last-byte-one");
    let publish_root = flow.judged_root_index;
    rewrite_root_record_and_reseal(&mut flow, publish_root, &|bytes| {
        assert_eq!(
            bytes[ROOT_RECORD_LAST_MAC_BYTE_OFFSET], 0,
            "加密关着时根记录的 MAC 恒 0"
        );
        bytes[ROOT_RECORD_LAST_MAC_BYTE_OFFSET] = 1;
    });
    flow
}

/// I-2.4 根槽尾（C728）：发布的根记录之后的头一个字节（偏移 457）写 1、重封根记录。core 不读这一段，两遍恢复照常；
/// CPU 池级 checker 对根环里每条自证过的根判 [457, 槽宽) 全 0，发布的根落了的状态上红 I-2.4。
fn record_new_pool_with_a_non_zero_byte_past_the_root_record(tag: &str) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-root-slot-byte-past-the-record-one");
    let publish_root = flow.judged_root_index;
    let record_end = usize::try_from(singlefs_format::ROOT_RECORD_BYTES).expect("457");
    rewrite_root_record_and_reseal(&mut flow, publish_root, &|bytes| {
        assert!(bytes.len() > record_end, "根槽宽过根记录");
        assert_eq!(bytes[record_end], 0, "第一版根记录之后写 0");
        bytes[record_end] = 1;
    });
    flow
}

/// 第 `root_write` 次写（根槽写）指着的树表单元的字节：按根记录里树表指针第 0 条位置条目找根槽写之前落在那个槽上的最后一次单元写。
fn tree_table_unit_bytes_named_by_the_root_write(
    flow: &RecordedFlow,
    root_write: usize,
) -> Vec<u8> {
    let root_bytes = flow.writes[root_write].bytes().expect("根槽写带字节");
    let location = ROOT_RECORD_TREE_TABLE_POINTER_FIRST_LOCATION_OFFSET;
    let device = u32::from_le_bytes(root_bytes[location..location + 4].try_into().expect("4"));
    let mut slot = [0u8; 8];
    slot[..6].copy_from_slice(&root_bytes[location + 4..location + 10]);
    let slot = u64::from_le_bytes(slot);
    flow.writes[..root_write]
        .iter()
        .rev()
        .find(|write| {
            write.kind == StepKind::UnitWrite
                && write.device.0 == device
                && write.offset.0 / 16384 == slot
        })
        .and_then(RetainedWrite::bytes)
        .expect("根槽写之前写过它指着的树表单元")
        .to_vec()
}

/// I-5.4 那一格「一条根只有一棵分配记录树」：发布的根（带树的一版）根记录偏移 342 那条分配记录树根本来全零，换成它自己树表里种类 3 那条的根指针、
/// 重封根记录。走读把它当树 0 再走一遍那棵树（另红 I-1.3、I-5.1），发布那条 journal 记录的新根段里这一项仍全零（另红 I-8.10）；
/// 两棵树指着同一个根，I-5.4 的区间判定只判一次、不红——红 I-5.4 的只有全零那一格。core 的只读恢复在带树的一版上不读这一项，两遍恢复照常。
fn record_new_pool_with_the_allocation_record_tree_root_of_the_root_record_not_zero(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(
        tag,
        "new-pool-root-record-allocation-record-tree-root-not-zero",
    );
    let publish_root = flow.judged_root_index;
    let tree_table = tree_table_unit_bytes_named_by_the_root_write(&flow, publish_root);
    let allocation_entry = only_tree_table_entry_of_kind(&tree_table, 3);
    let allocation_root_pointer =
        tree_table[allocation_entry + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET
            ..allocation_entry + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET + NODE_POINTER_BYTES]
            .to_vec();
    assert_ne!(
        allocation_root_pointer,
        vec![0u8; NODE_POINTER_BYTES],
        "带树那一版的分配记录树有根"
    );
    rewrite_root_record_and_reseal(&mut flow, publish_root, &|bytes| {
        let held = ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT_OFFSET;
        assert_eq!(
            bytes[held..held + NODE_POINTER_BYTES],
            [0u8; NODE_POINTER_BYTES],
            "带树的一版根记录里这一项本来全零"
        );
        bytes[held..held + NODE_POINTER_BYTES].copy_from_slice(&allocation_root_pointer);
    });
    flow
}

/// I-2.4 指针那一半：发布那张树表里 inode 树那条（种类 2）的根指针头部 `field` 那一段写成非第一版的取值、重封到根。
/// 池级 checker 读码 2 节点（`read_index_node`）时判 MAC / nonce 与 extent 偏移，CPU 与 GPU 走读内核都红 I-2.4。core 读树表条目时判加密、压缩
/// 那几段与 extent 偏移、把那一条当损坏拒（`TreeTableEntry::parse`），GPU 恢复内核在 `walk_to_file` 解树表条目那一处同拒：这一版两遍恢复都是读失败。
fn record_new_pool_with_a_non_first_version_head_on_the_inode_tree_root_pointer(
    tag: &str,
    field: PointerHeadFieldWrittenNonZero,
) -> RecordedFlow {
    let name = match field {
        PointerHeadFieldWrittenNonZero::ExtentOffset => {
            "new-pool-inode-tree-root-pointer-extent-offset-one"
        }
        PointerHeadFieldWrittenNonZero::FirstMacByte => {
            "new-pool-inode-tree-root-pointer-first-mac-byte-one"
        }
        PointerHeadFieldWrittenNonZero::AlgorithmType => {
            "new-pool-inode-tree-root-pointer-algorithm-type-one"
        }
    };
    record_new_pool_with_a_resealed_field(tag, name, &is_a_tree_table_unit, false, &|bytes| {
        let entry = only_tree_table_entry_of_kind(bytes, 2);
        write_the_pointer_head_field_non_zero_at(
            bytes,
            entry + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET,
            field,
        );
    })
}

// ───────────── 第 29 批 29-2：指针头部 extent 偏移，恢复那一半照 core 判、池级 checker 那一半补三处 ─────────────

/// inode 内部条目里子指针的偏移（D8（核心索引结构） 的字段表：分隔 key 8、身份引用 26 之后）。
const INODE_INTERNAL_ENTRY_CHILD_POINTER_OFFSET: usize = 34;
/// extent 树的 key 宽（locality 8、inode 8、文件偏移 8）。
const EXTENT_KEY_BYTES: usize = 24;
/// extent 上段叶条目宽：key 24、标签 1、指针 88。
const EXTENT_UPPER_LEAF_ENTRY_BYTES: u16 = 113;
/// extent 下段叶记录宽：key 24、数据指针 88。
const EXTENT_LOWER_LEAF_RECORD_BYTES: u16 = 112;
/// extent 上段叶条目的标签：1 指一个下段根、2 内联一个数据指针（标签之后就是那条指针）。
const EXTENT_UPPER_LEAF_TAG_LOWER_SEGMENT_ROOT: u8 = 1;
const EXTENT_UPPER_LEAF_TAG_INLINE_DATA_UNIT: u8 = 2;
/// 分配记录树的 key 宽（盘 4、槽 6）与内部条目宽（key 10、子指针 86）。
const ALLOCATION_RECORD_TREE_KEY_BYTES: usize = 10;
const ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES: u16 = 96;
/// inode 树根的 key 宽与内部条目宽（D8（核心索引结构） 的 inode 内部条目 120）。
const INODE_TREE_KEY_BYTES: usize = 8;
const INODE_INTERNAL_ENTRY_BYTES: u16 = 120;

/// 偏移 `pointer` 起那条指针头部 extent 偏移那 2 字节（本来 0）写成 1。
fn set_the_extent_offset_of_the_pointer_at(bytes: &mut [u8], pointer: usize) {
    let extent_offset = pointer + POINTER_EXTENT_OFFSET_OFFSET;
    assert_eq!(
        bytes[extent_offset..extent_offset + 2],
        [0u8; 2],
        "第一版指针的 extent 偏移恒 0"
    );
    bytes[extent_offset..extent_offset + 2].copy_from_slice(&1u16.to_le_bytes());
}

/// 指针头部里写成非第一版取值的那一段（D19（块指针的结构与宽度预算） 已定项 7 的字段表）：extent 偏移（偏移 32 的 2 字节）写 1，
/// 或 MAC 第 0 字节写 1（加密关着时 MAC 恒 0，D19 已定项 3），或算法类型（偏移 28 的 1 字节）写 1（第一版不加密，D9（加密） 已定项 10）。
/// core 读走读跟随的指针都经 `read_judging_the_encryption_compression_and_extent_offset_fields_from` 几段一并判，GPU 恢复内核各段都判
/// （第 30 批 30-2 起）；池级 checker 的 CPU 与 GPU 走读内核判算法类型那一段从第 32 批起（判决 `research/prompts/m3-formal-code-b30-r1-main-verification.md`
/// 第二节 M2 改法 F2）。
#[derive(Clone, Copy, Debug)]
enum PointerHeadFieldWrittenNonZero {
    ExtentOffset,
    FirstMacByte,
    AlgorithmType,
}

/// 偏移 `pointer` 起那条指针头部 `field` 那一段（本来 0）写成非 0。
fn write_the_pointer_head_field_non_zero_at(
    bytes: &mut [u8],
    pointer: usize,
    field: PointerHeadFieldWrittenNonZero,
) {
    match field {
        PointerHeadFieldWrittenNonZero::ExtentOffset => {
            set_the_extent_offset_of_the_pointer_at(bytes, pointer);
        }
        PointerHeadFieldWrittenNonZero::FirstMacByte => {
            assert_eq!(bytes[pointer], 0, "加密关着时指针头部的 MAC 恒 0");
            bytes[pointer] = 1;
        }
        PointerHeadFieldWrittenNonZero::AlgorithmType => {
            let algorithm_type = pointer + POINTER_ALGORITHM_TYPE_OFFSET;
            assert_eq!(
                bytes[algorithm_type], 0,
                "第一版指针头部的算法类型恒 0（不加密）"
            );
            bytes[algorithm_type] = 1;
        }
    }
}

/// 根记录实例表指针那一处：发布的根记录里实例表指针头部 extent 偏移写 1、重封根记录。池级 checker 走实例表链第 0 片时判它
/// （`walk_instance_table_chain`），CPU 与 GPU 走读内核都红 I-2.4。core 择根时把这条根当损坏拒（`root_record.rs` 的 `parse_slot` 第五关），
/// 退回上一条根（看 journal 那一遍再重放回这一版）；GPU 恢复内核择根同拒（`root_record_pointer_heads_are_all_the_first_version`）。
fn record_new_pool_with_an_extent_offset_on_the_instance_table_pointer_of_the_root_record(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(
        tag,
        "new-pool-root-record-instance-table-pointer-extent-offset-one",
    );
    let publish_root = flow.judged_root_index;
    rewrite_root_record_and_reseal(&mut flow, publish_root, &|bytes| {
        set_the_extent_offset_of_the_pointer_at(bytes, ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET);
    });
    flow
}

/// inode 内部条目子指针那一处：发布那一版 inode 树根（码 2、key 宽 8、条目宽 120）第一条内部条目的子指针头部 `field` 那一段写成非第一版的取值、
/// 重封到根。池级 checker 走 inode 内部条目时判它，CPU 与 GPU 走读内核都红 I-2.4；core 解这条内部条目时拒
/// （`records::parse_inode_internal_entry_or_refusal`），GPU 恢复内核在 `walk_to_file` 读 inode 树那一处同拒。
fn record_new_pool_with_a_non_first_version_head_on_an_inode_internal_entry_child_pointer(
    tag: &str,
    field: PointerHeadFieldWrittenNonZero,
) -> RecordedFlow {
    let name = match field {
        PointerHeadFieldWrittenNonZero::ExtentOffset => {
            "new-pool-inode-internal-entry-child-pointer-extent-offset-one"
        }
        PointerHeadFieldWrittenNonZero::FirstMacByte => {
            "new-pool-inode-internal-entry-child-pointer-first-mac-byte-one"
        }
        PointerHeadFieldWrittenNonZero::AlgorithmType => {
            "new-pool-inode-internal-entry-child-pointer-algorithm-type-one"
        }
    };
    let inode_tree_root = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && usize::from(index_node_key_width(bytes)) == INODE_TREE_KEY_BYTES
            && index_node_entry_width(bytes) == INODE_INTERNAL_ENTRY_BYTES
    };
    record_new_pool_with_a_resealed_field(tag, name, &inode_tree_root, false, &|bytes| {
        let first_entry = index_node_entries_start(INODE_TREE_KEY_BYTES);
        write_the_pointer_head_field_non_zero_at(
            bytes,
            first_entry + INODE_INTERNAL_ENTRY_CHILD_POINTER_OFFSET,
            field,
        );
    })
}

/// extent 上段叶条目里的那条指针（标签之后）在单元里的偏移；条目的标签要是 `expected_tag`。
fn pointer_of_the_first_extent_upper_leaf_entry(bytes: &[u8], expected_tag: u8) -> usize {
    let first_entry = index_node_entries_start(EXTENT_KEY_BYTES);
    assert_eq!(
        bytes[first_entry + EXTENT_KEY_BYTES],
        expected_tag,
        "上段叶第一条的标签"
    );
    first_entry + EXTENT_KEY_BYTES + 1
}

fn is_an_extent_upper_leaf(bytes: &[u8]) -> bool {
    unit_class(bytes) == 2
        && usize::from(index_node_key_width(bytes)) == EXTENT_KEY_BYTES
        && index_node_entry_width(bytes) == EXTENT_UPPER_LEAF_ENTRY_BYTES
}

/// extent 数据指针那一处（内联在上段叶里的那条）：发布那一版唯一的数据单元由上段叶第一条内联指着（标签 2），这条数据指针头部 `field` 那一段
/// 写成非第一版的取值、重封到根。池级 checker 走数据指针时判它（`walk_extent_data_pointer`），CPU 与 GPU 走读内核都红 I-2.4；
/// core 解上段叶条目时拒（`ExtentUpperLeafEntry::parse`），GPU 恢复内核读上段叶那一处同拒。
fn record_new_pool_with_a_non_first_version_head_on_the_inline_data_pointer(
    tag: &str,
    field: PointerHeadFieldWrittenNonZero,
) -> RecordedFlow {
    let name = match field {
        PointerHeadFieldWrittenNonZero::ExtentOffset => {
            "new-pool-inline-data-pointer-extent-offset-one"
        }
        PointerHeadFieldWrittenNonZero::FirstMacByte => {
            "new-pool-inline-data-pointer-first-mac-byte-one"
        }
        PointerHeadFieldWrittenNonZero::AlgorithmType => {
            "new-pool-inline-data-pointer-algorithm-type-one"
        }
    };
    record_new_pool_with_a_resealed_field(tag, name, &is_an_extent_upper_leaf, false, &|bytes| {
        let pointer = pointer_of_the_first_extent_upper_leaf_entry(
            bytes,
            EXTENT_UPPER_LEAF_TAG_INLINE_DATA_UNIT,
        );
        write_the_pointer_head_field_non_zero_at(bytes, pointer, field);
    })
}

/// 分配记录树内部条目子指针那一处：发布那一版分配记录树的根（码 2、key 宽 10、条目宽 96；一棵树里最后写出的内部节点）第一条内部条目的子指针
/// 头部 `field` 那一段写成非第一版的取值、重封到根。池级 checker 读分配记录树的码 2 节点时判它（`read_index_node`），CPU 与 GPU 走读内核都红 I-2.4；
/// core 读这条内部条目时拒（`allocation_record_tree` 读子指针那一处），GPU 恢复内核 `read_allocation_tree` 同拒。
fn record_new_pool_with_a_non_first_version_head_on_an_allocation_record_tree_child_pointer(
    tag: &str,
    field: PointerHeadFieldWrittenNonZero,
) -> RecordedFlow {
    let name = match field {
        PointerHeadFieldWrittenNonZero::ExtentOffset => {
            "new-pool-allocation-record-tree-child-pointer-extent-offset-one"
        }
        PointerHeadFieldWrittenNonZero::FirstMacByte => {
            "new-pool-allocation-record-tree-child-pointer-first-mac-byte-one"
        }
        PointerHeadFieldWrittenNonZero::AlgorithmType => {
            "new-pool-allocation-record-tree-child-pointer-algorithm-type-one"
        }
    };
    let allocation_internal_node = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && usize::from(index_node_key_width(bytes)) == ALLOCATION_RECORD_TREE_KEY_BYTES
            && index_node_entry_width(bytes) == ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES
    };
    record_new_pool_with_a_resealed_field(tag, name, &allocation_internal_node, false, &|bytes| {
        let first_entry = index_node_entries_start(ALLOCATION_RECORD_TREE_KEY_BYTES);
        write_the_pointer_head_field_non_zero_at(
            bytes,
            first_entry + ALLOCATION_RECORD_TREE_KEY_BYTES,
            field,
        );
    })
}

/// 新池新建文件（txg 3）、第一个文件顺序写成三个数据单元（txg 4）：txg 4 那一版 extent 上段叶那一条指一个下段根（标签 1），
/// 下段叶记三条数据指针。搭建用 `common_deletion_in_progress` 的 `pool_with_a_three_unit_file`，不往下走删除。
fn record_new_pool_through_a_three_unit_file(tag: &str, name: &'static str) -> RecordedFlow {
    let pool = pool_with_a_three_unit_file(tag);
    recorded_flow_of_the_pool(
        &pool,
        name,
        vec![
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(3),
                content: file_content(),
            },
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(4),
                content: content_of_three_data_units(),
            },
        ],
    )
}

/// 三个数据单元那条流上改坏最后一个 `select` 命中的单元（两份副本）的一处、重封到根。
fn record_three_unit_file_with_a_resealed_field(
    tag: &str,
    name: &'static str,
    select: &dyn Fn(&[u8]) -> bool,
    mutate: &dyn Fn(&mut [u8]),
) -> RecordedFlow {
    let mut flow = record_new_pool_through_a_three_unit_file(tag, name);
    let targets = last_unit_copies(&flow, name, select, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, mutate);
    flow
}

/// extent 下段根指针那一处：txg 4 那一版上段叶第一条（标签 1）指着的下段根指针头部 `field` 那一段写成非第一版的取值、重封到根。
/// 池级 checker 读下段根这个码 2 节点时判它（`read_index_node`），CPU 与 GPU 走读内核都红 I-2.4；core 解上段叶条目时拒
/// （`ExtentUpperLeafEntry::parse`），GPU 恢复内核读上段叶那一处同拒。
fn record_three_unit_file_with_a_non_first_version_head_on_the_lower_segment_root_pointer(
    tag: &str,
    field: PointerHeadFieldWrittenNonZero,
) -> RecordedFlow {
    let name = match field {
        PointerHeadFieldWrittenNonZero::ExtentOffset => {
            "three-unit-file-lower-segment-root-pointer-extent-offset-one"
        }
        PointerHeadFieldWrittenNonZero::FirstMacByte => {
            "three-unit-file-lower-segment-root-pointer-first-mac-byte-one"
        }
        PointerHeadFieldWrittenNonZero::AlgorithmType => {
            "three-unit-file-lower-segment-root-pointer-algorithm-type-one"
        }
    };
    record_three_unit_file_with_a_resealed_field(tag, name, &is_an_extent_upper_leaf, &|bytes| {
        let pointer = pointer_of_the_first_extent_upper_leaf_entry(
            bytes,
            EXTENT_UPPER_LEAF_TAG_LOWER_SEGMENT_ROOT,
        );
        write_the_pointer_head_field_non_zero_at(bytes, pointer, field);
    })
}

/// extent 数据指针那一处（下段叶记录里的那条）：txg 4 那一版下段叶（码 2、key 宽 24、条目宽 112）第一条记录的数据指针头部 `field` 那一段
/// 写成非第一版的取值、重封到根。池级 checker 走数据指针时判它（`walk_extent_data_pointer`），CPU 与 GPU 走读内核都红 I-2.4；core 解这条叶记录时拒
/// （`records::parse_extent_record`），GPU 恢复内核 `read_lower_segment` 读叶那一处同拒。
fn record_three_unit_file_with_a_non_first_version_head_on_a_lower_leaf_data_pointer(
    tag: &str,
    field: PointerHeadFieldWrittenNonZero,
) -> RecordedFlow {
    let name = match field {
        PointerHeadFieldWrittenNonZero::ExtentOffset => {
            "three-unit-file-lower-leaf-data-pointer-extent-offset-one"
        }
        PointerHeadFieldWrittenNonZero::FirstMacByte => {
            "three-unit-file-lower-leaf-data-pointer-first-mac-byte-one"
        }
        PointerHeadFieldWrittenNonZero::AlgorithmType => {
            "three-unit-file-lower-leaf-data-pointer-algorithm-type-one"
        }
    };
    let extent_lower_leaf = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && usize::from(index_node_key_width(bytes)) == EXTENT_KEY_BYTES
            && index_node_entry_width(bytes) == EXTENT_LOWER_LEAF_RECORD_BYTES
    };
    record_three_unit_file_with_a_resealed_field(tag, name, &extent_lower_leaf, &|bytes| {
        let first_record = index_node_entries_start(EXTENT_KEY_BYTES);
        write_the_pointer_head_field_non_zero_at(bytes, first_record + EXTENT_KEY_BYTES, field);
    })
}

// ───────────── 第 30 批 30-2：容器树子指针 extent 偏移（E1）、恢复内核种类 ≥ 32 的重复、恢复内核指针头部加密压缩那几段 ─────────────

/// 墓碑树与谱系树码 2 节点条目里子指针的偏移（分隔 key 8、身份引用 26 之后）与条目宽，key 宽是容器号 8 字节。
const CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET_IN_THE_ENTRY: usize = 34;
const CONTAINER_TREE_ENTRY_BYTES: u16 = 120;
const CONTAINER_NUMBER_KEY_BYTES: usize = 8;
/// 码 2 节点条目里身份引用的类型段（分隔 key 8、出生树 8 之后的 2 字节）：指码 3 容器的条目写容器的记录类型。
const CONTAINER_TREE_ENTRY_RECORD_TYPE_OFFSET: usize = 16;
/// 谱系容器的打包记录类型（`singlefs_core::unit::PACKED_TYPE_LINEAGE`）。
const PACKED_RECORD_TYPE_LINEAGE: u16 = 5;

/// 一个墓碑树或谱系树的码 2 根（层级 1、key 宽 8、条目宽 120），第一条条目指一片记录类型 `record_type` 的容器。
/// inode 树根形状相同，靠第一条条目的类型段分开（inode 叶容器的类型是 2）。
fn is_a_container_tree_root_over_containers_of_record_type(bytes: &[u8], record_type: u16) -> bool {
    unit_class(bytes) == 2
        && usize::from(index_node_key_width(bytes)) == CONTAINER_NUMBER_KEY_BYTES
        && index_node_entry_width(bytes) == CONTAINER_TREE_ENTRY_BYTES
        && read_u16_at(
            bytes,
            index_node_entries_start(CONTAINER_NUMBER_KEY_BYTES)
                + CONTAINER_TREE_ENTRY_RECORD_TYPE_OFFSET,
        ) == record_type
}

/// 容器树根第一条条目里指向容器的子指针头部 extent 偏移写 1。
fn set_the_extent_offset_of_the_first_container_pointer(bytes: &mut [u8]) {
    set_the_extent_offset_of_the_pointer_at(
        bytes,
        index_node_entries_start(CONTAINER_NUMBER_KEY_BYTES)
            + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET_IN_THE_ENTRY,
    );
}

/// E1 墓碑树那一处：删除中间态的流上墓碑树根第一条条目里指向墓碑容器的子指针头部 extent 偏移写 1、重封到根。
/// 池级 checker 走墓碑树时判它（`walk_container_tree_root`），CPU 与 GPU 走读内核都红 I-2.4；core 读容器树的码 2 节点时拒这条子指针
/// （`read_container_tree_index_nodes`），GPU 恢复内核 `read_container_tree_index_nodes` 同拒，都在找第一个文件那条记录之前。
fn record_deletion_with_an_extent_offset_on_the_tombstone_container_pointer(
    tag: &str,
) -> RecordedFlow {
    record_deletion_with_a_resealed_field(
        tag,
        "deletion-tombstone-container-pointer-extent-offset-one",
        &|bytes| {
            is_a_container_tree_root_over_containers_of_record_type(
                bytes,
                PACKED_RECORD_TYPE_TOMBSTONE,
            )
        },
        &set_the_extent_offset_of_the_first_container_pointer,
    )
}

/// E1 谱系树那一处：固定脚本到回退 D 的流上谱系树根第一条条目里指向谱系容器的子指针头部 extent 偏移写 1、重封到根。
/// 池级 checker 走谱系树时判它，CPU 与 GPU 走读内核都红 I-2.4；core 读容器树的码 2 节点时拒这条子指针（`read_container_tree_index_nodes`），
/// GPU 恢复内核 `read_container_tree_index_nodes` 同拒（`record_pointer_head_is_the_first_version` 连 extent 偏移一起判）。
fn record_rollback_with_an_extent_offset_on_the_lineage_container_pointer(
    tag: &str,
) -> RecordedFlow {
    record_rollback_with_a_resealed_field(
        tag,
        "rollback-lineage-container-pointer-extent-offset-one",
        &|bytes| {
            is_a_container_tree_root_over_containers_of_record_type(
                bytes,
                PACKED_RECORD_TYPE_LINEAGE,
            )
        },
        &set_the_extent_offset_of_the_first_container_pointer,
    )
}

/// 一个第一版写者写不出的种类码（≥ 32）：GPU 恢复内核的种类位图只装得下 0..31，这一档逐条与前面各条比。
const TREE_KIND_NOT_BELOW_THIRTY_TWO: u16 = 40;

/// I-9.16 ③ 种类 ≥ 32 那一档（判决 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L4 第二行）：发布那张树表里
/// 稀疏旁表与 deadlist 两条（根指针都全零）的种类都改成 40——种类 40 两条、7 与 8 一条都没有。core 的只读恢复在读任何一棵树之前拒
/// （`tree_table_entries_each_kind_at_most_once` 比整个 u16），GPU 恢复内核对种类 ≥ 32 的条目逐条与前面比、同拒。
fn record_new_pool_with_two_tree_table_entries_of_a_kind_not_below_thirty_two(
    tag: &str,
) -> RecordedFlow {
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-tree-table-two-entries-of-kind-forty",
        &is_a_tree_table_unit,
        false,
        &|bytes| {
            for from_kind in [TREE_KIND_SPARSE_SIDE_TABLE, TREE_KIND_DEADLIST] {
                let entry = only_tree_table_entry_of_kind(bytes, from_kind);
                let root_pointer = entry + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET;
                assert_eq!(
                    bytes[root_pointer..root_pointer + NODE_POINTER_BYTES],
                    [0u8; NODE_POINTER_BYTES],
                    "种类 {from_kind} 那条的根指针本来全零（这棵树第一版不建根，走读与恢复都不读它）"
                );
                let kind = entry + TREE_TABLE_ENTRY_KIND_OFFSET;
                bytes[kind..kind + 2]
                    .copy_from_slice(&TREE_KIND_NOT_BELOW_THIRTY_TWO.to_le_bytes());
            }
        },
    )
}

/// E1 墓碑树：池级 checker 与恢复两半整份比，CPU 红 I-2.4。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_tombstone_container_pointer_carries_an_extent_offset()
{
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_tombstone_container_pointer_carries_an_extent_offset",
        &record_deletion_with_an_extent_offset_on_the_tombstone_container_pointer(
            "crash-judge-gpu-i24-tombstone-container-pointer-extent-offset",
        ),
        "I-2.4",
    );
}

/// E1 谱系树：池级 checker 与恢复两半整份比，CPU 红 I-2.4。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_lineage_container_pointer_carries_an_extent_offset() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_lineage_container_pointer_carries_an_extent_offset",
        &record_rollback_with_an_extent_offset_on_the_lineage_container_pointer(
            "crash-judge-gpu-i24-lineage-container-pointer-extent-offset",
        ),
        "I-2.4",
    );
}

/// I-9.16 ③：树表里种类 40 有两条（恢复那一半 core 拒、GPU 恢复内核要同拒）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_carries_two_entries_of_a_kind_not_below_thirty_two(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_carries_two_entries_of_a_kind_not_below_thirty_two",
        &record_new_pool_with_two_tree_table_entries_of_a_kind_not_below_thirty_two(
            "crash-judge-gpu-i916-kind-forty-twice",
        ),
        "I-9.16",
    );
}

/// I-2.4：树表条目里 inode 树根指针的 MAC 第 0 字节非 0（core 解树表条目时拒、GPU 恢复内核同拒）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tree_table_root_pointer_carries_a_non_zero_mac_byte() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tree_table_root_pointer_carries_a_non_zero_mac_byte",
        &record_new_pool_with_a_non_first_version_head_on_the_inode_tree_root_pointer(
            "crash-judge-gpu-i24-tree-table-root-pointer-mac",
            PointerHeadFieldWrittenNonZero::FirstMacByte,
        ),
        "I-2.4",
    );
}

/// I-2.4：inode 内部条目子指针的 MAC 第 0 字节非 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_internal_entry_child_pointer_carries_a_non_zero_mac_byte(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_internal_entry_child_pointer_carries_a_non_zero_mac_byte",
        &record_new_pool_with_a_non_first_version_head_on_an_inode_internal_entry_child_pointer(
            "crash-judge-gpu-i24-inode-child-pointer-mac",
            PointerHeadFieldWrittenNonZero::FirstMacByte,
        ),
        "I-2.4",
    );
}

/// I-2.4：extent 上段叶内联的数据指针的 MAC 第 0 字节非 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inline_extent_data_pointer_carries_a_non_zero_mac_byte(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_inline_extent_data_pointer_carries_a_non_zero_mac_byte",
        &record_new_pool_with_a_non_first_version_head_on_the_inline_data_pointer(
            "crash-judge-gpu-i24-inline-data-pointer-mac",
            PointerHeadFieldWrittenNonZero::FirstMacByte,
        ),
        "I-2.4",
    );
}

/// I-2.4：分配记录树内部条目子指针的 MAC 第 0 字节非 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_allocation_record_tree_child_pointer_carries_a_non_zero_mac_byte(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_allocation_record_tree_child_pointer_carries_a_non_zero_mac_byte",
        &record_new_pool_with_a_non_first_version_head_on_an_allocation_record_tree_child_pointer(
            "crash-judge-gpu-i24-allocation-child-pointer-mac",
            PointerHeadFieldWrittenNonZero::FirstMacByte,
        ),
        "I-2.4",
    );
}

/// I-2.4：extent 上段叶指着的下段根指针的 MAC 第 0 字节非 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_segment_root_pointer_carries_a_non_zero_mac_byte(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_segment_root_pointer_carries_a_non_zero_mac_byte",
        &record_three_unit_file_with_a_non_first_version_head_on_the_lower_segment_root_pointer(
            "crash-judge-gpu-i24-lower-segment-root-pointer-mac",
            PointerHeadFieldWrittenNonZero::FirstMacByte,
        ),
        "I-2.4",
    );
}

/// I-2.4：extent 上段叶内联的数据指针的算法类型写 1（core 解上段叶条目时拒、GPU 恢复内核同拒；池级 checker 的 CPU 与 GPU 走读内核
/// 判算法类型那一段，第 30 批判决第二节 M2 改法 F2）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inline_extent_data_pointer_names_an_encryption_algorithm(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_inline_extent_data_pointer_names_an_encryption_algorithm",
        &record_new_pool_with_a_non_first_version_head_on_the_inline_data_pointer(
            "crash-judge-gpu-i24-inline-data-pointer-algorithm-type",
            PointerHeadFieldWrittenNonZero::AlgorithmType,
        ),
        "I-2.4",
    );
}

/// I-2.4：extent 下段叶记录的数据指针的 MAC 第 0 字节非 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_leaf_data_pointer_carries_a_non_zero_mac_byte(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_leaf_data_pointer_carries_a_non_zero_mac_byte",
        &record_three_unit_file_with_a_non_first_version_head_on_a_lower_leaf_data_pointer(
            "crash-judge-gpu-i24-lower-leaf-data-pointer-mac",
            PointerHeadFieldWrittenNonZero::FirstMacByte,
        ),
        "I-2.4",
    );
}

/// I-9.16 ③：树表里一种种类有两条。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_carries_two_entries_of_one_kind() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_carries_two_entries_of_one_kind",
        &record_new_pool_with_two_tree_table_entries_of_the_sparse_side_table_kind(
            "crash-judge-gpu-i916-kind-twice",
        ),
        "I-9.16",
    );
}

/// I-9.16 ④：树表非空而十一种里缺 deadlist（种类 1–4 之外的那七种之一，CPU 的 ④ 按十一种判时才红）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_lacks_the_deadlist_kind() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_lacks_the_deadlist_kind",
        &record_new_pool_with_no_tree_table_entry_of_the_deadlist_kind(
            "crash-judge-gpu-i916-kind-missing",
        ),
        "I-9.16",
    );
}

/// I-3.16 那道守：树表缺种类 3（分配记录树）那一形（[`record_new_pool_with_no_tree_table_entry_of_the_allocation_record_kind`]）。
/// 读法（写段收尾件认的）：最新根那一版一棵分配记录树都没有，按记录口径的全空聚簇段数就不判（`walk.rs` 记射程里没判，没有违例时整条报不适用，
/// 不报部分没判）。CPU 一侧先钉流造对了——整条落盘的镜像上 I-3.16 报不适用、理由点名「一棵分配记录树都没有」（走到了按记录口径数全空段那一步，
/// 不是更早那几格的射程里没判），I-9.16 红。流上发布的根落了的状态，CPU 与 GPU 扫描内核都不置 I-3.16 的判到位
/// （`crash_judge_checker_scan.wgsl` 里 `newest_allocation_record_trees_read == 0u` 那一支记射程里没判），每个状态逐位相同；没有这道守的扫描内核
/// 按空的记录表数出整段都空、与记账对不上而判红，与 CPU 不一致。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_the_statistics_of_a_newest_version_without_an_allocation_record_tree_are_not_judged(
) {
    const TEST_NAME: &str = "the_gpu_judge_agrees_with_the_cpu_that_the_statistics_of_a_newest_version_without_an_allocation_record_tree_are_not_judged";
    let flow = record_new_pool_with_no_tree_table_entry_of_the_allocation_record_kind(
        "crash-judge-gpu-i316-no-allocation-record-tree",
    );
    let mut image_after_the_whole_flow = flow.base.clone();
    image_after_the_whole_flow.apply_writes(&flow.writes);
    match pool_checker_verdict_on_the_image(&image_after_the_whole_flow, "I-3.16") {
        InvariantVerdict::NotApplicable(reason) => assert!(
            reason.contains("一棵分配记录树都没有"),
            "不适用的理由点名最新根那一版一棵分配记录树都没有（不是更早那几格）：{reason}"
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::PartiallyJudged(_)) => panic!(
            "树表缺种类 3、分配记录树的节点照旧在映射里：I-3.16 要走到按记录口径那一步、不判（整条报不适用），实际 {other_verdict:?}"
        ),
    }
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-9.16"),
        "流 {} 上树表缺种类 3，CPU 池级 checker 要红 I-9.16（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// I-2.4：根记录里算法类型 / nonce / MAC 那 29 字节有非 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_root_record_carries_a_non_zero_mac_byte() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_root_record_carries_a_non_zero_mac_byte",
        &record_new_pool_with_a_non_zero_byte_in_the_root_record_mac(
            "crash-judge-gpu-i24-root-mac",
        ),
        "I-2.4",
    );
}

/// I-2.4（C728）：根记录之后到槽宽那一段有一个非 0 字节；GPU 走读内核对每条自证过的根判 [457, 槽宽) 全 0，逐位跟上 CPU。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_root_slot_carries_a_non_zero_byte_past_the_root_record()
{
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_root_slot_carries_a_non_zero_byte_past_the_root_record",
        &record_new_pool_with_a_non_zero_byte_past_the_root_record("crash-judge-gpu-i24-root-tail"),
        "I-2.4",
    );
}

/// I-5.4：带树的一版根记录里还持有一棵分配记录树。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_version_with_trees_also_holds_an_allocation_record_tree_in_its_root_record(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_version_with_trees_also_holds_an_allocation_record_tree_in_its_root_record",
        &record_new_pool_with_the_allocation_record_tree_root_of_the_root_record_not_zero(
            "crash-judge-gpu-i54-held-root",
        ),
        "I-5.4",
    );
}

/// 一片分配记录叶（码 2、层级 0、key 宽 10）里第一对「同一块盘、前一条正好接到后一条」的记录，前一条跨度加一（已释放标志照留）：
/// 两条罩住同一个槽。返回改了几条（0 或 1）。抄自 harness 档 `checker_known_bad_images.rs` 的同名函数（两档不共用代码）。
fn widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(
    bytes: &mut [u8],
) -> usize {
    const ALLOCATION_KEY_WIDTH: u8 = 10;
    const ALLOCATION_RECORD_BYTES: usize = 20;
    const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x8000;
    if bytes[50] != 0 || index_node_key_width(bytes) != ALLOCATION_KEY_WIDTH {
        return 0;
    }
    let header_end = 86 + 2 * usize::from(ALLOCATION_KEY_WIDTH);
    let record_count = usize::from(u16::from_le_bytes(
        bytes[header_end - 4..header_end - 2].try_into().expect("2"),
    ));
    let entries_start = 115 + 2 * usize::from(ALLOCATION_KEY_WIDTH);
    let record_at = |leaf_bytes: &[u8], record_index: usize| {
        let start = entries_start + ALLOCATION_RECORD_BYTES * record_index;
        let device = u32::from_le_bytes(leaf_bytes[start..start + 4].try_into().expect("4"));
        let mut slot_bytes = [0u8; 8];
        slot_bytes[..6].copy_from_slice(&leaf_bytes[start + 4..start + 10]);
        let span_field = u16::from_le_bytes([leaf_bytes[start + 10], leaf_bytes[start + 11]]);
        (start, device, u64::from_le_bytes(slot_bytes), span_field)
    };
    for record_index in 0..record_count.saturating_sub(1) {
        let (start, device, slot, span_field) = record_at(bytes, record_index);
        let (_, next_device, next_slot, _) = record_at(bytes, record_index + 1);
        let span_slots = span_field & !ALLOCATION_RECORD_RELEASED_FLAG;
        if device == next_device && slot + u64::from(span_slots) == next_slot {
            let widened = (span_field & ALLOCATION_RECORD_RELEASED_FLAG) | (span_slots + 1);
            bytes[start + 10..start + 12].copy_from_slice(&widened.to_le_bytes());
            return 1;
        }
    }
    0
}

/// I-5.4 按版本那一层多收下一次挂载先施加的那一版（改法 RV，判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P2）：
/// 新池新建文件那次发布写的最后一片分配记录叶里两条记录罩住同一个槽，重封到根与 journal 记录。发布那条记录落了、根槽没落的那几个状态上，
/// 这一版是下一次挂载先施加的那一版，只有改法 RV 之后的 I-5.4 判得到；根槽落了的状态上它是最新的根。两边每个状态逐位比。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_two_allocation_records_cover_one_slot_in_the_version_the_next_mount_applies_first(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_two_allocation_records_cover_one_slot_in_the_version_the_next_mount_applies_first",
        &record_new_pool_with_a_resealed_field(
            "crash-judge-gpu-i54-overlapping-records",
            "new-pool-overlapping-allocation-records",
            &|bytes| {
                unit_class(bytes) == 2
                    && index_node_key_width(bytes) == 10
                    && bytes[50] == 0
                    && widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(
                        &mut bytes.to_vec(),
                    ) == 1
            },
            false,
            &|bytes| {
                assert_eq!(
                    widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(bytes),
                    1,
                    "挑叶时这一片改得动"
                );
            },
        ),
        "I-5.4",
    );
}

/// I-2.4：树表条目里码 2 节点的指针头部 extent 偏移非 0（池级 checker 与恢复两半都比）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_code_two_node_pointer_carries_an_extent_offset() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_code_two_node_pointer_carries_an_extent_offset",
        &record_new_pool_with_a_non_first_version_head_on_the_inode_tree_root_pointer(
            "crash-judge-gpu-i24-node-pointer-extent-offset",
            PointerHeadFieldWrittenNonZero::ExtentOffset,
        ),
        "I-2.4",
    );
}

/// I-2.4：根记录里实例表指针的头部 extent 偏移非 0（core 择根拒这条根）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_of_a_root_record_carries_an_extent_offset(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_of_a_root_record_carries_an_extent_offset",
        &record_new_pool_with_an_extent_offset_on_the_instance_table_pointer_of_the_root_record(
            "crash-judge-gpu-i24-root-instance-table-pointer-extent-offset",
        ),
        "I-2.4",
    );
}

/// I-2.4：inode 内部条目子指针的头部 extent 偏移非 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_internal_entry_child_pointer_carries_an_extent_offset(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_internal_entry_child_pointer_carries_an_extent_offset",
        &record_new_pool_with_a_non_first_version_head_on_an_inode_internal_entry_child_pointer(
            "crash-judge-gpu-i24-inode-child-pointer-extent-offset",
            PointerHeadFieldWrittenNonZero::ExtentOffset,
        ),
        "I-2.4",
    );
}

/// I-2.4：extent 上段叶内联的数据指针的头部 extent 偏移非 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inline_extent_data_pointer_carries_an_extent_offset() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_inline_extent_data_pointer_carries_an_extent_offset",
        &record_new_pool_with_a_non_first_version_head_on_the_inline_data_pointer(
            "crash-judge-gpu-i24-inline-data-pointer-extent-offset",
            PointerHeadFieldWrittenNonZero::ExtentOffset,
        ),
        "I-2.4",
    );
}

/// I-2.4：分配记录树内部条目子指针的头部 extent 偏移非 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_allocation_record_tree_child_pointer_carries_an_extent_offset(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_allocation_record_tree_child_pointer_carries_an_extent_offset",
        &record_new_pool_with_a_non_first_version_head_on_an_allocation_record_tree_child_pointer(
            "crash-judge-gpu-i24-allocation-child-pointer-extent-offset",
            PointerHeadFieldWrittenNonZero::ExtentOffset,
        ),
        "I-2.4",
    );
}

/// I-2.4：extent 上段叶指着的下段根指针的头部 extent 偏移非 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_segment_root_pointer_carries_an_extent_offset(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_segment_root_pointer_carries_an_extent_offset",
        &record_three_unit_file_with_a_non_first_version_head_on_the_lower_segment_root_pointer(
            "crash-judge-gpu-i24-lower-segment-root-pointer-extent-offset",
            PointerHeadFieldWrittenNonZero::ExtentOffset,
        ),
        "I-2.4",
    );
}

/// I-2.4：extent 下段叶记录的数据指针的头部 extent 偏移非 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_leaf_data_pointer_carries_an_extent_offset(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_leaf_data_pointer_carries_an_extent_offset",
        &record_three_unit_file_with_a_non_first_version_head_on_a_lower_leaf_data_pointer(
            "crash-judge-gpu-i24-lower-leaf-data-pointer-extent-offset",
            PointerHeadFieldWrittenNonZero::ExtentOffset,
        ),
        "I-2.4",
    );
}

// ───────────── 里程碑三·3 删除线：I-9.5、I-3.4、I-4.7、I-4.6 与 I-9.6 墓碑那一半（GPU 扫描内核的删除态扫描与 CPU 池级 checker 逐状态逐位对拍）─────────────

/// 码 2 / 码 3 单元头里的声明长度（偏移 8，2 字节）。
const UNIT_DECLARED_LENGTH_OFFSET: usize = 8;
/// 码 3 单元头里的记录数（偏移 69，2 字节）与记录区起点（含 29 字节预留位的头 136）。
const PACKED_UNIT_RECORD_COUNT_OFFSET: usize = 69;
const PACKED_UNIT_RECORDS_START: usize = 136;
/// inode 记录 140 字节：号 0、nlink 36（4 字节）、size 40、blocks 48（D8（核心索引结构） 已定项 6 字段表）；blocks = ⌈size ÷ 512⌉（I-9.15）。
const INODE_RECORD_BYTES: usize = 140;
const INODE_RECORD_NLINK_OFFSET: usize = 36;
const INODE_RECORD_SIZE_OFFSET: usize = 40;
const INODE_RECORD_BLOCKS_OFFSET: usize = 48;
const INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES: u64 = 512;
/// 打包记录类型：1 墓碑、2 inode。
const PACKED_RECORD_TYPE_TOMBSTONE: u16 = 1;
const PACKED_RECORD_TYPE_INODE: u16 = 2;
/// 码 2 节点头（key 宽 k）：最小 key 在 52、最大 key 在 52 + k、条目数在 82 + 2k、条目区从 115 + 2k 起（D18（块里携带什么信息） 已定项 18）。
const INDEX_NODE_SMALLEST_KEY_OFFSET: usize = 52;
/// `deleted_inodes` 根：key 宽 8、条目宽 8（D8（核心索引结构） 已定项 16）。
const DELETED_INODES_KEY_BYTES: usize = 8;
/// 记账树叶：key 宽 22、条目 34（统计量 2、树 8、设备 4、代 8、值 8、seq 4），设备段在 10、值在 22。
const ACCOUNTING_KEY_BYTES: usize = 22;
const ACCOUNTING_ROW_BYTES: usize = 34;
const ACCOUNTING_ROW_DEVICE_OFFSET: usize = 10;
const ACCOUNTING_ROW_VALUE_OFFSET: usize = 22;
/// 待删除但意图未完成占用（D5（快照 / 空间记账机制） 已定项 4 第 4 项）与不带设备维的设备段。
const STATISTIC_PENDING_DELETION_OCCUPANCY: u16 = 4;
const STATISTIC_NO_DEVICE_DIMENSION: u32 = 0xFFFF_FFFF;
/// I-9.5 正方向那一形塞进队列的号：inode 树里没有它的记录、extent 树里也没有它的单元。
const INODE_NUMBER_QUEUED_WITHOUT_A_RECORD: u64 = 7;
/// I-9.6 墓碑那一半那一形把墓碑的对象 ID 改成的值：远大于这条流的 inode 号水位（新池新建文件之后是 2）。
const TOMBSTONE_OBJECT_IDENTIFIER_PAST_THE_WATERMARK: u64 = 1 << 40;

fn index_node_entry_count_offset(key_width: usize) -> usize {
    82 + 2 * key_width
}
fn index_node_entries_start(key_width: usize) -> usize {
    115 + 2 * key_width
}
fn read_u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("2"))
}
fn read_u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("4"))
}
fn read_u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8"))
}

/// 删除线那几条流共用的收尾：`pool` 从 mkfs 之后录下的写切段，被判的是最后一次根槽写。
fn recorded_flow_of_the_pool(
    pool: &BuiltPool,
    name: &'static str,
    versions: Vec<PublishedVersion>,
) -> RecordedFlow {
    let operations = pool.retained_operations();
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("发布有根槽写");
    RecordedFlow {
        name,
        base: pool.memory_pool_after_mkfs(),
        writes,
        segments,
        judged_root_index,
        versions,
    }
}

/// 删除中间态的流（`common_deletion_in_progress` 的搭建）：新池新建文件（txg 3）、第一个文件顺序写成三个数据单元（txg 4）、unlink 意图（txg 5：
/// inode 1 的记录 nlink 写成 0、号进 `deleted_inodes`、墓碑树开放容器追加一条）、按批大小 1 删第一批（txg 6：剩两个单元、size 两个净荷）。
/// 恢复从意图那一版起答「没有文件」，所以带文件的版本只有 txg 3、4。
fn record_new_pool_through_the_first_deletion_batch(tag: &str, name: &'static str) -> RecordedFlow {
    let mut pool = pool_with_a_three_unit_file(tag);
    publish_the_intent(&mut pool);
    publish_the_next_step_in_batches_of_one(&mut pool);
    recorded_flow_of_the_pool(
        &pool,
        name,
        versions_with_the_first_file_before_its_deletion(),
    )
}

/// 删除线那几条流上读得出第一个文件的两版：新建文件（txg 3）与顺序写成三个数据单元（txg 4）；意图那一版起恢复答「没有文件」。
fn versions_with_the_first_file_before_its_deletion() -> Vec<PublishedVersion> {
    vec![
        PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
            content: file_content(),
        },
        PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(4),
            content: content_of_three_data_units(),
        },
    ]
}

/// 三个单元的文件按批大小 1 删几批删完：前两批各放掉尾部一个单元，最后一批删记录、队列里的号与最后一个单元
/// （`unlink_single_file_publishes.rs` 的 `the_final_batch_removes_the_record_the_queue_key_and_every_remaining_unit_in_the_same_publish`）。
const BATCHES_OF_ONE_THAT_DELETE_A_THREE_UNIT_FILE: usize = 3;

/// 删完的流：同 [`record_new_pool_through_the_first_deletion_batch`] 的搭建，再一批批删到最后一批（txg 6、7、8）。最后一批那一版起
/// extent 树只剩上段零条目根（D8（核心索引结构） 已定项 16 第 5 条）、没有下段与数据单元，inode 树里没有第一个文件那条记录，
/// 队列空；恢复从意图那一版起答「没有文件」，带文件的版本同样只有 txg 3、4。
///
/// # Panics
/// 最后一批那一版还有第一个文件的记录、队列里的号、数据单元或下段，或 extent 上段不是只剩一个节点：那这条流就没走到零条目根。
fn record_new_pool_through_the_last_deletion_batch(tag: &str, name: &'static str) -> RecordedFlow {
    let mut pool = pool_with_a_three_unit_file(tag);
    publish_the_intent(&mut pool);
    let last_batch = (0..BATCHES_OF_ONE_THAT_DELETE_A_THREE_UNIT_FILE)
        .map(|_batch_index| publish_the_next_step_in_batches_of_one(&mut pool))
        .last()
        .expect("删完要至少一批");
    assert!(
        last_batch.inode_record.is_none()
            && last_batch.deleted_inodes.is_empty()
            && last_batch.data_pointers.is_empty()
            && last_batch.extent_tree.lower_nodes.is_empty()
            && last_batch.extent_tree.upper_nodes.len() == 1,
        "第 {BATCHES_OF_ONE_THAT_DELETE_A_THREE_UNIT_FILE} 批是最后一批：记录、队列里的号、数据单元与下段都不剩，extent 树只剩上段零条目根（{:?}）",
        last_batch.extent_tree
    );
    recorded_flow_of_the_pool(
        &pool,
        name,
        versions_with_the_first_file_before_its_deletion(),
    )
}

/// 删除中间态的流上改坏最后一个 `select` 命中的单元（两份副本）的一处、重封到根。
fn record_deletion_with_a_resealed_field(
    tag: &str,
    name: &'static str,
    select: &dyn Fn(&[u8]) -> bool,
    mutate: &dyn Fn(&mut [u8]),
) -> RecordedFlow {
    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
    let targets = last_unit_copies(&flow, name, select, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, mutate);
    flow
}

fn is_a_deleted_inodes_root(bytes: &[u8]) -> bool {
    unit_class(bytes) == 2
        && usize::from(index_node_key_width(bytes)) == DELETED_INODES_KEY_BYTES
        && usize::from(index_node_entry_width(bytes)) == DELETED_INODES_KEY_BYTES
}
fn is_an_accounting_leaf(bytes: &[u8]) -> bool {
    unit_class(bytes) == 2
        && usize::from(index_node_key_width(bytes)) == ACCOUNTING_KEY_BYTES
        && usize::from(index_node_entry_width(bytes)) == ACCOUNTING_ROW_BYTES
}
fn is_an_inode_leaf_container(bytes: &[u8]) -> bool {
    unit_class(bytes) == 3 && packed_record_type(bytes) == PACKED_RECORD_TYPE_INODE
}
fn is_a_tombstone_container(bytes: &[u8]) -> bool {
    unit_class(bytes) == 3 && packed_record_type(bytes) == PACKED_RECORD_TYPE_TOMBSTONE
}

/// inode 叶容器里号 `inode` 那条记录的起点。
fn offset_of_the_inode_record(bytes: &[u8], inode: u64) -> usize {
    let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
    (0..record_count)
        .map(|index| PACKED_UNIT_RECORDS_START + index * INODE_RECORD_BYTES)
        .find(|offset| read_u64_at(bytes, *offset) == inode)
        .unwrap_or_else(|| panic!("这片 inode 叶容器里有 inode {inode} 的记录"))
}

/// I-9.5 正方向那一形：`deleted_inodes` 根 {1} 改成 {1, 7}（条目数加一、声明长度加 8、头里最大 key 改成 7），7 号在 inode 树里没有记录。
fn put_an_inode_number_without_a_record_into_the_deletion_queue_root(bytes: &mut [u8]) {
    let count_offset = index_node_entry_count_offset(DELETED_INODES_KEY_BYTES);
    let entry_count = usize::from(read_u16_at(bytes, count_offset));
    let declared_length = read_u16_at(bytes, UNIT_DECLARED_LENGTH_OFFSET);
    assert_eq!(
        usize::from(declared_length),
        entry_count * DELETED_INODES_KEY_BYTES,
        "队列根的声明长度是条目数 × 8"
    );
    let new_entry =
        index_node_entries_start(DELETED_INODES_KEY_BYTES) + entry_count * DELETED_INODES_KEY_BYTES;
    bytes[new_entry..new_entry + DELETED_INODES_KEY_BYTES]
        .copy_from_slice(&INODE_NUMBER_QUEUED_WITHOUT_A_RECORD.to_le_bytes());
    bytes[count_offset..count_offset + 2].copy_from_slice(
        &u16::try_from(entry_count + 1)
            .expect("装得进 u16")
            .to_le_bytes(),
    );
    bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
        &(declared_length + u16::try_from(DELETED_INODES_KEY_BYTES).expect("8")).to_le_bytes(),
    );
    let largest_key = INDEX_NODE_SMALLEST_KEY_OFFSET + DELETED_INODES_KEY_BYTES;
    bytes[largest_key..largest_key + DELETED_INODES_KEY_BYTES]
        .copy_from_slice(&INODE_NUMBER_QUEUED_WITHOUT_A_RECORD.to_le_bytes());
}

/// I-3.4 那一形：记账叶里待删除但意图未完成占用（标签 4、设备段 0xFFFF_FFFF）那一行的值 0 → 1。
fn set_the_pending_deletion_occupancy_row_to_one(bytes: &mut [u8]) {
    let entry_count = usize::from(read_u16_at(
        bytes,
        index_node_entry_count_offset(ACCOUNTING_KEY_BYTES),
    ));
    let row = (0..entry_count)
        .map(|index| index_node_entries_start(ACCOUNTING_KEY_BYTES) + index * ACCOUNTING_ROW_BYTES)
        .find(|row| {
            read_u16_at(bytes, *row) == STATISTIC_PENDING_DELETION_OCCUPANCY
                && read_u32_at(bytes, row + ACCOUNTING_ROW_DEVICE_OFFSET)
                    == STATISTIC_NO_DEVICE_DIMENSION
        })
        .expect("记账叶里有待删除但意图未完成占用那一行");
    assert_eq!(
        read_u64_at(bytes, row + ACCOUNTING_ROW_VALUE_OFFSET),
        0,
        "那一行恒 0"
    );
    bytes[row + ACCOUNTING_ROW_VALUE_OFFSET..row + ACCOUNTING_ROW_VALUE_OFFSET + 8]
        .copy_from_slice(&1u64.to_le_bytes());
}

/// I-4.6 那一形：inode 叶容器里第一个文件（删除中、nlink = 0）的 size 多加一个净荷容量，blocks 跟着按 ⌈size ÷ 512⌉ 改（I-9.15 照旧成立），
/// extent 树里的单元数不动。
fn raise_the_size_of_the_first_file_by_one_data_unit_payload(bytes: &mut [u8]) {
    let record = offset_of_the_inode_record(bytes, FIRST_FILE.0);
    let payload_capacity =
        u64::try_from(singlefs_core::unit::data_unit_payload_capacity()).expect("装得进 u64");
    let size = read_u64_at(bytes, record + INODE_RECORD_SIZE_OFFSET) + payload_capacity;
    bytes[record + INODE_RECORD_SIZE_OFFSET..record + INODE_RECORD_SIZE_OFFSET + 8]
        .copy_from_slice(&size.to_le_bytes());
    bytes[record + INODE_RECORD_BLOCKS_OFFSET..record + INODE_RECORD_BLOCKS_OFFSET + 8]
        .copy_from_slice(
            &size
                .div_ceil(INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES)
                .to_le_bytes(),
        );
}

/// I-9.6 墓碑那一半那一形：墓碑容器第一条墓碑的对象 ID 改成远大于 inode 号水位的值。
fn raise_the_first_tombstone_object_identifier_past_the_watermark(bytes: &mut [u8]) {
    assert!(
        read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET) >= 1,
        "墓碑容器里至少一条墓碑"
    );
    bytes[PACKED_UNIT_RECORDS_START..PACKED_UNIT_RECORDS_START + 8]
        .copy_from_slice(&TOMBSTONE_OBJECT_IDENTIFIER_PAST_THE_WATERMARK.to_le_bytes());
}

/// I-3.4 缺行那一形：记账叶里待删除但意图未完成占用那一行整行拿掉（后面的行往前挪、条目数减一、声明长度减 34；这一行不在首末，头里的 key 区间不变）。
fn drop_the_pending_deletion_occupancy_row(bytes: &mut [u8]) {
    let count_offset = index_node_entry_count_offset(ACCOUNTING_KEY_BYTES);
    let entry_count = usize::from(read_u16_at(bytes, count_offset));
    let entries_start = index_node_entries_start(ACCOUNTING_KEY_BYTES);
    let row_index = (0..entry_count)
        .find(|index| {
            let row = entries_start + index * ACCOUNTING_ROW_BYTES;
            read_u16_at(bytes, row) == STATISTIC_PENDING_DELETION_OCCUPANCY
                && read_u32_at(bytes, row + ACCOUNTING_ROW_DEVICE_OFFSET)
                    == STATISTIC_NO_DEVICE_DIMENSION
        })
        .expect("记账叶里有待删除但意图未完成占用那一行");
    assert!(
        row_index > 0 && row_index + 1 < entry_count,
        "那一行不在首末：拿掉它头里的 key 区间不变"
    );
    let row = entries_start + row_index * ACCOUNTING_ROW_BYTES;
    let rows_end = entries_start + entry_count * ACCOUNTING_ROW_BYTES;
    bytes.copy_within(row + ACCOUNTING_ROW_BYTES..rows_end, row);
    bytes[rows_end - ACCOUNTING_ROW_BYTES..rows_end].fill(0);
    bytes[count_offset..count_offset + 2].copy_from_slice(
        &u16::try_from(entry_count - 1)
            .expect("装得进 u16")
            .to_le_bytes(),
    );
    let declared_length = read_u16_at(bytes, UNIT_DECLARED_LENGTH_OFFSET);
    bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
        &(declared_length - u16::try_from(ACCOUNTING_ROW_BYTES).expect("34")).to_le_bytes(),
    );
}

/// 新池新建文件之后建一个空 inode（txg 4，号 2，size 0、没有数据单元，与第一个文件同在最左那片叶里）的流：
/// 那片叶（最后一个 inode 叶容器的两份副本）照 `mutate` 改一处、重封到根。
fn record_new_pool_with_a_new_inode_and_its_inode_leaf_resealed(
    tag: &str,
    name: &'static str,
    mutate: &dyn Fn(&mut [u8]),
) -> RecordedFlow {
    let pool = pool_with_new_inodes(tag, 1);
    let mut flow = recorded_flow_of_the_pool(
        &pool,
        name,
        vec![
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(3),
                content: file_content(),
            },
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(4),
                content: file_content(),
            },
        ],
    );
    let targets = last_unit_copies(&flow, name, &is_an_inode_leaf_container, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, mutate);
    flow
}

/// I-4.7 那一形（判决 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节 F1 的孤儿单元，在 checker 侧直接改字节造）：
/// 空 inode 那片叶里 1 号的记录拿掉（后面的记录往前挪、记录数减一、声明长度减 140），extent 树不动：1 号在 extent 树里还有数据单元、
/// inode 树里没有它的记录、号也不在队列里。
fn drop_the_record_of_the_first_file(bytes: &mut [u8]) {
    let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
    assert!(record_count >= 2, "第一个文件与空 inode 同在这片叶里");
    let dropped = offset_of_the_inode_record(bytes, FIRST_FILE.0);
    let records_end = PACKED_UNIT_RECORDS_START + record_count * INODE_RECORD_BYTES;
    bytes.copy_within(dropped + INODE_RECORD_BYTES..records_end, dropped);
    bytes[records_end - INODE_RECORD_BYTES..records_end].fill(0);
    bytes[PACKED_UNIT_RECORD_COUNT_OFFSET..PACKED_UNIT_RECORD_COUNT_OFFSET + 2].copy_from_slice(
        &u16::try_from(record_count - 1)
            .expect("装得进 u16")
            .to_le_bytes(),
    );
    let declared_length = read_u16_at(bytes, UNIT_DECLARED_LENGTH_OFFSET);
    bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
        &(declared_length - u16::try_from(INODE_RECORD_BYTES).expect("140")).to_le_bytes(),
    );
}

/// I-9.5 反方向那一形（C708、判决里说的僵尸记录）：空 inode 2 的记录 nlink 1 → 0，号不进队列（它没有数据单元，I-4.7、I-4.6 照旧成立）。
fn set_the_nlink_of_the_new_inode_to_zero(bytes: &mut [u8]) {
    let record = offset_of_the_inode_record(bytes, FIRST_FILE.0 + 1);
    assert_eq!(
        read_u32_at(bytes, record + INODE_RECORD_NLINK_OFFSET),
        1,
        "空 inode 的 nlink 是 1"
    );
    bytes[record + INODE_RECORD_NLINK_OFFSET..record + INODE_RECORD_NLINK_OFFSET + 4]
        .copy_from_slice(&0u32.to_le_bytes());
}

/// 删除中间态（意图之后、第一批之后）的每个状态上 GPU 与 CPU 整份相同（红位、池级 checker 的违例位、两遍恢复落到的根与结局：意图那一版起
/// 两边都把 nlink = 0 的第一个文件答成没有文件，C712（GPU 恢复内核不判 nlink = 0））；删除线四条在这条流上 CPU 判过而成立、一个状态都不红
/// （I-9.5 队列 {1} 与 nlink = 0 的号集合相同、I-3.4 标签 4 那一行在且为 0、I-4.7 号 1 在队列里、I-4.6 单元数等于 ⌈size ÷ 净荷容量⌉）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress() {
    let flow = record_new_pool_through_the_first_deletion_batch(
        "crash-judge-gpu-deletion-in-progress",
        "deletion-through-the-first-batch",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
    for invariant in ["I-9.5", "I-3.4", "I-4.7", "I-4.6"] {
        assert!(
            comparison.cpu_checker_held_invariants.contains(invariant),
            "删除中间态的流上 CPU 池级 checker 判过 {invariant} 而成立（成立过的：{:?}）",
            comparison.cpu_checker_held_invariants
        );
        assert!(
            !comparison.cpu_checker_red_invariants.contains(invariant),
            "删除中间态的流上 CPU 池级 checker 不红 {invariant}（红过的：{:?}）",
            comparison.cpu_checker_red_invariants
        );
    }
}

/// 删到最后一批（意图之后三批）的每个状态上 GPU 与 CPU 整份相同（红位、池级 checker 的违例位、两遍恢复落到的根与结局）：最后一批那一版起
/// extent 树只剩上段零条目根，CPU 冷走读把它当第一个文件删完之后的合法形态收下（`recovery.rs` 的 `extents_of_the_first_file`）、
/// inode 树里没有那条记录、答没有文件，GPU 恢复内核同判（`judge_extent_node` 只放过上段根兼叶的零条目，读第一个文件放过零条叶条目）；
/// 删除线四条在这条流上 CPU 判过而成立、一个状态都不红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_through_the_last_batch() {
    let flow = record_new_pool_through_the_last_deletion_batch(
        "crash-judge-gpu-deletion-through-the-last-batch",
        "deletion-through-the-last-batch",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_through_the_last_batch",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
    for invariant in ["I-9.5", "I-3.4", "I-4.7", "I-4.6"] {
        assert!(
            comparison.cpu_checker_held_invariants.contains(invariant),
            "删到最后一批的流上 CPU 池级 checker 判过 {invariant} 而成立（成立过的：{:?}）",
            comparison.cpu_checker_held_invariants
        );
        assert!(
            !comparison.cpu_checker_red_invariants.contains(invariant),
            "删到最后一批的流上 CPU 池级 checker 不红 {invariant}（红过的：{:?}）",
            comparison.cpu_checker_red_invariants
        );
    }
}

/// 一条流换一种切段，造出「之前的段全落、第 `segment_index` 段只差第 `missing_write_index` 次写」那一个状态：`segments` 把原来那一段拆成
/// 「这一段其余的写」与「只有这一次写」前后两段（别的段照旧），只展开后一段（下标 `segment_index + 1`）。持久集合与原来那一段全量展开
/// （[`CrashPointReplaySegmentExpansion::EveryProperSubset`]）枚举到的状态之一逐个相同；状态只有它与最后全落那一个。
struct SegmentsMissingOneWrite {
    segment_index: usize,
    missing_write_index: usize,
    segments: Vec<Vec<usize>>,
}

/// 写数超过 `limit`（按这个展开上限不展开）的每一段、段里每一次写各一种切法（[`SegmentsMissingOneWrite`]）。
///
/// # Panics
/// 这几段里有不是单元写的写：单元写是 COW、只取两态（`TearableInPlaceOverwrites` 认法第 1 条），拆段不改它落不落；原地覆写的撕裂镜像要重放
/// 同一段里更晚的重叠写，拆开之后那几次写归到前一段，造出的状态就不是原来那一段的了。
fn segments_missing_one_write_of_each_unexpanded_segment(
    flow: &RecordedFlow,
    limit: usize,
) -> Vec<SegmentsMissingOneWrite> {
    let mut variants = Vec::new();
    for (segment_index, segment) in flow.segments.iter().enumerate() {
        if segment.len() <= limit {
            continue;
        }
        for write_index in segment {
            assert_eq!(
                flow.writes[*write_index].kind,
                StepKind::UnitWrite,
                "流 {} 不展开的第 {segment_index} 段里第 {write_index} 次写要是单元写，拆段才不改它的落地",
                flow.name
            );
        }
        for missing_write_index in segment {
            let rest_of_the_segment: Vec<usize> = segment
                .iter()
                .copied()
                .filter(|write_index| write_index != missing_write_index)
                .collect();
            let mut segments = Vec::with_capacity(flow.segments.len() + 1);
            segments.extend(flow.segments[..segment_index].iter().cloned());
            segments.push(rest_of_the_segment);
            segments.push(vec![*missing_write_index]);
            segments.extend(flow.segments[segment_index + 1..].iter().cloned());
            variants.push(SegmentsMissingOneWrite {
                segment_index,
                missing_write_index: *missing_write_index,
                segments,
            });
        }
    }
    variants
}

/// 删除中间态那条流上，展开上限不展开的段（新建文件、顺序写三个单元、意图、第一批那几次发布的单元写，每段二三十个写）在
/// [`compare_on_every_state`] 里一个状态都不比：「这一次发布的单元写落了一半」时 GPU 的删除态扫描与恢复内核从没和 CPU 比过
/// （第 18 批代码轮攻方报告第四节）。这里每段每次写各比一次「之前的段全落、这一段只差这一次写」的状态（连同最后全落那一个），
/// GPU 与 CPU 整份相同；CPU 那一侧逐状态现判，不抄数。第一批那一段的这些状态上最新落盘的根是意图那一版，第一个文件 nlink = 0，
/// 两边都要答没有文件（C712（GPU 恢复内核不判 nlink = 0））。每种切法恰好两个状态在取卡之前判，没卡的机器上也判到。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_every_state_missing_one_write_of_an_unexpanded_segment_of_a_deletion_in_progress(
) {
    let flow = record_new_pool_through_the_first_deletion_batch(
        "crash-judge-gpu-deletion-missing-one-write",
        "deletion-through-the-first-batch-missing-one-write",
    );
    let limit = expand_up_to();
    let variants = segments_missing_one_write_of_each_unexpanded_segment(&flow, limit);
    assert!(
        !variants.is_empty(),
        "流 {} 按展开上限 {limit} 要有不展开的段（发布的单元写），这条用例才比得到它们",
        flow.name
    );
    let expansion_of = |variant: &SegmentsMissingOneWrite| {
        let expanded_segment_index = variant.segment_index + 1;
        move |segment_index: usize, _segment: &[usize]| {
            if segment_index == expanded_segment_index {
                CrashPointReplaySegmentExpansion::EveryProperSubset
            } else {
                CrashPointReplaySegmentExpansion::NotExpanded
            }
        }
    };
    for variant in &variants {
        assert_eq!(
            crash_point_replay_plan_state_count(
                &flow.base,
                &flow.writes,
                &variant.segments,
                &expansion_of(variant)
            ),
            2,
            "第 {} 段只差第 {} 次写的切法只有那一个状态与最后全落那一个",
            variant.segment_index,
            variant.missing_write_index
        );
    }
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_on_every_state_missing_one_write_of_an_unexpanded_segment_of_a_deletion_in_progress",
    ) else {
        return;
    };
    let mut failures = Vec::new();
    for variant in &variants {
        // 每种切法一个项名：同一条流在两类库里按切法各录一份
        let item_name = format!(
            "{}.segment-{}-missing-write-{}",
            flow.name, variant.segment_index, variant.missing_write_index
        );
        let Some(comparison) = compare_on_every_state_of_these_segments_as_the_item(
            cards,
            &item_name,
            &flow,
            &variant.segments,
            &expansion_of(variant),
        ) else {
            return;
        };
        if !comparison.mismatches.is_empty() {
            failures.push(format!(
                "第 {} 段只差第 {} 次写：\n{}",
                variant.segment_index,
                variant.missing_write_index,
                mismatch_listing(&comparison)
            ));
        }
    }
    eprintln!(
        "CRASH_JUDGE_GPU_MISSING_ONE_WRITE flow={} limit={limit} unexpanded_segment_lengths={:?} states_missing_one_write={} failed={}",
        flow.name,
        flow.segments
            .iter()
            .enumerate()
            .filter(|(_, segment)| segment.len() > limit)
            .map(|(segment_index, segment)| (segment_index, segment.len()))
            .collect::<Vec<_>>(),
        variants.len(),
        failures.len()
    );
    assert!(
        failures.is_empty(),
        "流 {}：{} 个只差一个写的状态 GPU 与 CPU 不一致：\n{}",
        flow.name,
        failures.len(),
        failures.join("\n\n")
    );
}

/// 队列 {1} → {1, 7}：7 号没有记录（I-9.5 正方向红），队列里两个不同的号（I-9.24 红，读队列在配对之前判）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_deletion_queue_names_an_inode_without_a_record() {
    let flow = record_deletion_with_a_resealed_field(
        "crash-judge-gpu-i95",
        "deletion-queue-names-an-inode-without-a-record",
        &is_a_deleted_inodes_root,
        &put_an_inode_number_without_a_record_into_the_deletion_queue_root,
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_the_deletion_queue_names_an_inode_without_a_record",
        &flow,
        &["I-9.5", "I-9.24"],
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_not_zero() {
    let flow = record_deletion_with_a_resealed_field(
        "crash-judge-gpu-i34",
        "deletion-pending-occupancy-row-one",
        &is_an_accounting_leaf,
        &set_the_pending_deletion_occupancy_row_to_one,
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_not_zero",
        &flow,
        "I-3.4",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_missing() {
    let flow = record_deletion_with_a_resealed_field(
        "crash-judge-gpu-i34-missing-row",
        "deletion-pending-occupancy-row-missing",
        &is_an_accounting_leaf,
        &drop_the_pending_deletion_occupancy_row,
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_missing",
        &flow,
        "I-3.4",
    );
}

/// 这条流上没有 nlink = 0 的记录（第一个文件的记录整条拿掉了），恢复两边都答「没有文件」，所以连恢复一起逐状态比。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_without_a_record_keeps_its_data_unit_outside_the_queue(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_without_a_record_keeps_its_data_unit_outside_the_queue",
        &record_new_pool_with_a_new_inode_and_its_inode_leaf_resealed(
            "crash-judge-gpu-i47",
            "new-pool-new-inode-first-file-record-dropped",
            &drop_the_record_of_the_first_file,
        ),
        "I-4.7",
    );
}

/// 这条流上 nlink = 0 的是空 inode 2，第一个文件照常可读，恢复两边都读出文件，所以连恢复一起逐状态比。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_record_of_nlink_zero_is_not_in_the_deletion_queue() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_record_of_nlink_zero_is_not_in_the_deletion_queue",
        &record_new_pool_with_a_new_inode_and_its_inode_leaf_resealed(
            "crash-judge-gpu-i95-zombie",
            "new-pool-new-inode-nlink-zero-outside-the-queue",
            &set_the_nlink_of_the_new_inode_to_zero,
        ),
        "I-9.5",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_size_of_an_inode_being_deleted_asks_for_more_data_units(
) {
    let flow = record_deletion_with_a_resealed_field(
        "crash-judge-gpu-i46",
        "deletion-size-past-the-data-units",
        &is_an_inode_leaf_container,
        &raise_the_size_of_the_first_file_by_one_data_unit_payload,
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_size_of_an_inode_being_deleted_asks_for_more_data_units",
        &flow,
        "I-4.6",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_published_tombstone_reaches_the_inode_number_watermark()
{
    let flow = record_deletion_with_a_resealed_field(
        "crash-judge-gpu-i96-tombstone",
        "deletion-tombstone-past-the-watermark",
        &is_a_tombstone_container,
        &raise_the_first_tombstone_object_identifier_past_the_watermark,
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_published_tombstone_reaches_the_inode_number_watermark",
        &flow,
        "I-9.6",
    );
}

// ───────────────────────── 里程碑三·3 墓碑与谱系那九条（第 23 批）与第 21 批留下的两处坏盘分叉 ─────────────────────────

/// 码 2 节点条目里身份引用的出生树（分隔 key 8 之后）与容器号（类型段 2 之后）。
const CONTAINER_TREE_ENTRY_BIRTH_TREE_OFFSET: usize = 8;
const CONTAINER_TREE_ENTRY_CONTAINER_NUMBER_OFFSET: usize = 18;
const CONTAINER_TREE_ENTRY_CONTAINER_BIRTH_OFFSET: usize = 26;
/// 墓碑树、谱系树条目类型段条款之外的一个值（墓碑 {0, 1}、谱系 {0, 5} 之外）。
const UNREGISTERED_CONTAINER_TREE_ENTRY_TYPE: u16 = 7;
/// 墓碑记录 40 字节：对象 ID 0、对象出生代 8、死亡 txg 32（D18（块里携带什么信息） 已定项 24 第 2 条）。
const TOMBSTONE_RECORD_BYTES: usize = 40;
const TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET: usize = 8;
const TOMBSTONE_RECORD_DEATH_TXG_OFFSET: usize = 32;
/// 改坏墓碑的对象 ID 时写的号：inode 树里没有这个号的记录，也不越过 inode 号水位（I-9.6 墓碑那一半照旧成立）。
const OBJECT_IDENTIFIER_WITHOUT_A_RECORD: u64 = 0;
/// I-9.21 那条流里墓碑容器旋钮压到的每片记录数：两次 unlink 各占一片，最左那片不是最右的、只装 1 条。
const TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB: u64 = 1;
/// N4-f、N4-j 那两条流里容器树旋钮压到的每个码 2 节点条目数：三片容器装不进一个根，根长到层级 2。
const CONTAINER_TREE_ENTRIES_PER_NODE_UNDER_THE_KNOB: usize = 2;
/// 码 2 节点头里层级那一字节的偏移（D18（块里携带什么信息） 已定项 18：42 树 ID / 50 层级 / 51 key 宽）。
const INDEX_NODE_LEVEL_OFFSET: usize = 50;

/// 墓碑树、谱系树、inode 树的码 2 节点里第一条条目的起点（key 是 8 字节的容器号或 inode 号）。
fn first_entry_of_a_container_number_node() -> usize {
    index_node_entries_start(CONTAINER_NUMBER_KEY_BYTES)
}

fn is_the_tombstone_tree_root(bytes: &[u8]) -> bool {
    is_a_container_tree_root_over_containers_of_record_type(bytes, PACKED_RECORD_TYPE_TOMBSTONE)
}

fn is_the_lineage_tree_root(bytes: &[u8]) -> bool {
    is_a_container_tree_root_over_containers_of_record_type(bytes, PACKED_RECORD_TYPE_LINEAGE)
}

/// 码 2 节点头里的条目数。
fn index_node_entry_count(bytes: &[u8]) -> u16 {
    read_u16_at(
        bytes,
        index_node_entry_count_offset(usize::from(index_node_key_width(bytes))),
    )
}

/// 码 2 节点头里的条目宽改成 `entry_width`（条目数 0：声明长度 0 = 0 × 条目宽照旧成立）。
fn set_the_entry_width_of_an_empty_index_node(bytes: &mut [u8], entry_width: u16) {
    assert_eq!(index_node_entry_count(bytes), 0, "零条目的码 2 节点");
    let entry_width_offset =
        index_node_entry_count_offset(usize::from(index_node_key_width(bytes))) + 2;
    bytes[entry_width_offset..entry_width_offset + 2].copy_from_slice(&entry_width.to_le_bytes());
}

/// 容器树根第一条条目里身份引用的容器号加 1（分隔 key 与子指针不动）。
fn add_one_to_the_container_number_of_the_first_entry(bytes: &mut [u8]) {
    let field =
        first_entry_of_a_container_number_node() + CONTAINER_TREE_ENTRY_CONTAINER_NUMBER_OFFSET;
    let container = read_u64_at(bytes, field);
    bytes[field..field + 8].copy_from_slice(&(container + 1).to_le_bytes());
}

/// 容器树根第一条条目的类型段写成 `entry_type`。
fn set_the_type_of_the_first_entry(bytes: &mut [u8], entry_type: u16) {
    let field = first_entry_of_a_container_number_node() + CONTAINER_TREE_ENTRY_RECORD_TYPE_OFFSET;
    bytes[field..field + 2].copy_from_slice(&entry_type.to_le_bytes());
}

/// 墓碑容器第一条墓碑偏移 `field` 起的 8 字节改成 `change` 给的值。
fn change_the_first_tombstone_field(bytes: &mut [u8], field: usize, change: impl Fn(u64) -> u64) {
    assert!(
        read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET) >= 1,
        "墓碑容器里至少一条墓碑"
    );
    let offset = PACKED_UNIT_RECORDS_START + field;
    let value = change(read_u64_at(bytes, offset));
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

/// 同 [`assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening`]，CPU 池级 checker 要红的是 `invariants` 里的每一条。
fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
    test_name: &str,
    flow: &RecordedFlow,
    invariants: &[&'static str],
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, flow, expand_up_to()) else {
        return;
    };
    for invariant in invariants {
        assert!(
            comparison.cpu_checker_red_invariants.contains(invariant),
            "流 {} 上 CPU 池级 checker 要红 {invariant}（红过的：{:?}）",
            flow.name,
            comparison.cpu_checker_red_invariants
        );
    }
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// I-9.17 容器那一支：固定脚本到回退 D 的流上谱系树根第一条条目（类型段 5、指谱系容器）身份引用里的容器号加 1、重封到根。
/// 容器头没动，条目身份与子头不符：CPU 池级 checker 判 I-9.17 红、不记走读失败（第 22 批），GPU 走读内核同判（在查这片容器走没走过之前判）。
/// 恢复读谱系树根时层级 1 的条目只数不读，两边恢复照旧。
fn record_rollback_with_the_lineage_root_entry_naming_another_container(tag: &str) -> RecordedFlow {
    record_rollback_with_a_resealed_field(
        tag,
        "rollback-lineage-root-entry-names-another-container",
        &is_the_lineage_tree_root,
        &add_one_to_the_container_number_of_the_first_entry,
    )
}

/// I-9.23 类型段不认得那一支：删除中间态的流上墓碑树根第一条条目的类型段 1 → 7、重封到根。CPU 先登记这条子指针的引用（层级 1 按容器两槽）
/// 再判 I-9.23 红、不往下走、不记走读失败；GPU 走读内核同样先登记引用再置位。删除态扫描读这棵墓碑树判不了，I-9.18 起那几条不判。
fn record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type(
    tag: &str,
) -> RecordedFlow {
    record_deletion_with_a_resealed_field(
        tag,
        "deletion-tombstone-root-entry-type-seven",
        &is_the_tombstone_tree_root,
        &|bytes| set_the_type_of_the_first_entry(bytes, UNREGISTERED_CONTAINER_TREE_ENTRY_TYPE),
    )
}

/// I-9.23 类型段 0 那一支（攻方 N4-a 那一形，`research/prompts/m3-batch22-code-r1-opus-output.md` 第一节）：删除中间态的流上墓碑树根（层级 1）
/// 第一条条目的类型段 1 → 0、重封到根。身份引用的出生树不是 0、孩子是码 3：CPU 只读孩子的单元头得码 3、与类型段 0 不符，判 I-9.23 红、
/// 按两槽登记引用、不往下走、不记走读失败（第 24 批；原来按节点长度读不出孩子，再因为层级 1 的节点里出现类型段 0 记走读失败，I-4.8、I-7.2、I-7.4 连带红）；
/// GPU 走读内核同判同登记。
fn record_deletion_with_the_tombstone_root_entry_of_type_zero(tag: &str) -> RecordedFlow {
    record_deletion_with_a_resealed_field(
        tag,
        "deletion-tombstone-root-entry-type-zero",
        &is_the_tombstone_tree_root,
        &|bytes| {
            assert_ne!(
                read_u64_at(
                    bytes,
                    first_entry_of_a_container_number_node()
                        + CONTAINER_TREE_ENTRY_BIRTH_TREE_OFFSET
                ),
                0,
                "指墓碑容器的条目身份引用里出生树不是 0"
            );
            set_the_type_of_the_first_entry(bytes, 0);
        },
    )
}

/// 容器树节点第一条条目的身份引用里出生树、容器号、容器出生代三段写 0（类型段那两字节不动）。
fn clear_the_identity_of_the_first_entry_but_its_type(bytes: &mut [u8]) {
    let first_entry = first_entry_of_a_container_number_node();
    for field in [
        CONTAINER_TREE_ENTRY_BIRTH_TREE_OFFSET,
        CONTAINER_TREE_ENTRY_CONTAINER_NUMBER_OFFSET,
        CONTAINER_TREE_ENTRY_CONTAINER_BIRTH_OFFSET,
    ] {
        bytes[first_entry + field..first_entry + field + 8].fill(0);
    }
}

/// I-9.23 类型段 0 那一支、身份三段也清零（攻方 N4-i 那一形）：删除中间态的流上墓碑树根（层级 1）第一条条目的类型段 1 → 0、
/// 出生树、容器号、容器出生代三段写 0、重封到根。身份引用那一半成立，只剩「子单元是码 2」那一小句不成立：孩子是码 3 容器。
/// CPU 只读孩子的单元头得码 3，判 I-9.23 红、不往下走、不记走读失败（第 24 批；原来按节点长度读不出孩子、这一小句不判，I-9.23 报成立，
/// 层级 1 的节点里出现类型段 0 记走读失败）；GPU 走读内核同判。
fn record_deletion_with_the_tombstone_root_entry_of_type_zero_and_no_identity(
    tag: &str,
) -> RecordedFlow {
    record_deletion_with_a_resealed_field(
        tag,
        "deletion-tombstone-root-entry-type-zero-no-identity",
        &is_the_tombstone_tree_root,
        &|bytes| {
            set_the_type_of_the_first_entry(bytes, 0);
            clear_the_identity_of_the_first_entry_but_its_type(bytes);
        },
    )
}

/// I-9.2 同形（判决 `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 N4 那一行「inode 树 I-9.2 同形一并查」）：
/// 新池新建文件的流上 inode 树根（层级 1）第一条条目的类型段 2 → 0、重封到根。条目说它指下一层码 2 节点，孩子是码 3 叶容器：
/// CPU 只读孩子的单元头得码 3、按 32 KiB 自证得过，判 I-9.2 红、按两槽登记引用、不往下走、不记走读失败（第 24 批；原来按节点长度读、
/// 校验和对不上，I-2.1 红、记走读失败）；GPU 走读内核同判同登记（原来类型段 0 的孩子整个状态判不了）。
fn record_new_pool_with_the_inode_root_entry_of_type_zero_naming_its_leaf_container(
    tag: &str,
) -> RecordedFlow {
    let inode_tree_root_over_leaf_containers = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && usize::from(index_node_key_width(bytes)) == INODE_TREE_KEY_BYTES
            && index_node_entry_width(bytes) == INODE_INTERNAL_ENTRY_BYTES
            && read_u16_at(
                bytes,
                index_node_entries_start(INODE_TREE_KEY_BYTES)
                    + CONTAINER_TREE_ENTRY_RECORD_TYPE_OFFSET,
            ) == PACKED_RECORD_TYPE_INODE
    };
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-inode-root-entry-type-zero-names-its-leaf-container",
        &inode_tree_root_over_leaf_containers,
        false,
        &|bytes| set_the_type_of_the_first_entry(bytes, 0),
    )
}

/// 节点指针里第一条位置条目的槽号（6 字节）的偏移：两条位置条目在 50 与 64（盘 4、槽 6、整单元校验和 4）。
const POINTER_FIRST_LOCATION_SLOT_OFFSET: usize = 54;

/// 攻方 H1（判决 `research/prompts/m3-batch24-code-r1-main-verification.md` 第二节 K2 N1 两故障 H1 那一行，第 26 批改）的 inode 那一形：
/// 新池新建文件的流上 inode 树根（层级 1）第一条条目的类型段 2 → 0、身份引用三段写 0、重封到根（除了孩子的类，这是一条合法的类型段 0 条目），
/// 再把它指着的叶容器盘序小的那一份写的前 16 KiB 换成改之前那个 inode 树根的原样字节（陈旧副本：头里类标签 2，恰是类型段 0 说的那一类；
/// 不补校验和，位置条目还是叶容器原来那一份的），盘序大的那一份不动。CPU 认孩子的类时头里是类型段那一类的副本也要按那一类的长度自证，
/// 第一份自证不过看第二份、认出码 3，判 I-9.2 红、不往下走、不记走读失败（改之前只认第一份的头、不验，I-9.2 报成立，另连带 I-7.2 等）；
/// GPU 走读内核同改（`w_child_unit_class_other_than_the_entry_type_names`）。
fn record_new_pool_with_the_inode_root_entry_of_type_zero_and_no_identity_over_a_leaf_container_whose_first_copy_is_stale(
    tag: &str,
) -> RecordedFlow {
    let name = "new-pool-inode-root-entry-type-zero-no-identity-first-leaf-copy-stale";
    let mut flow = record_new_pool_file_creation(tag, name);
    let inode_tree_root_over_leaf_containers = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && usize::from(index_node_key_width(bytes)) == INODE_TREE_KEY_BYTES
            && index_node_entry_width(bytes) == INODE_INTERNAL_ENTRY_BYTES
            && read_u16_at(
                bytes,
                index_node_entries_start(INODE_TREE_KEY_BYTES)
                    + CONTAINER_TREE_ENTRY_RECORD_TYPE_OFFSET,
            ) == PACKED_RECORD_TYPE_INODE
    };
    let inode_leaf_container = |bytes: &[u8]| {
        unit_class(bytes) == 3 && packed_record_type(bytes) == PACKED_RECORD_TYPE_INODE
    };
    let root_copies = last_unit_copies(&flow, name, &inode_tree_root_over_leaf_containers, false);
    let stale_bytes = flow.writes[root_copies[0]]
        .bytes()
        .expect("单元写带字节")
        .to_vec();
    let first_leaf_copy = last_unit_copies(&flow, name, &inode_leaf_container, true)[0];
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
        set_the_type_of_the_first_entry(bytes, 0);
        clear_the_identity_of_the_first_entry_but_its_type(bytes);
    });
    let root_after_the_change = flow.writes[root_copies[0]].bytes().expect("单元写带字节");
    let first_location_slot_offset = first_entry_of_a_container_number_node()
        + INODE_INTERNAL_ENTRY_CHILD_POINTER_OFFSET
        + POINTER_FIRST_LOCATION_SLOT_OFFSET;
    let mut slot_named_by_the_first_entry = [0u8; 8];
    slot_named_by_the_first_entry[..6].copy_from_slice(
        &root_after_the_change[first_location_slot_offset..first_location_slot_offset + 6],
    );
    assert_eq!(
        u64::from_le_bytes(slot_named_by_the_first_entry),
        flow.writes[first_leaf_copy].offset.0 / 16384,
        "inode 树根第一条条目指着的就是流里最后写的那片叶容器"
    );
    assert_eq!(unit_class(&stale_bytes), 2, "陈旧副本头里的类是码 2");
    let WrittenContents::Bytes(leaf) = &mut flow.writes[first_leaf_copy].contents else {
        panic!("单元写是普通写");
    };
    leaf[..stale_bytes.len()].copy_from_slice(&stale_bytes);
    flow
}

/// I-9.18：删除中间态的流上墓碑容器里那一条墓碑的对象 ID 1 → 0、重封到根。队列 {1} 里的号 1 在墓碑树里没有墓碑：CPU 判 I-9.18 红；
/// 号 0 没有记录，I-9.10 墓碑那一个等号与 I-9.19 没有对象，号 0 不越过水位，I-9.6 墓碑那一半照旧成立。
fn record_deletion_with_the_tombstone_naming_an_object_without_a_record(tag: &str) -> RecordedFlow {
    record_deletion_with_a_resealed_field(
        tag,
        "deletion-tombstone-names-an-object-without-a-record",
        &is_a_tombstone_container,
        &|bytes| {
            change_the_first_tombstone_field(bytes, 0, |_object| OBJECT_IDENTIFIER_WITHOUT_A_RECORD)
        },
    )
}

/// I-9.19：删除中间态的流上第一批那一版 inode 叶容器里第一个文件的记录 nlink 0 → 1、重封到根。意图那一版写的墓碑在第一批那一版的谱系上、
/// 指着号与对象出生代都相同而 nlink 1 的记录：CPU 判 I-9.19 红（队列里的号 nlink ≠ 0，I-9.5 同红）。
fn record_deletion_with_the_queued_record_back_at_nlink_one(tag: &str) -> RecordedFlow {
    record_deletion_with_a_resealed_field(
        tag,
        "deletion-queued-record-back-at-nlink-one",
        &is_an_inode_leaf_container,
        &|bytes| {
            let record = offset_of_the_inode_record(bytes, FIRST_FILE.0);
            assert_eq!(
                read_u32_at(bytes, record + INODE_RECORD_NLINK_OFFSET),
                0,
                "删除中的第一个文件 nlink 0"
            );
            bytes[record + INODE_RECORD_NLINK_OFFSET..record + INODE_RECORD_NLINK_OFFSET + 4]
                .copy_from_slice(&1u32.to_le_bytes());
        },
    )
}

/// I-9.20 管全部墓碑那几句：删除中间态的流上墓碑的死亡 txg 改成 0、重封到根。对象出生代 > 死亡 txg：CPU 判 I-9.20 红；
/// 死亡 txg 0 仍在谱系上（没有回退的一版从 0 到自己的 txg 整段在链上），I-9.18 照旧成立。
fn record_deletion_with_the_tombstone_dying_at_txg_zero(tag: &str) -> RecordedFlow {
    record_deletion_with_a_resealed_field(
        tag,
        "deletion-tombstone-dies-at-txg-zero",
        &is_a_tombstone_container,
        &|bytes| {
            change_the_first_tombstone_field(
                bytes,
                TOMBSTONE_RECORD_DEATH_TXG_OFFSET,
                |_death_txg| 0,
            )
        },
    )
}

/// I-9.20 不重号那一句：删除中间态的流上墓碑容器里那一条墓碑原样再追加一条（记录数加一、声明长度加 40）、重封到根。
/// 两条都在谱系上、(号, 对象出生代) 相同：CPU 判 I-9.20 红。
fn record_deletion_with_the_tombstone_written_twice(tag: &str) -> RecordedFlow {
    record_deletion_with_a_resealed_field(
        tag,
        "deletion-tombstone-written-twice",
        &is_a_tombstone_container,
        &|bytes| {
            let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
            assert_eq!(record_count, 1, "意图那一次只写一条墓碑");
            let first = PACKED_UNIT_RECORDS_START;
            bytes.copy_within(
                first..first + TOMBSTONE_RECORD_BYTES,
                first + TOMBSTONE_RECORD_BYTES,
            );
            bytes[PACKED_UNIT_RECORD_COUNT_OFFSET..PACKED_UNIT_RECORD_COUNT_OFFSET + 2]
                .copy_from_slice(&2u16.to_le_bytes());
            let declared_length = read_u16_at(bytes, UNIT_DECLARED_LENGTH_OFFSET);
            bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
                &(declared_length + u16::try_from(TOMBSTONE_RECORD_BYTES).expect("40"))
                    .to_le_bytes(),
            );
        },
    )
}

/// I-9.29 与 I-9.10 墓碑那一个等号：删除中间态的流上墓碑的对象出生代加 1、重封到根。号 1 的记录出生代与谱系上墓碑的不同：
/// CPU 判 I-9.10（按号认）、I-9.29（同号两个出生代）红，I-9.18 跟着红（没有出生代对得上的墓碑）。
fn record_deletion_with_the_tombstone_object_birth_off_its_record(tag: &str) -> RecordedFlow {
    record_deletion_with_a_resealed_field(
        tag,
        "deletion-tombstone-object-birth-off-its-record",
        &is_a_tombstone_container,
        &|bytes| {
            change_the_first_tombstone_field(
                bytes,
                TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET,
                |object_birth| object_birth + 1,
            )
        },
    )
}

/// I-9.4 第四句：新池新建文件、再建一个空 inode（2 号，与第一个文件同在最左那片叶里）的流上那片叶的两条记录对调、重封到根。
/// 记录序 [2, 1] 不严格递增：CPU 判 I-9.4 红（号都 ≥ 容器号、叶序上的第二、三句照旧成立）；恢复按号找第一个文件，不看次序。
fn record_new_pool_with_the_two_records_of_the_inode_leaf_swapped(tag: &str) -> RecordedFlow {
    record_new_pool_with_a_new_inode_and_its_inode_leaf_resealed(
        tag,
        "new-pool-new-inode-leaf-records-swapped",
        &|bytes| {
            let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
            assert_eq!(record_count, 2, "第一个文件与空 inode 两条");
            let first = offset_of_the_inode_record(bytes, FIRST_FILE.0);
            let second = offset_of_the_inode_record(bytes, FIRST_FILE.0 + 1);
            assert!(first < second, "写者按号升序写记录");
            let first_record = bytes[first..first + INODE_RECORD_BYTES].to_vec();
            bytes.copy_within(second..second + INODE_RECORD_BYTES, first);
            bytes[second..second + INODE_RECORD_BYTES].copy_from_slice(&first_record);
        },
    )
}

/// 删到最后一批的流上最后一个零条目的码 2 节点（key 宽 `key_width`）条目宽改成 `entry_width`、重封到根：第 21 批 G 留下的坏盘分叉。
fn record_deletion_through_the_last_batch_with_an_empty_root_of_another_entry_width(
    tag: &str,
    name: &'static str,
    key_width: u8,
    entry_width: u16,
) -> RecordedFlow {
    let mut flow = record_new_pool_through_the_last_deletion_batch(tag, name);
    let empty_root = move |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == key_width
            && index_node_entry_count(bytes) == 0
    };
    let targets = last_unit_copies(&flow, name, &empty_root, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
        set_the_entry_width_of_an_empty_index_node(bytes, entry_width);
    });
    flow
}

/// 零条目、层级 0 的 extent 上段根，条目宽 113 → 24（= key 宽）。core 冷走读逐条解条目时才核宽（`extent_tree.rs` 的 `upper_leaf_entries`），
/// 零条目时答没有文件；GPU 恢复内核同判（`read_upper_segment` 只在有条目时核宽）。池级 checker 两边都按字段表判 I-1.10 红。
fn record_deletion_through_the_last_batch_with_a_narrow_empty_extent_root(
    tag: &str,
) -> RecordedFlow {
    const EXTENT_KEY_WIDTH: u8 = 24;
    record_deletion_through_the_last_batch_with_an_empty_root_of_another_entry_width(
        tag,
        "deletion-through-the-last-batch-empty-extent-root-of-entry-width-twenty-four",
        EXTENT_KEY_WIDTH,
        u16::from(EXTENT_KEY_WIDTH),
    )
}

/// 零条目的 inode 树根（层级 1），条目宽 120 → 8（= key 宽）。core 冷走读逐条 `inode_internal_entry_parsed_and_judged` 时才核宽，
/// 零条目时答没有文件；GPU 恢复内核同判（`walk_to_file` 只在有条目时核宽）。池级 checker 两边都按字段表判 I-1.10 红。
fn record_deletion_through_the_last_batch_with_a_narrow_empty_inode_root(
    tag: &str,
) -> RecordedFlow {
    let inode_key_width = u8::try_from(INODE_TREE_KEY_BYTES).expect("8");
    record_deletion_through_the_last_batch_with_an_empty_root_of_another_entry_width(
        tag,
        "deletion-through-the-last-batch-empty-inode-root-of-entry-width-eight",
        inode_key_width,
        u16::from(inode_key_width),
    )
}

/// 从新池新建文件（txg 3）到 `pool` 现行那一版，每一版都读得出第一个文件（这几条流不碰它）。
fn versions_with_the_first_file_through_the_last_publish(
    pool: &BuiltPool,
) -> Vec<PublishedVersion> {
    (3..=pool.output.root.checkpoint_txg.0)
        .map(|txg| PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(txg),
            content: file_content(),
        })
        .collect()
}

/// I-9.21：新池新建文件、建两个空 inode（2、3 号），写入口装上只供测试的墓碑容器旋钮（一片 1 条），unlink 2、3 号各到底：
/// 墓碑树根两条条目各指一片容器、各装 1 条。这是旋钮下的合法镜像；CPU 判器走池级 checker 的旧入口（当没装旋钮），
/// 不是最右的那片只装 1 条、不是 815，判 I-9.21 红；GPU 扫描内核只有没装旋钮那一支，同判。
fn record_two_empty_inodes_unlinked_with_one_tombstone_per_container(tag: &str) -> RecordedFlow {
    let mut pool = pool_with_new_inodes(tag, 2);
    let capacities = ContainerTreeCapacities {
        tombstone_container: TombstoneContainerCapacity::CappedForTests(
            TombstoneRecordsPerContainerCappedForTests::new(
                TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
            )
            .expect("1..=815"),
        ),
        lineage_container: LineageContainerCapacity::FromTheContainerFormat,
        retirement_container:
            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
        node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    };
    for inode in [FIRST_FILE.0 + 1, FIRST_FILE.0 + 2] {
        unlink_an_empty_inode_to_the_end_under_the_capacities(&mut pool, inode, capacities);
    }
    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
    recorded_flow_of_the_pool(
        &pool,
        "two-empty-inodes-unlinked-one-tombstone-per-container",
        versions,
    )
}

/// 写入口装上只供测试的容器树旋钮 `capacities`，把空 inode `inode` unlink 到底：意图那一次，再走空（空 inode 没有数据单元，直接是最后一批）。
fn unlink_an_empty_inode_to_the_end_under_the_capacities(
    pool: &mut BuiltPool,
    inode: u64,
    capacities: ContainerTreeCapacities,
) {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    writer.set_container_tree_capacities(capacities);
    let mut current = publish_unlink_intent(
        &mut writer,
        &mut pool.allocator,
        &pool.output,
        InodeNumber(inode),
        WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
        INSTANCE_OF_THE_PUBLISHES,
    )
    .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的意图：{refusal:?}"));
    publish_unlink_steps_until_the_queue_is_empty(
        &mut writer,
        &mut pool.allocator,
        &mut current,
        UnlinkBatchSize::OneExtentLowerLeaf,
        INSTANCE_OF_THE_PUBLISHES,
    )
    .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的各批：{refusal:?}"));
    pool.output = current;
}

/// 墓碑容器一片 1 条、容器树一个节点 2 条（`crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs` 的
/// `capacities_of_one_record_and_two_entries`）：unlink 第三个 inode 时墓碑树根满、长到层级 2。
fn capacities_of_one_tombstone_per_container_and_two_entries_per_node() -> ContainerTreeCapacities {
    ContainerTreeCapacities {
        tombstone_container: TombstoneContainerCapacity::CappedForTests(
            TombstoneRecordsPerContainerCappedForTests::new(
                TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
            )
            .expect("1..=815"),
        ),
        lineage_container: LineageContainerCapacity::FromTheContainerFormat,
        retirement_container:
            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
        node_entries: ContainerTreeNodeEntries::CappedForTests(
            ContainerTreeNodeEntriesCappedForTests::new(
                CONTAINER_TREE_ENTRIES_PER_NODE_UNDER_THE_KNOB,
            )
            .expect("2..=135"),
        ),
    }
}

/// 层级 2 的墓碑树根：码 2、key 宽 8、条目宽 120、层级 2、第一条条目类型段 0（指层级 1 的码 2 节点）。
/// 这几条流里谱系树是零根、inode 树只有一片叶容器（根层级 1），层级 2 的这种节点只有墓碑树根。
fn is_a_tombstone_tree_root_of_level_two(bytes: &[u8]) -> bool {
    unit_class(bytes) == 2
        && usize::from(index_node_key_width(bytes)) == CONTAINER_NUMBER_KEY_BYTES
        && index_node_entry_width(bytes) == CONTAINER_TREE_ENTRY_BYTES
        && bytes[INDEX_NODE_LEVEL_OFFSET] == 2
        && read_u16_at(
            bytes,
            first_entry_of_a_container_number_node() + CONTAINER_TREE_ENTRY_RECORD_TYPE_OFFSET,
        ) == 0
}

/// 新池新建文件、建五个空 inode（2–6 号），写入口装 [`capacities_of_one_tombstone_per_container_and_two_entries_per_node`]，
/// unlink 2、3、4 号各到底：三片墓碑容器各装 1 条，第三次 unlink 的意图那次墓碑树根长到层级 2（根两条类型段 0 的条目，
/// 各指一个层级 1 的码 2 节点）。旋钮下的合法镜像；CPU 判器走旧入口当没装旋钮，I-9.21 红（同 I-9.21 那条流）。
/// `turn_the_root_ring_over` 时再建 `PUBLISHES_THAT_TURN_THE_ROOT_RING_OVER`（`common_deletion_in_progress`）次空 inode：
/// 墓碑树长高之前那几版都转出回退候选集，层级 1 那两个节点只剩现行这棵墓碑树（与之后照抄它的各版）引用。
fn record_a_tombstone_tree_grown_to_level_two(
    tag: &str,
    name: &'static str,
    turn_the_root_ring_over: bool,
) -> RecordedFlow {
    let mut pool = pool_with_new_inodes(tag, 5);
    let capacities = capacities_of_one_tombstone_per_container_and_two_entries_per_node();
    for inode in [FIRST_FILE.0 + 1, FIRST_FILE.0 + 2, FIRST_FILE.0 + 3] {
        unlink_an_empty_inode_to_the_end_under_the_capacities(&mut pool, inode, capacities);
    }
    assert_eq!(
        pool.output.tombstone_tree_index_nodes.shape.root_level(),
        2,
        "第三片墓碑容器让根长到层级 2"
    );
    if turn_the_root_ring_over {
        turn_the_root_ring_over_by_creating_inodes(&mut pool);
    }
    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
    recorded_flow_of_the_pool(&pool, name, versions)
}

/// I-9.23 类型段 = 记录类型那一支、孩子其实是码 2 节点（攻方 N4-f 那一形）：[`record_a_tombstone_tree_grown_to_level_two`] 的流上
/// 层级 2 的墓碑树根第一条条目类型段 0 → 1、重封到根。条目说它指一片墓碑容器，孩子是层级 1 的码 2 节点：
/// CPU 只读孩子的单元头得码 2、与类型段 1 不符，判 I-9.23 红、按孩子自述的类登记一槽引用、不往下走、不按数据单元长度去读、不记走读失败
/// （第 24 批；原来按 32 KiB 读那个 16 KiB 的节点，I-2.1 红、记走读失败，I-4.8、I-7.2、I-7.4 连带红）；GPU 走读内核同判同登记。
/// `turn_the_root_ring_over` 是攻方 N4-j 那一形：同一刀落在根环转过一圈之后，那个节点下面的容器没有别的版本再引用。
fn record_a_tombstone_tree_of_level_two_whose_first_root_entry_names_its_node_as_a_container(
    tag: &str,
    name: &'static str,
    turn_the_root_ring_over: bool,
) -> RecordedFlow {
    let mut flow = record_a_tombstone_tree_grown_to_level_two(tag, name, turn_the_root_ring_over);
    let targets = last_unit_copies(&flow, name, &is_a_tombstone_tree_root_of_level_two, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
        set_the_type_of_the_first_entry(bytes, PACKED_RECORD_TYPE_TOMBSTONE);
    });
    flow
}

/// inode 记录里对象出生代那 8 字节的偏移（D18（块里携带什么信息） 已定项 16 的 inode 记录字段表：inode 号 0、对象出生代 8）。
const INODE_RECORD_OBJECT_BIRTH_OFFSET: usize = 8;
/// 实例表单元的打包记录类型（D18（块里携带什么信息） 已定项 11）。
const PACKED_RECORD_TYPE_INSTANCE_TABLE: u16 = 4;
/// 翻坏一个单元的载荷时翻的那一字节离单元末尾几字节（`checker_known_bad_images.rs` 的 `break_both_copies_of_the_unit` 同一处）：
/// 不重封，位置条目里的整单元校验和对不上，两份都读不出。
const BYTES_FROM_THE_END_OF_THE_FLIPPED_PAYLOAD_BYTE: usize = 100;

/// 被崩溃恢复抛弃的 C 那一段历史录成的流，与录制时 C 那次发布在录制流里占的那一截（[`record_a_root_abandoned_by_a_crash_recovery`] 交回）。
struct FlowWithARootAbandonedByACrashRecovery {
    flow: RecordedFlow,
    /// `flow.writes` 每次写在录制流（扣掉 mkfs 那几步）里的下标。
    stream_indexes: Vec<usize>,
    /// 录制流（扣掉 mkfs 那几步）里 C 那次发布之前的步数与到它为止的步数：C 那次发布的写，录制流下标落在这两个数之间。
    operations_before_the_abandoned_root: usize,
    operations_through_the_abandoned_root: usize,
    abandoned_root_checkpoint_txg: u64,
}

/// A (1, 3)、B (1, 4) → 重开取号 2（写行、暖机）→ C (2, 8)；崩溃恢复抛弃 C
/// （`common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`：C 的根槽与数据单元暂时读不出、见证 C 的系统配置槽写回轮换之前那一代，
/// 择根落到 C 之前那条根，实例 3 写行、暖机，再把暂存的字节原样写回）。录成名叫 `name` 的流（每一版的内容都登记进 `versions`），
/// 判的根是录制流里最后一次根槽写。
fn record_a_root_abandoned_by_a_crash_recovery(
    tag: &str,
    name: &'static str,
) -> FlowWithARootAbandonedByACrashRecovery {
    let mut pool = build_pool(tag);
    let mut recorder = CrashPointRecorder::new(pool.stream.clone(), pool.mkfs_operation_count);
    record_pool_build(&mut recorder, &pool);
    let mut versions = vec![PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(3),
        content: file_content(),
    }];
    let second = overwrite_step(
        &mut recorder,
        &mut pool,
        &second_content(),
        InstanceGeneration(1),
    );
    let second_txg = second.root.checkpoint_txg.0;
    versions.push(PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(second_txg),
        content: second_content(),
    });
    pool.output = second;
    let mounted = mount_writable_step(&mut recorder, &mut pool);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    for txg in second_txg + 1..=pool.output.root.checkpoint_txg.0 {
        versions.push(PublishedVersion {
            instance: InstanceGeneration(2),
            checkpoint_txg: CheckpointTxg(txg),
            content: second_content(),
        });
    }
    let operations_before_the_third = pool.stream.operation_count() - pool.mkfs_operation_count;
    let third = overwrite_step(
        &mut recorder,
        &mut pool,
        &third_content(),
        InstanceGeneration(2),
    );
    let operations_through_the_third = pool.stream.operation_count() - pool.mkfs_operation_count;
    versions.push(PublishedVersion {
        instance: InstanceGeneration(2),
        checkpoint_txg: third.root.checkpoint_txg,
        content: third_content(),
    });
    pool.output = third.clone();
    let abandoning =
        abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &third);
    assert_eq!(
        (
            abandoning.output.instance,
            abandoning.output.effective_root.checkpoint_txg.0 + 1,
            abandoning.output.row_publish.root().checkpoint_txg.0,
        ),
        (
            InstanceGeneration(3),
            third.root.checkpoint_txg.0,
            third.root.checkpoint_txg.0 + 1
        ),
        "恢复落到 C 之前那条根，实例 3 写行接在 C 的 txg 之后"
    );
    for txg in
        abandoning.output.row_publish.root().checkpoint_txg.0..=pool.output.root.checkpoint_txg.0
    {
        versions.push(PublishedVersion {
            instance: InstanceGeneration(3),
            checkpoint_txg: CheckpointTxg(txg),
            content: second_content(),
        });
    }
    let operations = pool.retained_operations();
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("根槽写");
    FlowWithARootAbandonedByACrashRecovery {
        flow: RecordedFlow {
            name,
            base: pool.memory_pool_after_mkfs(),
            writes,
            segments,
            judged_root_index,
            versions,
        },
        stream_indexes,
        operations_before_the_abandoned_root: operations_before_the_third,
        operations_through_the_abandoned_root: operations_through_the_third,
        abandoned_root_checkpoint_txg: third.root.checkpoint_txg.0,
    }
}

/// I-9.29（同号只许一个出生代） 判不了那一形（`crates/singlefs-checker/invariants/09-inode树结构.md` I-9.29 那一行「最新根的实例表读不出、被崩溃恢复抛弃的根滤不掉时记判不了」；
/// 第 23 批 X 的 harness 坏镜像 `an_abandoned_root_naming_another_object_birth_leaves_the_one_birth_invariant_partially_judged_once_the_newest_instance_table_does_not_read_back`
/// 同一段历史，录成流）：A (1, 3)、B (1, 4) → 重开取号 2（写行、暖机）→ C (2, 8)；崩溃恢复抛弃 C
/// （`common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`：C 的根槽与数据单元暂时读不出、见证 C 的系统配置槽写回轮换之前那一代，
/// 择根落到 C 之前那条根，实例 3 写行、暖机，再把暂存的字节原样写回）。录成流之后 C 那片 inode 叶容器两份里第一个文件那条记录的对象出生代加一、
/// 重封到 C 的根（C 那一版与现行线上各版同号不同出生代），再照 `change` 改最新根（实例 3）指着的实例表（[`ChangeOfTheNewestInstanceTable`]）。
/// 交回流，与改实例表之前这条流整条落盘之后的镜像（候选集滤得掉 C 的对照）。
fn record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed(
    tag: &str,
    change: ChangeOfTheNewestInstanceTable,
) -> (RecordedFlow, MemoryPool) {
    let FlowWithARootAbandonedByACrashRecovery {
        mut flow,
        stream_indexes,
        operations_before_the_abandoned_root: operations_before_the_third,
        operations_through_the_abandoned_root: operations_through_the_third,
        abandoned_root_checkpoint_txg: _,
    } = record_a_root_abandoned_by_a_crash_recovery(tag, change.flow_name());
    let inode_leaf_of_the_third: Vec<usize> = stream_indexes
        .iter()
        .enumerate()
        .filter(|(write_index, stream_index)| {
            (operations_before_the_third..operations_through_the_third).contains(*stream_index)
                && flow.writes[*write_index].kind == StepKind::UnitWrite
                && flow.writes[*write_index].bytes().is_some_and(|bytes| {
                    unit_class(bytes) == 3 && packed_record_type(bytes) == PACKED_RECORD_TYPE_INODE
                })
        })
        .map(|(write_index, _)| write_index)
        .collect();
    assert_eq!(
        inode_leaf_of_the_third.len(),
        2,
        "C 那次发布写一片 inode 叶容器、两份副本"
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &inode_leaf_of_the_third, &|bytes| {
        let object_birth =
            offset_of_the_inode_record(bytes, FIRST_FILE.0) + INODE_RECORD_OBJECT_BIRTH_OFFSET;
        let another_object_birth = read_u64_at(bytes, object_birth) + 1;
        bytes[object_birth..object_birth + 8].copy_from_slice(&another_object_birth.to_le_bytes());
    });
    let mut image_with_the_newest_instance_table_readable = flow.base.clone();
    image_with_the_newest_instance_table_readable.apply_writes(&flow.writes);
    let is_an_instance_table_unit = |bytes: &[u8]| {
        unit_class(bytes) == 3 && packed_record_type(bytes) == PACKED_RECORD_TYPE_INSTANCE_TABLE
    };
    let newest_instance_table = last_unit_copies(
        &flow,
        "实例 3 写行那次写的实例表",
        &is_an_instance_table_unit,
        false,
    );
    for write_index in &newest_instance_table {
        assert!(
            stream_indexes[*write_index] >= operations_through_the_third,
            "最后写的那张实例表是实例 3 写行那次写的"
        );
    }
    match change {
        ChangeOfTheNewestInstanceTable::BothCopiesFlippedWithoutResealing => {
            for write_index in newest_instance_table {
                let WrittenContents::Bytes(bytes) = &mut flow.writes[write_index].contents else {
                    panic!("单元写是普通写");
                };
                let flipped = bytes.len() - BYTES_FROM_THE_END_OF_THE_FLIPPED_PAYLOAD_BYTE;
                bytes[flipped] ^= 0xff;
            }
        }
        ChangeOfTheNewestInstanceTable::RecordOfAnUnregisteredKindInsertedBeforeTheChainRecord => {
            corrupt_units_and_reseal_up_to_the_root(
                &mut flow,
                &newest_instance_table,
                &insert_a_record_of_an_unregistered_kind_before_the_chain_record,
            );
        }
        ChangeOfTheNewestInstanceTable::SecondChainRecordInsertedInTheMiddleOfThePage => {
            corrupt_units_and_reseal_up_to_the_root(
                &mut flow,
                &newest_instance_table,
                &insert_a_copy_of_the_chain_record_before_the_chain_record,
            );
        }
        ChangeOfTheNewestInstanceTable::FirstRowFlagsSetToTheRetiredBit => {
            corrupt_units_and_reseal_up_to_the_root(
                &mut flow,
                &newest_instance_table,
                &set_the_flags_of_the_first_row_to_the_retired_bit,
            );
        }
        ChangeOfTheNewestInstanceTable::FirstRowFlagsSetToTheRetiredBitAndItsPublishedTxgRaisedToTheNewestRoot => {
            let newest_root_txg = flow
                .versions
                .iter()
                .map(|version| version.checkpoint_txg.0)
                .max()
                .expect("流里至少一版");
            corrupt_units_and_reseal_up_to_the_root(&mut flow, &newest_instance_table, &|bytes| {
                set_the_flags_of_the_first_row_to_the_retired_bit(bytes);
                let first_row_published_txg =
                    PACKED_UNIT_RECORDS_START + INSTANCE_TABLE_ROW_PUBLISHED_TXG_OFFSET;
                bytes[first_row_published_txg..first_row_published_txg + 8]
                    .copy_from_slice(&newest_root_txg.to_le_bytes());
            });
        }
        ChangeOfTheNewestInstanceTable::SecondPageWhosePointerOnDeviceZeroNamesTheFirstPage => {
            let (_, other_rows) = rows_of_the_newest_instance_table_split_off_the_abandoned_instance(
                &flow,
                &newest_instance_table,
            );
            rebuild_the_newest_instance_table_as_a_chain(
                &mut flow,
                &newest_instance_table,
                &[other_rows, Vec::new()],
                PagesAfterTheFirstInTheFlow::Intact,
            );
            let first_page_copies = last_unit_copies(
                &flow,
                "改造成两片之后的第 0 片",
                &is_an_instance_table_unit,
                false,
            );
            let first_page_slot = flow.writes[first_page_copies[0]].offset.0 / SLOT_BYTES;
            corrupt_units_and_reseal_up_to_the_root(&mut flow, &first_page_copies, &|bytes| {
                point_the_first_location_of_the_chain_record_at_slot(bytes, first_page_slot);
            });
        }
        ChangeOfTheNewestInstanceTable::SecondPageHoldingTheRowOfTheAbandonedInstanceNamedThroughAPointerWhoseMacIsNotZero => {
            let (rows_of_the_abandoned_instance, other_rows) =
                rows_of_the_newest_instance_table_split_off_the_abandoned_instance(
                    &flow,
                    &newest_instance_table,
                );
            rebuild_the_newest_instance_table_as_a_chain(
                &mut flow,
                &newest_instance_table,
                &[other_rows, rows_of_the_abandoned_instance],
                PagesAfterTheFirstInTheFlow::Intact,
            );
            let first_page_copies = last_unit_copies(
                &flow,
                "改造成两片之后的第 0 片",
                &is_an_instance_table_unit,
                false,
            );
            corrupt_units_and_reseal_up_to_the_root(
                &mut flow,
                &first_page_copies,
                &set_the_first_mac_byte_of_the_chain_record_pointer,
            );
        }
        ChangeOfTheNewestInstanceTable::SecondPageHoldingTheRowOfTheAbandonedInstanceNamedThroughAPointerWhoseExtentOffsetIsNotZero => {
            let (rows_of_the_abandoned_instance, other_rows) =
                rows_of_the_newest_instance_table_split_off_the_abandoned_instance(
                    &flow,
                    &newest_instance_table,
                );
            rebuild_the_newest_instance_table_as_a_chain(
                &mut flow,
                &newest_instance_table,
                &[other_rows, rows_of_the_abandoned_instance],
                PagesAfterTheFirstInTheFlow::Intact,
            );
            let first_page_copies = last_unit_copies(
                &flow,
                "改造成两片之后的第 0 片",
                &is_an_instance_table_unit,
                false,
            );
            corrupt_units_and_reseal_up_to_the_root(
                &mut flow,
                &first_page_copies,
                &set_the_extent_offset_of_the_chain_record_pointer,
            );
        }
        ChangeOfTheNewestInstanceTable::ChainRecordSayingANextPageWithoutAPointerAndNoRowOfTheAbandonedInstance => {
            let (rows_of_the_abandoned_instance, other_rows) =
                rows_of_the_newest_instance_table_split_off_the_abandoned_instance(
                    &flow,
                    &newest_instance_table,
                );
            assert_eq!(rows_of_the_abandoned_instance.len(), 1, "实例 2 那一行恰好一行");
            let mut records = other_rows;
            records.push(chain_record_saying_a_next_page_without_a_pointer());
            corrupt_units_and_reseal_up_to_the_root(&mut flow, &newest_instance_table, &|bytes| {
                replace_the_records_of_an_instance_table_page(bytes, &records);
            });
        }
        ChangeOfTheNewestInstanceTable::SecondPageHoldingTheRowOfTheAbandonedInstanceUnreadable => {
            let (rows_of_the_abandoned_instance, other_rows) =
                rows_of_the_newest_instance_table_split_off_the_abandoned_instance(
                    &flow,
                    &newest_instance_table,
                );
            rebuild_the_newest_instance_table_as_a_chain(
                &mut flow,
                &newest_instance_table,
                &[other_rows, rows_of_the_abandoned_instance],
                PagesAfterTheFirstInTheFlow::BothCopiesFlippedWithoutResealing,
            );
        }
        ChangeOfTheNewestInstanceTable::SpreadOverPagesWithoutTheRowOfTheAbandonedInstance(
            pages,
        ) => {
            let (_, other_rows) = rows_of_the_newest_instance_table_split_off_the_abandoned_instance(
                &flow,
                &newest_instance_table,
            );
            let mut rows_of_each_page = vec![other_rows];
            rows_of_each_page.resize(pages.count(), Vec::new());
            rebuild_the_newest_instance_table_as_a_chain(
                &mut flow,
                &newest_instance_table,
                &rows_of_each_page,
                PagesAfterTheFirstInTheFlow::Intact,
            );
        }
    }
    (flow, image_with_the_newest_instance_table_readable)
}

/// 最新根（实例 3）指着的那张实例表在流里怎么改（两份副本一起改）。
#[derive(Clone, Copy, Debug)]
enum ChangeOfTheNewestInstanceTable {
    /// 两份各翻一个载荷字节、不重封：位置条目里的整单元校验和对不上，两份都读不出（第 23 批 X 那一形，攻方 N5-d）。
    /// 实例 3 的根落了、C 写回了的状态上，候选集滤不掉被抛弃的 C：CPU 报 I-9.29 部分没判、不红，GPU 扫描内核那张表不比、不置位
    /// （照旧比的话 C 那条记录与现行线同号不同出生代，GPU 红 I-9.29、CPU 不红）。
    BothCopiesFlippedWithoutResealing,
    /// 链指针记录之前插一条没登记种类（kind 2）的记录、重封到根（攻方 N5-a，`research/prompts/m3-batch23-code-r1-opus-output.md`「一、N5 打中」）：
    /// I-3.14 不成立，这一片解不开（用户 2026-10-06 给 D18（块里携带什么信息） 已定项 11 补「kind 不是 0 或 1 ⇒ 这一片解不开」，第 10k 批）：
    /// 主走读不收这一片的行、走不到末片，候选集滤不掉被抛弃的 C，CPU 报 I-9.29 部分没判、红 I-3.14，GPU 那张表不比、置 I-3.14 那一位。
    RecordOfAnUnregisteredKindInsertedBeforeTheChainRecord,
    /// 链指针记录之前插一条一模一样的链指针记录（kind 1、无下一片）、重封到根：I-3.14「链指针记录只在一片末尾」不成立，处置同上一形。
    SecondChainRecordInsertedInTheMiddleOfThePage,
    /// 第一行（kind 0）的 flags 写成退役的 bit0、重封到根：I-3.14「行的 flags 为 0」不成立（D18（块里携带什么信息） 已定项 11「第一版恒 0、
    /// 非 0 拒收」），处置同上两形。
    FirstRowFlagsSetToTheRetiredBit,
    /// 同上一形，第一行（实例 1 那一行）的 T 另抬到流里最新那条根的 txg（第 10n 批）：只读不判的读法（池级 checker 的
    /// `instance_table_rows_of_root_without_judging`、GPU 扫描内核的 `r_instance_rows`）拒收这一片，I-3.12 两边都不判这张表；
    /// GPU 那条读法不判 flags、照收这一行时，指着这张表的实例 3 那几条根上 GPU 红 I-3.12 而 CPU 不红。记录核对器那条读法
    /// （`InstanceTableOfRootRecord::read`、GPU 恢复内核的 `load_instance_table_of_the_effective_root`）同样拒收：实例 2 那一行照在，
    /// GPU 照收这张表时按它把 C 那几次发布判成被抛弃、不罩，CPU 读不出这张表、照罩。
    FirstRowFlagsSetToTheRetiredBitAndItsPublishedTxgRaisedToTheNewestRoot,
    /// 拆成两片（第 0 片放实例 2 之外的行，第 1 片只有末片的链指针记录），第 0 片链指针记录里盘 0 那条位置条目的槽写成第 0 片自己的槽、
    /// 整单元校验和仍是第 1 片的，盘 1 那条完好（攻方 H-b，`research/prompts/m3-batch25-code-r1-opus-output.md` 第 1.3 节；第 10n 批）。
    /// 读第 1 片时盘 0 那一份对不上、读回的是盘 1 那一份：「走过」按读回那一份的落点认，第 1 片没走过，主走读走到末片、门开，
    /// 候选集里没有实例 2 那一行、滤不掉 C，CPU 红 I-9.29。GPU 按第 0 条位置条目认「走过」时第 1 片被当成走过、门关，GPU 不红而 CPU 红。
    SecondPageWhosePointerOnDeviceZeroNamesTheFirstPage,
    /// 拆成两片：第 0 片放实例 2 之外的行、链指针记录指第 1 片；第 1 片放实例 2 那一行与末片的链指针记录，两份完好。第 0 片链指针记录里
    /// 那条指针 MAC 的头一个字节写 1、位置条目不动，重封到根（I-3.14 第五形：有下一片而位置指针头部有一段不是第一版的取值，用户 2026-10-06
    /// 定 K2-1 甲，第 10u 批）。CPU 三个读法都当第 0 片解不开：主走读红 I-3.14、行不收、链不往下走（那条指针 I-2.4 不判）、门关；
    /// 只读不判的读法与记录核对器读恢复那一版的实例表（`InstanceTableOfRootRecord::read`）读不出；引用集合那一遍记数不全。
    /// GPU 照走第 1 片时：走读内核不红 I-3.14、红 I-2.4，恢复内核读出实例 2 那一行、把 C 那几次发布判成被抛弃，都与 CPU 不一致。
    SecondPageHoldingTheRowOfTheAbandonedInstanceNamedThroughAPointerWhoseMacIsNotZero,
    /// 同上一形，MAC 不动，第 0 片链指针记录里那条指针的 extent 偏移写 1（第 10x 批，第 10t 批代码轮判决 K4-3：GPU 两处第五形判法
    /// 只删 extent 偏移那一截时 MAC 那一条流照绿）。CPU 三个读法与 GPU 三个内核该判的同上一形：走读内核与扫描内核经
    /// `w_chain_record_kind` 的 `pointer_extent_offset_is_zero`，恢复内核经 `next.extent_offset_zero`。
    SecondPageHoldingTheRowOfTheAbandonedInstanceNamedThroughAPointerWhoseExtentOffsetIsNotZero,
    /// 唯一那一片里去掉被抛弃的实例 2 那一行，链指针记录的「有无下一片」写 1、指针照旧全零（不合法的链指针记录）、重封到根
    /// （判决 `research/prompts/m3-batch25-code-r1-main-verification.md` 第三节第 2 条，GPU 关门那一侧「链指针记录不合法」一形）：
    /// 主走读没走到末片，门关，CPU 报 I-9.29 部分没判、不红，GPU 不置位、不比。门要是开了，候选集里没有实例 2 那一行、滤不掉 C，
    /// 比就红——GPU 走读内核把「链指针记录不合法」也当走到末片时，这条流上 GPU 红 I-9.29 而 CPU 不红。
    ChainRecordSayingANextPageWithoutAPointerAndNoRowOfTheAbandonedInstance,
    /// 拆成两片：第 0 片放实例 2 之外的行、链指针记录指第 1 片；第 1 片放实例 2 那一行与末片的链指针记录，两份各翻一个载荷字节、
    /// 不重封（第 1 片两份都读不出；同一判决第 2 条「单片读不出」一形）。主走读读到第 0 片、第 1 片读不出，没走到末片，门关：
    /// CPU 报 I-9.29 部分没判、不红，GPU 不置位、不比（门要是开了，候选集里没有实例 2 那一行，比就红）。
    SecondPageHoldingTheRowOfTheAbandonedInstanceUnreadable,
    /// 拆成 `pages` 片：第 0 片放实例 2 之外的行，其余各片只有链指针记录（攻方 H-c 那一形，`research/prompts/m3-batch25-code-r1-opus-output.md`
    /// 第 2.1 节）。主走读走到末片、门开，候选集里没有实例 2 那一行、滤不掉 C，C 与现行线同号不同出生代，CPU 红 I-9.29。
    SpreadOverPagesWithoutTheRowOfTheAbandonedInstance(InstanceTableChainPages),
}

impl ChangeOfTheNewestInstanceTable {
    const fn flow_name(self) -> &'static str {
        match self {
            Self::BothCopiesFlippedWithoutResealing => {
                "abandoned-root-naming-another-object-birth-newest-instance-table-unreadable"
            }
            Self::RecordOfAnUnregisteredKindInsertedBeforeTheChainRecord => {
                "abandoned-root-naming-another-object-birth-newest-instance-table-with-an-unregistered-record-kind"
            }
            Self::SecondChainRecordInsertedInTheMiddleOfThePage => {
                "abandoned-root-naming-another-object-birth-newest-instance-table-with-a-chain-record-in-the-middle"
            }
            Self::FirstRowFlagsSetToTheRetiredBit => {
                "abandoned-root-naming-another-object-birth-newest-instance-table-with-row-flags-not-zero"
            }
            Self::FirstRowFlagsSetToTheRetiredBitAndItsPublishedTxgRaisedToTheNewestRoot => {
                "abandoned-root-naming-another-object-birth-newest-instance-table-with-row-flags-not-zero-and-its-txg-not-below-the-root"
            }
            Self::SecondPageWhosePointerOnDeviceZeroNamesTheFirstPage => {
                "abandoned-root-naming-another-object-birth-newest-instance-table-second-page-pointer-on-device-zero-naming-the-first-page"
            }
            Self::SecondPageHoldingTheRowOfTheAbandonedInstanceNamedThroughAPointerWhoseMacIsNotZero => {
                "abandoned-root-naming-another-object-birth-newest-instance-table-second-page-named-through-a-pointer-whose-mac-is-not-zero"
            }
            Self::SecondPageHoldingTheRowOfTheAbandonedInstanceNamedThroughAPointerWhoseExtentOffsetIsNotZero => {
                "abandoned-root-naming-another-object-birth-newest-instance-table-second-page-named-through-a-pointer-whose-extent-offset-is-not-zero"
            }
            Self::ChainRecordSayingANextPageWithoutAPointerAndNoRowOfTheAbandonedInstance => {
                "abandoned-root-naming-another-object-birth-newest-instance-table-with-an-invalid-chain-record"
            }
            Self::SecondPageHoldingTheRowOfTheAbandonedInstanceUnreadable => {
                "abandoned-root-naming-another-object-birth-newest-instance-table-second-page-unreadable"
            }
            Self::SpreadOverPagesWithoutTheRowOfTheAbandonedInstance(
                InstanceTableChainPages::AsManyAsTheGpuWalks,
            ) => "abandoned-root-naming-another-object-birth-newest-instance-table-of-256-pages",
            Self::SpreadOverPagesWithoutTheRowOfTheAbandonedInstance(
                InstanceTableChainPages::OneMoreThanTheGpuWalks,
            ) => "abandoned-root-naming-another-object-birth-newest-instance-table-of-257-pages",
        }
    }
}

/// GPU 走读内核沿实例表链最多走几片（`crash_judge_checker.wgsl` 的 `INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED`；CPU 不设上限）。
const GPU_INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED: usize = 256;

/// GPU 走读内核走满 [`GPU_INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED`] 片、链还没完时交回的判不了原因字
/// （`crash_judge_checker.wgsl` 的 `UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED`：「条目太多」一类 3、站点号 61）。
const GPU_UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED: u32 = 3 | (61 << 8);

/// GPU 扫描内核沿实例表链读满 [`GPU_INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED`] 片、链还没完时交回的判不了原因字（`crash_judge_checker_scan.wgsl`，
/// 「条目太多」一类 3）：候选根的引用集合那一处（`UNDECIDABLE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST_LONGER_THAN_THE_PAGES_SCANNED`，站点号 37）、
/// 一条根自己那张实例表的全部行那一处（`UNDECIDABLE_INSTANCE_TABLE_ROWS_LONGER_THAN_THE_PAGES_SCANNED`，I-7.9 与 I-3.12 读它，站点号 38）。
const GPU_UNDECIDABLE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST_LONGER_THAN_THE_PAGES_SCANNED: u32 =
    3 | (37 << 8);
const GPU_UNDECIDABLE_INSTANCE_TABLE_ROWS_LONGER_THAN_THE_PAGES_SCANNED: u32 = 3 | (38 << 8);

/// 走读内核与扫描内核各写一份实例表链的片数上限（两个内核各自编译，共用段 `crash_judge_checker_common.wgsl` 里没有它）：两份都与
/// [`GPU_INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED`] 相同（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第三节第 2 条
/// 「上限用命名常量」）。只读两个内核的源码，不用卡。
#[test]
fn the_walk_kernel_and_the_scan_kernel_follow_instance_table_chains_up_to_the_same_number_of_pages()
{
    let declaration = format!(
        "const INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED: u32 = {GPU_INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED}u;"
    );
    for (kernel, source) in [
        (
            "crash_judge_checker.wgsl",
            include_str!("../src/crash_judge_checker.wgsl"),
        ),
        (
            "crash_judge_checker_scan.wgsl",
            include_str!("../src/crash_judge_checker_scan.wgsl"),
        ),
    ] {
        assert_eq!(
            source.matches(declaration.as_str()).count(),
            1,
            "{kernel} 里恰好一处 `{declaration}`"
        );
    }
}

/// 改造出来的实例表链有几片。
#[derive(Clone, Copy, Debug)]
enum InstanceTableChainPages {
    /// 恰好是 GPU 走读内核走得完的片数：两边都走到末片、都比。
    AsManyAsTheGpuWalks,
    /// 比 GPU 走读内核走得完的多一片：CPU 走到末片、比；GPU 走满上限链还没完，记判不了交 CPU。
    OneMoreThanTheGpuWalks,
}

impl InstanceTableChainPages {
    const fn count(self) -> usize {
        match self {
            Self::AsManyAsTheGpuWalks => GPU_INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED,
            Self::OneMoreThanTheGpuWalks => GPU_INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED + 1,
        }
    }
}

/// 被崩溃恢复抛弃的 C 那一版的实例代号（流里 C 由实例 2 发布，恢复之后的实例 3 在它的实例表里给实例 2 写一行）。
const INSTANCE_OF_THE_ABANDONED_ROOT: u32 = 2;
/// 实例表一行里实例代号（4 字节）的偏移；第 0 字节是种类。
const INSTANCE_TABLE_ROW_INSTANCE_OFFSET: usize = 1;
const INSTANCE_TABLE_RECORD_KIND_ROW: u8 = 0;
/// 链指针记录里「有无下一片」那一字节的偏移，与之后 86 字节节点指针的偏移。
const INSTANCE_TABLE_CHAIN_RECORD_HAS_A_NEXT_PAGE_OFFSET: usize = 1;
const INSTANCE_TABLE_CHAIN_RECORD_POINTER_OFFSET: usize = 2;
/// 打包记录单元头里容器号（8 字节）的偏移：实例表那一片的容器号就是它在链上的片号。
const PACKED_UNIT_CONTAINER_NUMBER_OFFSET: usize = 53;
/// 节点指针里两条位置条目的起点（盘 4、槽 6、整单元校验和 4）。
const POINTER_LOCATION_ENTRY_OFFSETS: [usize; 2] = [50, 64];
/// 一个槽的字节数（实例表一片占两槽）。
const SLOT_BYTES: u64 = 16384;
/// 改造实例表链时第 1 片起写在哪：从这个槽起每片两槽（4 GiB 的盘上流里没写过、基镜像里全零的那一截，写之前逐片核）。
const FIRST_SLOT_OF_THE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST: u64 = 200_000;

/// 实例表一片里的记录，按记录序。
fn records_of_an_instance_table_page(bytes: &[u8]) -> Vec<Vec<u8>> {
    let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
    (0..record_count)
        .map(|record_index| {
            let start = PACKED_UNIT_RECORDS_START + record_index * INSTANCE_TABLE_RECORD_BYTES;
            bytes[start..start + INSTANCE_TABLE_RECORD_BYTES].to_vec()
        })
        .collect()
}

/// 实例表一片的载荷换成 `records`：记录数与声明长度跟着改，载荷里记录之后的字节写 0（重封归调用方）。
fn replace_the_records_of_an_instance_table_page(bytes: &mut [u8], records: &[Vec<u8>]) {
    bytes[PACKED_UNIT_RECORDS_START..].fill(0);
    for (record_index, record) in records.iter().enumerate() {
        let start = PACKED_UNIT_RECORDS_START + record_index * INSTANCE_TABLE_RECORD_BYTES;
        bytes[start..start + INSTANCE_TABLE_RECORD_BYTES].copy_from_slice(record);
    }
    bytes[PACKED_UNIT_RECORD_COUNT_OFFSET..PACKED_UNIT_RECORD_COUNT_OFFSET + 2].copy_from_slice(
        &u16::try_from(records.len())
            .expect("一片装得下")
            .to_le_bytes(),
    );
    bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
        &u16::try_from(records.len() * INSTANCE_TABLE_RECORD_BYTES)
            .expect("一片装得下")
            .to_le_bytes(),
    );
}

/// 最新那张实例表（单片）里的行分成两份：被抛弃的实例 2 那几行，与其余各行（都按原来的次序）。核过它是单片、最后一条是末片的链指针记录。
fn rows_of_the_newest_instance_table_split_off_the_abandoned_instance(
    flow: &RecordedFlow,
    newest_instance_table: &[usize],
) -> (Vec<Vec<u8>>, Vec<Vec<u8>>) {
    let page = flow.writes[newest_instance_table[0]]
        .bytes()
        .expect("单元写带字节");
    let mut records = records_of_an_instance_table_page(page);
    let chain_record = records.pop().expect("实例表一片里至少有链指针记录");
    assert_eq!(
        chain_record,
        chain_record_of_the_last_page(),
        "最新那张实例表只有一片：链指针记录说没有下一片、指针全零"
    );
    assert!(
        records
            .iter()
            .all(|record| record[0] == INSTANCE_TABLE_RECORD_KIND_ROW),
        "链指针记录之前都是行"
    );
    records.into_iter().partition(|row| {
        read_u32_at(row, INSTANCE_TABLE_ROW_INSTANCE_OFFSET) == INSTANCE_OF_THE_ABANDONED_ROOT
    })
}

/// 末片的链指针记录：种类 1、没有下一片、指针全零。
fn chain_record_of_the_last_page() -> Vec<u8> {
    let mut record = vec![0u8; INSTANCE_TABLE_RECORD_BYTES];
    record[0] = INSTANCE_TABLE_RECORD_KIND_CHAIN_RECORD;
    record
}

/// 不合法的链指针记录：说有下一片，指针却全零（D18（块里携带什么信息） 已定项 11 的有无下一片与指针配不上）。
fn chain_record_saying_a_next_page_without_a_pointer() -> Vec<u8> {
    let mut record = chain_record_of_the_last_page();
    record[INSTANCE_TABLE_CHAIN_RECORD_HAS_A_NEXT_PAGE_OFFSET] = 1;
    record
}

/// 照 `pointer`（一条实例表指针）的头与两条位置条目的盘，两条位置条目的槽换成 `slot`、整单元校验和换成 `whole_unit_checksum`。
fn pointer_naming_a_page_at(pointer: &[u8], slot: u64, whole_unit_checksum: u32) -> Vec<u8> {
    let mut pointer = pointer.to_vec();
    for location in POINTER_LOCATION_ENTRY_OFFSETS {
        pointer[location + 4..location + 10].copy_from_slice(&slot.to_le_bytes()[..6]);
        pointer[location + 10..location + 14].copy_from_slice(&whole_unit_checksum.to_le_bytes());
    }
    pointer
}

/// 指下一片的链指针记录：照 `pointer_to_the_first_page`（根记录里指第 0 片的那条实例表指针）的头，两条位置条目换成下一片的槽与整单元 CRC。
fn chain_record_naming_the_next_page(
    pointer_to_the_first_page: &[u8],
    next_page_slot: u64,
    next_page_whole_unit_checksum: u32,
) -> Vec<u8> {
    let pointer = pointer_naming_a_page_at(
        pointer_to_the_first_page,
        next_page_slot,
        next_page_whole_unit_checksum,
    );
    let mut record = chain_record_of_the_last_page();
    record[INSTANCE_TABLE_CHAIN_RECORD_HAS_A_NEXT_PAGE_OFFSET] = 1;
    record[INSTANCE_TABLE_CHAIN_RECORD_POINTER_OFFSET
        ..INSTANCE_TABLE_CHAIN_RECORD_POINTER_OFFSET + NODE_POINTER_BYTES]
        .copy_from_slice(&pointer);
    record
}

/// 节点指针第 `location` 条位置条目里的盘与槽。
fn device_and_slot_of_a_pointer_location(pointer: &[u8], location: usize) -> (u32, u64) {
    let mut slot = [0u8; 8];
    slot[..6].copy_from_slice(&pointer[location + 4..location + 10]);
    (read_u32_at(pointer, location), u64::from_le_bytes(slot))
}

/// 改造出来的实例表链上第 1 片起各片怎么写进流。
#[derive(Clone, Copy, Debug)]
enum PagesAfterTheFirstInTheFlow {
    /// 照造出来的字节写，两份都读得出。
    Intact,
    /// 两份各翻一个载荷字节、不重封（前一片链指针记录里的整单元 CRC 按翻之前算）：两份都读不出。
    BothCopiesFlippedWithoutResealing,
}

/// 照 `template`（实例表的一片）的头造一段链：第 i 片（片号 `first_page_index + i`）放 `rows_of_each_page[i]` 再补链指针记录
/// （末片说没有下一片，其余指下一片：照 `pointer_head`（一条实例表指针）的头，两条位置条目换成下一片的槽与整单元 CRC），容器号写片号、重封；
/// 片号 k 的那一片在 [`FIRST_SLOT_OF_THE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST`] + 2k 槽。交回各片的 (槽, 字节)，按片号升序。
fn instance_table_pages_from(
    template: &[u8],
    pointer_head: &[u8],
    first_page_index: usize,
    rows_of_each_page: &[Vec<Vec<u8>>],
) -> Vec<(u64, Vec<u8>)> {
    let mut pages_from_the_last: Vec<(u64, Vec<u8>)> = Vec::new();
    let mut next_page: Option<(u64, u32)> = None;
    for (index_in_the_run, rows) in rows_of_each_page.iter().enumerate().rev() {
        let page_index =
            u64::try_from(first_page_index + index_in_the_run).expect("片号装得进 u64");
        let mut records = rows.clone();
        records.push(match next_page {
            Some((slot, whole_unit_checksum)) => {
                chain_record_naming_the_next_page(pointer_head, slot, whole_unit_checksum)
            }
            None => chain_record_of_the_last_page(),
        });
        let mut page = template.to_vec();
        replace_the_records_of_an_instance_table_page(&mut page, &records);
        page[PACKED_UNIT_CONTAINER_NUMBER_OFFSET..PACKED_UNIT_CONTAINER_NUMBER_OFFSET + 8]
            .copy_from_slice(&page_index.to_le_bytes());
        reseal_unit(&mut page);
        let slot = FIRST_SLOT_OF_THE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST + 2 * page_index;
        next_page = Some((slot, crc32c(&page)));
        pages_from_the_last.push((slot, page));
    }
    pages_from_the_last.reverse();
    pages_from_the_last
}

/// `pages`（[`instance_table_pages_from`] 交回的 (槽, 字节)）在 `devices` 两块盘上各写一份的单元写，照 `how` 写：核过每一片的槽在这块盘上
/// 流里没写过、基镜像里全零。
fn writes_of_instance_table_pages(
    flow: &RecordedFlow,
    devices: [singlefs_core::address::DeviceIdentity; 2],
    pages: Vec<(u64, Vec<u8>)>,
    how: PagesAfterTheFirstInTheFlow,
) -> Vec<RetainedWrite> {
    let mut page_writes: Vec<RetainedWrite> = Vec::new();
    for (slot, page) in pages {
        let page_offset = slot * SLOT_BYTES;
        let page_end = page_offset + u64::try_from(page.len()).expect("单元长装得进 u64");
        for device in devices {
            assert!(
                flow.writes.iter().all(|write| write.device != device
                    || write.offset.0 + write.length_in_bytes() <= page_offset
                    || write.offset.0 >= page_end),
                "实例表那一片写的槽 {slot} 在盘 {} 上流里没写过",
                device.0
            );
            assert!(
                singlefs_core::recovery::PoolReader::read(
                    &flow.base,
                    device,
                    singlefs_core::address::DeviceOffsetInBytes(page_offset),
                    page.len(),
                )
                .expect("基镜像里读得到这一截")
                .iter()
                .all(|byte| *byte == 0),
                "实例表那一片写的槽 {slot} 在盘 {} 的基镜像里全零",
                device.0
            );
            let mut written = page.clone();
            match how {
                PagesAfterTheFirstInTheFlow::Intact => {}
                PagesAfterTheFirstInTheFlow::BothCopiesFlippedWithoutResealing => {
                    let flipped = written.len() - BYTES_FROM_THE_END_OF_THE_FLIPPED_PAYLOAD_BYTE;
                    written[flipped] ^= 0xff;
                }
            }
            page_writes.push(RetainedWrite {
                device,
                kind: StepKind::UnitWrite,
                is_force_unit_access: false,
                offset: singlefs_core::address::DeviceOffsetInBytes(page_offset),
                contents: WrittenContents::Bytes(written),
            });
        }
    }
    page_writes
}

/// 一条实例表指针两条位置条目的盘。
fn devices_of_an_instance_table_pointer(
    pointer: &[u8],
) -> [singlefs_core::address::DeviceIdentity; 2] {
    POINTER_LOCATION_ENTRY_OFFSETS.map(|location| {
        singlefs_core::address::DeviceIdentity(
            device_and_slot_of_a_pointer_location(pointer, location).0,
        )
    })
}

/// 最新根（实例 3）指着的那张单片实例表（`first_page_copies` 是写它的两份副本）改造成一条 `rows_of_each_page.len()` 片的链：
/// 第 k 片放 `rows_of_each_page[k]` 再补链指针记录（末片说没有下一片，其余指下一片）。第 0 片就地改、重封到根；第 1 片起照第 0 片的头造
/// （[`instance_table_pages_from`]），两份写在与第 0 片同一对盘上，照 `pages_after_the_first` 写进流（[`writes_of_instance_table_pages`]）；
/// 这些写另起一段、插在写第 0 片那一段之前（第 0 片落了的状态上它们都已落）。
fn rebuild_the_newest_instance_table_as_a_chain(
    flow: &mut RecordedFlow,
    first_page_copies: &[usize],
    rows_of_each_page: &[Vec<Vec<u8>>],
    pages_after_the_first: PagesAfterTheFirstInTheFlow,
) {
    let first_page = flow.writes[first_page_copies[0]]
        .bytes()
        .expect("单元写带字节")
        .to_vec();
    let first_page_slot = flow.writes[first_page_copies[0]].offset.0 / SLOT_BYTES;
    let pointer_to_the_first_page = flow
        .writes
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
        .filter_map(RetainedWrite::bytes)
        .map(|root| {
            root[ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET
                ..ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + NODE_POINTER_BYTES]
                .to_vec()
        })
        .find(|pointer| {
            device_and_slot_of_a_pointer_location(pointer, POINTER_LOCATION_ENTRY_OFFSETS[0]).1
                == first_page_slot
        })
        .expect("流里有根记录指着最新那张实例表");
    let built_pages_after_the_first = instance_table_pages_from(
        &first_page,
        &pointer_to_the_first_page,
        1,
        &rows_of_each_page[1..],
    );
    let mut first_page_records = rows_of_each_page[0].clone();
    first_page_records.push(match built_pages_after_the_first.first() {
        Some((slot, page)) => {
            chain_record_naming_the_next_page(&pointer_to_the_first_page, *slot, crc32c(page))
        }
        None => chain_record_of_the_last_page(),
    });
    corrupt_units_and_reseal_up_to_the_root(flow, first_page_copies, &|bytes| {
        replace_the_records_of_an_instance_table_page(bytes, &first_page_records);
    });
    let page_writes = writes_of_instance_table_pages(
        flow,
        devices_of_an_instance_table_pointer(&pointer_to_the_first_page),
        built_pages_after_the_first,
        pages_after_the_first,
    );
    let first_page_write = *first_page_copies.iter().min().expect("第 0 片至少一份");
    insert_a_segment_of_writes_before_the_segment_of(flow, first_page_write, page_writes);
}

/// `writes` 另起一段，插在 `write_index` 那次写所在那一段之前（那一段之前的各段之后）：之后各次写的下标与 `judged_root_index` 跟着挪。
fn insert_a_segment_of_writes_before_the_segment_of(
    flow: &mut RecordedFlow,
    write_index: usize,
    writes: Vec<RetainedWrite>,
) {
    let segment_of_the_write = flow
        .segments
        .iter()
        .position(|segment| segment.contains(&write_index))
        .expect("那次写在某一段里");
    let insertion = *flow.segments[segment_of_the_write]
        .iter()
        .min()
        .expect("段里至少一次写");
    assert!(
        flow.segments[..segment_of_the_write]
            .iter()
            .flatten()
            .all(|index| *index < insertion)
            && flow.segments[segment_of_the_write..]
                .iter()
                .flatten()
                .all(|index| *index >= insertion),
        "段按写序排：插进来的写在那一段之前、之前各段之后"
    );
    let inserted = writes.len();
    flow.writes.splice(insertion..insertion, writes);
    for segment in &mut flow.segments {
        for index in segment.iter_mut() {
            if *index >= insertion {
                *index += inserted;
            }
        }
    }
    flow.segments.insert(
        segment_of_the_write,
        (insertion..insertion + inserted).collect(),
    );
    if flow.judged_root_index >= insertion {
        flow.judged_root_index += inserted;
    }
}

/// 实例表一条记录的字节数（D18（块里携带什么信息） 已定项 11）；第 0 字节是种类：1 链指针记录，2 没登记。
const INSTANCE_TABLE_RECORD_BYTES: usize = 88;
const INSTANCE_TABLE_RECORD_KIND_CHAIN_RECORD: u8 = 1;
const INSTANCE_TABLE_RECORD_KIND_UNREGISTERED: u8 = 2;

/// 实例表一片里链指针记录（最后一条）往后挪一条，原位插一条全零、种类 2 的记录，记录数与声明长度各加一条（重封归调用方）。
fn insert_a_record_of_an_unregistered_kind_before_the_chain_record(bytes: &mut [u8]) {
    let mut record_of_an_unregistered_kind = vec![0u8; INSTANCE_TABLE_RECORD_BYTES];
    record_of_an_unregistered_kind[0] = INSTANCE_TABLE_RECORD_KIND_UNREGISTERED;
    insert_a_record_before_the_chain_record(bytes, &record_of_an_unregistered_kind);
}

/// 实例表一片里链指针记录之前插一条它自己的拷贝：片中间多一条链指针记录（重封归调用方）。
fn insert_a_copy_of_the_chain_record_before_the_chain_record(bytes: &mut [u8]) {
    let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
    assert!(record_count >= 1, "实例表一片里至少有链指针记录");
    let chain_record_start =
        PACKED_UNIT_RECORDS_START + (record_count - 1) * INSTANCE_TABLE_RECORD_BYTES;
    let chain_record =
        bytes[chain_record_start..chain_record_start + INSTANCE_TABLE_RECORD_BYTES].to_vec();
    insert_a_record_before_the_chain_record(bytes, &chain_record);
}

/// 实例表行记录（kind 0）里 flags 字节的偏移（kind 1 + 实例代号 4 + T 8 + W 8，D18（块里携带什么信息） 已定项 11）与退役的 bit0。
const INSTANCE_TABLE_ROW_FLAGS_OFFSET: usize = 21;
const INSTANCE_TABLE_ROW_FLAGS_RETIRED_BIT: u8 = 0b1;

/// 实例表一片末尾那条指下一片的链指针记录里，第 0 条位置条目的槽换成 `slot`（盘与整单元校验和不动；重封归调用方）。
fn point_the_first_location_of_the_chain_record_at_slot(bytes: &mut [u8], slot: u64) {
    let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
    assert!(record_count >= 1, "实例表一片里至少有链指针记录");
    let chain_record_start =
        PACKED_UNIT_RECORDS_START + (record_count - 1) * INSTANCE_TABLE_RECORD_BYTES;
    assert_eq!(
        (
            bytes[chain_record_start],
            bytes[chain_record_start + INSTANCE_TABLE_CHAIN_RECORD_HAS_A_NEXT_PAGE_OFFSET]
        ),
        (INSTANCE_TABLE_RECORD_KIND_CHAIN_RECORD, 1),
        "最后一条是指下一片的链指针记录"
    );
    let first_location_slot = chain_record_start
        + INSTANCE_TABLE_CHAIN_RECORD_POINTER_OFFSET
        + POINTER_LOCATION_ENTRY_OFFSETS[0]
        + 4;
    bytes[first_location_slot..first_location_slot + 6].copy_from_slice(&slot.to_le_bytes()[..6]);
}

/// 指针头部 MAC 16 字节的头一个字节在指针里的偏移（D19（块指针的结构与宽度预算） 已定项 11 的偏移表：MAC 从偏移 0 起）。
const POINTER_MAC_FIRST_BYTE_OFFSET: usize = 0;

/// 实例表一片末尾那条指下一片的链指针记录里，位置指针 MAC 的头一个字节写 1（I-3.14 第五形：有下一片而位置指针头部不是第一版的取值；
/// 位置条目不动，指的仍是下一片；重封归调用方）。
fn set_the_first_mac_byte_of_the_chain_record_pointer(bytes: &mut [u8]) {
    let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
    assert!(record_count >= 1, "实例表一片里至少有链指针记录");
    let chain_record_start =
        PACKED_UNIT_RECORDS_START + (record_count - 1) * INSTANCE_TABLE_RECORD_BYTES;
    assert_eq!(
        (
            bytes[chain_record_start],
            bytes[chain_record_start + INSTANCE_TABLE_CHAIN_RECORD_HAS_A_NEXT_PAGE_OFFSET]
        ),
        (INSTANCE_TABLE_RECORD_KIND_CHAIN_RECORD, 1),
        "最后一条是指下一片的链指针记录"
    );
    let first_mac_byte = chain_record_start
        + INSTANCE_TABLE_CHAIN_RECORD_POINTER_OFFSET
        + POINTER_MAC_FIRST_BYTE_OFFSET;
    assert_eq!(
        bytes[first_mac_byte], 0,
        "照第 0 片指针的头造的链指针 MAC 全零"
    );
    bytes[first_mac_byte] = 1;
}

/// 实例表一片末尾那条指下一片的链指针记录里，位置指针的 extent 偏移写 1（I-3.14 第五形 extent 偏移那一段，第 10x 批；
/// MAC 与位置条目不动，指的仍是下一片；重封归调用方）。
fn set_the_extent_offset_of_the_chain_record_pointer(bytes: &mut [u8]) {
    let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
    assert!(record_count >= 1, "实例表一片里至少有链指针记录");
    let chain_record_start =
        PACKED_UNIT_RECORDS_START + (record_count - 1) * INSTANCE_TABLE_RECORD_BYTES;
    assert_eq!(
        (
            bytes[chain_record_start],
            bytes[chain_record_start + INSTANCE_TABLE_CHAIN_RECORD_HAS_A_NEXT_PAGE_OFFSET]
        ),
        (INSTANCE_TABLE_RECORD_KIND_CHAIN_RECORD, 1),
        "最后一条是指下一片的链指针记录"
    );
    set_the_extent_offset_of_the_pointer_at(
        bytes,
        chain_record_start + INSTANCE_TABLE_CHAIN_RECORD_POINTER_OFFSET,
    );
}

/// 实例表一片的第一条（实例 3 写行那一片里是一行）flags 写成退役的 bit0（重封归调用方）。
fn set_the_flags_of_the_first_row_to_the_retired_bit(bytes: &mut [u8]) {
    let first_row_flags = PACKED_UNIT_RECORDS_START + INSTANCE_TABLE_ROW_FLAGS_OFFSET;
    assert_eq!(
        (bytes[PACKED_UNIT_RECORDS_START], bytes[first_row_flags]),
        (0, 0),
        "实例表这一片的第一条是行、flags 是 0"
    );
    bytes[first_row_flags] = INSTANCE_TABLE_ROW_FLAGS_RETIRED_BIT;
}

/// 实例表一片里链指针记录（最后一条）往后挪一条，原位放 `record`，记录数与声明长度各加一条（重封归调用方）。
fn insert_a_record_before_the_chain_record(bytes: &mut [u8], record: &[u8]) {
    let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
    assert!(record_count >= 1, "实例表一片里至少有链指针记录");
    let chain_record_start =
        PACKED_UNIT_RECORDS_START + (record_count - 1) * INSTANCE_TABLE_RECORD_BYTES;
    assert_eq!(
        bytes[chain_record_start], INSTANCE_TABLE_RECORD_KIND_CHAIN_RECORD,
        "实例表一片的最后一条是链指针记录"
    );
    let chain_record =
        bytes[chain_record_start..chain_record_start + INSTANCE_TABLE_RECORD_BYTES].to_vec();
    bytes[chain_record_start..chain_record_start + INSTANCE_TABLE_RECORD_BYTES]
        .copy_from_slice(record);
    bytes[chain_record_start + INSTANCE_TABLE_RECORD_BYTES
        ..chain_record_start + 2 * INSTANCE_TABLE_RECORD_BYTES]
        .copy_from_slice(&chain_record);
    let new_record_count = record_count + 1;
    bytes[PACKED_UNIT_RECORD_COUNT_OFFSET..PACKED_UNIT_RECORD_COUNT_OFFSET + 2].copy_from_slice(
        &u16::try_from(new_record_count)
            .expect("一片装得下")
            .to_le_bytes(),
    );
    bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
        &u16::try_from(new_record_count * INSTANCE_TABLE_RECORD_BYTES)
            .expect("一片装得下")
            .to_le_bytes(),
    );
}

/// 实例表一行里本实例最后一次发布的 `checkpoint_txg`（T，8 字节）的偏移（种类 1 字节、实例代号 4 字节之后）。
const INSTANCE_TABLE_ROW_PUBLISHED_TXG_OFFSET: usize = 5;
/// journal 记录新根段里实例表指针（86 字节）离新根段起点的偏移（walk.rs 的 `NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER`）。
const JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER_OFFSET: usize = 188;
/// C 自己那张实例表里改了 T 的那一行的实例代号：实例 2 写行那次写的表里只有实例 1 那一行。
const INSTANCE_OF_THE_ROW_MOVED_TO_THE_ABANDONED_ROOT_TXG: u32 = 1;

/// O-1 那一形（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 K1 J1 那一行、第三节第 2 条）：被崩溃恢复抛弃的 C
/// （[`record_a_root_abandoned_by_a_crash_recovery`]；实例 3 的根落了之后 C 在回退候选集外）自己那张实例表换成一条 257 片的新链。
/// C 的根记录原来指着实例 2 写行那次写的单片实例表（实例 2 的几条根共用），改指新链的第 0 片：第 0 片放那张表的行，实例 1 那一行的 T 改成
/// C 自己的 txg（I-3.12「T < 这条根的 checkpoint_txg」不成立），其余各片只有链指针记录（[`instance_table_pages_from`]，头照那张表）；
/// C 那次发布的根槽写（发布时一次、恢复之后原样写回一次）与 journal 记录新根段里的实例表指针都改指第 0 片、重封；新链各片两份另起一段，
/// 插在 C 那次发布的记录与根槽写之前（引用它们的写落了的状态上它们都已落）。CPU 一侧先钉流造对了：整条落盘的镜像上 I-3.12 红。
fn record_an_abandoned_root_whose_own_instance_table_is_a_chain_longer_than_the_gpu_walks(
    tag: &str,
) -> RecordedFlow {
    let FlowWithARootAbandonedByACrashRecovery {
        mut flow,
        stream_indexes,
        operations_before_the_abandoned_root,
        operations_through_the_abandoned_root,
        abandoned_root_checkpoint_txg,
    } = record_a_root_abandoned_by_a_crash_recovery(
        tag,
        "abandoned-root-own-instance-table-of-257-pages-with-a-row-at-its-own-txg",
    );
    let root_slot_write_of_the_publish: Vec<usize> = (0..flow.writes.len())
        .filter(|write_index| {
            (operations_before_the_abandoned_root..operations_through_the_abandoned_root)
                .contains(&stream_indexes[*write_index])
                && flow.writes[*write_index].kind == StepKind::RootRecordFua
        })
        .collect();
    assert_eq!(
        root_slot_write_of_the_publish.len(),
        1,
        "C 那次发布写一次根槽"
    );
    let root_of_the_abandoned = flow.writes[root_slot_write_of_the_publish[0]]
        .bytes()
        .expect("根槽写带字节")
        .to_vec();
    assert_eq!(
        read_u64_at(&root_of_the_abandoned, ROOT_RECORD_CHECKPOINT_TXG_OFFSET),
        abandoned_root_checkpoint_txg,
        "那次根槽写写的是 C"
    );
    let instance_of_the_abandoned = read_u32_at(&root_of_the_abandoned, 24);
    let root_slot_writes_of_the_abandoned: Vec<usize> = (0..flow.writes.len())
        .filter(|write_index| {
            flow.writes[*write_index].kind == StepKind::RootRecordFua
                && flow.writes[*write_index].bytes() == Some(root_of_the_abandoned.as_slice())
        })
        .collect();
    assert_eq!(
        root_slot_writes_of_the_abandoned.len(),
        2,
        "C 的根槽写两次：发布时一次，恢复之后原样写回一次"
    );
    let journal_records_of_the_abandoned: Vec<usize> = (0..flow.writes.len())
        .filter(|write_index| {
            flow.writes[*write_index].kind == StepKind::JournalRecord
                && flow.writes[*write_index].bytes().is_some_and(|bytes| {
                    read_u32_at(bytes, JOURNAL_RECORD_INSTANCE_OFFSET) == instance_of_the_abandoned
                        && read_u64_at(bytes, JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET)
                            == abandoned_root_checkpoint_txg
                })
        })
        .collect();
    assert!(
        !journal_records_of_the_abandoned.is_empty(),
        "C 那次发布写了 journal 记录"
    );
    let pointer_to_the_shared_table = root_of_the_abandoned
        [ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET
            ..ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + NODE_POINTER_BYTES]
        .to_vec();
    let journal_pointer_field = JOURNAL_NEW_ROOT_SEGMENT_OFFSET
        + JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER_OFFSET
        ..JOURNAL_NEW_ROOT_SEGMENT_OFFSET
            + JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER_OFFSET
            + NODE_POINTER_BYTES;
    for record in &journal_records_of_the_abandoned {
        assert_eq!(
            &flow.writes[*record].bytes().expect("记录写带字节")[journal_pointer_field.clone()],
            pointer_to_the_shared_table.as_slice(),
            "C 那次发布的记录新根段里的实例表指针与 C 的根记录相同"
        );
    }
    let (shared_table_device, shared_table_slot) = device_and_slot_of_a_pointer_location(
        &pointer_to_the_shared_table,
        POINTER_LOCATION_ENTRY_OFFSETS[0],
    );
    let shared_table = flow.writes[..root_slot_write_of_the_publish[0]]
        .iter()
        .rev()
        .find(|write| {
            write.kind == StepKind::UnitWrite
                && write.device.0 == shared_table_device
                && write.offset.0 / SLOT_BYTES == shared_table_slot
        })
        .and_then(RetainedWrite::bytes)
        .expect("C 之前流里写过 C 指着的那张实例表")
        .to_vec();
    assert!(
        unit_class(&shared_table) == 3
            && packed_record_type(&shared_table) == PACKED_RECORD_TYPE_INSTANCE_TABLE,
        "C 的根记录指着一片实例表"
    );
    let mut rows = records_of_an_instance_table_page(&shared_table);
    assert_eq!(
        rows.pop(),
        Some(chain_record_of_the_last_page()),
        "C 指着的那张实例表只有一片"
    );
    let mut moved_rows = 0usize;
    for row in &mut rows {
        if row[0] == INSTANCE_TABLE_RECORD_KIND_ROW
            && read_u32_at(row, INSTANCE_TABLE_ROW_INSTANCE_OFFSET)
                == INSTANCE_OF_THE_ROW_MOVED_TO_THE_ABANDONED_ROOT_TXG
        {
            row[INSTANCE_TABLE_ROW_PUBLISHED_TXG_OFFSET
                ..INSTANCE_TABLE_ROW_PUBLISHED_TXG_OFFSET + 8]
                .copy_from_slice(&abandoned_root_checkpoint_txg.to_le_bytes());
            moved_rows += 1;
        }
    }
    assert_eq!(moved_rows, 1, "那张表里实例 1 恰好一行");
    let mut rows_of_each_page = vec![rows];
    rows_of_each_page.resize(
        InstanceTableChainPages::OneMoreThanTheGpuWalks.count(),
        Vec::new(),
    );
    let pages = instance_table_pages_from(
        &shared_table,
        &pointer_to_the_shared_table,
        0,
        &rows_of_each_page,
    );
    let pointer_to_the_new_first_page = pointer_naming_a_page_at(
        &pointer_to_the_shared_table,
        pages[0].0,
        crc32c(&pages[0].1),
    );
    for root_slot_write in &root_slot_writes_of_the_abandoned {
        rewrite_root_record_and_reseal(&mut flow, *root_slot_write, &|bytes| {
            bytes[ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET
                ..ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + NODE_POINTER_BYTES]
                .copy_from_slice(&pointer_to_the_new_first_page);
        });
    }
    for record in &journal_records_of_the_abandoned {
        rewrite_journal_record_and_reseal(&mut flow, *record, &|bytes| {
            bytes[journal_pointer_field.clone()].copy_from_slice(&pointer_to_the_new_first_page);
        });
    }
    let page_writes = writes_of_instance_table_pages(
        &flow,
        devices_of_an_instance_table_pointer(&pointer_to_the_shared_table),
        pages,
        PagesAfterTheFirstInTheFlow::Intact,
    );
    let first_write_naming_the_new_chain = *root_slot_writes_of_the_abandoned
        .iter()
        .chain(&journal_records_of_the_abandoned)
        .min()
        .expect("至少一次写");
    insert_a_segment_of_writes_before_the_segment_of(
        &mut flow,
        first_write_naming_the_new_chain,
        page_writes,
    );
    let mut image_with_every_write = flow.base.clone();
    image_with_every_write.apply_writes(&flow.writes);
    assert!(
        matches!(
            pool_checker_verdict_on_the_image(&image_with_every_write, "I-3.12"),
            InvariantVerdict::Violated(_)
        ),
        "流 {}：CPU 沿 C 自己那张 257 片的实例表读到末片，实例 1 那一行的 T 等于 C 的 txg，I-3.12 红：{:?}",
        flow.name,
        pool_checker_verdict_on_the_image(&image_with_every_write, "I-3.12")
    );
    flow
}

/// O-1：回退候选集外的根（被崩溃恢复抛弃的 C）自己那张实例表链 257 片
/// （[`record_an_abandoned_root_whose_own_instance_table_is_a_chain_longer_than_the_gpu_walks`]）。CPU 读链不设片数上限，读到末片、判 I-3.12 红；
/// GPU 走满上限链还没完，池级 checker 记判不了（实例表链过长那三种原因字之一），不当读不出、跳过 I-3.12。逐状态：池级 checker 判了的，
/// 违例位与 CPU 逐位相同；判不了的，原因字只能是实例表链过长，且至少一个这样的状态上 CPU 红 I-3.12；恢复内核判不了的，原因只能是
/// 沿链读满 256 片还有下一片（[`GPU_RECOVERY_REASONS_LEFT_TO_THE_CPU_ON_AN_INSTANCE_TABLE_CHAIN_OF_257_PAGES`]，恢复读到 C 的状态）；判不了就回落 CPU
/// （`gpu_verdict_items` 交回 None），回落之后每个状态的红项都与 CPU 相同。
/// 记哪一个原因字是推的（读码，没跑过）：走读内核把根环里每条自证过的根都另起一份走读走一遍（最新根、候选根、被抛弃的根、F 之下的根各一个阶段，
/// `crash_judge_checker.wgsl` 的 `walk_and_hand_off`），每次先走它的实例表链，链 257 片先在那里记 61，扫描内核不跑；扫描内核那两处（37、38）
/// 是走读内核哪天不再走某一类根时的接手，这条流上落不到。
#[test]
fn the_gpu_judge_leaves_to_the_cpu_an_instance_table_chain_longer_than_the_pages_it_walks_on_a_root_outside_the_candidates(
) {
    let test_name = "the_gpu_judge_leaves_to_the_cpu_an_instance_table_chain_longer_than_the_pages_it_walks_on_a_root_outside_the_candidates";
    let flow =
        record_an_abandoned_root_whose_own_instance_table_is_a_chain_longer_than_the_gpu_walks(
            "crash-judge-gpu-i312-abandoned-root-instance-table-of-257-pages",
        );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    let mut states_left_to_the_cpu_by_a_chain_limit = 0u64;
    let mut rows_red_on_a_state_left_to_the_cpu_by_a_chain_limit = false;
    let mut disagreements: Vec<String> = Vec::new();
    for (ordinal, gpu_words) in comparison.gpu_words_of_each_state.iter().enumerate() {
        let cpu_checker_bits = comparison.cpu_checker_bits_of_each_state[ordinal];
        let cpu_red_items =
            &comparison.cpu_red_items_compared_by_the_cross_check_of_each_state[ordinal];
        match gpu_words[POOL_CHECKER_UNDECIDABLE_WORD] {
            0 => {
                let gpu_checker_bits = pool_checker_violated_bits(gpu_words);
                if !pool_checker_bits_agree_in_the_cross_check(gpu_checker_bits, cpu_checker_bits) {
                    disagreements.push(format!(
                        "状态 {ordinal}：GPU 池级 checker 判了，违例 {:?}；CPU 违例 {:?}",
                        invariant_names(gpu_checker_bits),
                        invariant_names(cpu_checker_bits)
                    ));
                }
            }
            GPU_UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED
            | GPU_UNDECIDABLE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST_LONGER_THAN_THE_PAGES_SCANNED
            | GPU_UNDECIDABLE_INSTANCE_TABLE_ROWS_LONGER_THAN_THE_PAGES_SCANNED => {
                states_left_to_the_cpu_by_a_chain_limit += 1;
                if invariant_names(cpu_checker_bits).contains(&"I-3.12") {
                    rows_red_on_a_state_left_to_the_cpu_by_a_chain_limit = true;
                }
            }
            other_reason => disagreements.push(format!(
                "状态 {ordinal}：GPU 池级 checker 判不了的原因字 {other_reason:#x} 不是实例表链过长"
            )),
        }
        // 恢复读到 C（链 257 片）的状态上恢复内核沿链读满 256 片还有下一片，按设计交 CPU（E9 那一趟这条流 56 个状态记 4）
        disagreements.extend(
            recovery_kernel_left_the_state_to_the_cpu_for_an_unlisted_reason(
                ordinal,
                gpu_words,
                GPU_RECOVERY_REASONS_LEFT_TO_THE_CPU_ON_AN_INSTANCE_TABLE_CHAIN_OF_257_PAGES,
            ),
        );
        // 内核不置位的那几条（第一版是 I-1.13）两边都去掉再比（`red_items_compared_by_the_cross_check`）
        let concluded = gpu_verdict_items(gpu_words).map_or_else(
            || cpu_red_items.clone(),
            |items| red_items_compared_by_the_cross_check(&items),
        );
        if concluded != *cpu_red_items {
            disagreements.push(format!(
                "状态 {ordinal}：判不了就回落 CPU 之后的红项 {concluded} 与 CPU 的 {cpu_red_items} 不同"
            ));
        }
    }
    eprintln!(
        "CRASH_JUDGE_GPU_CHAIN_LIMIT flow={} states={} left_to_the_cpu_by_a_chain_limit={states_left_to_the_cpu_by_a_chain_limit}",
        flow.name, comparison.states
    );
    assert!(
        states_left_to_the_cpu_by_a_chain_limit > 0,
        "流 {}：C 的实例表链 257 片、C 的根读得出的状态上 GPU 池级 checker 要记判不了（实例表链过长）",
        flow.name
    );
    assert!(
        rows_red_on_a_state_left_to_the_cpu_by_a_chain_limit,
        "流 {}：GPU 因链过长判不了的状态里，至少一个 CPU 红 I-3.12（CPU 读到末片、判）",
        flow.name
    );
    assert!(
        disagreements.is_empty(),
        "流 {}：{} 处不一致（前几个）：\n{}",
        flow.name,
        disagreements.len(),
        disagreements
            .iter()
            .take(6)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// 删 → 回退 → 再删的合法历史（第 22 批 C 报告的合法历史之一）：新池新建文件、建空 inode 2（txg 4），把它 unlink 到底，挂着回退到 txg 4
/// （谱系 (回退那次的 txg, 4)；墓碑树随现行那一版，头一条墓碑的死亡 txg 落在被回退抛弃的那一段里、不在谱系上），再把它 unlink 到底
/// （又一条 (2, 同一个对象出生代) 的墓碑，在谱系上）。不按谱系滤的写法在这里判 I-9.20 不重号红。
fn record_new_inode_unlinked_rolled_back_and_unlinked_again(tag: &str) -> RecordedFlow {
    let mut pool = pool_with_new_inodes(tag, 1);
    let before_the_intent = pool.output.clone();
    unlink_to_the_end(&mut pool, FIRST_FILE.0 + 1);
    roll_back_to(&mut pool, &before_the_intent);
    unlink_to_the_end(&mut pool, FIRST_FILE.0 + 1);
    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
    recorded_flow_of_the_pool(
        &pool,
        "new-inode-unlinked-rolled-back-and-unlinked-again",
        versions,
    )
}

/// 退役记录 34 字节（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 2 条）：被退役身份的出生树 0、打包记录类型 8、容器号 10、
/// 容器出生代 18、死亡 txg 26；退役容器是码 3、打包记录类型 3。
const RETIREMENT_RECORD_BYTES: usize = 34;
const RETIREMENT_RECORD_CONTAINER_OFFSET: usize = 10;
const RETIREMENT_RECORD_CONTAINER_BIRTH_OFFSET: usize = 18;
const RETIREMENT_RECORD_DEATH_TXG_OFFSET: usize = 26;
const PACKED_RECORD_TYPE_RETIREMENT: u16 = 3;
/// 一片 inode 叶容器装 233 条（`singlefs_format::INODE_LEAF_RECORDS`）：第一个文件之外再建这么多个空 inode，叶就是满的一片加一条的一片。
const INODE_RECORDS_PER_LEAF: u64 = 233;

fn is_a_retirement_container(bytes: &[u8]) -> bool {
    unit_class(bytes) == 3 && packed_record_type(bytes) == PACKED_RECORD_TYPE_RETIREMENT
}

/// 退役容器第一条退役记录偏移 `field` 起的 8 字节改成 `change` 给的值。
fn change_the_first_retirement_record_field(
    bytes: &mut [u8],
    field: usize,
    change: impl Fn(u64) -> u64,
) {
    assert!(
        read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET) >= 1,
        "退役容器里至少一条退役记录"
    );
    let offset = PACKED_UNIT_RECORDS_START + field;
    let value = change(read_u64_at(bytes, offset));
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

/// 删完的流（[`record_new_pool_through_the_last_deletion_batch`]：最后一批删掉唯一那片 inode 叶，同一次发布开出退役树、写一条退役记录，
/// 死亡 txg = 最后一批那次的 txg 8）上改坏那片退役容器（两份副本）的一处、重封到根。
fn record_retirement_with_a_resealed_field(
    tag: &str,
    name: &'static str,
    mutate: &dyn Fn(&mut [u8]),
) -> RecordedFlow {
    let mut flow = record_new_pool_through_the_last_deletion_batch(tag, name);
    let targets = last_unit_copies(&flow, name, &is_a_retirement_container, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, mutate);
    flow
}

/// I-9.32 与 I-9.35：删完的流上那条退役记录的死亡 txg 8 → 9、重封到根。死亡 txg 大于这一版的 txg 8：CPU 判 I-9.32 红；
/// 死于这一版的退役记录一条都没有，父版（txg 7）那片叶的身份没记：I-9.35 漏记红。
fn record_retirement_dying_after_its_version(tag: &str) -> RecordedFlow {
    record_retirement_with_a_resealed_field(tag, "retirement-dies-after-its-version", &|bytes| {
        change_the_first_retirement_record_field(
            bytes,
            RETIREMENT_RECORD_DEATH_TXG_OFFSET,
            |death_txg| death_txg + 1,
        );
    })
}

/// I-9.33：删完的流上那条退役记录原样再追加一条（记录数加一、声明长度加 34）、重封到根。两条都在谱系上、身份相同：CPU 判 I-9.33 红；
/// 死于这一版的身份集照旧是那一片，I-9.35 照旧成立。
fn record_retirement_written_twice(tag: &str) -> RecordedFlow {
    record_retirement_with_a_resealed_field(tag, "retirement-written-twice", &|bytes| {
        let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
        assert_eq!(record_count, 1, "删到底那一批只写一条退役记录");
        let first = PACKED_UNIT_RECORDS_START;
        bytes.copy_within(
            first..first + RETIREMENT_RECORD_BYTES,
            first + RETIREMENT_RECORD_BYTES,
        );
        bytes[PACKED_UNIT_RECORD_COUNT_OFFSET..PACKED_UNIT_RECORD_COUNT_OFFSET + 2]
            .copy_from_slice(&2u16.to_le_bytes());
        let declared_length = read_u16_at(bytes, UNIT_DECLARED_LENGTH_OFFSET);
        bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
            &(declared_length + u16::try_from(RETIREMENT_RECORD_BYTES).expect("34")).to_le_bytes(),
        );
    })
}

/// I-9.35 漏记那一向：删完的流上那条退役记录的死亡 txg 8 → 7、重封到根。容器出生代 3 < 7 ≤ 8、死于 7 也在谱系上，I-9.32、I-9.33 照旧成立；
/// 死于这一版（8）的退役记录一条都没有，父版（txg 7）那片叶的身份没记：CPU 只红 I-9.35。
fn record_retirement_dying_at_the_parent_version(tag: &str) -> RecordedFlow {
    record_retirement_with_a_resealed_field(
        tag,
        "retirement-dies-at-the-parent-version",
        &|bytes| {
            change_the_first_retirement_record_field(
                bytes,
                RETIREMENT_RECORD_DEATH_TXG_OFFSET,
                |death_txg| death_txg - 1,
            );
        },
    )
}

/// I-9.34 与 I-9.35：新池新建文件之外再建 233 个空 inode（叶装成满的一片 [1..=233] 与一条的一片 [234]），unlink 2 号到底：最后一批左片吸收右片、
/// 右片身份消失，写一条 (右片身份, 这次的 txg) 的退役记录。把这条退役记录的容器号、容器出生代改成幸存左片的、重封到根：它在谱系上、指着这一版的活叶，
/// CPU 判 I-9.34 红；死于这一版的身份集成了左片、右片没记，I-9.35 漏记多记一起红。
fn record_absorption_with_the_retirement_record_naming_the_surviving_left_leaf(
    tag: &str,
) -> RecordedFlow {
    const NAME: &str = "absorption-retirement-record-names-the-surviving-left-leaf";
    let mut pool = pool_with_new_inodes(tag, INODE_RECORDS_PER_LEAF);
    let identities_before: Vec<_> = pool
        .output
        .inode_leaf_containers
        .iter()
        .map(|container| container.contents.identity)
        .collect();
    let [left, right] = identities_before[..] else {
        panic!("234 条记录装成两片叶：{identities_before:?}");
    };
    unlink_to_the_end(&mut pool, FIRST_FILE.0 + 1);
    let retired: Vec<_> = pool
        .output
        .retirement_containers
        .iter()
        .flat_map(|container| container.contents.records.iter())
        .map(|record| record.retired)
        .collect();
    assert_eq!(retired, [right], "左吸收右：退役树里一条右片的退役记录");
    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
    let mut flow = recorded_flow_of_the_pool(&pool, NAME, versions);
    let targets = last_unit_copies(&flow, NAME, &is_a_retirement_container, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &move |bytes| {
        change_the_first_retirement_record_field(
            bytes,
            RETIREMENT_RECORD_CONTAINER_OFFSET,
            |_container| left.container,
        );
        change_the_first_retirement_record_field(
            bytes,
            RETIREMENT_RECORD_CONTAINER_BIRTH_OFFSET,
            |_container_birth| left.container_birth.0,
        );
    });
    flow
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_retirement_record_dies_after_its_version() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_retirement_record_dies_after_its_version",
        &record_retirement_dying_after_its_version("crash-judge-gpu-i932-retirement-death-txg"),
        &["I-9.32", "I-9.35"],
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_retirement_record_is_written_twice() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_retirement_record_is_written_twice",
        &record_retirement_written_twice("crash-judge-gpu-i933-retirement-written-twice"),
        "I-9.33",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_retirement_record_names_the_surviving_left_leaf() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_retirement_record_names_the_surviving_left_leaf",
        &record_absorption_with_the_retirement_record_naming_the_surviving_left_leaf(
            "crash-judge-gpu-i934-retirement-names-a-live-leaf",
        ),
        &["I-9.34", "I-9.35"],
    );
}

/// I-9.36 退役树那一形（第 53 批，判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F2；攻方 `opus_probe_container_closed_early`）：
/// 新池新建文件之外再建 233 个空 inode（叶 [1..=233]、[234]），写入口装退役容器一片 1 条的旋钮，unlink 234 到底（最右那片删空、退役一条），
/// 再建 1 个（235 开新叶）、unlink 235 到底（又退役一条）：退役树两片容器各 1 条。旋钮下的合法镜像；CPU 判器走池级 checker 的旧入口
/// （当没装旋钮），不是最右的那片只装 1 条、不是 959，判 I-9.36 红；GPU 扫描内核只有没装旋钮那一支，同判。
fn record_two_leaves_retired_with_one_retirement_record_per_container(tag: &str) -> RecordedFlow {
    let mut pool = pool_with_new_inodes(tag, INODE_RECORDS_PER_LEAF);
    let capacities = ContainerTreeCapacities {
        tombstone_container: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container: LineageContainerCapacity::FromTheContainerFormat,
        retirement_container:
            singlefs_core::retirement_tree::RetirementContainerCapacity::CappedForTests(
                singlefs_core::retirement_tree::RetirementRecordsPerContainerCappedForTests::new(1)
                    .expect("1..=959"),
            ),
        node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    };
    let rightmost_leaf_inode = FIRST_FILE.0 + INODE_RECORDS_PER_LEAF;
    unlink_an_empty_inode_to_the_end_under_the_capacities(
        &mut pool,
        rightmost_leaf_inode,
        capacities,
    );
    common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    unlink_an_empty_inode_to_the_end_under_the_capacities(
        &mut pool,
        rightmost_leaf_inode + 1,
        capacities,
    );
    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
    recorded_flow_of_the_pool(
        &pool,
        "two-leaves-retired-one-retirement-record-per-container",
        versions,
    )
}

/// I-9.36 谱系树那一形：新池新建文件、建 1 个空 inode（txg 4），分配器装上谱系容器一片 1 条的旋钮（回退的发布路径照分配器上的开关取容量），
/// 挂着回退到第一个文件那一版（txg 3）两次：谱系树两片容器各 1 条。旋钮下的合法镜像；CPU 判器走旧入口判 I-9.36 红，GPU 同判。
fn record_two_rollbacks_with_one_lineage_record_per_container(tag: &str) -> RecordedFlow {
    let mut pool = build_pool(tag);
    let first_file_version = pool.output.clone();
    common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    pool.allocator
        .set_deletion_switches(singlefs_core::unlink::DeletionSwitches {
            batch_size: UnlinkBatchSize::OneExtentLowerLeaf,
            tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
            lineage_container_capacity: LineageContainerCapacity::CappedForTests(
                singlefs_core::lineage_tree::LineageRecordsPerContainerCappedForTests::new(1)
                    .expect("1..=1359"),
            ),
            retirement_container_capacity:
                singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
            container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
        });
    roll_back_to(&mut pool, &first_file_version);
    roll_back_to(&mut pool, &first_file_version);
    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
    recorded_flow_of_the_pool(
        &pool,
        "two-rollbacks-one-lineage-record-per-container",
        versions,
    )
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_retirement_container_closes_before_it_is_full() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_retirement_container_closes_before_it_is_full",
        &record_two_leaves_retired_with_one_retirement_record_per_container(
            "crash-judge-gpu-i936-retirement-container-closed-early",
        ),
        "I-9.36",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_lineage_container_closes_before_it_is_full() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_lineage_container_closes_before_it_is_full",
        &record_two_rollbacks_with_one_lineage_record_per_container(
            "crash-judge-gpu-i936-lineage-container-closed-early",
        ),
        "I-9.36",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_retirement_record_of_a_gone_leaf_dies_at_the_parent_version(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_retirement_record_of_a_gone_leaf_dies_at_the_parent_version",
        &record_retirement_dying_at_the_parent_version(
            "crash-judge-gpu-i935-retirement-death-txg-at-the-parent",
        ),
        "I-9.35",
    );
}

/// 墓碑与谱系那九条（I-9.24 在删除态扫描里判；I-9.22 崩溃点重放恒不适用）。
const TOMBSTONE_AND_LINEAGE_INVARIANTS: [&str; 9] = [
    "I-9.17", "I-9.18", "I-9.19", "I-9.20", "I-9.21", "I-9.22", "I-9.23", "I-9.24", "I-9.29",
];

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_lineage_root_entry_names_another_container_than_its_child(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_lineage_root_entry_names_another_container_than_its_child",
        &record_rollback_with_the_lineage_root_entry_naming_another_container(
            "crash-judge-gpu-i917-lineage-entry-container",
        ),
        "I-9.17",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_carries_an_unregistered_type() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_carries_an_unregistered_type",
        &record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type(
            "crash-judge-gpu-i923-tombstone-entry-type-seven",
        ),
        "I-9.23",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_carries_an_identity()
{
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_the_identity_without_a_walk_failure(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_carries_an_identity",
        &record_deletion_with_the_tombstone_root_entry_of_type_zero(
            "crash-judge-gpu-i923-tombstone-entry-type-zero",
        ),
        "I-9.23",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_queued_inode_has_no_tombstone_on_the_lineage() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_queued_inode_has_no_tombstone_on_the_lineage",
        &record_deletion_with_the_tombstone_naming_an_object_without_a_record(
            "crash-judge-gpu-i918-tombstone-object",
        ),
        "I-9.18",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_on_the_lineage_names_a_live_object() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_on_the_lineage_names_a_live_object",
        &record_deletion_with_the_queued_record_back_at_nlink_one("crash-judge-gpu-i919-nlink-one"),
        "I-9.19",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_dies_before_its_object_is_born() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_dies_before_its_object_is_born",
        &record_deletion_with_the_tombstone_dying_at_txg_zero(
            "crash-judge-gpu-i920-death-txg-zero",
        ),
        "I-9.20",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_on_the_lineage_is_written_twice() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_on_the_lineage_is_written_twice",
        &record_deletion_with_the_tombstone_written_twice("crash-judge-gpu-i920-tombstone-twice"),
        "I-9.20",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_names_another_object_birth_than_the_record() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_names_another_object_birth_than_the_record",
        &record_deletion_with_the_tombstone_object_birth_off_its_record(
            "crash-judge-gpu-i929-tombstone-object-birth",
        ),
        &["I-9.29", "I-9.10"],
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_non_rightmost_tombstone_container_is_not_full() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_non_rightmost_tombstone_container_is_not_full",
        &record_two_empty_inodes_unlinked_with_one_tombstone_per_container(
            "crash-judge-gpu-i921-one-tombstone-per-container",
        ),
        "I-9.21",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_records_of_an_inode_leaf_are_out_of_order() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_records_of_an_inode_leaf_are_out_of_order",
        &record_new_pool_with_the_two_records_of_the_inode_leaf_swapped(
            "crash-judge-gpu-i94-records-swapped",
        ),
        "I-9.4",
    );
}

/// 恢复那一半整份比（看 journal 与不看那一遍落到的根与结局）：零条目的 extent 上段根条目宽窄于字段表，两边都答没有文件。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_an_empty_extent_root_narrower_than_its_field_table_holds_no_file(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_that_an_empty_extent_root_narrower_than_its_field_table_holds_no_file",
        &record_deletion_through_the_last_batch_with_a_narrow_empty_extent_root(
            "crash-judge-gpu-empty-extent-root-narrow",
        ),
        "I-1.10",
    );
}

/// 同上，零条目的 inode 树根。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_an_empty_inode_root_narrower_than_its_field_table_holds_no_file(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_that_an_empty_inode_root_narrower_than_its_field_table_holds_no_file",
        &record_deletion_through_the_last_batch_with_a_narrow_empty_inode_root(
            "crash-judge-gpu-empty-inode-root-narrow",
        ),
        "I-1.10",
    );
}

/// 删 → 回退 → 再删的每个状态上 GPU 与 CPU 整份相同；CPU 池级 checker 在这条流上判过 I-9.18、I-9.19、I-9.20、I-9.24、I-9.29 而成立，
/// 墓碑与谱系那九条一个状态都不红（谱系外那条旧墓碑不进 I-9.20 不重号、不进 I-9.29 的表）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_an_unlink_rolled_back_and_unlinked_again() {
    let flow = record_new_inode_unlinked_rolled_back_and_unlinked_again(
        "crash-judge-gpu-unlink-rolled-back-and-unlinked-again",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_on_every_state_of_an_unlink_rolled_back_and_unlinked_again",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
    for invariant in ["I-9.18", "I-9.19", "I-9.20", "I-9.24", "I-9.29"] {
        assert!(
            comparison.cpu_checker_held_invariants.contains(invariant),
            "删 → 回退 → 再删的流上 CPU 池级 checker 判过 {invariant} 而成立（成立过的：{:?}）",
            comparison.cpu_checker_held_invariants
        );
    }
    for invariant in TOMBSTONE_AND_LINEAGE_INVARIANTS {
        assert!(
            !comparison.cpu_checker_red_invariants.contains(invariant),
            "删 → 回退 → 再删的流上 CPU 池级 checker 不红 {invariant}（红过的：{:?}）",
            comparison.cpu_checker_red_invariants
        );
    }
}

/// 墓碑树、谱系树条目的类型段与孩子的真实类不符时，原来的读法跟着连带红的那几条（攻方 N4 那几形，
/// `research/prompts/m3-batch22-code-r1-opus-output.md` 第一节）：按类型段定的长度读孩子读不出（I-2.1），记走读失败让最新根走不完
/// （I-7.2、I-4.8、I-7.4）。
const INVARIANTS_A_WALK_FAILURE_REDDENS: [&str; 4] = ["I-2.1", "I-4.8", "I-7.2", "I-7.4"];

/// 墓碑树、谱系树条目的类型段与孩子的真实类不符的流（攻方 N4 那几形）：CPU 池级 checker 红 `identity_invariant`，
/// [`INVARIANTS_A_WALK_FAILURE_REDDENS`] 一个状态都不红（孩子的类标签只读单元头、不按类型段定读多长，不符就判身份、不往下走、不记走读失败，
/// 判决 `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 N4 那一行）；GPU 每个状态逐位相同。
fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_the_identity_without_a_walk_failure(
    test_name: &str,
    flow: &RecordedFlow,
    identity_invariant: &'static str,
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison
            .cpu_checker_red_invariants
            .contains(identity_invariant),
        "流 {} 上 CPU 池级 checker 要红 {identity_invariant}（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    for invariant in INVARIANTS_A_WALK_FAILURE_REDDENS {
        assert!(
            !comparison.cpu_checker_red_invariants.contains(invariant),
            "流 {} 上身份不符不记走读失败，CPU 池级 checker 不红 {invariant}（红过的：{:?}）",
            flow.name,
            comparison.cpu_checker_red_invariants
        );
    }
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_without_an_identity_names_a_container(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_the_identity_without_a_walk_failure(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_without_an_identity_names_a_container",
        &record_deletion_with_the_tombstone_root_entry_of_type_zero_and_no_identity(
            "crash-judge-gpu-i923-tombstone-entry-type-zero-no-identity",
        ),
        "I-9.23",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_the_container_type_names_a_node_below_the_root(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_the_identity_without_a_walk_failure(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_the_container_type_names_a_node_below_the_root",
        &record_a_tombstone_tree_of_level_two_whose_first_root_entry_names_its_node_as_a_container(
            "crash-judge-gpu-i923-level-two-entry-names-a-node",
            "tombstone-tree-of-level-two-first-root-entry-names-its-node-as-a-container",
            false,
        ),
        "I-9.23",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_the_container_type_names_a_node_below_the_root_after_the_root_ring_turned_over(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_the_identity_without_a_walk_failure(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_the_container_type_names_a_node_below_the_root_after_the_root_ring_turned_over",
        &record_a_tombstone_tree_of_level_two_whose_first_root_entry_names_its_node_as_a_container(
            "crash-judge-gpu-i923-level-two-entry-names-a-node-ring-turned-over",
            "tombstone-tree-of-level-two-first-root-entry-names-its-node-as-a-container-ring-turned-over",
            true,
        ),
        "I-9.23",
    );
}

/// 池级 checker（CPU，旧入口）在一份镜像上判 `invariant` 的结果：镜像先经对拍那一类的库（写进去再读回，用户 2026-10-13 定：
/// CPU 参照也从库读，`gpu_test_start::cpu_reference_table_read_back_from_the_pairing_library`），判的是读回的那一份。
///
/// # Panics
/// 只录那一趟（`SINGLEFS_CRASH_AMPLIFICATION_STAGE=record`）：取卡之前的前提要从库里读回才判得了，这一趟不判。
// cpu-reference-for-gpu-comparison
fn pool_checker_verdict_on_the_image(image: &MemoryPool, invariant: &str) -> InvariantVerdict {
    let read_back = cpu_reference_table_read_back_from_the_pairing_library(
        &RecordedGivenStatesTable {
            table_name: "pool-checker-precondition".to_string(),
            base: image.clone(),
            writes: Vec::new(),
            segments: Vec::new(),
            seams: Vec::new(),
            persisted_sets: vec![Vec::new()],
        },
    )
    .unwrap_or_else(|| {
        panic!(
            "只录那一趟：取卡之前判 {invariant} 的前提要从库里读回那张镜像才判得了；先跑录入段，再不设选段或设 judge 跑判段"
        )
    });
    check_pool_image(&read_back.base)
        .into_iter()
        .find(|(name, _)| *name == invariant)
        .map(|(_, verdict)| verdict)
        .unwrap_or_else(|| panic!("池级 checker 报了 {invariant}"))
}

/// I-9.29 判不了那一形的流（[`record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed`] 取
/// [`ChangeOfTheNewestInstanceTable::BothCopiesFlippedWithoutResealing`]）：
/// CPU 一侧先钉流造对了——翻坏实例表之前整条落盘的镜像上 C 被滤掉、I-9.29 真被评估过且成立；翻坏之后整条落盘的镜像上 I-9.29 报部分没判。
/// 流上 C 还是最新根的那几个状态，C 的实例表读得出、C 进候选集，同号不同出生代照比，CPU 红 I-9.29；GPU 每个状态逐位相同
/// （实例 3 的根落了、C 写回了的状态上 GPU 照旧比的话红 I-9.29 而 CPU 不红）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_naming_another_object_birth_is_not_compared_once_the_newest_instance_table_does_not_read_back(
) {
    let (flow, image_with_the_newest_instance_table_readable) =
        record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed(
            "crash-judge-gpu-i929-newest-instance-table-unreadable",
            ChangeOfTheNewestInstanceTable::BothCopiesFlippedWithoutResealing,
        );
    assert_eq!(
        pool_checker_verdict_on_the_image(&image_with_the_newest_instance_table_readable, "I-9.29"),
        InvariantVerdict::Holds,
        "最新根的实例表读得出：被抛弃的 C 按它滤掉、不进候选集，现行线上各版同号同出生代，I-9.29 真被评估过且成立"
    );
    let mut image_with_the_newest_instance_table_unreadable = flow.base.clone();
    image_with_the_newest_instance_table_unreadable.apply_writes(&flow.writes);
    match pool_checker_verdict_on_the_image(&image_with_the_newest_instance_table_unreadable, "I-9.29") {
        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
            unjudged.starts_with("最新根的实例表读不出"),
            "部分没判的说明点名最新根的实例表读不出：{unjudged}"
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::NotApplicable(_)) => panic!(
            "最新根的实例表两份都翻坏、候选集滤不掉被抛弃的 C：I-9.29 要报部分没判，实际 {other_verdict:?}"
        ),
    }
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_naming_another_object_birth_is_not_compared_once_the_newest_instance_table_does_not_read_back",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-9.29"),
        "流 {} 上 C 还是最新根的状态 CPU 红 I-9.29（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// I-3.14（实例表每片记录形态合法）不成立的三形各一条流（[`record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed`]
/// 取 `change`）共用的判法。CPU 一侧先钉流造对了——整条落盘的镜像上最新根那一片解不开（用户 2026-10-06 给 D18（块里携带什么信息） 已定项 11
/// 补「kind 不是 0 或 1 ⇒ 这一片解不开」，第 10k 批）：主走读不收它的行、走不到末片，I-3.14 红，I-9.29 报部分没判。
/// 流上 C 还是最新根的那几个状态 C 的实例表读得出、C 进候选集，CPU 红 I-9.29；实例 3 的根落了的状态 CPU 红 I-3.14；GPU 每个状态逐位相同
/// （GPU 照收这一片的行、走到末片时，实例 3 的根落了、C 写回了的状态上 GPU 比那张表而红 I-9.29，CPU 不比）。
/// `invariants_the_cpu_never_reddens` 里的每一条，整条落盘的镜像上不是违例、流上一个状态都不红（CPU 一侧钉流造对了的另一半）。
fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_page_that_does_not_decode(
    test_name: &str,
    tag: &str,
    change: ChangeOfTheNewestInstanceTable,
    invariants_the_cpu_never_reddens: &[&str],
) {
    let (flow, _) =
        record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed(
            tag, change,
        );
    let mut image_with_the_page_that_does_not_decode = flow.base.clone();
    image_with_the_page_that_does_not_decode.apply_writes(&flow.writes);
    assert!(
        matches!(
            pool_checker_verdict_on_the_image(&image_with_the_page_that_does_not_decode, "I-3.14"),
            InvariantVerdict::Violated(_)
        ),
        "{change:?}：最新根的实例表那一片记录形态不合法，I-3.14 红"
    );
    match pool_checker_verdict_on_the_image(&image_with_the_page_that_does_not_decode, "I-9.29") {
        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
            unjudged.starts_with("最新根的实例表读不出"),
            "{change:?}：部分没判的说明点名最新根的实例表读不出：{unjudged}"
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::NotApplicable(_)) => panic!(
            "{change:?}：最新根的实例表那一片解不开、候选集滤不掉被抛弃的 C：I-9.29 要报部分没判，实际 {other_verdict:?}"
        ),
    }
    for invariant in invariants_the_cpu_never_reddens {
        let verdict =
            pool_checker_verdict_on_the_image(&image_with_the_page_that_does_not_decode, invariant);
        assert!(
            !matches!(verdict, InvariantVerdict::Violated(_)),
            "{change:?}：整条落盘的镜像上 {invariant} 不是违例，实际 {verdict:?}"
        );
    }
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    for invariant in ["I-9.29", "I-3.14"] {
        assert!(
            comparison.cpu_checker_red_invariants.contains(invariant),
            "流 {} 上 CPU 红 {invariant}（红过的：{:?}）",
            flow.name,
            comparison.cpu_checker_red_invariants
        );
    }
    for invariant in invariants_the_cpu_never_reddens {
        assert!(
            !comparison.cpu_checker_red_invariants.contains(*invariant),
            "流 {} 上 CPU 一个状态都不红 {invariant}（红过的：{:?}）",
            flow.name,
            comparison.cpu_checker_red_invariants
        );
    }
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// 攻方 N5-a 那一形的流（[`ChangeOfTheNewestInstanceTable::RecordOfAnUnregisteredKindInsertedBeforeTheChainRecord`]；判决
/// `research/prompts/m3-batch23-code-r1-main-verification.md` 第三节第 1 条「GPU 同改，对拍补 N5-a」），判法见
/// [`assert_the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_page_that_does_not_decode`]。第 10k 批之前这一片照收、C 滤掉、
/// I-9.29 成立；kind 2 那一片解不开之后改钉部分没判。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_carrying_a_record_of_an_unregistered_kind_does_not_decode(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_page_that_does_not_decode(
        "the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_carrying_a_record_of_an_unregistered_kind_does_not_decode",
        "crash-judge-gpu-i929-newest-instance-table-unregistered-record-kind",
        ChangeOfTheNewestInstanceTable::RecordOfAnUnregisteredKindInsertedBeforeTheChainRecord,
        &[],
    );
}

/// 片中间多一条链指针记录那一形的流（[`ChangeOfTheNewestInstanceTable::SecondChainRecordInsertedInTheMiddleOfThePage`]），判法同上。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_with_a_chain_record_in_the_middle_does_not_decode(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_page_that_does_not_decode(
        "the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_with_a_chain_record_in_the_middle_does_not_decode",
        "crash-judge-gpu-i314-newest-instance-table-chain-record-in-the-middle",
        ChangeOfTheNewestInstanceTable::SecondChainRecordInsertedInTheMiddleOfThePage,
        &[],
    );
}

/// 行的 flags 不是 0 那一形的流（[`ChangeOfTheNewestInstanceTable::FirstRowFlagsSetToTheRetiredBit`]），判法同上。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_with_row_flags_not_zero_does_not_decode(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_page_that_does_not_decode(
        "the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_with_row_flags_not_zero_does_not_decode",
        "crash-judge-gpu-i314-newest-instance-table-row-flags-not-zero",
        ChangeOfTheNewestInstanceTable::FirstRowFlagsSetToTheRetiredBit,
        &[],
    );
}

/// I-3.14 第五形的流（[`ChangeOfTheNewestInstanceTable::SecondPageHoldingTheRowOfTheAbandonedInstanceNamedThroughAPointerWhoseMacIsNotZero`]，
/// 第 10u 批），判法同上，另钉 CPU 一个状态都不红 I-2.4：那条指针不跟随、它的头部不判。三个内核都要把这一形当解不开——
/// 走读内核与扫描内核经 `crash_judge_checker_common.wgsl` 的 `w_chain_record_kind`，恢复内核在 `load_instance_table_of_the_effective_root` 里判。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_whose_chain_pointer_mac_is_not_zero_does_not_decode(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_page_that_does_not_decode(
        "the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_whose_chain_pointer_mac_is_not_zero_does_not_decode",
        "crash-judge-gpu-i314-newest-instance-table-chain-pointer-mac-not-zero",
        ChangeOfTheNewestInstanceTable::SecondPageHoldingTheRowOfTheAbandonedInstanceNamedThroughAPointerWhoseMacIsNotZero,
        &["I-2.4"],
    );
}

/// I-3.14 第五形 extent 偏移那一段的流（[`ChangeOfTheNewestInstanceTable::SecondPageHoldingTheRowOfTheAbandonedInstanceNamedThroughAPointerWhoseExtentOffsetIsNotZero`]，
/// 第 10x 批，第 10t 批代码轮判决 K4-3），判法与另钉的同上一条：GPU 两处判法只删 extent 偏移那一截（`w_chain_record_kind` 的
/// `pointer_extent_offset_is_zero`、恢复内核的 `next.extent_offset_zero`）时 MAC 那一条照绿，这一条红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_whose_chain_pointer_extent_offset_is_not_zero_does_not_decode(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_page_that_does_not_decode(
        "the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_whose_chain_pointer_extent_offset_is_not_zero_does_not_decode",
        "crash-judge-gpu-i314-newest-instance-table-chain-pointer-extent-offset-not-zero",
        ChangeOfTheNewestInstanceTable::SecondPageHoldingTheRowOfTheAbandonedInstanceNamedThroughAPointerWhoseExtentOffsetIsNotZero,
        &["I-2.4"],
    );
}

/// 行的 flags 不是 0、那一行的 T 抬到最新根的 txg 那一形的流
/// （[`ChangeOfTheNewestInstanceTable::FirstRowFlagsSetToTheRetiredBitAndItsPublishedTxgRaisedToTheNewestRoot`]，第 10n 批），判法同上，
/// 另钉 CPU 一个状态都不红 I-3.12：只读不判的读法拒收这一片，指着它的根不判。GPU 扫描内核 `r_instance_rows` 不判 flags 时照收这一行、
/// 红 I-3.12；GPU 恢复内核 `load_instance_table_of_the_effective_root` 不判 flags 时照收这张表、按实例 2 那一行不罩 C 那几次发布，
/// 两处都与 CPU 不一致。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_whose_row_flags_are_not_zero_is_not_read_for_the_row_txg_invariant(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_page_that_does_not_decode(
        "the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_whose_row_flags_are_not_zero_is_not_read_for_the_row_txg_invariant",
        "crash-judge-gpu-i312-newest-instance-table-row-flags-not-zero-txg-not-below-the-root",
        ChangeOfTheNewestInstanceTable::FirstRowFlagsSetToTheRetiredBitAndItsPublishedTxgRaisedToTheNewestRoot,
        &["I-3.12"],
    );
}

/// 攻方 H-b 那一形的流（[`ChangeOfTheNewestInstanceTable::SecondPageWhosePointerOnDeviceZeroNamesTheFirstPage`]，第 10n 批）：
/// CPU 一侧先钉流造对了——改之前整条落盘的镜像上 I-9.29 真被评估过且成立；改之后整条落盘的镜像上主走读读回第 1 片盘 1 那一份、
/// 按它的落点认「走过」、走到末片，门开，候选集里没有实例 2 那一行、滤不掉 C，I-9.29 红（不是部分没判）。
/// GPU 每个状态逐位相同（GPU 走读内核按第 0 条位置条目认「走过」时第 1 片被当成走过、门关，实例 3 的根落了、C 写回了的状态上
/// GPU 不红 I-9.29 而 CPU 红）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_second_instance_table_page_whose_pointer_on_device_zero_names_the_first_page_is_walked_from_the_copy_read_back(
) {
    let (flow, image_with_the_newest_instance_table_readable) =
        record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed(
            "crash-judge-gpu-i929-newest-instance-table-second-page-pointer-on-device-zero-naming-the-first-page",
            ChangeOfTheNewestInstanceTable::SecondPageWhosePointerOnDeviceZeroNamesTheFirstPage,
        );
    assert_eq!(
        pool_checker_verdict_on_the_image(&image_with_the_newest_instance_table_readable, "I-9.29"),
        InvariantVerdict::Holds,
        "改实例表之前：被抛弃的 C 按最新根的实例表滤掉，I-9.29 真被评估过且成立"
    );
    let mut image_with_every_write = flow.base.clone();
    image_with_every_write.apply_writes(&flow.writes);
    let verdict = pool_checker_verdict_on_the_image(&image_with_every_write, "I-9.29");
    assert!(
        matches!(verdict, InvariantVerdict::Violated(_)),
        "流 {}：主走读按读回那一份认「走过」、走到第 1 片（末片），候选集里没有实例 2 那一行，I-9.29 红，实际 {verdict:?}",
        flow.name
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_that_a_second_instance_table_page_whose_pointer_on_device_zero_names_the_first_page_is_walked_from_the_copy_read_back",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-9.29"),
        "流 {} 上 CPU 红 I-9.29（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_names_a_leaf_container()
{
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_the_identity_without_a_walk_failure(
        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_names_a_leaf_container",
        &record_new_pool_with_the_inode_root_entry_of_type_zero_naming_its_leaf_container(
            "crash-judge-gpu-i92-inode-root-entry-type-zero",
        ),
        "I-9.2",
    );
}

/// H1 那一形的流（[`record_new_pool_with_the_inode_root_entry_of_type_zero_and_no_identity_over_a_leaf_container_whose_first_copy_is_stale`]）：
/// CPU 池级 checker 红 I-9.2、一个状态都不红 I-7.2（改之前只认第一份陈旧副本的头，I-9.2 报成立，记走读失败让最新根走不完、I-7.2 连带红）；
/// 陈旧副本自己连带的 I-2.1 那几条照红，不在这里钉；GPU 每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_without_an_identity_names_a_leaf_container_whose_first_copy_is_stale(
) {
    let test_name = "the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_without_an_identity_names_a_leaf_container_whose_first_copy_is_stale";
    let flow =
        record_new_pool_with_the_inode_root_entry_of_type_zero_and_no_identity_over_a_leaf_container_whose_first_copy_is_stale(
            "crash-judge-gpu-i92-inode-root-entry-type-zero-first-leaf-copy-stale",
        );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-9.2"),
        "流 {} 上 CPU 池级 checker 要红 I-9.2（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        !comparison.cpu_checker_red_invariants.contains("I-7.2"),
        "流 {} 上第一份陈旧副本不挡住第二份：CPU 池级 checker 不红 I-7.2（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// 单元头里类标签的偏移。
const UNIT_HEADER_CLASS_TAG_OFFSET: usize = 6;
/// 不在登记表里的单元类标签（登记的只有 1、2、3）：孩子头里写成它之后，按哪一类的长度都认不出它是哪一类。
const UNIT_CLASS_NOT_REGISTERED: u8 = 9;

/// inode 树根（层级 1，条目指叶容器）：码 2、key 宽 8、条目宽 120、第一条条目的类型段是 2。
fn is_the_inode_tree_root_over_leaf_containers(bytes: &[u8]) -> bool {
    unit_class(bytes) == 2
        && usize::from(index_node_key_width(bytes)) == INODE_TREE_KEY_BYTES
        && index_node_entry_width(bytes) == INODE_INTERNAL_ENTRY_BYTES
        && read_u16_at(
            bytes,
            index_node_entries_start(INODE_TREE_KEY_BYTES)
                + CONTAINER_TREE_ENTRY_RECORD_TYPE_OFFSET,
        ) == PACKED_RECORD_TYPE_INODE
}

/// inode 树读不出孩子的那一形（第 26 批 H 报告「交主 agent」第 1、2 条，`records/2026-10-01-删除与并发事务.md` 第 27 批；
/// harness 坏镜像 `image_whose_inode_root_entry_names_a_child_that_does_not_read_back` 同一形，录成流）：新池新建文件的流上 inode 叶容器两份
/// 头里的类标签 3 → 9、不重封（[`corrupt_units_and_carry_their_whole_unit_checksums_up_to_the_root`] 取
/// [`OwnChecksumsOfTheChangedUnits::LeftAsBeforeTheChange`]：指着它的位置条目换成改后的整单元 CRC、一路重封到根），
/// 再把 inode 树根第一条条目的类型段 2 → `entry_type`、重封到根（身份引用不动）。孩子头里的类标签不登记，按哪一类都认不出；
/// 类型段 0 按 16 KiB 去读，校验和对不上。
fn record_new_pool_with_the_inode_root_entry_of_another_type_over_a_leaf_container_of_an_unregistered_class(
    tag: &str,
    name: &'static str,
    entry_type: u16,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, name);
    let leaf_copies = last_unit_copies(&flow, name, &is_an_inode_leaf_container, false);
    let leaf_slot = flow.writes[leaf_copies[0]].offset.0 / SLOT_BYTES;
    corrupt_units_and_carry_their_whole_unit_checksums_up_to_the_root(
        &mut flow,
        &leaf_copies,
        &|bytes| bytes[UNIT_HEADER_CLASS_TAG_OFFSET] = UNIT_CLASS_NOT_REGISTERED,
        OwnChecksumsOfTheChangedUnits::LeftAsBeforeTheChange,
    );
    let root_copies = last_unit_copies(
        &flow,
        name,
        &is_the_inode_tree_root_over_leaf_containers,
        false,
    );
    let root = flow.writes[root_copies[0]].bytes().expect("单元写带字节");
    assert_eq!(
        device_and_slot_of_a_pointer_location(
            &root[first_entry_of_a_container_number_node()
                + INODE_INTERNAL_ENTRY_CHILD_POINTER_OFFSET..],
            POINTER_LOCATION_ENTRY_OFFSETS[0],
        )
        .1,
        leaf_slot,
        "inode 树根第一条条目指着的就是改了类标签的那片叶容器"
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
        set_the_type_of_the_first_entry(bytes, entry_type);
    });
    flow
}

/// 第 27 批：inode 树根条目类型段 0、身份引用三段不为 0（容器号、出生代照旧），孩子读不出（类标签改成不登记的码、不重封）：
/// CPU 池级 checker 按 16 KiB 读孩子读不出，I-9.2 里不看孩子就判得了的身份三段照判、红（`judge_an_inode_entry_whose_child_does_not_read_back`），
/// I-2.1 红、记走读失败；GPU 走读内核读不出那一支同判（第 26 批 H 改的 `w_walk_inode_root` 那一支），每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_with_an_identity_names_a_child_that_does_not_read_back(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_with_an_identity_names_a_child_that_does_not_read_back",
        &record_new_pool_with_the_inode_root_entry_of_another_type_over_a_leaf_container_of_an_unregistered_class(
            "crash-judge-gpu-i92-inode-root-entry-type-zero-unreadable-child",
            "new-pool-inode-root-entry-type-zero-over-an-unreadable-child",
            0,
        ),
        &["I-9.2", "I-2.1"],
    );
}

/// 第 27 批：inode 树根条目类型段 2 → 7（不在 {0, 2} 里），孩子读不出（同上一条的叶容器）：CPU 池级 checker 不往下走孩子，
/// 按层级 1 下面的叶容器两槽登记引用、判 I-9.2 红，不记 I-2.1、不记走读失败（与墓碑树、谱系树类型段不认得那一支同判）；
/// GPU 走读内核同改，每个状态逐位相同（照旧按 16 KiB 读孩子的话 GPU 红 I-2.1、记走读失败，CPU 不红）。
/// 「不记 I-2.1」只对这一形成立（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 H-J3a 那一行）：第 29 批起这一支
/// 每一份副本照判 I-2.1，头里的类标签 9 不登记、16 KiB 与 32 KiB 两种长度都比，这片叶容器的位置条目换成了改后的整单元 CRC，按 32 KiB 对得上；
/// 位置条目里的校验和本身坏了的那一形见
/// [`the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered_type_names_a_child_whose_two_location_checksums_are_wrong`]。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered_type_names_a_child_that_does_not_read_back(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_the_identity_without_a_walk_failure(
        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered_type_names_a_child_that_does_not_read_back",
        &record_new_pool_with_the_inode_root_entry_of_another_type_over_a_leaf_container_of_an_unregistered_class(
            "crash-judge-gpu-i92-inode-root-entry-type-seven-unreadable-child",
            "new-pool-inode-root-entry-type-seven-over-an-unreadable-child",
            UNREGISTERED_CONTAINER_TREE_ENTRY_TYPE,
        ),
        "I-9.2",
    );
}

// ───────────────────────── 第 29 批：走读不往下读的孩子每一份副本照判 I-2.1（H-J3a、H-J3b） ─────────────────────────

/// 码 2 节点第一条条目子指针里第 `location` 条位置条目（[`POINTER_LOCATION_ENTRY_OFFSETS`] 之一）的整单元校验和第 0 字节翻一下：
/// 孩子不动，那一份按哪一种长度都与位置条目里的校验和对不上。
fn flip_a_byte_of_a_location_checksum_in_the_first_entry(bytes: &mut [u8], location: usize) {
    let field = first_entry_of_a_container_number_node()
        + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
        + location
        + LOCATION_ENTRY_UNIT_CHECKSUM_OFFSET;
    bytes[field] ^= 0xFF;
}

/// 攻方 B1 那一形（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 H-J3a 那一行；harness 坏镜像
/// `an_inode_tree_entry_with_an_unregistered_type_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants`
/// 同一形，录成流）：新池新建文件的流上 inode 树根（层级 1）第一条条目的类型段 2 → 7、子指针两条位置条目的整单元校验和各翻一字节，重封到根；
/// 叶容器不动。
fn record_new_pool_with_the_inode_root_entry_of_an_unregistered_type_over_two_wrong_location_checksums(
    tag: &str,
) -> RecordedFlow {
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-inode-root-entry-type-seven-two-location-checksums-wrong",
        &is_the_inode_tree_root_over_leaf_containers,
        false,
        &|bytes| {
            set_the_type_of_the_first_entry(bytes, UNREGISTERED_CONTAINER_TREE_ENTRY_TYPE);
            for location in POINTER_LOCATION_ENTRY_OFFSETS {
                flip_a_byte_of_a_location_checksum_in_the_first_entry(bytes, location);
            }
        },
    )
}

/// 第 29 批 H-J3a：类型段不认得那一支不往下走孩子，两份副本照判 I-2.1、都对不上、记走读失败：CPU 池级 checker 红 I-9.2、I-2.1、I-4.8、
/// I-7.2、I-7.4（第 27 批这一支两样都不记，只红 I-9.2）；GPU 走读内核同改（`w_judge_every_copy_of_a_child_the_walk_does_not_go_into`），
/// 每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered_type_names_a_child_whose_two_location_checksums_are_wrong(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered_type_names_a_child_whose_two_location_checksums_are_wrong",
        &record_new_pool_with_the_inode_root_entry_of_an_unregistered_type_over_two_wrong_location_checksums(
            "crash-judge-gpu-i21-inode-root-entry-type-seven-two-location-checksums-wrong",
        ),
        &["I-9.2", "I-2.1", "I-4.8", "I-7.2", "I-7.4"],
    );
}

/// H-J3b 那一形（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 H-J3b 那一行；harness 坏镜像
/// `an_inode_tree_entry_of_type_zero_over_a_leaf_container_whose_first_location_checksum_is_wrong_reddens_the_checksum_invariant` 同一形，录成流）：
/// 新池新建文件的流上 inode 树根（层级 1）第一条条目的类型段 2 → 0、子指针第一条位置条目的整单元校验和翻一字节，重封到根；叶容器不动。
fn record_new_pool_with_the_inode_root_entry_of_type_zero_over_a_wrong_first_location_checksum(
    tag: &str,
) -> RecordedFlow {
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-inode-root-entry-type-zero-first-location-checksum-wrong",
        &is_the_inode_tree_root_over_leaf_containers,
        false,
        &|bytes| {
            set_the_type_of_the_first_entry(bytes, 0);
            flip_a_byte_of_a_location_checksum_in_the_first_entry(
                bytes,
                POINTER_LOCATION_ENTRY_OFFSETS[0],
            );
        },
    )
}

/// 第 29 批 H-J3b：第二份自证成码 3（类型段 0 说的是码 2），走「孩子是另一类」那一支，不往下走；第一份按哪一种长度都对不上，照判 I-2.1：
/// CPU 池级 checker 红 I-9.2、I-2.1、I-4.8、I-7.4（第二份对得上、不记走读失败，I-7.2 不红；第 29 批之前这一支不判 I-2.1）；
/// GPU 走读内核同改，每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_names_a_leaf_container_whose_first_location_checksum_is_wrong(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_names_a_leaf_container_whose_first_location_checksum_is_wrong",
        &record_new_pool_with_the_inode_root_entry_of_type_zero_over_a_wrong_first_location_checksum(
            "crash-judge-gpu-i21-inode-root-entry-type-zero-first-location-checksum-wrong",
        ),
        &["I-9.2", "I-2.1", "I-4.8", "I-7.4"],
    );
}

// ───────────────────────── 第 28 批：读不出孩子时照判出生树那一小句（H-B）、孩子两份都看（H-V）、第二份副本的自证 ─────────────────────────

/// 墓碑树、谱系树码 2 节点条目里子指针的偏移：与 inode 树内部条目同形（分隔 key 8 + 身份引用 26 之后）。
const CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET: usize = INODE_INTERNAL_ENTRY_CHILD_POINTER_OFFSET;
/// 位置条目里整单元校验和的偏移（盘 4、槽 6 之后）。
const LOCATION_ENTRY_UNIT_CHECKSUM_OFFSET: usize = 10;

/// 一个单元的这几份副本各翻一个载荷字节、不重封、不补引用链：指着它的位置条目还是翻之前的整单元 CRC，这几份都读不出。
fn flip_a_payload_byte_of_each_copy_without_resealing(flow: &mut RecordedFlow, copies: &[usize]) {
    for copy in copies {
        let WrittenContents::Bytes(bytes) = &mut flow.writes[*copy].contents else {
            panic!("单元写是普通写");
        };
        let flipped_byte = bytes.len() - BYTES_FROM_THE_END_OF_THE_FLIPPED_PAYLOAD_BYTE;
        bytes[flipped_byte] ^= 0xFF;
    }
}

/// 节点第一条条目的子指针第 0 条位置条目指的槽就是 `copies` 那个单元的槽（钉住改的条目指着要弄坏的那个孩子）。
fn assert_the_first_entry_names_the_unit_of(
    flow: &RecordedFlow,
    node_copy: usize,
    copies: &[usize],
    what: &str,
) {
    let node = flow.writes[node_copy].bytes().expect("单元写带字节");
    assert_eq!(
        device_and_slot_of_a_pointer_location(
            &node[first_entry_of_a_container_number_node()
                + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET..],
            POINTER_LOCATION_ENTRY_OFFSETS[0],
        )
        .1,
        flow.writes[copies[0]].offset.0 / SLOT_BYTES,
        "{what}"
    );
}

/// 节点第一条条目里 `field`（条目内偏移）起那 8 字节（出生树那两段之一）加 1。
fn add_one_to_a_birth_tree_of_the_first_entry(bytes: &mut [u8], field: usize) {
    let offset = first_entry_of_a_container_number_node() + field;
    let birth_tree = read_u64_at(bytes, offset);
    bytes[offset..offset + 8].copy_from_slice(&(birth_tree + 1).to_le_bytes());
}

/// H-B inode 那一形（判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第三节第 1 条；harness 坏镜像
/// `an_inode_tree_entry_whose_identity_birth_tree_is_one_off_over_a_leaf_that_does_not_read_back_reddens_the_inode_entry_invariant` 同一形，录成流）：
/// 新池新建文件的流上 inode 树根（层级 1）第一条条目身份引用的出生树加 1、重封到根，它指的叶容器两份各翻一个载荷字节、不重封（两份都读不出）。
fn record_new_pool_with_the_inode_root_entry_identity_birth_tree_one_off_over_a_leaf_that_does_not_read_back(
    tag: &str,
) -> RecordedFlow {
    let name = "new-pool-inode-root-entry-identity-birth-tree-one-off-unreadable-leaf";
    let mut flow = record_new_pool_file_creation(tag, name);
    let leaf_copies = last_unit_copies(&flow, name, &is_an_inode_leaf_container, false);
    let root_copies = last_unit_copies(
        &flow,
        name,
        &is_the_inode_tree_root_over_leaf_containers,
        false,
    );
    assert_the_first_entry_names_the_unit_of(
        &flow,
        root_copies[0],
        &leaf_copies,
        "inode 树根第一条条目指着流里最后写的那片叶容器",
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
        add_one_to_a_birth_tree_of_the_first_entry(bytes, CONTAINER_TREE_ENTRY_BIRTH_TREE_OFFSET);
    });
    flip_a_payload_byte_of_each_copy_without_resealing(&mut flow, &leaf_copies);
    flow
}

/// 第 28 批 H-B：CPU 池级 checker 读不出孩子时照判「子指针头部的出生树 == 身份引用的出生树」、红 I-9.2（改之前类型段 2 那一支整支不判），
/// 叶容器读不出、I-2.1 红；GPU 走读内核读不出那一支同判（`w_walk_inode_root`），每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_identity_birth_tree_is_one_off_over_a_leaf_that_does_not_read_back(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_identity_birth_tree_is_one_off_over_a_leaf_that_does_not_read_back",
        &record_new_pool_with_the_inode_root_entry_identity_birth_tree_one_off_over_a_leaf_that_does_not_read_back(
            "crash-judge-gpu-i92-inode-root-entry-identity-birth-tree-unreadable-leaf",
        ),
        &["I-9.2", "I-2.1"],
    );
}

/// H-B 墓碑那一形（harness 坏镜像
/// `a_tombstone_tree_entry_whose_child_pointer_birth_tree_is_one_off_over_a_container_that_does_not_read_back_reddens_the_tombstone_entry_invariant`
/// 同一形，录成流）：删除中间态的流上墓碑树根（层级 1）第一条条目子指针头部的出生树加 1、重封到根，它指的墓碑容器两份各翻一个载荷字节、
/// 不重封（两份都读不出）。
fn record_deletion_with_the_tombstone_root_entry_pointer_birth_tree_one_off_over_a_container_that_does_not_read_back(
    tag: &str,
) -> RecordedFlow {
    let name = "deletion-tombstone-root-entry-pointer-birth-tree-one-off-unreadable-container";
    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
    let container_copies = last_unit_copies(&flow, name, &is_a_tombstone_container, false);
    let root_copies = last_unit_copies(&flow, name, &is_the_tombstone_tree_root, false);
    assert_the_first_entry_names_the_unit_of(
        &flow,
        root_copies[0],
        &container_copies,
        "墓碑树根第一条条目指着流里最后写的那片墓碑容器",
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
        add_one_to_a_birth_tree_of_the_first_entry(
            bytes,
            CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET + POINTER_BIRTH_TREE_OFFSET,
        );
    });
    flip_a_payload_byte_of_each_copy_without_resealing(&mut flow, &container_copies);
    flow
}

/// 第 28 批 H-B：CPU 池级 checker 容器读不出时照判「子指针头部出生树 == 身份引用出生树」、红 I-9.23（改之前只记走读失败），
/// 容器读不出、I-2.1 红；GPU 走读内核容器树读不出那一支同判（`w_walk_container_tree_root`），每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_pointer_birth_tree_is_one_off_over_a_container_that_does_not_read_back(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_pointer_birth_tree_is_one_off_over_a_container_that_does_not_read_back",
        &record_deletion_with_the_tombstone_root_entry_pointer_birth_tree_one_off_over_a_container_that_does_not_read_back(
            "crash-judge-gpu-i923-tombstone-root-entry-pointer-birth-tree-unreadable-container",
        ),
        &["I-9.23", "I-2.1"],
    );
}

/// 新池新建文件的流上 inode 叶容器盘序大的那一份的前 16 KiB 换成改之前那个 inode 树根（码 2）的原样字节；`checksum_of_the_second_location`
/// 说 inode 树根第一条条目子指针第 1 条位置条目的整单元校验和怎么办：换成那 16 KiB 的 CRC（重封到根，那一份自证成码 2），或不动（那一份自证不过）。
#[derive(Clone, Copy, Debug)]
enum ChecksumOfTheSecondLocation {
    MatchesTheNodeBytes,
    LeftAsTheLeafChecksum,
}

/// 见 [`ChecksumOfTheSecondLocation`]：流名 `name`，叶容器盘序小的那一份不动（码 3，按它的位置条目自证得过）。
fn record_new_pool_with_the_second_copy_of_the_inode_leaf_holding_the_inode_root_bytes(
    tag: &str,
    name: &'static str,
    checksum_of_the_second_location: ChecksumOfTheSecondLocation,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, name);
    hold_the_inode_root_bytes_in_the_second_copy_of_the_inode_leaf(
        &mut flow,
        name,
        checksum_of_the_second_location,
    );
    flow
}

/// [`record_new_pool_with_the_second_copy_of_the_inode_leaf_holding_the_inode_root_bytes`] 在一条录好的新池新建文件流上做的那一步：
/// 流里最后那片 inode 叶容器盘序大的那一份换成最后那份 inode 树根（码 2）的字节，位置条目照 `checksum_of_the_second_location`。
fn hold_the_inode_root_bytes_in_the_second_copy_of_the_inode_leaf(
    flow: &mut RecordedFlow,
    name: &'static str,
    checksum_of_the_second_location: ChecksumOfTheSecondLocation,
) {
    let leaf_copies = last_unit_copies(flow, name, &is_an_inode_leaf_container, false);
    let second_leaf_copy =
        if flow.writes[leaf_copies[0]].device.0 < flow.writes[leaf_copies[1]].device.0 {
            leaf_copies[1]
        } else {
            leaf_copies[0]
        };
    let root_copies = last_unit_copies(
        flow,
        name,
        &is_the_inode_tree_root_over_leaf_containers,
        false,
    );
    assert_the_first_entry_names_the_unit_of(
        flow,
        root_copies[0],
        &leaf_copies,
        "inode 树根第一条条目指着流里最后写的那片叶容器",
    );
    let node_bytes = flow.writes[root_copies[0]]
        .bytes()
        .expect("单元写带字节")
        .to_vec();
    assert_eq!(
        unit_class(&node_bytes),
        2,
        "换进去的那 16 KiB 头里的类是码 2"
    );
    let second_location = first_entry_of_a_container_number_node()
        + INODE_INTERNAL_ENTRY_CHILD_POINTER_OFFSET
        + POINTER_LOCATION_ENTRY_OFFSETS[1];
    assert_eq!(
        device_and_slot_of_a_pointer_location(
            &flow.writes[root_copies[0]].bytes().expect("单元写带字节")
                [first_entry_of_a_container_number_node()
                    + INODE_INTERNAL_ENTRY_CHILD_POINTER_OFFSET..],
            POINTER_LOCATION_ENTRY_OFFSETS[1],
        )
        .0,
        flow.writes[second_leaf_copy].device.0,
        "第 1 条位置条目是盘序大的那块盘"
    );
    match checksum_of_the_second_location {
        ChecksumOfTheSecondLocation::MatchesTheNodeBytes => {
            let checksum_of_the_node_bytes = crc32c(&node_bytes);
            corrupt_units_and_reseal_up_to_the_root(flow, &root_copies, &|bytes| {
                let field = second_location + LOCATION_ENTRY_UNIT_CHECKSUM_OFFSET;
                bytes[field..field + 4].copy_from_slice(&checksum_of_the_node_bytes.to_le_bytes());
            });
        }
        ChecksumOfTheSecondLocation::LeftAsTheLeafChecksum => {}
    }
    let WrittenContents::Bytes(leaf) = &mut flow.writes[second_leaf_copy].contents else {
        panic!("单元写是普通写");
    };
    leaf[..node_bytes.len()].copy_from_slice(&node_bytes);
}

/// 第 28 批 H-V（判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第三节第 2 条；攻方 `research/prompts/m3-batch26-code-r1-opus-output.md`
/// 第 1.3 节那一形，条目不改、自证成另一类的那份在盘序大的那一侧）：inode 树根第一条条目类型段 2，叶容器盘 0 那一份是码 3、自证得过，
/// 盘 1 那一份是码 2、按改过的位置条目也自证得过。CPU 两份都看、认出码 2，红 I-9.2（改之前在第一份自证过的副本处停、交回 None，I-9.2 报成立）；
/// GPU `w_child_unit_class_other_than_the_entry_type_names` 同改，每个状态逐位相同。这条流也走 GPU 第二份副本那一支自证得过的那一侧。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_second_copy_of_an_inode_leaf_verifies_as_a_node() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_the_second_copy_of_an_inode_leaf_verifies_as_a_node",
        &record_new_pool_with_the_second_copy_of_the_inode_leaf_holding_the_inode_root_bytes(
            "crash-judge-gpu-i92-inode-leaf-second-copy-verifying-as-a-node",
            "new-pool-inode-leaf-second-copy-verifying-as-a-node",
            ChecksumOfTheSecondLocation::MatchesTheNodeBytes,
        ),
        &["I-9.2"],
    );
}

/// 第 28 批正推线索①（判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第三节第 3 条「GPU 第二份副本那一支补对拍流与变异」）：
/// 同上一条，位置条目不改：盘 1 那一份头里是码 2、按叶容器的整单元校验和自证不过（陈旧副本）。CPU 不把它当另一类，照类型段 2 读盘 0 那一份，
/// I-9.2 不红，那一份对不上校验和、I-2.1 红；GPU 第二份副本那一支也要自证（只认头里的类的话 GPU 红 I-9.2、CPU 不红），每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_second_copy_of_an_inode_leaf_is_a_stale_node_that_does_not_verify(
) {
    let test_name = "the_gpu_judge_agrees_with_the_cpu_when_the_second_copy_of_an_inode_leaf_is_a_stale_node_that_does_not_verify";
    let flow = record_new_pool_with_the_second_copy_of_the_inode_leaf_holding_the_inode_root_bytes(
        "crash-judge-gpu-i92-inode-leaf-second-copy-stale-node",
        "new-pool-inode-leaf-second-copy-stale-node",
        ChecksumOfTheSecondLocation::LeftAsTheLeafChecksum,
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-2.1"),
        "流 {} 上 CPU 池级 checker 要红 I-2.1（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        !comparison.cpu_checker_red_invariants.contains("I-9.2"),
        "流 {} 上自证不过的第二份不算另一类：CPU 池级 checker 不红 I-9.2（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

// ───────────────── 第 30 批：H-V 子树（F-W）、实例表链上每一片判 I-1.2 的出生身份那一半（F-IT） ─────────────────

/// 叶容器第一条记录（新池新建文件那一版只有它一条）的 blocks 加 1：size 3000 要 6，写成 7，I-9.15 红。
fn add_one_to_the_blocks_of_the_first_record_of_an_inode_leaf(bytes: &mut [u8]) {
    let blocks_offset = PACKED_UNIT_RECORDS_START + INODE_RECORD_BLOCKS_OFFSET;
    let blocks = read_u64_at(bytes, blocks_offset);
    bytes[blocks_offset..blocks_offset + 8].copy_from_slice(&(blocks + 1).to_le_bytes());
}

/// 第 30 批 F-W（判决 `research/prompts/m3-batch28-code-r1-main-verification.md` 第二节「K2 H-V 两份都看之后判错」那一行、第三节第 1 条；
/// harness 坏镜像 `an_inode_leaf_with_blocks_off_the_logical_length_beside_a_copy_verifying_as_a_node_reddens_the_blocks_invariant_on_either_device`
/// 同一形，录成流）：新池新建文件的流上 inode 叶容器两份第一条记录的 blocks 加 1（重封、沿引用链补到根），
/// 再照 [`hold_the_inode_root_bytes_in_the_second_copy_of_the_inode_leaf`] 把盘序大的那一份换成 inode 树根的码 2 字节、位置条目跟着换。
fn record_new_pool_with_blocks_off_in_the_inode_leaf_and_its_second_copy_holding_the_inode_root_bytes(
    tag: &str,
) -> RecordedFlow {
    let name = "new-pool-inode-leaf-blocks-off-second-copy-verifying-as-a-node";
    let mut flow = record_new_pool_file_creation(tag, name);
    let leaf_copies = last_unit_copies(&flow, name, &is_an_inode_leaf_container, false);
    corrupt_units_and_reseal_up_to_the_root(
        &mut flow,
        &leaf_copies,
        &add_one_to_the_blocks_of_the_first_record_of_an_inode_leaf,
    );
    hold_the_inode_root_bytes_in_the_second_copy_of_the_inode_leaf(
        &mut flow,
        name,
        ChecksumOfTheSecondLocation::MatchesTheNodeBytes,
    );
    flow
}

/// F-W 那一形：叶容器盘 0 那一份是码 3、带 blocks 的违例、自证得过，盘 1 那一份自证成码 2。CPU 认出码 2、I-9.2 红，另一份按类型段那一类自证过，
/// 照类型段 2 那一支走盘 0 那一份、判出 I-9.15（改之前见别类就不往下走，I-9.15 报不适用）；槽数按类型段那一类登记，不连带 I-3.1、I-3.11、I-5.1。
/// GPU 走读内核 `w_walk_inode_root` 同改（`w_some_copy_of_the_child_verifies_as_the_class_the_entry_type_names`），每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_leaf_copy_of_the_entry_type_carries_a_violation_beside_a_copy_verifying_as_a_node(
) {
    let test_name = "the_gpu_judge_agrees_with_the_cpu_when_a_leaf_copy_of_the_entry_type_carries_a_violation_beside_a_copy_verifying_as_a_node";
    let flow = record_new_pool_with_blocks_off_in_the_inode_leaf_and_its_second_copy_holding_the_inode_root_bytes(
        "crash-judge-gpu-i92-inode-leaf-blocks-off-second-copy-verifying-as-a-node",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    for invariant in ["I-9.2", "I-9.15"] {
        assert!(
            comparison.cpu_checker_red_invariants.contains(invariant),
            "流 {} 上 CPU 池级 checker 要红 {invariant}（红过的：{:?}）",
            flow.name,
            comparison.cpu_checker_red_invariants
        );
    }
    for invariant_a_wrong_slot_count_drags_in in ["I-3.1", "I-3.11", "I-5.1"] {
        assert!(
            !comparison
                .cpu_checker_red_invariants
                .contains(invariant_a_wrong_slot_count_drags_in),
            "流 {} 上槽数按类型段那一类登记：CPU 池级 checker 不红 {invariant_a_wrong_slot_count_drags_in}（红过的：{:?}）",
            flow.name,
            comparison.cpu_checker_red_invariants
        );
    }
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// 码 2 / 码 3 指针头里出生身份的三段（D19（块指针的结构与宽度预算） 已定项 11 的偏移表）：诞生代号 8 字节在 42、写序实例 4 字节在 78、
/// 尾段（码 2 / 码 3 是出生序号）4 字节在 82。
#[derive(Clone, Copy, Debug)]
enum BirthIdentityFieldOfANodePointer {
    BirthTxg,
    Instance,
    Tail,
}

impl BirthIdentityFieldOfANodePointer {
    fn offset_in_the_pointer(self) -> usize {
        match self {
            BirthIdentityFieldOfANodePointer::BirthTxg => 42,
            BirthIdentityFieldOfANodePointer::Instance => 78,
            BirthIdentityFieldOfANodePointer::Tail => 82,
        }
    }
}

/// 攻方 F-IT 那三形（`research/prompts/m3-batch28-code-r1-opus-output.md` 第 4.2 节）：新池新建文件的流上，发布的根记录里实例表指针头部出生身份的
/// `field` 那一段最低字节加 1、重封根记录，发布那几条 journal 记录新根段里同一条指针的同一段跟着改、重封（新根段与同代的根逐字节相同，I-8.10 不红）。
/// 实例表那一片头里的诞生代号、写序实例、出生序号都是 0：指针记的不再与它相同。
fn record_new_pool_with_the_instance_table_pointer_birth_identity_off_by_one_in(
    tag: &str,
    name: &'static str,
    field: BirthIdentityFieldOfANodePointer,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, name);
    let field_offset = field.offset_in_the_pointer();
    let publish_root = flow.judged_root_index;
    rewrite_root_record_and_reseal(&mut flow, publish_root, &|bytes| {
        let offset = ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + field_offset;
        bytes[offset] = bytes[offset].wrapping_add(1);
    });
    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
        let offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET
            + NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER
            + field_offset;
        bytes[offset] = bytes[offset].wrapping_add(1);
    });
    flow
}

/// F-IT 的判法（判决 `research/prompts/m3-batch28-code-r1-main-verification.md` 第二节「实例表链上的 I-1.2」那一行、第三节第 2 条）：CPU 池级 checker
/// 走实例表链每一片都判指针与头的出生身份（`walk.rs` `walk_instance_table_chain`），这条流上只红 I-1.2；GPU 走读内核
/// `w_walk_instance_table_chain` 每一片同判（改之前链循环不调 `w_judge_birth_identity`，I-1.2 报成立），每个状态逐位相同。
fn assert_the_gpu_judge_agrees_with_the_cpu_on_an_instance_table_pointer_whose_birth_identity_is_off_by_one_in(
    test_name: &str,
    tag: &str,
    name: &'static str,
    field: BirthIdentityFieldOfANodePointer,
) {
    let flow = record_new_pool_with_the_instance_table_pointer_birth_identity_off_by_one_in(
        tag, name, field,
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert_eq!(
        comparison.cpu_checker_red_invariants,
        std::collections::BTreeSet::from(["I-1.2"]),
        "流 {} 上 CPU 池级 checker 只红 I-1.2（实例表指针头部 {field:?} 那一段与那一片头里的不符）",
        flow.name
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_birth_txg_than_its_page(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_an_instance_table_pointer_whose_birth_identity_is_off_by_one_in(
        "the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_birth_txg_than_its_page",
        "crash-judge-gpu-i12-instance-table-pointer-birth-txg",
        "new-pool-instance-table-pointer-birth-txg-plus-one",
        BirthIdentityFieldOfANodePointer::BirthTxg,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_instance_than_its_page(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_an_instance_table_pointer_whose_birth_identity_is_off_by_one_in(
        "the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_instance_than_its_page",
        "crash-judge-gpu-i12-instance-table-pointer-instance",
        "new-pool-instance-table-pointer-instance-plus-one",
        BirthIdentityFieldOfANodePointer::Instance,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_birth_sequence_than_its_page(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_an_instance_table_pointer_whose_birth_identity_is_off_by_one_in(
        "the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_birth_sequence_than_its_page",
        "crash-judge-gpu-i12-instance-table-pointer-birth-sequence",
        "new-pool-instance-table-pointer-birth-sequence-plus-one",
        BirthIdentityFieldOfANodePointer::Tail,
    );
}

/// GPU 关门那一侧（判决 `research/prompts/m3-batch25-code-r1-main-verification.md` 第三节第 2 条）：最新根的实例表照 `change` 改成主走读走不到末片的样子，
/// 门要关。CPU 一侧先钉流造对了——改之前整条落盘的镜像上 I-9.29 真被评估过且成立；改之后整条落盘的镜像上 I-9.29 报部分没判。
/// 流上 C 还是最新根的那几个状态 CPU 红 I-9.29；GPU 每个状态逐位相同（实例 3 的根落了、C 写回了的状态上，GPU 门要是开了，
/// 候选集里没有实例 2 那一行、滤不掉 C，GPU 红 I-9.29 而 CPU 不红）。
fn assert_the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_newest_instance_table_stops_before_its_last_page(
    test_name: &str,
    tag: &str,
    change: ChangeOfTheNewestInstanceTable,
) {
    let (flow, image_with_the_newest_instance_table_readable) =
        record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed(
            tag, change,
        );
    assert_eq!(
        pool_checker_verdict_on_the_image(&image_with_the_newest_instance_table_readable, "I-9.29"),
        InvariantVerdict::Holds,
        "改实例表之前：被抛弃的 C 按最新根的实例表滤掉，I-9.29 真被评估过且成立"
    );
    let mut image_with_every_write = flow.base.clone();
    image_with_every_write.apply_writes(&flow.writes);
    match pool_checker_verdict_on_the_image(&image_with_every_write, "I-9.29") {
        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
            unjudged.starts_with("最新根的实例表读不出"),
            "部分没判的说明点名最新根的实例表读不出：{unjudged}"
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::NotApplicable(_)) => panic!(
            "流 {}：主走读沿最新根的实例表链走不到末片，I-9.29 要报部分没判，实际 {other_verdict:?}",
            flow.name
        ),
    }
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-9.29"),
        "流 {} 上 C 还是最新根的状态 CPU 红 I-9.29（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// 链指针记录不合法那一形（[`ChangeOfTheNewestInstanceTable::ChainRecordSayingANextPageWithoutAPointerAndNoRowOfTheAbandonedInstance`]）：
/// GPU 走读内核只在链指针记录说「没有下一片」时置「走到末片」（`chain == 0u`），把不合法的也当末片时这条流红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_chain_record_of_the_newest_instance_table_is_invalid(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_newest_instance_table_stops_before_its_last_page(
        "the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_chain_record_of_the_newest_instance_table_is_invalid",
        "crash-judge-gpu-i929-newest-instance-table-invalid-chain-record",
        ChangeOfTheNewestInstanceTable::ChainRecordSayingANextPageWithoutAPointerAndNoRowOfTheAbandonedInstance,
    );
}

/// 单片读不出那一形（[`ChangeOfTheNewestInstanceTable::SecondPageHoldingTheRowOfTheAbandonedInstanceUnreadable`]）：第 0 片读得出、第 1 片两份都读不出。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_second_page_of_the_newest_instance_table_does_not_read_back(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_newest_instance_table_stops_before_its_last_page(
        "the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_second_page_of_the_newest_instance_table_does_not_read_back",
        "crash-judge-gpu-i929-newest-instance-table-second-page-unreadable",
        ChangeOfTheNewestInstanceTable::SecondPageHoldingTheRowOfTheAbandonedInstanceUnreadable,
    );
}

/// 攻方 H-c 那一形的流，录 `pages` 片（[`ChangeOfTheNewestInstanceTable::SpreadOverPagesWithoutTheRowOfTheAbandonedInstance`]），
/// CPU 一侧先钉流造对了：整条落盘的镜像上主走读走到第 `pages` 片（末片）、门开，C 滤不掉，I-9.29 红。
fn record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_spread_over_pages(
    tag: &str,
    pages: InstanceTableChainPages,
) -> RecordedFlow {
    let (flow, _) =
        record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed(
            tag,
            ChangeOfTheNewestInstanceTable::SpreadOverPagesWithoutTheRowOfTheAbandonedInstance(
                pages,
            ),
        );
    let mut image_with_every_write = flow.base.clone();
    image_with_every_write.apply_writes(&flow.writes);
    assert!(
        matches!(
            pool_checker_verdict_on_the_image(&image_with_every_write, "I-9.29"),
            InvariantVerdict::Violated(_)
        ),
        "流 {}：CPU 主走读沿 {} 片的链走到末片、候选集里没有实例 2 那一行，被抛弃的 C 与现行线同号不同出生代，I-9.29 红：{:?}",
        flow.name,
        pages.count(),
        pool_checker_verdict_on_the_image(&image_with_every_write, "I-9.29")
    );
    flow
}

/// H-c 对照那一形：链恰好 256 片（GPU 走读内核走得完的片数），两边都走到末片、都比，每个状态逐位相同、一个状态都不判不了。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_chain_of_as_many_pages_as_the_gpu_walks(
) {
    let flow =
        record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_spread_over_pages(
            "crash-judge-gpu-i929-newest-instance-table-of-256-pages",
            InstanceTableChainPages::AsManyAsTheGpuWalks,
        );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_chain_of_as_many_pages_as_the_gpu_walks",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-9.29"),
        "流 {} 上 CPU 红 I-9.29（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致或判不了（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// H-c（判决 `research/prompts/m3-batch25-code-r1-main-verification.md` 第二节 H-c 那一行与第三节第 1 条）：链 257 片，比 GPU 走读内核走得完的多一片。
/// CPU 走到末片、比、红 I-9.29；GPU 走读内核走满上限链还没完，池级 checker 记判不了、原因字是实例表链过长
/// （[`GPU_UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED`]），不当没走到末片（当成没走到末片的话门关、不比，GPU 不红 I-9.29 而 CPU 红）。
/// 逐状态：池级 checker 判了的，违例位与 CPU 逐位相同；判不了的，原因字只能是实例表链过长，且至少一个这样的状态上 CPU 红 I-9.29；
/// 恢复内核判不了的，原因只能是沿链读满 256 片还有下一片（[`GPU_RECOVERY_REASONS_LEFT_TO_THE_CPU_ON_AN_INSTANCE_TABLE_CHAIN_OF_257_PAGES`]，
/// 恢复落到那条最新根的状态）；照流水线那样判不了就回落 CPU（`gpu_verdict_items` 交回 None），回落之后每个状态的红项都与 CPU 相同。
#[test]
fn the_gpu_judge_leaves_to_the_cpu_a_newest_instance_table_chain_longer_than_the_pages_it_walks() {
    let test_name =
        "the_gpu_judge_leaves_to_the_cpu_a_newest_instance_table_chain_longer_than_the_pages_it_walks";
    let flow =
        record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_spread_over_pages(
            "crash-judge-gpu-i929-newest-instance-table-of-257-pages",
            InstanceTableChainPages::OneMoreThanTheGpuWalks,
        );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    let mut states_left_to_the_cpu_by_the_chain_limit = 0u64;
    let mut one_birth_red_on_a_state_left_to_the_cpu_by_the_chain_limit = false;
    let mut disagreements: Vec<String> = Vec::new();
    for (ordinal, gpu_words) in comparison.gpu_words_of_each_state.iter().enumerate() {
        let cpu_checker_bits = comparison.cpu_checker_bits_of_each_state[ordinal];
        let cpu_red_items =
            &comparison.cpu_red_items_compared_by_the_cross_check_of_each_state[ordinal];
        match gpu_words[POOL_CHECKER_UNDECIDABLE_WORD] {
            0 => {
                let gpu_checker_bits = pool_checker_violated_bits(gpu_words);
                if !pool_checker_bits_agree_in_the_cross_check(gpu_checker_bits, cpu_checker_bits) {
                    disagreements.push(format!(
                        "状态 {ordinal}：GPU 池级 checker 判了，违例 {:?}；CPU 违例 {:?}",
                        invariant_names(gpu_checker_bits),
                        invariant_names(cpu_checker_bits)
                    ));
                }
            }
            GPU_UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED => {
                states_left_to_the_cpu_by_the_chain_limit += 1;
                if invariant_names(cpu_checker_bits).contains(&"I-9.29") {
                    one_birth_red_on_a_state_left_to_the_cpu_by_the_chain_limit = true;
                }
            }
            other_reason => disagreements.push(format!(
                "状态 {ordinal}：GPU 池级 checker 判不了的原因字 {other_reason:#x} 不是实例表链过长"
            )),
        }
        // 恢复落到链 257 片那条最新根的状态上恢复内核沿链读满 256 片还有下一片，按设计交 CPU（E9 那一趟这条流 28 个状态记 4）
        disagreements.extend(
            recovery_kernel_left_the_state_to_the_cpu_for_an_unlisted_reason(
                ordinal,
                gpu_words,
                GPU_RECOVERY_REASONS_LEFT_TO_THE_CPU_ON_AN_INSTANCE_TABLE_CHAIN_OF_257_PAGES,
            ),
        );
        // 内核不置位的那几条（第一版是 I-1.13）两边都去掉再比（`red_items_compared_by_the_cross_check`）
        let concluded = gpu_verdict_items(gpu_words).map_or_else(
            || cpu_red_items.clone(),
            |items| red_items_compared_by_the_cross_check(&items),
        );
        if concluded != *cpu_red_items {
            disagreements.push(format!(
                "状态 {ordinal}：判不了就回落 CPU 之后的红项 {concluded} 与 CPU 的 {cpu_red_items} 不同"
            ));
        }
    }
    eprintln!(
        "CRASH_JUDGE_GPU_CHAIN_LIMIT flow={} states={} left_to_the_cpu_by_the_chain_limit={states_left_to_the_cpu_by_the_chain_limit}",
        flow.name, comparison.states
    );
    assert!(
        states_left_to_the_cpu_by_the_chain_limit > 0,
        "流 {}：最新根的实例表链 257 片的状态上 GPU 池级 checker 要记判不了（实例表链过长）",
        flow.name
    );
    assert!(
        one_birth_red_on_a_state_left_to_the_cpu_by_the_chain_limit,
        "流 {}：GPU 因链过长判不了的状态里，至少一个 CPU 红 I-9.29（CPU 走到末片、比）",
        flow.name
    );
    assert!(
        disagreements.is_empty(),
        "流 {}：{} 处不一致（前几个）：\n{}",
        flow.name,
        disagreements.len(),
        disagreements
            .iter()
            .take(6)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

// ─── 第 35 批：由记录造出来的两类版本取带末条标志那条记录的新根段（FL）、树表指针全零照 core 跟过去（Q1、Q3） ───

/// 新根段七段在记录与根记录里的偏移（新根段从记录偏移 99 起；D23（journal 的角色与格式） 已定项 15 / 已定项 4 的字段表，与 walk.rs 的
/// `NEW_ROOT_SEGMENT_FIELDS_COMPARED_WITH_THE_ROOT` 同一张表）：(新根段里的起点, 根记录里的起点, 字节数)。
const NEW_ROOT_SEGMENT_FIELDS_IN_THE_ROOT_RECORD: [(usize, usize, usize); 7] = [
    (0, 36, 86),
    (86, 256, 86),
    (172, 122, 8),
    (180, 130, 8),
    (188, 170, 86),
    (274, 342, 86),
    (360, 20, 4),
];

/// journal 记录头里实例代号（4 字节）、计数器（6 字节）、`checkpoint_txg`（8 字节）的偏移（D23（journal 的角色与格式） 已定项 17 的字段表）。
const JOURNAL_RECORD_INSTANCE_OFFSET: usize = 16;
const JOURNAL_RECORD_COUNTER_OFFSET: usize = 20;
const JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET: usize = 26;
/// 码 2 节点头里诞生代号那 8 字节的偏移扣掉 2k（D18（块里携带什么信息） 已定项 7：诞生代号 52 + 2k）。
const INDEX_NODE_BIRTH_TXG_OFFSET_BEFORE_THE_KEYS: usize = 52;

/// 流里 (实例 1, `checkpoint_txg`) 那次发布的每一份 journal 记录写：(写下标, 计数器)。
fn journal_record_writes_of_the_publish(
    flow: &RecordedFlow,
    checkpoint_txg: u64,
) -> Vec<(usize, u64)> {
    flow.writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::JournalRecord)
        .filter_map(|(index, write)| {
            let bytes = write.bytes()?;
            let instance = u32::from_le_bytes(
                bytes[JOURNAL_RECORD_INSTANCE_OFFSET..JOURNAL_RECORD_INSTANCE_OFFSET + 4]
                    .try_into()
                    .expect("4"),
            );
            let txg = u64::from_le_bytes(
                bytes[JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET
                    ..JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET + 8]
                    .try_into()
                    .expect("8"),
            );
            let mut counter = [0u8; 8];
            counter[..6].copy_from_slice(
                &bytes[JOURNAL_RECORD_COUNTER_OFFSET..JOURNAL_RECORD_COUNTER_OFFSET + 6],
            );
            (instance == 1 && txg == checkpoint_txg).then_some((index, u64::from_le_bytes(counter)))
        })
        .collect()
}

/// 三个数据单元那条流里第一个文件顺序写那次发布（实例 1、txg 4，切成三条记录）的**第一条**记录（两块盘各一份）新根段七段换成
/// txg 3 那条根的（写侧违反 D23（journal 的角色与格式） 已定项 15「每条记录都带同一份新根段」的样子），重封、同实例后面的记录重链。
fn write_the_root_of_the_first_file_into_the_first_record_of_the_three_unit_publish(
    flow: &mut RecordedFlow,
) {
    let root_of_the_first_file: Vec<u8> = flow
        .writes
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
        .filter_map(RetainedWrite::bytes)
        .find(|bytes| {
            u32::from_le_bytes(bytes[24..28].try_into().expect("4")) == 1
                && u64::from_le_bytes(bytes[28..36].try_into().expect("8")) == 3
        })
        .expect("流里有第一个文件那条根 (1, 3) 的根槽写")
        .to_vec();
    let records = journal_record_writes_of_the_publish(flow, 4);
    let first_counter = records
        .iter()
        .map(|(_, counter)| *counter)
        .min()
        .expect("顺序写那次发布有记录");
    assert_eq!(
        records.len(),
        6,
        "流 {}：顺序写那次发布切成三条记录、两块盘各一份",
        flow.name
    );
    for (index, _) in records
        .iter()
        .filter(|(_, counter)| *counter == first_counter)
    {
        rewrite_journal_record_and_reseal(flow, *index, &|bytes| {
            for (in_the_segment, in_the_root, length) in NEW_ROOT_SEGMENT_FIELDS_IN_THE_ROOT_RECORD
            {
                let segment = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + in_the_segment;
                bytes[segment..segment + length]
                    .copy_from_slice(&root_of_the_first_file[in_the_root..in_the_root + length]);
            }
        });
    }
}

/// 一片分配记录叶由 txg 4 那次发布写出（三个数据单元那条流上是顺序写那次，两次单条覆盖写那条流上是第一次覆盖写）、
/// 且里面有一对改得动的记录（[`widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts`]）。
fn is_an_allocation_leaf_born_at_txg_four_that_can_be_widened(bytes: &[u8]) -> bool {
    if unit_class(bytes) != 2 || index_node_key_width(bytes) != 10 || bytes[50] != 0 {
        return false;
    }
    let birth_txg_offset = INDEX_NODE_BIRTH_TXG_OFFSET_BEFORE_THE_KEYS + 2 * 10;
    u64::from_le_bytes(
        bytes[birth_txg_offset..birth_txg_offset + 8]
            .try_into()
            .expect("8"),
    ) == 4
        && widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(
            &mut bytes.to_vec(),
        ) == 1
}

/// 改法 FL 下一次挂载那一支（判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第二节 P1-b，攻方报告的 s3 形）：
/// 三个数据单元那条流上，顺序写那次发布（txg 4，三条记录）写的分配记录叶里两条记录罩住同一个槽（重封到根与三条记录），
/// 再把第一条记录的新根段写成 txg 3 那条根的。三条记录都落了、根槽没落的状态上，CPU 认下一次挂载先施加的那一版取重放第一次施加的发布的末条，
/// 红 I-5.4；GPU 扫描内核 `record_row_of_the_version_the_next_mount_applies_first` 读走读内核同一份重放交来的那一条、红在同样的状态上
/// （取第一条就走 txg 3 的树、不红）。
fn record_three_unit_file_with_a_stale_first_record_over_two_allocation_records_covering_one_slot(
    tag: &str,
) -> RecordedFlow {
    let name = "three-unit-file-stale-first-record-over-overlapping-allocation-records";
    let mut flow = record_three_unit_file_with_a_resealed_field(
        tag,
        name,
        &is_an_allocation_leaf_born_at_txg_four_that_can_be_widened,
        &|bytes| {
            assert_eq!(
                widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(bytes),
                1,
                "挑叶时这一片改得动"
            );
        },
    );
    write_the_root_of_the_first_file_into_the_first_record_of_the_three_unit_publish(&mut flow);
    flow
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_first_record_of_the_next_mount_version_carries_the_previous_root_over_two_allocation_records_covering_one_slot(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_first_record_of_the_next_mount_version_carries_the_previous_root_over_two_allocation_records_covering_one_slot",
        &record_three_unit_file_with_a_stale_first_record_over_two_allocation_records_covering_one_slot(
            "crash-judge-gpu-fl-next-mount-version",
        ),
        "I-5.4",
    );
}

/// 三个数据单元的第一个文件那段历史（`pool_with_a_three_unit_file`：新池新建文件 txg 3、顺序写三个数据单元那次发布 txg 4、三条记录）
/// 录到 txg 4 的根槽写之前停——崩在那次根槽 FUA 之前，三条记录两块盘都落了。被判的根是 (1, 3) 那一次根槽写；版本是 txg 3、txg 4 两版
/// （txg 4 那一版只在三条记录里，记录全落的状态上恢复由记录施加它）。
fn record_three_unit_file_up_to_the_root_of_its_three_record_publish(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let pool = pool_with_a_three_unit_file(tag);
    let operations = pool.retained_operations();
    let classifier = geometry();
    let operations_after_mkfs = &operations[pool.mkfs_operation_count..];
    let root_of_the_three_unit_publish = operations_after_mkfs
        .iter()
        .position(|retained| {
            classifier.classify(&retained.operation) == StepKind::RootRecordFua
                && retained.contents.as_ref().is_some_and(|bytes| {
                    u32::from_le_bytes(bytes[24..28].try_into().expect("4")) == 1
                        && u64::from_le_bytes(bytes[28..36].try_into().expect("8")) == 4
                })
        })
        .expect("流里有顺序写那次发布 (1, 4) 的根槽写");
    let (writes, segments, _stream_indexes_before_the_crash) =
        writes_and_segments_with_stream_indexes(
            &operations_after_mkfs[..root_of_the_three_unit_publish],
            &classifier,
        );
    RecordedFlow {
        name,
        base: pool.memory_pool_after_mkfs(),
        judged_root_index: writes
            .iter()
            .rposition(|write| write.kind == StepKind::RootRecordFua)
            .expect("顺序写之前有新建文件那次发布 (1, 3) 的根槽写"),
        writes,
        segments,
        versions: vec![
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(3),
                content: file_content(),
            },
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(4),
                content: content_of_three_data_units(),
            },
        ],
    }
}

/// [`record_three_unit_file_up_to_the_root_of_its_three_record_publish`] 那一段上，`rewrite_the_records_before_the_mount` 改那次发布的记录。
/// 再从这一段写全落的那张崩溃镜像起新进程（另一条录制流）录可写挂载（施加那三条记录、取号 2、写行、暖机两次）与实例 2 的一次覆盖写，
/// 两段写接成一条流。txg 4 那一版只在它的三条记录里：挂载之前的状态上它是下一次挂载先施加的那一版，挂载写了实例 1 的行之后它是由记录施加出来、
/// 根槽从没落盘的那一版（攻方报告 P1-b 的 s4 形）。挂载读到的就是这条流里那三条记录（判决 `research/prompts/m3-formal-code-b35-r1-main-verification.md`
/// 第二节 P3 与第三节第 1 条 R2：此前录完整段历史再从流里拿掉 txg 4 的根槽写，录制时挂载读到的是 txg 4 的根、什么都没施加，
/// 写出的实例表行（W）与之后的发布、根槽之后的系统配置轮换都与合法历史不同）。
fn record_three_unit_file_applied_only_by_its_records_after_a_mount(
    tag: &str,
    name: &'static str,
    rewrite_the_records_before_the_mount: &dyn Fn(&mut RecordedFlow),
) -> RecordedFlow {
    let mut flow = record_three_unit_file_up_to_the_root_of_its_three_record_publish(tag, name);
    rewrite_the_records_before_the_mount(&mut flow);

    let classifier = geometry();
    let stream_of_the_new_process = SharedStream::retaining_contents();
    let every_write_landed = vec![true; flow.writes.len()];
    let mut devices = common::crash_state_devices(
        &flow.base,
        &flow.writes,
        &every_write_landed,
        &stream_of_the_new_process,
    );
    let mut mounted = singlefs_core::mount::mount_writable(&parameters(), &mut devices)
        .expect("崩在 txg 4 根槽之前的镜像上新进程可写挂载：施加那三条记录、取号 2");
    assert_eq!(
        (
            mounted.output.chosen_root.checkpoint_txg,
            mounted.output.effective_root.checkpoint_txg,
            mounted.output.journal.prefix_applied
        ),
        (CheckpointTxg(3), CheckpointTxg(4), 3),
        "流 {name}：择 (1, 3) 的根，施加 txg 4 的三条记录，现行版是 (1, 4)"
    );
    let current = mounted
        .current
        .into_file_version()
        .expect("施加了 txg 4，现行那一版带文件");
    let overwrite = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut mounted.allocator,
            &current,
            FirstFile {
                content: &third_content(),
                write_time_seconds: common::FIXED_WRITE_TIME_SECONDS + 120,
            },
            InstanceGeneration(2),
        )
        .expect("实例 2 覆盖写")
    };
    let (
        writes_of_the_new_process,
        segments_of_the_new_process,
        _stream_indexes_of_the_new_process,
    ) = writes_and_segments_with_stream_indexes(
        &stream_of_the_new_process.retained_operations(),
        &classifier,
    );
    let first_write_of_the_new_process = flow.writes.len();
    flow.writes.extend(writes_of_the_new_process);
    flow.segments
        .extend(segments_of_the_new_process.into_iter().map(|segment| {
            segment
                .into_iter()
                .map(|index| first_write_of_the_new_process + index)
                .collect()
        }));
    flow.judged_root_index = flow
        .writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("覆盖写那次发布有根槽写");
    let overwrite_txg = overwrite.root.checkpoint_txg.0;
    flow.versions = vec![
        PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
            content: file_content(),
        },
        PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(4),
            content: content_of_three_data_units(),
        },
    ];
    for txg in 5..overwrite_txg {
        flow.versions.push(PublishedVersion {
            instance: InstanceGeneration(2),
            checkpoint_txg: CheckpointTxg(txg),
            content: content_of_three_data_units(),
        });
    }
    flow.versions.push(PublishedVersion {
        instance: InstanceGeneration(2),
        checkpoint_txg: CheckpointTxg(overwrite_txg),
        content: third_content(),
    });
    flow
}

/// 在流尾另起一段，把 `checkpoint_txg` 那次发布（实例 1）的每一份 journal 记录写（两块盘各一份）抄一遍、`rewrite` 改过、重封，
/// 同盘里计数器接着的那一条的反向链按前一条改后的头重算：这一段全落的状态上盘面就是「整段历史写完之后那几条记录被改成这样」。
/// 流里原来那几次记录写不动，挂载录制时读到的仍是它们。
fn append_the_records_of_the_publish_rewritten(
    flow: &mut RecordedFlow,
    checkpoint_txg: u64,
    rewrite: &dyn Fn(&mut [u8]),
) {
    let mut records = journal_record_writes_of_the_publish(flow, checkpoint_txg);
    records.sort_by_key(|(index, counter)| (flow.writes[*index].device, *counter));
    let mut rewritten: Vec<RetainedWrite> = Vec::new();
    let mut previous_on_the_device: Option<(DeviceIdentity, u64, u32)> = None;
    for (index, counter) in records {
        let mut write = flow.writes[index].clone();
        let device = write.device;
        let WrittenContents::Bytes(bytes) = &mut write.contents else {
            panic!("记录写是普通写");
        };
        rewrite(bytes);
        if let Some((_, _, chain)) =
            previous_on_the_device.filter(|(previous_device, previous_counter, _)| {
                *previous_device == device && previous_counter + 1 == counter
            })
        {
            bytes[91..95].copy_from_slice(&chain.to_le_bytes());
        }
        reseal_journal_record(bytes);
        previous_on_the_device = Some((device, counter, journal_chain_value(bytes)));
        rewritten.push(write);
    }
    assert!(
        !rewritten.is_empty(),
        "流 {} 里有 txg {checkpoint_txg} 那次发布的 journal 记录",
        flow.name
    );
    append_a_segment_of_writes(flow, rewritten);
}

/// 改法 FL 由记录施加出来的那一版那一支（攻方报告 P1-b 的 s4 形）：txg 4 只在三条记录里、第一条的新根段写成 txg 3 那条根的。
/// 挂载之后的状态上 txg 4 是由记录施加出来的那一版：CPU 取末条那一条走它自己的树，GPU 走读内核拼 `R_DATA` 那一段同取末条、
/// 每个状态逐位相同（取先到的那一条就走 txg 3 的树，txg 4 自己的单元没走到，I-3.1、I-3.5 在 GPU 一侧红出来）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_version_applied_only_by_its_records_is_walked_from_its_last_record_when_the_first_carries_the_previous_root(
) {
    let flow = record_three_unit_file_applied_only_by_its_records_after_a_mount(
        "crash-judge-gpu-fl-version-applied-only-by-records",
        "three-unit-file-applied-only-by-its-records-stale-first-record",
        &write_the_root_of_the_first_file_into_the_first_record_of_the_three_unit_publish,
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_that_a_version_applied_only_by_its_records_is_walked_from_its_last_record_when_the_first_carries_the_previous_root",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// Q3 甲（用户 2026-10-06 定「甲 同改」）与 Q1 甲下一次挂载那一支：新池新建文件那次发布的根记录与它的记录（两块盘各一份）里
/// 树表指针整条写零、重封。根落了的状态上最新根照 core 跟过去读不出（CPU 红 I-2.1、I-4.8、I-7.2、I-7.4），记录落了、根没落的状态上
/// 重放施加出来的那一版走不完（CPU 红 I-4.1）；GPU 走读内核 `w_walk_tree_table_and_mapping` 同判、每个状态逐位相同。
fn record_new_pool_with_the_tree_table_pointer_of_the_publish_written_all_zero(
    tag: &str,
) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-publish-tree-table-pointer-all-zero");
    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
        bytes
            [JOURNAL_NEW_ROOT_SEGMENT_OFFSET..JOURNAL_NEW_ROOT_SEGMENT_OFFSET + NODE_POINTER_BYTES]
            .fill(0);
    });
    let publish_root = flow.judged_root_index;
    rewrite_root_record_and_reseal(&mut flow, publish_root, &|bytes| {
        bytes[36..36 + NODE_POINTER_BYTES].fill(0);
    });
    flow
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_pointer_of_the_root_and_of_its_record_is_all_zero(
) {
    let flow = record_new_pool_with_the_tree_table_pointer_of_the_publish_written_all_zero(
        "crash-judge-gpu-all-zero-tree-table-pointer",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_pointer_of_the_root_and_of_its_record_is_all_zero",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    for invariant in ["I-2.1", "I-7.2", "I-4.1"] {
        assert!(
            comparison.cpu_checker_red_invariants.contains(invariant),
            "流 {} 上 CPU 池级 checker 要红 {invariant}（红过的：{:?}）",
            flow.name,
            comparison.cpu_checker_red_invariants
        );
    }
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// Q1 甲由记录施加出来的那一版那一支：同上那条挂载之后的流（挂载读的是干净的三条记录），覆盖写之后在流尾另起一段，把 txg 4 的三条记录
/// （两块盘各一份）抄一遍、新根段里的树表指针都写零、重封、重链（挂载之后改记录，`checker_known_bad_images.rs` 同一造法）。
/// 那一段落了的状态上 txg 4 是由记录施加出来的那一版，CPU 照 core 跟过去、这一版走不完（红 I-2.1、I-4.8、I-7.4）；GPU 走读内核同判。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_records_of_a_version_applied_only_by_them_carry_an_all_zero_tree_table_pointer(
) {
    let mut flow = record_three_unit_file_applied_only_by_its_records_after_a_mount(
        "crash-judge-gpu-all-zero-tree-table-pointer-applied-only-by-records",
        "three-unit-file-applied-only-by-its-records-all-zero-tree-table-pointer",
        &|_records_left_as_written| {},
    );
    append_the_records_of_the_publish_rewritten(&mut flow, 4, &|bytes| {
        bytes
            [JOURNAL_NEW_ROOT_SEGMENT_OFFSET..JOURNAL_NEW_ROOT_SEGMENT_OFFSET + NODE_POINTER_BYTES]
            .fill(0);
    });
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_when_the_records_of_a_version_applied_only_by_them_carry_an_all_zero_tree_table_pointer",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    for invariant in ["I-2.1", "I-4.8", "I-7.4"] {
        assert!(
            comparison.cpu_checker_red_invariants.contains(invariant),
            "流 {} 上 CPU 池级 checker 要红 {invariant}（红过的：{:?}）",
            flow.name,
            comparison.cpu_checker_red_invariants
        );
    }
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

// ─── 第 36 批：下一次挂载先施加的那一版照 I-4.1 那一份重放施加的第一次发布认（改法 R1 的 GPU 一半） ───

/// I-5.4 在下一次挂载先施加的那一版上的违例说明的开头（`walk.rs` 按版本那一层的称呼）。
const VIOLATION_ON_THE_VERSION_THE_NEXT_MOUNT_APPLIES_FIRST_AT_TXG_FOUR: &str =
    "下一次挂载先施加的那一版（实例 1、txg 4）";

/// 改法 R1 断号那一形（判决 `research/prompts/m3-formal-code-b35-r1-main-verification.md` 第二节 P1 与第三节第 1 条，攻方
/// `cut=records-2..2-of-2`；harness 档 `checker_known_bad_images.rs` 的
/// `a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first` 同形）：三个数据单元那段历史录到顺序写那次发布
/// （txg 4，计数器 4、5、6 三条记录）的根槽写之前停（崩在那次根槽 FUA 之前，R2 的合法造法），那次发布写的分配记录叶里两条记录罩住同一个槽，
/// 重封进三条记录、后面的记录重链。三条记录两块盘六次写在同一段里、每个子集都展开：首条两份都没落而后两条落了的状态在里面。
fn record_three_unit_file_crashed_before_its_root_over_two_allocation_records_covering_one_slot(
    tag: &str,
) -> RecordedFlow {
    let name = "three-unit-file-crashed-before-its-root-over-overlapping-allocation-records";
    let mut flow = record_three_unit_file_up_to_the_root_of_its_three_record_publish(tag, name);
    let targets = last_unit_copies(
        &flow,
        name,
        &is_an_allocation_leaf_born_at_txg_four_that_can_be_widened,
        false,
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
        assert_eq!(
            widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(bytes),
            1,
            "挑叶时这一片改得动"
        );
    });
    let records = journal_record_writes_of_the_publish(&flow, 4);
    let (first_record_write, _) = *records.first().expect("顺序写那次发布有记录");
    let segment_of_the_records = flow
        .segments
        .iter()
        .find(|segment| segment.contains(&first_record_write))
        .expect("记录写在某一段里");
    assert!(
        records.len() == 6
            && records
                .iter()
                .all(|(index, _)| segment_of_the_records.contains(index))
            && segment_of_the_records.len() <= DEFAULT_EXPAND_UP_TO,
        "流 {name}：txg 4 的三条记录两块盘六次写在同一段里、那一段不多于 {DEFAULT_EXPAND_UP_TO} 次写（每个子集都展开）"
    );
    flow
}

/// 断号那一形（[`record_three_unit_file_crashed_before_its_root_over_two_allocation_records_covering_one_slot`]）。CPU 一侧先钉流造对了：
/// 三条记录都落了的镜像上 txg 4 是下一次挂载先施加的那一版，I-5.4 红在它身上；首条（计数器 4）两份都不落、其余照落的镜像上 core 断号
/// 不施加 txg 4，CPU 照同一份重放不认它，I-5.4 在 txg 3 那一版上判过而成立。GPU 扫描内核读走读内核那一份重放交来的第一次发布，
/// 每个状态逐位相同（只看带末条标志的那一条就认计数器 6，断号的状态上 GPU 红 I-5.4 而 CPU 不红）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first(
) {
    let flow =
        record_three_unit_file_crashed_before_its_root_over_two_allocation_records_covering_one_slot(
            "crash-judge-gpu-r1-first-record-not-landed",
        );
    let mut image_with_every_record_landed = flow.base.clone();
    image_with_every_record_landed.apply_writes(&flow.writes);
    match pool_checker_verdict_on_the_image(&image_with_every_record_landed, "I-5.4") {
        InvariantVerdict::Violated(detail) => assert!(
            detail.starts_with(VIOLATION_ON_THE_VERSION_THE_NEXT_MOUNT_APPLIES_FIRST_AT_TXG_FOUR),
            "三条记录都落了：I-5.4 红在下一次挂载先施加的那一版 txg 4 上：{detail}"
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => panic!(
            "三条记录都落了：txg 4 是下一次挂载先施加的那一版，I-5.4 要红，实际 {other_verdict:?}"
        ),
    }
    let records = journal_record_writes_of_the_publish(&flow, 4);
    let first_counter = records
        .iter()
        .map(|(_, counter)| *counter)
        .min()
        .expect("顺序写那次发布有记录");
    let copies_of_the_first_record: Vec<usize> = records
        .iter()
        .filter(|(_, counter)| *counter == first_counter)
        .map(|(index, _)| *index)
        .collect();
    assert_eq!(copies_of_the_first_record.len(), 2, "首条两块盘各一份");
    let writes_without_the_first_record: Vec<RetainedWrite> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(index, _)| !copies_of_the_first_record.contains(index))
        .map(|(_, write)| write.clone())
        .collect();
    let mut image_with_the_first_record_not_landed = flow.base.clone();
    image_with_the_first_record_not_landed.apply_writes(&writes_without_the_first_record);
    assert_eq!(
        pool_checker_verdict_on_the_image(&image_with_the_first_record_not_landed, "I-5.4"),
        InvariantVerdict::Holds,
        "首条两份都没落、后两条落了：core 断号不施加 txg 4，CPU 不认它，I-5.4 在 txg 3 那一版上判过而成立"
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-5.4"),
        "流 {} 上 CPU 池级 checker 要红 I-5.4（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// 流里 (实例 1, `checkpoint_txg`) 那条根的最后一次根槽写，抄一份、根记录树表指针那一段里翻一个字节（不重封：那个根槽读不出）。
fn torn_copy_of_the_root_write_for_txg(flow: &RecordedFlow, checkpoint_txg: u64) -> RetainedWrite {
    let mut torn = flow
        .writes
        .iter()
        .rev()
        .find(|write| {
            write.kind == StepKind::RootRecordFua
                && write.bytes().is_some_and(|bytes| {
                    u32::from_le_bytes(bytes[24..28].try_into().expect("4")) == 1
                        && u64::from_le_bytes(
                            bytes[ROOT_RECORD_CHECKPOINT_TXG_OFFSET
                                ..ROOT_RECORD_CHECKPOINT_TXG_OFFSET + 8]
                                .try_into()
                                .expect("8"),
                        ) == checkpoint_txg
                })
        })
        .unwrap_or_else(|| panic!("流 {} 里有 (1, {checkpoint_txg}) 的根槽写", flow.name))
        .clone();
    let WrittenContents::Bytes(bytes) = &mut torn.contents else {
        panic!("根槽写是普通写");
    };
    let flipped = usize::try_from(common::BYTE_INSIDE_THE_ROOT_RECORDS_TREE_TABLE_POINTER)
        .expect("根记录里的下标");
    bytes[flipped] ^= 0xff;
    torn
}

/// 改法 R1 的另一半（harness 档
/// `a_replay_stopped_by_a_named_unit_of_a_later_publish_still_takes_the_first_publish_as_the_version_the_next_mount_applies_first` 同形）：
/// 新池新建文件（A，txg 3）之后同一个进程、实例 1 里再覆盖写两次：B (1, 4)、C (1, 5)，各一条记录（计数器 4、5）；整段历史照录。
/// B 写的分配记录叶里两条记录罩住同一个槽，重封到 B 的根与记录、C 的记录重链。流尾另起一段（R2 改记录那一条流的造法：之前的写不动）：
/// B、C 两次的根槽各翻一个字节（两个根槽都读不出，与 harness 档那一形的 `tear_the_root_slot_written_for_txg` 同一处），
/// C 那条记录第一个点名单元第二份落点上那一份翻一个字节、不重封（介质损坏）。这一段全落的状态上 core 择 A 的根、施加 B、停在 C 之前。
fn record_two_single_record_publishes_stopped_by_a_named_unit_of_the_second(
    tag: &str,
) -> RecordedFlow {
    let name =
        "two-single-record-publishes-only-in-their-records-stopped-by-a-named-unit-of-the-second";
    let mut pool = build_pool(tag);
    common::overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &second_content());
    common::overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &third_content());
    let mut flow = recorded_flow_of_the_pool(
        &pool,
        name,
        vec![
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(3),
                content: file_content(),
            },
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(4),
                content: second_content(),
            },
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(5),
                content: third_content(),
            },
        ],
    );
    let targets = last_unit_copies(
        &flow,
        name,
        &is_an_allocation_leaf_born_at_txg_four_that_can_be_widened,
        false,
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
        assert_eq!(
            widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(bytes),
            1,
            "挑叶时这一片改得动"
        );
    });
    let (record_of_the_second_publish, _) = *journal_record_writes_of_the_publish(&flow, 5)
        .first()
        .expect("C 有记录");
    let record_bytes = flow.writes[record_of_the_second_publish]
        .bytes()
        .expect("记录写带字节");
    assert!(
        u32::from_le_bytes(record_bytes[12..16].try_into().expect("4")) > 0,
        "C 那条记录有点名项"
    );
    let second_location = JOURNAL_RECORD_HEADER_BYTES + 14;
    let named_device = u32::from_le_bytes(
        record_bytes[second_location..second_location + 4]
            .try_into()
            .expect("4"),
    );
    let mut named_slot = [0u8; 8];
    named_slot[..6].copy_from_slice(&record_bytes[second_location + 4..second_location + 10]);
    let named_offset = u64::from_le_bytes(named_slot) * SLOT_BYTES;
    let mut flipped_named_unit = flow
        .writes
        .iter()
        .rev()
        .find(|write| {
            write.kind == StepKind::UnitWrite
                && write.device.0 == named_device
                && write.offset.0 == named_offset
        })
        .expect("流里有 C 第一个点名单元第二份那一次单元写")
        .clone();
    let WrittenContents::Bytes(unit_bytes) = &mut flipped_named_unit.contents else {
        panic!("单元写是普通写");
    };
    unit_bytes[1000] ^= 0xff;
    let torn_roots =
        [4u64, 5].map(|checkpoint_txg| torn_copy_of_the_root_write_for_txg(&flow, checkpoint_txg));
    append_a_segment_of_writes(
        &mut flow,
        torn_roots.into_iter().chain([flipped_named_unit]).collect(),
    );
    flow
}

/// 前缀停在后面那次发布的点名单元上那一形（[`record_two_single_record_publishes_stopped_by_a_named_unit_of_the_second`]）。
/// CPU 一侧先钉流造对了：整条落盘的镜像上 core 的只读恢复施加 B 一条、验不过 C 一条、落在 (1, 4)；I-5.4 照认 B（停之前已整次收下的第一次发布）、
/// 红在它身上，I-4.1 报不适用（起点含介质损坏）。GPU 走读内核那一份重放停在 C 时照交回 B 的末条，扫描内核同红，每个状态逐位相同
/// （停在点名单元上就交 NONE 的话，那个状态上 GPU 不红 I-5.4 而 CPU 红）。
#[test]
// cpu-reference-for-gpu-comparison
fn the_gpu_judge_agrees_with_the_cpu_that_a_replay_stopped_by_a_named_unit_of_a_later_publish_still_takes_the_first_publish_as_the_version_the_next_mount_applies_first(
) {
    let flow = record_two_single_record_publishes_stopped_by_a_named_unit_of_the_second(
        "crash-judge-gpu-r1-stopped-by-a-named-unit",
    );
    // 取卡之前的 CPU 前提判的是整条落盘那一张镜像经对拍那一类的库读回的那一份（用户 2026-10-13 定：CPU 参照也从库读）；只录那一趟当场返回
    let Some(image) =
        image_with_every_write_landed_read_back_from_the_pairing_library(&flow.base, &flow.writes)
    else {
        return;
    };
    let report =
        singlefs_core::recovery::recover(&image, singlefs_core::recovery::JournalPolicy::Consult);
    assert_eq!(
        (
            report.effective_root,
            report.journal.prefix_applied,
            report.journal.verification_failed
        ),
        (Some((InstanceGeneration(1), CheckpointTxg(4))), 1, 1),
        "整条落盘：core 施加 B 那一条、验不过 C 那一条，落在 (1, 4)"
    );
    match pool_checker_verdict_on_the_image(&image, "I-5.4") {
        InvariantVerdict::Violated(detail) => assert!(
            detail.starts_with(VIOLATION_ON_THE_VERSION_THE_NEXT_MOUNT_APPLIES_FIRST_AT_TXG_FOUR),
            "重放停在 C 之前、B 照认：I-5.4 红在下一次挂载先施加的那一版 B 上：{detail}"
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("重放停在 C 之前、B 照认：I-5.4 要红，实际 {other_verdict:?}")
        }
    }
    match pool_checker_verdict_on_the_image(&image, "I-4.1") {
        InvariantVerdict::NotApplicable(reason) => assert!(
            reason.contains("介质损坏"),
            "I-4.1 报不适用的理由要说是起点含介质损坏：{reason}"
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("前缀停在 C 的点名单元上时 I-4.1 该报不适用，实际 {other_verdict:?}")
        }
    }
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_that_a_replay_stopped_by_a_named_unit_of_a_later_publish_still_takes_the_first_publish_as_the_version_the_next_mount_applies_first",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

// ─── 第 31 批（删除线）：容器树四处走读不往下读的孩子（含 F-L0）、容器树两处 F-W、inode 类型段 0 另一份自证成码 2、候选根链途经已走过的片 ───

/// F-L0（判决 `research/prompts/m3-batch29-code-r1-main-verification.md` 第二节 K2 L1 第三支那一行、第三节第 1 条；harness 坏镜像
/// `a_tombstone_tree_entry_of_type_zero_without_an_identity_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants`
/// 同一形，录成流）：删除中间态的流上墓碑树根（层级 1）第一条条目类型段 1 → 0、身份三段清零、子指针两条位置条目的整单元校验和各翻一字节，
/// 重封到根；墓碑容器不动。
fn record_deletion_with_the_tombstone_root_entry_of_type_zero_and_no_identity_over_two_wrong_location_checksums(
    tag: &str,
) -> RecordedFlow {
    record_deletion_with_a_resealed_field(
        tag,
        "deletion-tombstone-root-entry-type-zero-no-identity-two-location-checksums-wrong",
        &is_the_tombstone_tree_root,
        &|bytes| {
            set_the_type_of_the_first_entry(bytes, 0);
            clear_the_identity_of_the_first_entry_but_its_type(bytes);
            for location in POINTER_LOCATION_ENTRY_OFFSETS {
                flip_a_byte_of_a_location_checksum_in_the_first_entry(bytes, location);
            }
        },
    )
}

/// 第 31 批 F-L0：孩子两份按头里的码 3 都对不上、不是另一类，走类型段 0 那一支；身份引用那一半成立，层级 1 的节点下面不往下走、记走读失败，
/// 每一份副本照判 I-2.1：CPU 池级 checker 红 I-2.1、I-4.8、I-7.2、I-7.4（第 30 批这一支只记走读失败、I-2.1 不红）；GPU 走读内核
/// `w_walk_container_tree_root` 类型段 0、`n.level <= 1u` 那一处同改，每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_without_an_identity_names_a_child_whose_two_location_checksums_are_wrong(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_without_an_identity_names_a_child_whose_two_location_checksums_are_wrong",
        &record_deletion_with_the_tombstone_root_entry_of_type_zero_and_no_identity_over_two_wrong_location_checksums(
            "crash-judge-gpu-i21-tombstone-root-entry-type-zero-no-identity-two-location-checksums-wrong",
        ),
        &["I-2.1", "I-4.8", "I-7.2", "I-7.4"],
    );
}

/// 墓碑树那一侧的 H-J3a（harness 坏镜像
/// `a_tombstone_tree_entry_with_an_unregistered_type_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants`
/// 同一形，录成流）：删除中间态的流上墓碑树根第一条条目类型段 1 → 7、子指针两条位置条目的整单元校验和各翻一字节，重封到根。
fn record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_over_two_wrong_location_checksums(
    tag: &str,
) -> RecordedFlow {
    record_deletion_with_a_resealed_field(
        tag,
        "deletion-tombstone-root-entry-type-seven-two-location-checksums-wrong",
        &is_the_tombstone_tree_root,
        &|bytes| {
            set_the_type_of_the_first_entry(bytes, UNREGISTERED_CONTAINER_TREE_ENTRY_TYPE);
            for location in POINTER_LOCATION_ENTRY_OFFSETS {
                flip_a_byte_of_a_location_checksum_in_the_first_entry(bytes, location);
            }
        },
    )
}

/// 容器树类型段不认得那一支（`w_walk_container_tree_root` 里 `entry_type != record_type` 那一处）：CPU 红 I-9.23、I-2.1、I-4.8、I-7.2、I-7.4，
/// GPU 每个状态逐位相同（判决同一份第二节 K1 L1 测试尖度那一行：容器树这一处调用没有对拍流）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_child_whose_two_location_checksums_are_wrong(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_child_whose_two_location_checksums_are_wrong",
        &record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_over_two_wrong_location_checksums(
            "crash-judge-gpu-i21-tombstone-root-entry-type-seven-two-location-checksums-wrong",
        ),
        &["I-9.23", "I-2.1", "I-4.8", "I-7.2", "I-7.4"],
    );
}

/// 墓碑树那一侧的 H-J3b（harness 坏镜像
/// `a_tombstone_tree_entry_of_type_zero_over_a_container_whose_first_location_checksum_is_wrong_reddens_the_checksum_invariant` 同一形，录成流）：
/// 删除中间态的流上墓碑树根第一条条目类型段 1 → 0、子指针第一条位置条目的整单元校验和翻一字节，重封到根；墓碑容器不动。
fn record_deletion_with_the_tombstone_root_entry_of_type_zero_over_a_wrong_first_location_checksum(
    tag: &str,
) -> RecordedFlow {
    record_deletion_with_a_resealed_field(
        tag,
        "deletion-tombstone-root-entry-type-zero-first-location-checksum-wrong",
        &is_the_tombstone_tree_root,
        &|bytes| {
            set_the_type_of_the_first_entry(bytes, 0);
            flip_a_byte_of_a_location_checksum_in_the_first_entry(
                bytes,
                POINTER_LOCATION_ENTRY_OFFSETS[0],
            );
        },
    )
}

/// 容器树类型段 0、孩子是另一类那一支（`class_of_the_child_of_an_internal_entry != NONE` 那一处）：第二份自证成码 3，不往下走；
/// 第一份对不上，照判 I-2.1：CPU 红 I-9.23、I-2.1、I-4.8、I-7.4（第二份对得上、不记走读失败），GPU 每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_whose_first_location_checksum_is_wrong(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_whose_first_location_checksum_is_wrong",
        &record_deletion_with_the_tombstone_root_entry_of_type_zero_over_a_wrong_first_location_checksum(
            "crash-judge-gpu-i21-tombstone-root-entry-type-zero-first-location-checksum-wrong",
        ),
        &["I-9.23", "I-2.1", "I-4.8", "I-7.4"],
    );
}

/// 容器树类型段 = 记录类型、孩子是另一类那一支（`class_of_the_child_of_a_container_entry != NONE` 那一处）：
/// [`record_a_tombstone_tree_grown_to_level_two`] 的流上层级 2 的墓碑树根第一条条目类型段 0 → 1、子指针第一条位置条目的整单元校验和翻一字节，
/// 重封到根。孩子是层级 1 的码 2 节点：第一份按 16 KiB 对不上，第二份自证成码 2、没有一份是码 3，不往下走、按一槽登记引用。
fn record_a_tombstone_tree_of_level_two_whose_first_root_entry_names_its_node_as_a_container_over_a_wrong_first_location_checksum(
    tag: &str,
) -> RecordedFlow {
    let name = "tombstone-tree-of-level-two-first-root-entry-names-its-node-as-a-container-first-location-checksum-wrong";
    let mut flow = record_a_tombstone_tree_grown_to_level_two(tag, name, false);
    let targets = last_unit_copies(&flow, name, &is_a_tombstone_tree_root_of_level_two, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
        set_the_type_of_the_first_entry(bytes, PACKED_RECORD_TYPE_TOMBSTONE);
        flip_a_byte_of_a_location_checksum_in_the_first_entry(
            bytes,
            POINTER_LOCATION_ENTRY_OFFSETS[0],
        );
    });
    flow
}

/// 见上：CPU 红 I-9.23、I-2.1（第一份那一份；第二份对得上、不记走读失败），GPU 每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_the_container_type_names_a_node_whose_first_location_checksum_is_wrong(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_the_container_type_names_a_node_whose_first_location_checksum_is_wrong",
        &record_a_tombstone_tree_of_level_two_whose_first_root_entry_names_its_node_as_a_container_over_a_wrong_first_location_checksum(
            "crash-judge-gpu-i21-level-two-entry-names-a-node-first-location-checksum-wrong",
        ),
        &["I-9.23", "I-2.1"],
    );
}

/// 码 3 打包记录单元头里记录宽（2 字节）的偏移（D18（块里携带什么信息） 已定项 11：记录数 69、记录宽 71）。
const PACKED_UNIT_RECORD_WIDTH_OFFSET: usize = 71;

/// 墓碑容器头里的记录宽 40 → 39：I-1.7 记录宽那一步判红（重封归调用方）。
fn set_the_record_width_of_a_tombstone_container_one_below(bytes: &mut [u8]) {
    let width = read_u16_at(bytes, PACKED_UNIT_RECORD_WIDTH_OFFSET);
    assert_eq!(
        usize::from(width),
        TOMBSTONE_RECORD_BYTES,
        "墓碑容器的记录宽是登记的 40"
    );
    bytes[PACKED_UNIT_RECORD_WIDTH_OFFSET..PACKED_UNIT_RECORD_WIDTH_OFFSET + 2]
        .copy_from_slice(&(width - 1).to_le_bytes());
}

/// 删除中间态的流上墓碑容器两份的记录宽 40 → 39（重封、沿引用链补到根），再把盘序大的那一份的前 16 KiB 换成墓碑树根（码 2）的原样字节，
/// 墓碑树根第一条条目子指针第二条位置条目的整单元校验和换成那 16 KiB 的 CRC、`change_the_first_entry` 再改一处，重封到根
/// （攻方 `research/prompts/m3-batch28-code-r1-opus-output.md` 第 1.1 节墓碑那一形；harness 坏镜像
/// `a_tombstone_container_with_a_wrong_record_width_beside_a_copy_verifying_as_a_node_reddens_the_packed_container_invariant_on_either_device`
/// 位置条目 1 那一形，录成流）。盘序小的那一份是码 3、带违例、按它的位置条目自证得过。
fn record_deletion_with_a_wrong_record_width_in_the_tombstone_container_and_its_second_copy_holding_the_root_bytes(
    tag: &str,
    name: &'static str,
    change_the_first_entry: &dyn Fn(&mut [u8]),
) -> RecordedFlow {
    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
    let container_copies = last_unit_copies(&flow, name, &is_a_tombstone_container, false);
    corrupt_units_and_reseal_up_to_the_root(
        &mut flow,
        &container_copies,
        &set_the_record_width_of_a_tombstone_container_one_below,
    );
    let second_container_copy =
        if flow.writes[container_copies[0]].device.0 < flow.writes[container_copies[1]].device.0 {
            container_copies[1]
        } else {
            container_copies[0]
        };
    let root_copies = last_unit_copies(&flow, name, &is_the_tombstone_tree_root, false);
    assert_the_first_entry_names_the_unit_of(
        &flow,
        root_copies[0],
        &container_copies,
        "墓碑树根第一条条目指着流里最后写的那片墓碑容器",
    );
    let node_bytes = flow.writes[root_copies[0]]
        .bytes()
        .expect("单元写带字节")
        .to_vec();
    assert_eq!(
        unit_class(&node_bytes),
        2,
        "换进去的那 16 KiB 头里的类是码 2"
    );
    let second_location = first_entry_of_a_container_number_node()
        + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
        + POINTER_LOCATION_ENTRY_OFFSETS[1];
    assert_eq!(
        device_and_slot_of_a_pointer_location(
            &node_bytes[first_entry_of_a_container_number_node()
                + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET..],
            POINTER_LOCATION_ENTRY_OFFSETS[1],
        )
        .0,
        flow.writes[second_container_copy].device.0,
        "第 1 条位置条目是盘序大的那块盘"
    );
    let checksum_of_the_node_bytes = crc32c(&node_bytes);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
        let field = second_location + LOCATION_ENTRY_UNIT_CHECKSUM_OFFSET;
        bytes[field..field + 4].copy_from_slice(&checksum_of_the_node_bytes.to_le_bytes());
        change_the_first_entry(bytes);
    });
    let WrittenContents::Bytes(container) = &mut flow.writes[second_container_copy].contents else {
        panic!("单元写是普通写");
    };
    container[..node_bytes.len()].copy_from_slice(&node_bytes);
    flow
}

/// 墓碑树根第一条条目不再改别处（类型段照旧是 1）。
fn leave_the_first_entry_as_it_is(_bytes: &mut [u8]) {}

/// 第 31 批：容器树类型段 = 记录类型那一处的 F-W（第 30 批 M 交回第 4 条：GPU 两处没有对拍流）：孩子盘 1 那一份自证成码 2，另一份是码 3、
/// 按 32 KiB 自证得过：CPU 认出码 2、I-9.23 红，再照类型段 1 那一支走盘 0 那一份、判出 I-1.7（见别类就停的话 I-1.7 报成立）；
/// GPU 走读内核 `class_of_the_child_of_a_container_entry = NONE` 那一处同改，每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_container_copy_of_the_entry_type_carries_a_violation_beside_a_copy_verifying_as_a_node(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_container_copy_of_the_entry_type_carries_a_violation_beside_a_copy_verifying_as_a_node",
        &record_deletion_with_a_wrong_record_width_in_the_tombstone_container_and_its_second_copy_holding_the_root_bytes(
            "crash-judge-gpu-i923-tombstone-container-record-width-second-copy-verifying-as-a-node",
            "deletion-tombstone-container-record-width-second-copy-verifying-as-a-node",
            &leave_the_first_entry_as_it_is,
        ),
        &["I-9.23", "I-1.7"],
    );
}

/// 第 31 批：容器树类型段 0 那一处的 F-W：同上一条的镜像，墓碑树根第一条条目再把类型段 1 → 0。孩子盘 0 那一份自证成码 3（另一类），
/// 盘 1 那一份就是类型段 0 那一类（码 2）、按 16 KiB 自证得过：CPU 判 I-9.23 红，照类型段 0 那一支走那一份——层级 1 的节点下面不往下走、
/// 两份照判 I-2.1（都对得上）、记走读失败，红 I-7.2（见别类就停的话不记走读失败、I-7.2 不红）；GPU 走读内核
/// `class_of_the_child_of_an_internal_entry = NONE` 那一处同改，每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_with_a_copy_verifying_as_a_node(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_with_a_copy_verifying_as_a_node",
        &record_deletion_with_a_wrong_record_width_in_the_tombstone_container_and_its_second_copy_holding_the_root_bytes(
            "crash-judge-gpu-i923-tombstone-entry-type-zero-container-second-copy-verifying-as-a-node",
            "deletion-tombstone-entry-type-zero-container-second-copy-verifying-as-a-node",
            &|bytes| set_the_type_of_the_first_entry(bytes, 0),
        ),
        &["I-9.23", "I-7.2"],
    );
}

/// GPU 结论第 1 字：恢复内核判不了（第 0 字带 [`verdict_bits::UNDECIDABLE`]）时是它的原因（`crash_judge_recovery.wgsl` 写结论那一段）。
/// 恢复内核的原因只有不带站点的 1–8（`crash_judge_common.wgsl` 的 `UNDECIDABLE_*`），池级 checker 原因字（[`POOL_CHECKER_UNDECIDABLE_WORD`]）
/// 多半带站点（低 8 位是原因、其上是站点号），是两套码：同值的几个只是两边共用同一个常量，不能拿一张码表同时比两个字。
const GPU_RECOVERY_UNDECIDABLE_REASON_WORD: usize = 1;

/// GPU 恢复内核沿实例表链读满 256 片、最后一片还说有下一片时记的原因（`crash_judge_common.wgsl` 的 `UNDECIDABLE_TOO_DEEP`）。
/// 恢复内核沿链读的三处（`crash_judge_common.wgsl` 的 `instance_table_chain_readable`、`crash_judge_recovery.wgsl` 的
/// `load_rows_of_the_instance_table_chain` 与 `load_instance_table_of_the_effective_root`）都设 256 片的上限；CPU 读链不设上限，
/// 链更长的状态按设计交 CPU（`load_rows_of_the_instance_table_chain` 上面那段注释：「链长过 256 片的记判不了」）。
const GPU_RECOVERY_UNDECIDABLE_TOO_DEEP: u32 = 4;

/// 实例表链 257 片那三条流上恢复内核许交 CPU 的原因：恢复读到链 257 片的那条根的状态上，恢复内核沿链读满 256 片还有下一片，
/// 记 [`GPU_RECOVERY_UNDECIDABLE_TOO_DEEP`]、交 CPU；别的原因照旧算不一致。同造法、链 256 片的流不用它（每个状态两个内核都要判得了）。
const GPU_RECOVERY_REASONS_LEFT_TO_THE_CPU_ON_AN_INSTANCE_TABLE_CHAIN_OF_257_PAGES: &[u32] =
    &[GPU_RECOVERY_UNDECIDABLE_TOO_DEEP];

/// 「判不了就回落 CPU」那几条用例里恢复内核那一半：恢复内核判不了的状态 `gpu_verdict_items` 也交回 None、回落 CPU，
/// 只看池级 checker 原因字的话这种状态既不计数、也不断言。原因不在 `recovery_reasons_left_to_the_cpu`（恢复内核自己那套码）里
/// 就交回一句不一致（给空的就是这条流上恢复内核每个状态都该判得了）。
fn recovery_kernel_left_the_state_to_the_cpu_for_an_unlisted_reason(
    ordinal: usize,
    gpu_words: &[u32; VERDICT_WORDS],
    recovery_reasons_left_to_the_cpu: &[u32],
) -> Option<String> {
    let recovery_reason = gpu_words[GPU_RECOVERY_UNDECIDABLE_REASON_WORD];
    (gpu_words[0] & (1 << verdict_bits::UNDECIDABLE) != 0
        && !recovery_reasons_left_to_the_cpu.contains(&recovery_reason))
    .then(|| {
        format!(
            "状态 {ordinal}：GPU 恢复内核判不了、原因 {recovery_reason:#x} 不在 {recovery_reasons_left_to_the_cpu:#x?} 里（这条流只许因点名的原因交 CPU）"
        )
    })
}

/// GPU 走读内核记判不了「类型段 0 的内部节点」时交回的原因字（`crash_judge_common.wgsl` 的 `UNDECIDABLE_TOO_DEEP`）。
const GPU_UNDECIDABLE_TOO_DEEP: u32 = 4;

/// 逐状态比 GPU 与 CPU，GPU 池级 checker 只许在 `reasons_left_to_the_cpu` 这几种原因字上判不了：判了的，违例位与 CPU 逐位相同；
/// 判不了的，原因字在 `reasons_left_to_the_cpu` 里；恢复内核判不了的，第 1 字的原因要在 `recovery_reasons_left_to_the_cpu` 里
/// （恢复内核自己那套码，见 [`GPU_RECOVERY_UNDECIDABLE_REASON_WORD`]）；至少一个状态的原因字在 `reasons_one_state_must_carry` 里；
/// 照流水线那样判不了就回落 CPU（`gpu_verdict_items` 交回 None），回落之后每个状态的红项都与 CPU 相同。
/// 本机没有 NVIDIA 独显适配器时报「本次未跑」返回。
fn assert_the_gpu_judge_leaves_to_the_cpu_only_states_undecidable_for(
    test_name: &str,
    flow: &RecordedFlow,
    reasons_left_to_the_cpu: &[u32],
    recovery_reasons_left_to_the_cpu: &[u32],
    reasons_one_state_must_carry: &[u32],
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, flow, expand_up_to()) else {
        return;
    };
    let mut states_left_to_the_cpu = 0u64;
    let mut states_carrying_a_required_reason = 0u64;
    let mut disagreements: Vec<String> = Vec::new();
    for (ordinal, gpu_words) in comparison.gpu_words_of_each_state.iter().enumerate() {
        let cpu_checker_bits = comparison.cpu_checker_bits_of_each_state[ordinal];
        let cpu_red_items =
            &comparison.cpu_red_items_compared_by_the_cross_check_of_each_state[ordinal];
        let reason = gpu_words[POOL_CHECKER_UNDECIDABLE_WORD];
        if reason == 0 {
            let gpu_checker_bits = pool_checker_violated_bits(gpu_words);
            if !pool_checker_bits_agree_in_the_cross_check(gpu_checker_bits, cpu_checker_bits) {
                disagreements.push(format!(
                    "状态 {ordinal}：GPU 池级 checker 判了，违例 {:?}；CPU 违例 {:?}",
                    invariant_names(gpu_checker_bits),
                    invariant_names(cpu_checker_bits)
                ));
            }
        } else if reasons_left_to_the_cpu.contains(&reason) {
            states_left_to_the_cpu += 1;
            if reasons_one_state_must_carry.contains(&reason) {
                states_carrying_a_required_reason += 1;
            }
        } else {
            disagreements.push(format!(
                "状态 {ordinal}：GPU 池级 checker 判不了的原因字 {reason:#x} 不在 {reasons_left_to_the_cpu:#x?} 里"
            ));
        }
        disagreements.extend(
            recovery_kernel_left_the_state_to_the_cpu_for_an_unlisted_reason(
                ordinal,
                gpu_words,
                recovery_reasons_left_to_the_cpu,
            ),
        );
        // 内核不置位的那几条（第一版是 I-1.13）两边都去掉再比（`red_items_compared_by_the_cross_check`）
        let concluded = gpu_verdict_items(gpu_words).map_or_else(
            || cpu_red_items.clone(),
            |items| red_items_compared_by_the_cross_check(&items),
        );
        if concluded != *cpu_red_items {
            disagreements.push(format!(
                "状态 {ordinal}：判不了就回落 CPU 之后的红项 {concluded} 与 CPU 的 {cpu_red_items} 不同"
            ));
        }
    }
    eprintln!(
        "CRASH_JUDGE_GPU_LEFT_TO_THE_CPU flow={} states={} left_to_the_cpu={states_left_to_the_cpu} carrying_a_required_reason={states_carrying_a_required_reason}",
        flow.name, comparison.states
    );
    assert!(
        states_carrying_a_required_reason > 0,
        "流 {}：至少一个状态上 GPU 池级 checker 要记判不了、原因字在 {reasons_one_state_must_carry:#x?} 里",
        flow.name
    );
    assert!(
        disagreements.is_empty(),
        "流 {}：{} 处不一致（前几个）：\n{}",
        flow.name,
        disagreements.len(),
        disagreements
            .iter()
            .take(6)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// 第 30 批 M 交回第 3 条那一形：新池新建文件的流上 inode 叶容器盘序大的那一份换成 inode 树根（码 2）的原样字节、位置条目跟着换
/// （[`hold_the_inode_root_bytes_in_the_second_copy_of_the_inode_leaf`]），再把 inode 树根第一条条目类型段 2 → 0、重封到根。
fn record_new_pool_with_the_inode_root_entry_of_type_zero_over_a_leaf_whose_second_copy_holds_the_inode_root_bytes(
    tag: &str,
) -> RecordedFlow {
    let name = "new-pool-inode-root-entry-type-zero-leaf-second-copy-verifying-as-a-node";
    let mut flow = record_new_pool_file_creation(tag, name);
    hold_the_inode_root_bytes_in_the_second_copy_of_the_inode_leaf(
        &mut flow,
        name,
        ChecksumOfTheSecondLocation::MatchesTheNodeBytes,
    );
    let root_copies = last_unit_copies(
        &flow,
        name,
        &is_the_inode_tree_root_over_leaf_containers,
        false,
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
        set_the_type_of_the_first_entry(bytes, 0);
    });
    flow
}

/// 第 30 批 M 交回第 3 条：inode 树根条目类型段 0，孩子盘 0 那一份自证成码 3（另一类）、盘 1 那一份就是码 2、按 16 KiB 自证得过。
/// CPU 判 I-9.2 红、照类型段 0 那一支走那一份（F-W）；GPU 走读内核 inode 树只接层级 1、类型段 2，F-W 之后落到「类型段 0 的内部节点」那一句，
/// 池级 checker 记判不了（[`GPU_UNDECIDABLE_TOO_DEEP`]），交回 CPU 判。逐状态：判了的与 CPU 逐位相同，判不了的只许是这一种原因字、至少一个状态是，
/// 回落 CPU 之后红项相同。
#[test]
fn the_gpu_judge_leaves_to_the_cpu_an_inode_root_entry_of_type_zero_over_a_leaf_with_a_copy_verifying_as_a_node(
) {
    assert_the_gpu_judge_leaves_to_the_cpu_only_states_undecidable_for(
        "the_gpu_judge_leaves_to_the_cpu_an_inode_root_entry_of_type_zero_over_a_leaf_with_a_copy_verifying_as_a_node",
        &record_new_pool_with_the_inode_root_entry_of_type_zero_over_a_leaf_whose_second_copy_holds_the_inode_root_bytes(
            "crash-judge-gpu-i92-inode-root-entry-type-zero-leaf-second-copy-verifying-as-a-node",
        ),
        &[GPU_UNDECIDABLE_TOO_DEEP],
        // 恢复内核这条流上每个状态都判得了（第 59 批 E9 那一趟逐状态核过：判不了的 9 个状态都只是池级 checker 记的）；
        // 池级 checker 那一张里的 4 是走读内核的原因字，不是恢复内核的码
        &[],
        &[GPU_UNDECIDABLE_TOO_DEEP],
    );
}

/// 最新根那张实例表改造出来的第 1 片写在哪一槽：在 [`FIRST_SLOT_OF_THE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST`] 起那一截（候选根的新链占着）之前，
/// 同样是流里没写过、基镜像里全零的那一截（写之前由 [`writes_of_instance_table_pages`] 核）。
const SLOT_OF_THE_SECOND_PAGE_OF_THE_NEWEST_INSTANCE_TABLE: u64 = 199_000;
/// 最新根那条链第 1 片的链指针记录指到候选根那条链的第几片（攻方 `research/prompts/m3-batch29-code-r1-opus-output.md` 第四节 `share=(1, 100)`）。
const PAGE_OF_THE_CANDIDATE_CHAIN_THE_NEWEST_CHAIN_RUNS_INTO: usize = 100;
/// 实例 3（崩溃恢复抛弃 C 之后那次可写挂载取的号）：它写行那次发布的根是候选根，最后一次暖机的根是最新根。
const INSTANCE_OF_THE_MOUNT_AFTER_THE_RECOVERY: u32 = 3;
/// 根记录里写序实例（4 字节）的偏移。
const ROOT_RECORD_INSTANCE_OFFSET: usize = 24;

/// 攻方 L2 那一形（判决 `research/prompts/m3-batch29-code-r1-main-verification.md` 第二节 K1 L2 两句走不走得到那一行、第三节第 3 条；
/// 攻方 `research/prompts/m3-batch29-code-r1-opus-output.md` 第四节 `newest=… other=257 role=candidate share=(1, 100)`）：
/// 被崩溃恢复抛弃 C 的那一段历史（[`record_a_root_abandoned_by_a_crash_recovery`]）里，实例 3 的根都指着实例 3 写行那次写的单片实例表 T。
/// 实例 3 写行那条根 R（回退候选集里、不是最新根）改指一条 257 片的新链：第 0 片放 T 的行，其余各片只有链指针记录（头照 T）；
/// R 的根槽写与它那次发布的 journal 记录新根段里的实例表指针改指新链第 0 片、重封，新链各片两份另起一段插在它们之前。
/// 再把 T 改成两片：第 0 片放原来的行、链指针记录指第 1 片，第 1 片只有一条链指针记录、指 R 那条链的第 100 片（容器号 100，
/// 在最新根那条链上是第 2 片：身份不对，坏镜像），T 重封到根，第 1 片两份另起一段插在写 T 那一段之前。
/// GPU 走读内核在上下文 0 里先走最新根的链、走到 R 那条链的第 100 片标成走过，再走候选根 R 的链走到第 100 片合并、停，不记站点 61；
/// 扫描内核从头读 R 的链、读满 256 片链还没完，记判不了（站点 37 或 38）。CPU 一侧先钉流造对了：整条落盘的镜像上最新根那条链第 2 片身份不对，I-1.1 红。
fn record_a_candidate_root_whose_own_instance_table_chain_is_longer_than_the_gpu_scans_and_the_newest_chain_runs_into_it(
    tag: &str,
) -> RecordedFlow {
    let FlowWithARootAbandonedByACrashRecovery { mut flow, .. } =
        record_a_root_abandoned_by_a_crash_recovery(
            tag,
            "candidate-root-own-instance-table-of-257-pages-the-newest-chain-runs-into",
        );
    let root_slot_writes_of_the_mount_after_the_recovery: Vec<usize> = (0..flow.writes.len())
        .filter(|write_index| {
            flow.writes[*write_index].kind == StepKind::RootRecordFua
                && flow.writes[*write_index].bytes().is_some_and(|root| {
                    read_u32_at(root, ROOT_RECORD_INSTANCE_OFFSET)
                        == INSTANCE_OF_THE_MOUNT_AFTER_THE_RECOVERY
                })
        })
        .collect();
    let txg_of = |recorded_flow: &RecordedFlow, write_index: usize| {
        read_u64_at(
            recorded_flow.writes[write_index]
                .bytes()
                .expect("根槽写带字节"),
            ROOT_RECORD_CHECKPOINT_TXG_OFFSET,
        )
    };
    let candidate_root_txg = root_slot_writes_of_the_mount_after_the_recovery
        .iter()
        .map(|write_index| txg_of(&flow, *write_index))
        .min()
        .expect("实例 3 至少写一次根槽");
    let newest_root_txg = root_slot_writes_of_the_mount_after_the_recovery
        .iter()
        .map(|write_index| txg_of(&flow, *write_index))
        .max()
        .expect("实例 3 至少写一次根槽");
    assert!(
        candidate_root_txg < newest_root_txg,
        "实例 3 写行那条根之后还有暖机的根：R 不是最新根"
    );
    let pointer_of = |recorded_flow: &RecordedFlow, write_index: usize| {
        recorded_flow.writes[write_index]
            .bytes()
            .expect("根槽写带字节")[ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET
            ..ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + NODE_POINTER_BYTES]
            .to_vec()
    };
    let pointer_to_the_table_of_the_mount =
        pointer_of(&flow, root_slot_writes_of_the_mount_after_the_recovery[0]);
    assert!(
        root_slot_writes_of_the_mount_after_the_recovery
            .iter()
            .all(|write_index| pointer_of(&flow, *write_index) == pointer_to_the_table_of_the_mount),
        "实例 3 的根都指着同一张实例表 T"
    );
    let root_slot_writes_of_the_candidate: Vec<usize> =
        root_slot_writes_of_the_mount_after_the_recovery
            .iter()
            .copied()
            .filter(|write_index| txg_of(&flow, *write_index) == candidate_root_txg)
            .collect();
    let journal_records_of_the_candidate: Vec<usize> = (0..flow.writes.len())
        .filter(|write_index| {
            flow.writes[*write_index].kind == StepKind::JournalRecord
                && flow.writes[*write_index].bytes().is_some_and(|bytes| {
                    read_u32_at(bytes, JOURNAL_RECORD_INSTANCE_OFFSET)
                        == INSTANCE_OF_THE_MOUNT_AFTER_THE_RECOVERY
                        && read_u64_at(bytes, JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET)
                            == candidate_root_txg
                })
        })
        .collect();
    assert!(
        !journal_records_of_the_candidate.is_empty(),
        "R 那次发布写了 journal 记录"
    );
    let journal_pointer_field = JOURNAL_NEW_ROOT_SEGMENT_OFFSET
        + JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER_OFFSET
        ..JOURNAL_NEW_ROOT_SEGMENT_OFFSET
            + JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER_OFFSET
            + NODE_POINTER_BYTES;
    for record in &journal_records_of_the_candidate {
        assert_eq!(
            &flow.writes[*record].bytes().expect("记录写带字节")[journal_pointer_field.clone()],
            pointer_to_the_table_of_the_mount.as_slice(),
            "R 那次发布的记录新根段里的实例表指针与 R 的根记录相同"
        );
    }
    let table_copies_of_the_mount = |recorded_flow: &RecordedFlow| -> Vec<usize> {
        let locations = POINTER_LOCATION_ENTRY_OFFSETS.map(|location| {
            device_and_slot_of_a_pointer_location(&pointer_to_the_table_of_the_mount, location)
        });
        (0..recorded_flow.writes.len())
            .filter(|write_index| {
                let write = &recorded_flow.writes[*write_index];
                write.kind == StepKind::UnitWrite
                    && locations.contains(&(write.device.0, write.offset.0 / SLOT_BYTES))
                    && write.bytes().is_some_and(|bytes| {
                        unit_class(bytes) == 3
                            && packed_record_type(bytes) == PACKED_RECORD_TYPE_INSTANCE_TABLE
                    })
            })
            .collect()
    };
    let table_copies_before_the_chain = table_copies_of_the_mount(&flow);
    assert_eq!(
        table_copies_before_the_chain.len(),
        2,
        "实例 3 写行那次写了 T 两份"
    );
    let table_of_the_mount = flow.writes[table_copies_before_the_chain[0]]
        .bytes()
        .expect("单元写带字节")
        .to_vec();
    let mut rows = records_of_an_instance_table_page(&table_of_the_mount);
    assert_eq!(
        rows.pop(),
        Some(chain_record_of_the_last_page()),
        "T 只有一片"
    );
    let mut rows_of_each_page_of_the_candidate_chain = vec![rows.clone()];
    rows_of_each_page_of_the_candidate_chain.resize(
        InstanceTableChainPages::OneMoreThanTheGpuWalks.count(),
        Vec::new(),
    );
    let candidate_pages = instance_table_pages_from(
        &table_of_the_mount,
        &pointer_to_the_table_of_the_mount,
        0,
        &rows_of_each_page_of_the_candidate_chain,
    );
    let (slot_of_the_page_run_into, page_run_into) =
        candidate_pages[PAGE_OF_THE_CANDIDATE_CHAIN_THE_NEWEST_CHAIN_RUNS_INTO].clone();
    let pointer_to_the_first_candidate_page = pointer_naming_a_page_at(
        &pointer_to_the_table_of_the_mount,
        candidate_pages[0].0,
        crc32c(&candidate_pages[0].1),
    );
    for root_slot_write in &root_slot_writes_of_the_candidate {
        rewrite_root_record_and_reseal(&mut flow, *root_slot_write, &|bytes| {
            bytes[ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET
                ..ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + NODE_POINTER_BYTES]
                .copy_from_slice(&pointer_to_the_first_candidate_page);
        });
    }
    for record in &journal_records_of_the_candidate {
        rewrite_journal_record_and_reseal(&mut flow, *record, &|bytes| {
            bytes[journal_pointer_field.clone()]
                .copy_from_slice(&pointer_to_the_first_candidate_page);
        });
    }
    let devices = devices_of_an_instance_table_pointer(&pointer_to_the_table_of_the_mount);
    let candidate_page_writes = writes_of_instance_table_pages(
        &flow,
        devices,
        candidate_pages,
        PagesAfterTheFirstInTheFlow::Intact,
    );
    let first_write_naming_the_candidate_chain = *root_slot_writes_of_the_candidate
        .iter()
        .chain(&journal_records_of_the_candidate)
        .min()
        .expect("至少一次写");
    insert_a_segment_of_writes_before_the_segment_of(
        &mut flow,
        first_write_naming_the_candidate_chain,
        candidate_page_writes,
    );
    let mut second_page_of_the_table = table_of_the_mount.clone();
    replace_the_records_of_an_instance_table_page(
        &mut second_page_of_the_table,
        &[chain_record_naming_the_next_page(
            &pointer_to_the_table_of_the_mount,
            slot_of_the_page_run_into,
            crc32c(&page_run_into),
        )],
    );
    second_page_of_the_table
        [PACKED_UNIT_CONTAINER_NUMBER_OFFSET..PACKED_UNIT_CONTAINER_NUMBER_OFFSET + 8]
        .copy_from_slice(&1u64.to_le_bytes());
    reseal_unit(&mut second_page_of_the_table);
    let mut first_page_records = rows;
    first_page_records.push(chain_record_naming_the_next_page(
        &pointer_to_the_table_of_the_mount,
        SLOT_OF_THE_SECOND_PAGE_OF_THE_NEWEST_INSTANCE_TABLE,
        crc32c(&second_page_of_the_table),
    ));
    let table_copies = table_copies_of_the_mount(&flow);
    assert_eq!(table_copies.len(), 2, "插段之后 T 还是两份");
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &table_copies, &|bytes| {
        replace_the_records_of_an_instance_table_page(bytes, &first_page_records);
    });
    let second_page_writes = writes_of_instance_table_pages(
        &flow,
        devices,
        vec![(
            SLOT_OF_THE_SECOND_PAGE_OF_THE_NEWEST_INSTANCE_TABLE,
            second_page_of_the_table,
        )],
        PagesAfterTheFirstInTheFlow::Intact,
    );
    let first_table_write = *table_copies.iter().min().expect("T 至少一份");
    insert_a_segment_of_writes_before_the_segment_of(
        &mut flow,
        first_table_write,
        second_page_writes,
    );
    let mut image_with_every_write = flow.base.clone();
    image_with_every_write.apply_writes(&flow.writes);
    assert!(
        matches!(
            pool_checker_verdict_on_the_image(&image_with_every_write, "I-1.1"),
            InvariantVerdict::Violated(_)
        ),
        "流 {}：最新根那条链第 2 片是 R 那条链的第 100 片、容器号不是 2，CPU 判 I-1.1 红：{:?}",
        flow.name,
        pool_checker_verdict_on_the_image(&image_with_every_write, "I-1.1")
    );
    flow
}

/// 第 31 批（判决 `research/prompts/m3-batch29-code-r1-main-verification.md` 第三节第 3 条：扫描内核站点 37 / 38 那两句留作兜底，补走得到它们的那一形）：
/// [`record_a_candidate_root_whose_own_instance_table_chain_is_longer_than_the_gpu_scans_and_the_newest_chain_runs_into_it`]。
/// CPU 读链不设片数上限；GPU 池级 checker 判不了的原因字只许是实例表链过长那三种（R 还是最新根的状态上走读内核走满记 61），
/// 且至少一个状态落在扫描内核那两处（37 或 38，推的：读码，攻方读码模型 `l2_chain_model.py` 也是这样，没在卡上跑过）；
/// 判了的与 CPU 逐位相同，判不了就回落 CPU 之后红项相同。
#[test]
fn the_gpu_judge_leaves_to_the_cpu_a_candidate_instance_table_chain_longer_than_the_pages_it_scans_once_the_newest_chain_runs_into_it(
) {
    assert_the_gpu_judge_leaves_to_the_cpu_only_states_undecidable_for(
        "the_gpu_judge_leaves_to_the_cpu_a_candidate_instance_table_chain_longer_than_the_pages_it_scans_once_the_newest_chain_runs_into_it",
        &record_a_candidate_root_whose_own_instance_table_chain_is_longer_than_the_gpu_scans_and_the_newest_chain_runs_into_it(
            "crash-judge-gpu-candidate-instance-table-of-257-pages-the-newest-chain-runs-into",
        ),
        &[
            GPU_UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED,
            GPU_UNDECIDABLE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST_LONGER_THAN_THE_PAGES_SCANNED,
            GPU_UNDECIDABLE_INSTANCE_TABLE_ROWS_LONGER_THAN_THE_PAGES_SCANNED,
        ],
        // 恢复读到 R（链 257 片）的状态上恢复内核沿链读满 256 片还有下一片，按设计交 CPU（E9 那一趟这条流 16 个状态记 4）
        GPU_RECOVERY_REASONS_LEFT_TO_THE_CPU_ON_AN_INSTANCE_TABLE_CHAIN_OF_257_PAGES,
        &[
            GPU_UNDECIDABLE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST_LONGER_THAN_THE_PAGES_SCANNED,
            GPU_UNDECIDABLE_INSTANCE_TABLE_ROWS_LONGER_THAN_THE_PAGES_SCANNED,
        ],
    );
}

// ─── 第 32 批（删除线）：「走过」按读回那一份的落点认（F-KEY）、类型段 0 认孩子的类只认自证过的副本（F-ID-b）、单元区里没写过的槽读出全零（F-零槽） ───

/// 位置条目里槽号的起点与字节数（盘 4 + 槽 6 + 整单元校验和 4）。
const LOCATION_ENTRY_SLOT_OFFSET: usize = 4;
const LOCATION_ENTRY_SLOT_BYTES: usize = 6;

/// 码 2 节点第一条条目子指针里第 `location` 条位置条目（[`POINTER_LOCATION_ENTRY_OFFSETS`] 之一）的整单元校验和写成 `checksum`（盘与槽不动）。
fn set_the_checksum_of_a_location_in_the_first_entry(
    bytes: &mut [u8],
    location: usize,
    checksum: u32,
) {
    let field = first_entry_of_a_container_number_node()
        + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
        + location
        + LOCATION_ENTRY_UNIT_CHECKSUM_OFFSET;
    bytes[field..field + 4].copy_from_slice(&checksum.to_le_bytes());
}

/// 码 2 节点第一条条目子指针里第 `location` 条位置条目的槽写成 `slot`、整单元校验和写成 `checksum`（盘不动）。
fn set_the_slot_and_checksum_of_a_location_in_the_first_entry(
    bytes: &mut [u8],
    location: usize,
    slot: u64,
    checksum: u32,
) {
    let field = first_entry_of_a_container_number_node()
        + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
        + location;
    bytes[field + LOCATION_ENTRY_SLOT_OFFSET
        ..field + LOCATION_ENTRY_SLOT_OFFSET + LOCATION_ENTRY_SLOT_BYTES]
        .copy_from_slice(&slot.to_le_bytes()[..LOCATION_ENTRY_SLOT_BYTES]);
    set_the_checksum_of_a_location_in_the_first_entry(bytes, location, checksum);
}

/// 翻进校验和里的几位：那一份按哪种长度都对不上。
const CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY: u32 = 0x5a5a_5a5a;

/// 两份副本里盘序小、盘序大的那一份（写下标）。
fn copies_in_device_order(flow: &RecordedFlow, copies: &[usize]) -> (usize, usize) {
    if flow.writes[copies[0]].device.0 < flow.writes[copies[1]].device.0 {
        (copies[0], copies[1])
    } else {
        (copies[1], copies[0])
    }
}

/// F-KEY（判决 `research/prompts/m3-batch31-code-r1-main-verification.md` 第二节 K2 观察 O3 那一行、第三节第 2 条；harness 坏镜像
/// `a_tombstone_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before`
/// 位置条目 0 那一形，录成流）：删除中间态的流上墓碑容器两份的记录宽 40 → 39（重封、沿引用链补到根），再把墓碑树根第一条条目子指针第 0 条位置条目的槽
/// 改成流里最后写的 inode 树根的槽（这一遍在墓碑树之前走过）、整单元校验和写成那个根的 CRC 翻几位（这一份对不上），重封到根。
/// 读者读回的是盘 1 那一份带违例的容器。
fn record_deletion_with_a_wrong_record_width_in_the_tombstone_container_whose_first_location_names_the_inode_root(
    tag: &str,
) -> RecordedFlow {
    let name = "deletion-tombstone-container-record-width-first-location-naming-the-inode-root";
    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
    let container_copies = last_unit_copies(&flow, name, &is_a_tombstone_container, false);
    corrupt_units_and_reseal_up_to_the_root(
        &mut flow,
        &container_copies,
        &set_the_record_width_of_a_tombstone_container_one_below,
    );
    let inode_root_copies = last_unit_copies(
        &flow,
        name,
        &is_the_inode_tree_root_over_leaf_containers,
        false,
    );
    let inode_root_slot = flow.writes[inode_root_copies[0]].offset.0 / SLOT_BYTES;
    let wrong_checksum_of_the_inode_root = crc32c(
        flow.writes[inode_root_copies[0]]
            .bytes()
            .expect("单元写带字节"),
    ) ^ CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY;
    let root_copies = last_unit_copies(&flow, name, &is_the_tombstone_tree_root, false);
    assert_the_first_entry_names_the_unit_of(
        &flow,
        root_copies[0],
        &container_copies,
        "墓碑树根第一条条目指着流里最后写的那片墓碑容器",
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
        set_the_slot_and_checksum_of_a_location_in_the_first_entry(
            bytes,
            POINTER_LOCATION_ENTRY_OFFSETS[0],
            inode_root_slot,
            wrong_checksum_of_the_inode_root,
        );
    });
    flow
}

/// 第 32 批 F-KEY：CPU 池级 checker「走过」按读回那一份（盘 1）的落点认，那片容器照判，红 I-1.7（另有 I-2.1 等：位置条目 0 那一份对不上、
/// 按两槽登记进了 inode 树根的槽）；GPU 走读内核 `w_visit_the_copy_read_back` 同改，每个状态逐位相同（按第 0 条位置条目认的话两边都不判那片容器）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_first_location_of_a_tombstone_container_with_a_wrong_record_width_names_a_node_walked_before(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_the_first_location_of_a_tombstone_container_with_a_wrong_record_width_names_a_node_walked_before",
        &record_deletion_with_a_wrong_record_width_in_the_tombstone_container_whose_first_location_names_the_inode_root(
            "crash-judge-gpu-i17-tombstone-container-record-width-first-location-naming-the-inode-root",
        ),
        &["I-1.7", "I-2.1"],
    );
}

/// F-ID-b（判决 `research/prompts/m3-batch31-code-r1-main-verification.md` 第二节 K2 N1 盘序翻面 A3 那一行，用户 2026-10-07 定；harness 坏镜像
/// `a_tombstone_tree_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_tombstone_entry_invariant_holding_on_either_device`
/// 位置条目 0 是 `c16` 那一形，录成流）：删除中间态的流上墓碑树根第一条条目类型段 1 → 0、身份三段清零，子指针第 0 条位置条目的整单元校验和换成
/// 墓碑容器前 16 KiB 的 CRC（盘 0 那一份头里是码 3、按 32 KiB 自证不过），第 1 条换成墓碑树根原样字节的 CRC，重封到根；盘 1 那份容器的前 16 KiB
/// 换成墓碑树根（码 2）的原样字节（按 16 KiB 自证得过）。
fn record_deletion_with_the_tombstone_root_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_the_root_bytes(
    tag: &str,
) -> RecordedFlow {
    let name = "deletion-tombstone-root-entry-type-zero-container-checked-at-sixteen-kibibytes-beside-the-root-bytes";
    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
    let container_copies = last_unit_copies(&flow, name, &is_a_tombstone_container, false);
    let (first_container_copy, second_container_copy) =
        copies_in_device_order(&flow, &container_copies);
    let root_copies = last_unit_copies(&flow, name, &is_the_tombstone_tree_root, false);
    assert_the_first_entry_names_the_unit_of(
        &flow,
        root_copies[0],
        &container_copies,
        "墓碑树根第一条条目指着流里最后写的那片墓碑容器",
    );
    let node_bytes = flow.writes[root_copies[0]]
        .bytes()
        .expect("单元写带字节")
        .to_vec();
    assert_eq!(
        unit_class(&node_bytes),
        2,
        "换进去的那 16 KiB 头里的类是码 2"
    );
    let container = flow.writes[first_container_copy]
        .bytes()
        .expect("单元写带字节")
        .to_vec();
    assert_eq!(unit_class(&container), 3, "墓碑容器是码 3");
    let checksum_of_the_first_sixteen_kibibytes = crc32c(&container[..node_bytes.len()]);
    let checksum_of_the_node_bytes = crc32c(&node_bytes);
    assert_eq!(
        device_and_slot_of_a_pointer_location(
            &node_bytes[first_entry_of_a_container_number_node()
                + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET..],
            POINTER_LOCATION_ENTRY_OFFSETS[1],
        )
        .0,
        flow.writes[second_container_copy].device.0,
        "第 1 条位置条目是盘序大的那块盘"
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
        set_the_type_of_the_first_entry(bytes, 0);
        clear_the_identity_of_the_first_entry_but_its_type(bytes);
        set_the_checksum_of_a_location_in_the_first_entry(
            bytes,
            POINTER_LOCATION_ENTRY_OFFSETS[0],
            checksum_of_the_first_sixteen_kibibytes,
        );
        set_the_checksum_of_a_location_in_the_first_entry(
            bytes,
            POINTER_LOCATION_ENTRY_OFFSETS[1],
            checksum_of_the_node_bytes,
        );
    });
    let WrittenContents::Bytes(second_container) = &mut flow.writes[second_container_copy].contents
    else {
        panic!("单元写是普通写");
    };
    second_container[..node_bytes.len()].copy_from_slice(&node_bytes);
    flow
}

/// 第 32 批 F-ID-b：CPU 池级 checker 认「孩子是码 2」只看按自己那一类长度自证过的副本（盘 1 那一份码 2），I-9.23 不红；F-L0 按头里的码 3 比盘 0 那一份
/// 对不上，红 I-2.1、I-4.8、I-7.2、I-7.4；GPU 走读内核 `w_class_of_the_first_copy_verifying_at_the_length_of_its_class` 同改，每个状态逐位相同
/// （取第一份按 16 KiB 对得上的话两边都认成码 3、I-9.23 红）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_checked_at_sixteen_kibibytes_beside_a_node(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_checked_at_sixteen_kibibytes_beside_a_node",
        &record_deletion_with_the_tombstone_root_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_the_root_bytes(
            "crash-judge-gpu-i923-tombstone-root-entry-type-zero-sixteen-kibibyte-copy-beside-a-node",
        ),
        &["I-2.1", "I-7.2"],
    );
}

/// 流里谁都没写过、基镜像里全零的一段：两块盘上这一槽起 32 KiB（写之前由 [`assert_nothing_in_the_flow_wrote_the_slot`] 核），在单元区里、
/// 离 [`SLOT_OF_THE_SECOND_PAGE_OF_THE_NEWEST_INSTANCE_TABLE`] 那几条流用的那一截也远。
const SLOT_NOTHING_IN_THE_FLOW_WROTE: u64 = 190_000;

/// `slot` 起 32 KiB 在两块盘上流里没写过、基镜像里全零。
fn assert_nothing_in_the_flow_wrote_the_slot(flow: &RecordedFlow, slot: u64) {
    let start = slot * SLOT_BYTES;
    let end = start + 2 * SLOT_BYTES;
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        assert!(
            flow.writes.iter().all(|write| write.device != device
                || write.offset.0 + write.length_in_bytes() <= start
                || write.offset.0 >= end),
            "槽 {slot} 起 32 KiB 在盘 {} 上流里没写过",
            device.0
        );
        assert!(
            singlefs_core::recovery::PoolReader::read(
                &flow.base,
                device,
                singlefs_core::address::DeviceOffsetInBytes(start),
                usize::try_from(end - start).expect("32768"),
            )
            .expect("基镜像里读得到这一截")
            .iter()
            .all(|byte| *byte == 0),
            "槽 {slot} 起 32 KiB 在盘 {} 的基镜像里全零",
            device.0
        );
    }
}

/// F-零槽（判决 `research/prompts/m3-batch29-code-r1-main-verification.md` 第二节 K2 L1 位置条目指向没写过的槽那一行、第三节第 2 条；攻方
/// `research/prompts/m3-batch29-code-r1-opus-output.md` 第三节 `tomb-L1-loc1zero16-type7` 那一形，录成流）：删除中间态的流上墓碑树根第一条条目
/// 类型段 1 → 7、子指针第 1 条位置条目的槽改成 [`SLOT_NOTHING_IN_THE_FLOW_WROTE`]、整单元校验和写成 16 KiB 全零的 CRC，重封到根。
fn record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_whose_second_location_names_a_slot_nothing_wrote(
    tag: &str,
) -> RecordedFlow {
    let name =
        "deletion-tombstone-root-entry-type-seven-second-location-naming-a-slot-nothing-wrote";
    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
    assert_nothing_in_the_flow_wrote_the_slot(&flow, SLOT_NOTHING_IN_THE_FLOW_WROTE);
    let checksum_of_sixteen_zero_kibibytes =
        crc32c(&vec![0u8; usize::try_from(SLOT_BYTES).expect("16384")]);
    let root_copies = last_unit_copies(&flow, name, &is_the_tombstone_tree_root, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
        set_the_type_of_the_first_entry(bytes, UNREGISTERED_CONTAINER_TREE_ENTRY_TYPE);
        set_the_slot_and_checksum_of_a_location_in_the_first_entry(
            bytes,
            POINTER_LOCATION_ENTRY_OFFSETS[1],
            SLOT_NOTHING_IN_THE_FLOW_WROTE,
            checksum_of_sixteen_zero_kibibytes,
        );
    });
    flow
}

/// 第 32 批 F-零槽：CPU 的稀疏读者对没写过的槽读出全零，头里类标签 0 不登记、16 KiB 与 32 KiB 两种长度都比，全零 16 KiB 的 CRC 对得上，
/// 那一份 I-2.1 成立；孩子按层级 1 的形状登记两槽引用，红 I-9.23、I-3.1、I-3.11（攻方 `RESULT … form=tomb-L1-loc1zero16-type7 red=[I-3.1,I-3.11,I-9.23]`）。
/// GPU 读者 `read_at` 在单元区里找不到落点、整段在盘内时交回全零单元（`crash_judge_common.wgsl` 的 `zero_unit_version_over_a_slot_nothing_wrote`），
/// 每个状态逐位相同（改之前那个槽读不出，GPU 多红 I-2.1、I-4.8、I-7.4）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_nothing_wrote_with_the_checksum_of_zeros(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_nothing_wrote_with_the_checksum_of_zeros",
        &record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_whose_second_location_names_a_slot_nothing_wrote(
            "crash-judge-gpu-i21-tombstone-root-entry-type-seven-second-location-naming-a-slot-nothing-wrote",
        ),
        &["I-9.23", "I-3.1"],
    );
}

/// 回退流上 A（txg 3）那片 inode 叶容器（码 3、打包记录类型 2、头里诞生代号 3：回退那一版 D 照抄 A 的 inode 树，两版经同一个树根引用它）。
fn is_the_inode_leaf_container_born_at_txg_three(bytes: &[u8]) -> bool {
    const PACKED_UNIT_BIRTH_TXG_OFFSET: usize = 73;
    unit_class(bytes) == 3
        && packed_record_type(bytes) == 2
        && u64::from_le_bytes(
            bytes[PACKED_UNIT_BIRTH_TXG_OFFSET..PACKED_UNIT_BIRTH_TXG_OFFSET + 8]
                .try_into()
                .expect("8 字节"),
        ) == 3
}

/// 分叉形（判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第二节 K1 / K2 P1 那一行③、第三节第 1 条；攻方
/// `research/prompts/m3-batch32-code-r1-opus-output.md` 第 1.4 节 `raise_birth=true corrupt_loc0=true direct=false`，录成流）：
/// 固定脚本到回退 D 的流上，A 那片 inode 叶容器 L 两份头里的诞生代号 3 → 9（D 的 txg，重封、沿引用链补到根：A、D 两版照旧经同一个树根引用 L），
/// 再把盘序小的那一份写成别的字节（不补校验和，那个槽上没有别的单元）。harness 坏镜像
/// `a_shared_leaf_container_born_after_the_rollback_target_with_a_garbled_copy_reddens_the_birth_and_appearance_rules_on_either_device` 同一形。
fn record_rollback_whose_shared_leaf_container_is_born_after_the_target_with_its_first_copy_garbled(
    tag: &str,
) -> RecordedFlow {
    const ROLLBACK_PUBLISH_TXG: u64 = 9;
    let mut flow = record_fixed_script_through_rollback(tag);
    flow.name = "rollback-shared-leaf-container-born-after-the-target-first-copy-garbled";
    let name = flow.name;
    let leaf_copies = last_unit_copies(
        &flow,
        name,
        &is_the_inode_leaf_container_born_at_txg_three,
        false,
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &leaf_copies, &|bytes| {
        bytes[73..81].copy_from_slice(&ROLLBACK_PUBLISH_TXG.to_le_bytes());
    });
    let (first_copy, _second_copy) = copies_in_device_order(&flow, &leaf_copies);
    let WrittenContents::Bytes(first) = &mut flow.writes[first_copy].contents else {
        panic!("单元写是普通写");
    };
    first.fill(0xa5);
    flow
}

/// 第 33 批 allrbmap：L 盘 0 那一份读不出、读回的是盘 1 那一份。CPU 池级 checker 的诞生代号、并进前一版与 I-3.5 的折叠表都按读回那一份的落点认，
/// A 那条根（txg 3，不是最新的候选根）引用 L、L 读得出的那一份诞生代号 9 > 3，红 I-4.9；D 引用、C 不引用的 inode 树根那一格子树里 L 的诞生代号
/// 大于树根指针的出生 txg，红 I-3.5。GPU 走读内核同改（`w_read_unit_to_walk` 记下读回那一份的落点），每个状态逐位相同
/// （改之前 GPU 把诞生代号记在盘 0 那个落点的走过表行上、那一行不存在，两条都漏）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_shared_leaf_container_born_after_the_rollback_target_reads_back_from_its_second_copy(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_shared_leaf_container_born_after_the_rollback_target_reads_back_from_its_second_copy",
        &record_rollback_whose_shared_leaf_container_is_born_after_the_target_with_its_first_copy_garbled(
            "crash-judge-gpu-i49-shared-leaf-container-born-after-the-target-first-copy-garbled",
        ),
        &["I-4.9", "I-3.5"],
    );
}

/// 两份副本里离盘尾近、槽号装不进 32 位的位置：槽号加 2³²（`zero_unit_version_over_a_slot_nothing_wrote` 的守卫按 `offset.y >> (32 − SLOT_SHIFT)` 判时
/// 落在 [2³², 2³⁶) 的槽号回绕成低 32 位）。
const SLOTS_IN_THIRTY_TWO_BITS: u64 = 1 << 32;

/// P4 回绕（判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第二节 K1 / K2 P4 回绕那一行、第三节第 2 条；正推
/// `research/prompts/m3-batch32-code-r1-sonnet-output.md` 第八节第 1 条）：同 F-零槽那条流
/// （[`record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_whose_second_location_names_a_slot_nothing_wrote`]），
/// 第 1 条位置条目的槽写成 `slot`、整单元校验和写成 16 KiB 全零的 CRC。
fn record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_whose_second_location_names_the_slot(
    tag: &str,
    name: &'static str,
    slot: u64,
) -> RecordedFlow {
    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
    assert_nothing_in_the_flow_wrote_the_slot(&flow, SLOT_NOTHING_IN_THE_FLOW_WROTE);
    let checksum_of_sixteen_zero_kibibytes =
        crc32c(&vec![0u8; usize::try_from(SLOT_BYTES).expect("16384")]);
    let root_copies = last_unit_copies(&flow, name, &is_the_tombstone_tree_root, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
        set_the_type_of_the_first_entry(bytes, UNREGISTERED_CONTAINER_TREE_ENTRY_TYPE);
        set_the_slot_and_checksum_of_a_location_in_the_first_entry(
            bytes,
            POINTER_LOCATION_ENTRY_OFFSETS[1],
            slot,
            checksum_of_sixteen_zero_kibibytes,
        );
    });
    flow
}

/// P4 回绕：槽号是 [`SLOT_NOTHING_IN_THE_FLOW_WROTE`] 加 2³²。CPU 的稀疏读者越过盘尾读不到，那一份 I-2.1 红（I-4.8、I-7.4 跟着）；
/// GPU `read_at` 的全零单元守卫按 `SLOT_SHIFT` 判、拒槽号 ≥ 2³²，同样读不到，每个状态逐位相同（守卫按 `offset.y >> (32 − SLOT_SHIFT)` 判时
/// 回绕成槽 190000、交回全零单元，I-2.1 成立，少红三条）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_past_thirty_two_bits_with_the_checksum_of_zeros(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_past_thirty_two_bits_with_the_checksum_of_zeros",
        &record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_whose_second_location_names_the_slot(
            "crash-judge-gpu-i21-tombstone-root-entry-type-seven-second-location-past-thirty-two-bits",
            "deletion-tombstone-root-entry-type-seven-second-location-naming-a-slot-past-thirty-two-bits",
            SLOTS_IN_THIRTY_TWO_BITS + SLOT_NOTHING_IN_THE_FLOW_WROTE,
        ),
        &["I-2.1", "I-9.23"],
    );
}

/// P4 回绕：槽号是 2³² − 1（装得进 32 位，`first_slot + 读的槽数` 那一加在 32 位里回绕成 0）。CPU 越过盘尾读不到，I-2.1 红；GPU 先判起点
/// 不越过空段终点、再拿差比，同样读不到，每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_the_last_slot_in_thirty_two_bits_with_the_checksum_of_zeros(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_the_last_slot_in_thirty_two_bits_with_the_checksum_of_zeros",
        &record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_whose_second_location_names_the_slot(
            "crash-judge-gpu-i21-tombstone-root-entry-type-seven-second-location-last-slot-in-thirty-two-bits",
            "deletion-tombstone-root-entry-type-seven-second-location-naming-the-last-slot-in-thirty-two-bits",
            SLOTS_IN_THIRTY_TWO_BITS - 1,
        ),
        &["I-2.1", "I-9.23"],
    );
}

/// 攻方那一形（`research/prompts/m3-batch32-code-r1-opus-output.md` 第二节 `P4 p4 type=7 next_written=true sum=zero32`，录成流）：
/// 同 F-零槽那条流，第 1 条位置条目指着 [`SLOT_NOTHING_IN_THE_FLOW_WROTE`]、整单元校验和写成 32 KiB 全零的 CRC；它的下一槽在两块盘的基镜像里
/// 写进 16 KiB 非零字节（流里谁都没写过，每个状态都在）。
fn record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_naming_a_slot_nothing_wrote_before_a_written_slot_with_the_checksum_of_thirty_two_zero_kibibytes(
    tag: &str,
) -> RecordedFlow {
    let name = "deletion-tombstone-root-entry-type-seven-second-location-naming-a-slot-nothing-wrote-before-a-written-slot";
    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
    assert_nothing_in_the_flow_wrote_the_slot(&flow, SLOT_NOTHING_IN_THE_FLOW_WROTE);
    let next_slot_offset = (SLOT_NOTHING_IN_THE_FLOW_WROTE + 1) * SLOT_BYTES;
    let bytes_of_the_next_slot = vec![0x5au8; usize::try_from(SLOT_BYTES).expect("16384")];
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        flow.base.devices.get_mut(&device).expect("盘在池里").write(
            singlefs_core::address::DeviceOffsetInBytes(next_slot_offset),
            &bytes_of_the_next_slot,
        );
    }
    let checksum_of_thirty_two_zero_kibibytes =
        crc32c(&vec![0u8; usize::try_from(2 * SLOT_BYTES).expect("32768")]);
    let root_copies = last_unit_copies(&flow, name, &is_the_tombstone_tree_root, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
        set_the_type_of_the_first_entry(bytes, UNREGISTERED_CONTAINER_TREE_ENTRY_TYPE);
        set_the_slot_and_checksum_of_a_location_in_the_first_entry(
            bytes,
            POINTER_LOCATION_ENTRY_OFFSETS[1],
            SLOT_NOTHING_IN_THE_FLOW_WROTE,
            checksum_of_thirty_two_zero_kibibytes,
        );
    });
    flow
}

/// 攻方那一形：CPU 头里类标签 0 不登记，按 16 KiB 读出全零、按 32 KiB 读出「16 KiB 零 + 下一槽的字节」，两种长度都对不上 32 KiB 全零的 CRC，I-2.1 红。
/// GPU F-α 那一处比 32 KiB 时另按 32 KiB 读一次（空段只装得下一槽，读不到），同样对不上，每个状态逐位相同（拿按 16 KiB 读回的全零切片的 32 KiB
/// 检查字去比时判对得上，少红 I-2.1、I-4.8、I-7.4）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_nothing_wrote_before_a_written_slot_with_the_checksum_of_thirty_two_zero_kibibytes(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_nothing_wrote_before_a_written_slot_with_the_checksum_of_thirty_two_zero_kibibytes",
        &record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_naming_a_slot_nothing_wrote_before_a_written_slot_with_the_checksum_of_thirty_two_zero_kibibytes(
            "crash-judge-gpu-i21-tombstone-root-entry-type-seven-slot-nothing-wrote-before-a-written-slot",
        ),
        &["I-2.1", "I-9.23"],
    );
}

fn leave_the_records_before_the_mount_as_they_are(_flow: &mut RecordedFlow) {}

/// C744 ①（`.claude/kb/checks-owed.md` C744；正推 `research/prompts/m3-batch10o-code-r1-sonnet-output.md` K3-2 第 1 类）：三个数据单元那次发布
/// （txg 4）只落了三条记录、根槽没落，新进程可写挂载施加它、取号 2、写行、暖机，再以实例 2 覆盖写一次（记录照旧，
/// [`record_three_unit_file_applied_only_by_its_records_after_a_mount`]）。挂载写了实例 2 的行、覆盖写的记录落了而根槽没落的状态上，
/// txg 4 是由记录施加出来的那一版、覆盖写那一版是下一次挂载先施加的那一版，两样同在（推的，读录制的次序）：扫描内核
/// `judge_allocation_records_disjoint` 与 `judge_allocation_generations_against_unit_births` 的末圈与前两段都走到。每个状态 GPU 与 CPU 逐位相同。
/// 两样同在的状态至少一个（判决 `research/prompts/m3-batch33s-code-r1-main-verification.md` 第二节 S6 行）：CPU 看 journal 那一遍报的根（所选根，
/// 根环里最新的那条）是实例 2 的（挂载写行那一版起每条根的实例表都带实例 1 那一行 (1, 4)，txg 4 就是由记录施加出来的那一版），
/// 又从它往上施加了记录（实际走的那条根不是它：覆盖写那一版是下一次挂载先施加的那一版）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_states_holding_a_version_applied_by_its_records_beside_the_version_the_next_mount_applies_first(
) {
    const INSTANCE_OF_THE_MOUNT: u32 = 2;
    let test_name = "the_gpu_judge_agrees_with_the_cpu_on_states_holding_a_version_applied_by_its_records_beside_the_version_the_next_mount_applies_first";
    let flow = record_three_unit_file_applied_only_by_its_records_after_a_mount(
        "crash-judge-gpu-c744-version-applied-by-records-beside-the-next-mount-version",
        "three-unit-file-applied-only-by-its-records-beside-the-version-the-next-mount-applies-first",
        &leave_the_records_before_the_mount_as_they_are,
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    let states_holding_both_versions = comparison
        .cpu_consulted_reported_root_of_each_state
        .iter()
        .zip(&comparison.cpu_recovery_words_of_each_state)
        .filter(|(reported_root, recovery_words)| {
            let effective_root_reported = recovery_words[0] != u32::MAX;
            reported_root.is_some_and(|(instance, txg)| {
                instance == INSTANCE_OF_THE_MOUNT
                    && effective_root_reported
                    && (recovery_words[0], u64::from(recovery_words[1]))
                        != (instance, txg & 0xFFFF_FFFF)
            })
        })
        .count();
    eprintln!(
        "CRASH_JUDGE_GPU_C744 flow={} states={} states_holding_both_versions={states_holding_both_versions}",
        flow.name, comparison.states
    );
    assert!(
        states_holding_both_versions > 0,
        "流 {}：至少一个状态同时带由记录施加出来的那一版（txg 4）与下一次挂载先施加的那一版（所选根是实例 {INSTANCE_OF_THE_MOUNT} 的、又施加了记录）",
        flow.name
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// F-ID-b 两份都没自证过（用户 2026-10-07 定「当读不出、这一小句不判」；判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第三节第 3 条；
/// harness 坏镜像 `a_tombstone_root_entry_of_type_zero_over_two_copies_that_do_not_verify_leaves_the_tombstone_entry_invariant_holding_on_either_device`
/// 位置条目 0 是 `C16`、1 是 `BAD` 那一格，录成流）：删除中间态的流上墓碑树根第一条条目类型段 1 → 0、身份三段清零，子指针第 0 条位置条目的
/// 整单元校验和换成墓碑容器前 16 KiB 的 CRC（头里是码 3、按 32 KiB 自证不过），第 1 条换成容器整单元 CRC 翻几位，重封到根。
fn record_deletion_with_the_tombstone_root_entry_of_type_zero_over_two_copies_that_do_not_verify(
    tag: &str,
) -> RecordedFlow {
    let name = "deletion-tombstone-root-entry-type-zero-two-copies-that-do-not-verify";
    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
    let container_copies = last_unit_copies(&flow, name, &is_a_tombstone_container, false);
    let root_copies = last_unit_copies(&flow, name, &is_the_tombstone_tree_root, false);
    assert_the_first_entry_names_the_unit_of(
        &flow,
        root_copies[0],
        &container_copies,
        "墓碑树根第一条条目指着流里最后写的那片墓碑容器",
    );
    let container = flow.writes[container_copies[0]]
        .bytes()
        .expect("单元写带字节")
        .to_vec();
    assert_eq!(unit_class(&container), 3, "墓碑容器是码 3");
    let checksum_of_the_first_sixteen_kibibytes =
        crc32c(&container[..usize::try_from(SLOT_BYTES).expect("16384")]);
    let wrong_checksum_of_the_container =
        crc32c(&container) ^ CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY;
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
        set_the_type_of_the_first_entry(bytes, 0);
        clear_the_identity_of_the_first_entry_but_its_type(bytes);
        set_the_checksum_of_a_location_in_the_first_entry(
            bytes,
            POINTER_LOCATION_ENTRY_OFFSETS[0],
            checksum_of_the_first_sixteen_kibibytes,
        );
        set_the_checksum_of_a_location_in_the_first_entry(
            bytes,
            POINTER_LOCATION_ENTRY_OFFSETS[1],
            wrong_checksum_of_the_container,
        );
    });
    flow
}

/// 第 33 批 F-ID-b 两份都没自证过：CPU 池级 checker 当读不出、「子单元是码 2」那一小句不判，I-9.23 不红；F-L0 每份照判 I-2.1（码 3 按 32 KiB 对不上、
/// 翻过位的对不上），红 I-2.1。GPU 走读内核去掉「照改之前取第一份按 16 KiB 对得上的」那一读，每个状态逐位相同（留着它的话 GPU 读出码 3、红 I-9.23）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_child_whose_two_copies_do_not_verify(
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_child_whose_two_copies_do_not_verify",
    ) else {
        return;
    };
    let flow = record_deletion_with_the_tombstone_root_entry_of_type_zero_over_two_copies_that_do_not_verify(
        "crash-judge-gpu-i923-tombstone-root-entry-type-zero-two-copies-that-do-not-verify",
    );
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-2.1")
            && !comparison.cpu_checker_red_invariants.contains("I-9.23"),
        "流 {} 上 CPU 红 I-2.1、一个状态都不红 I-9.23（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// 删完的流（[`record_new_pool_through_the_last_deletion_batch`]）带「没有文件的那几版」：意图那一版（txg 5）起没有文件，三批（txg 6–8）不进表、
/// 照比它旧的最新那一项（意图那一版）判（D8（核心索引结构） 已定项 16 第 4 条；第 33 批 R 报告「停下交主 agent」第二项）。
/// 恢复落到 txg 5–8 报没有文件的状态上 CPU 的 oracle 一个违例都不记；GPU 恢复内核的期望版本表第 5 字记这一版有没有文件、`classify_oracle` 照认，
/// 每个状态逐位相同（不认的话这几版报没有文件判成「更旧的根下面有文件」）。另钉 CPU 两遍 oracle 与记录核对器在每个状态上一个违例都没有、
/// 至少一个状态看 journal 那一遍落到意图那一版 (1, 5) 报没有文件（判决 `research/prompts/m3-batch33s-code-r1-main-verification.md` 第二节 S5 行：
/// 「没有文件的那几版」写错身份时 CPU、GPU 一起判违例、逐位仍相同，只比两边对不对得上看不出来）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_the_deleted_flow_against_a_version_table_without_a_file_from_the_intent_on(
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_on_the_deleted_flow_against_a_version_table_without_a_file_from_the_intent_on",
    ) else {
        return;
    };
    let flow = record_new_pool_through_the_last_deletion_batch(
        "crash-judge-gpu-oracle-deleted-flow-without-a-file-from-the-intent-on",
        "deleted-flow-against-a-version-table-without-a-file-from-the-intent-on",
    );
    let versions_without_a_file = [PublishedVersionWithoutAFile {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(5),
    }];
    let Some(comparison) = compare_on_every_state_against_the_version_table(
        cards,
        &flow,
        &versions_without_a_file,
        expand_up_to(),
    ) else {
        return;
    };
    assert_eq!(
        comparison.cpu_red, 0,
        "流 {}：CPU 两遍 oracle 与记录核对器在每个状态上一个违例都没有（意图那一版起没有文件，表里照认）",
        flow.name
    );
    let intent_version = (
        INSTANCE_OF_THE_PUBLISHES.0,
        u32::try_from(versions_without_a_file[0].checkpoint_txg.0).expect("txg 5 装得进 u32"),
    );
    assert!(
        comparison
            .cpu_recovery_words_of_each_state
            .iter()
            .any(|words| (words[0], words[1]) == intent_version
                && words[2] == RECOVERY_WORD_CLASS_NO_FILE),
        "流 {}：至少一个状态看 journal 那一遍落到意图那一版 {intent_version:?}、报没有文件",
        flow.name
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

// ─── 第 36 批（删除线）：GPU 恢复内核的 F2（F2_EXACT 那一形）、I-3.5 按全部版本走完之后的折叠表取单元（H-S1-fold）、read_at 拼读（H-S2-tail） ───

/// 根槽写里自证校验和字段的偏移（`reseal_root_record` 重封的那 32 字节）。
const ROOT_RECORD_CHECKSUM_FIELD_OFFSET: usize = 138;

/// 流里写出 txg `txg` 那条根的根槽写（恰好一次）。
fn root_write_of_txg(flow: &RecordedFlow, txg: u64) -> usize {
    let root_writes: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| {
            write.kind == StepKind::RootRecordFua
                && write.bytes().is_some_and(|bytes| {
                    u64::from_le_bytes(
                        bytes[ROOT_RECORD_CHECKPOINT_TXG_OFFSET
                            ..ROOT_RECORD_CHECKPOINT_TXG_OFFSET + 8]
                            .try_into()
                            .expect("8 字节"),
                    ) == txg
                })
        })
        .map(|(index, _)| index)
        .collect();
    let [root_write] = root_writes.as_slice() else {
        panic!("流里写出 txg {txg} 那条根的根槽写恰好一次：{root_writes:?}")
    };
    *root_write
}

/// F2 那两条流的底：新池新建文件（txg 3）→ 同一个进程、实例 1 里覆盖写成第二份内容（V，txg 4）→ 再覆盖写成第三份内容（R，txg 5），整条录下来；
/// R 那次发布的树表两份副本各改坏末字节、不重封（[`break_both_copies_of_the_tree_table_named_by_the_root_write`]）：R 的根槽落了的状态上 R 走读失败。
/// 每次发布按单元、记录、根槽、系统配置轮换的次序落盘（`transaction::persist_publish_writes`），R 的根槽落了而它的系统配置轮换没落的状态上，
/// c_见证 是 V 那次发布末条记录的计数器，系统配置见证不到 R。
fn record_two_overwrites_whose_second_tree_table_does_not_read_back(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let mut pool = build_pool(tag);
    overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &second_content());
    overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &third_content());
    let mut flow = recorded_flow_of_the_pool(
        &pool,
        name,
        vec![
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(3),
                content: file_content(),
            },
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(4),
                content: second_content(),
            },
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(5),
                content: third_content(),
            },
        ],
    );
    assert_eq!(
        root_write_of_txg(&flow, R_OF_THE_F2_FLOWS.1),
        flow.judged_root_index,
        "流里最后一条根槽写是 R 的"
    );
    let root_of_r = flow.judged_root_index;
    break_both_copies_of_the_tree_table_named_by_the_root_write(&mut flow, root_of_r);
    flow
}

/// F2 那两条流上的三版：V⁻（新池新建文件）、V（第一次覆盖写，已确认）、R（第二次覆盖写，走读失败），(实例代号, txg)。
const V_MINUS_OF_THE_F2_FLOWS: (u32, u64) = (1, 3);
const V_OF_THE_F2_FLOWS: (u32, u64) = (1, 4);
const R_OF_THE_F2_FLOWS: (u32, u64) = (1, 5);

/// 看 journal 那一遍 oracle 判「落到更旧的根」的那一项（`CrashStateJudgement::red_items` 里的写法）。
fn consulted_oracle_item(kind: CrashPointReplayOracleViolationKind) -> String {
    format!(
        "{CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE}:{}",
        kind.name()
    )
}

/// 看 journal 那一遍在状态 `ordinal` 上实际走的那条根是 `version`（恢复字只带 txg 低 32 位）、结局类别是 `class`。
fn consulted_recovery_lands_on(
    comparison: &Comparison,
    ordinal: usize,
    version: (u32, u64),
    class: u32,
) -> bool {
    let words = comparison.cpu_recovery_words_of_each_state[ordinal];
    (words[0], u64::from(words[1]), words[2]) == (version.0, version.1, class)
}

/// F2 设计里那一退（判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T5 行「设计那一退照旧落 V」）：R 的根槽落了、走读失败，
/// 系统配置没见证 R、见证到 V，CPU 恢复退到 V 本身、读出第二份内容，oracle 判「落到更旧的根」（oracle 不认 F2 那一退，判决第三节第 4 条不改）。
/// GPU 恢复内核照 `recovery::newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to` 退（`crash_judge_recovery.wgsl` 的同名函数），每个状态逐位相同、
/// 落到同一条根。没有它的 GPU 在这几个状态上报读不出（第 33 批 T 合入之后对拍红的那 52 条流同一个病根）。F2 改成 F2_NEWEST（第 39 批，判决
/// `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 K1 / K2 W4 H4 行）之后这一退不变：[V, R) 里最新、不被 R 的实例表抛弃的根就是 V。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_recovery_whose_newest_root_does_not_read_back_falls_back_to_the_version_the_system_configuration_witnessed(
) {
    let test_name = "the_gpu_judge_agrees_with_the_cpu_that_a_recovery_whose_newest_root_does_not_read_back_falls_back_to_the_version_the_system_configuration_witnessed";
    let flow = record_two_overwrites_whose_second_tree_table_does_not_read_back(
        "crash-judge-gpu-f2-fall-back-to-the-witnessed-version",
        "two-overwrites-second-tree-table-unreadable-falls-back-to-the-witnessed-version",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    let recovered_to_an_older_root =
        consulted_oracle_item(CrashPointReplayOracleViolationKind::RecoveredToAnOlderRoot);
    let states_falling_back_to_v = (0..comparison.cpu_red_items_of_each_state.len())
        .filter(|ordinal| {
            comparison.cpu_red_items_of_each_state[*ordinal].contains(&recovered_to_an_older_root)
                && consulted_recovery_lands_on(
                    &comparison,
                    *ordinal,
                    V_OF_THE_F2_FLOWS,
                    RECOVERY_WORD_CLASS_FILE_READ,
                )
        })
        .count();
    eprintln!(
        "CRASH_JUDGE_GPU_F2 flow={} states={} states_falling_back_to_v={states_falling_back_to_v}",
        flow.name, comparison.states
    );
    assert!(
        states_falling_back_to_v > 0,
        "流 {}：R 的根槽落了而系统配置没见证 R 的状态上，CPU 看 journal 那一遍退到 V {V_OF_THE_F2_FLOWS:?}、读出第二份内容",
        flow.name
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// 攻方 A 形（判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T5 行，攻方 `research/prompts/m3-batch33t-code-r1-opus-output.md`
/// 第一节 1）：同上一条流，V 的根槽写自证校验和字段翻一位（V 的根槽读不出），R 走读失败、系统配置见证到 V 的状态上，F2_NEWEST（第 39 批）只在
/// [V, R) 里找：V 的根读不出、区间里再没有别的根，就不退——CPU 报读不出（落到的根照旧是 R），一个状态都不退到 V⁻；GPU 恢复内核同判，每个状态逐位相同。
/// 退到「比所选根旧、不超过见证上限的最新可读根」（第 33 批 T 的写法）或不设下界，会落到 V⁻、把已确认的 V 越过去，这条红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_past_the_witnessed_version_whose_root_does_not_read_back(
) {
    let test_name = "the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_past_the_witnessed_version_whose_root_does_not_read_back";
    let mut flow = record_two_overwrites_whose_second_tree_table_does_not_read_back(
        "crash-judge-gpu-f2-witnessed-root-unreadable",
        "two-overwrites-second-tree-table-unreadable-and-the-witnessed-root-unreadable",
    );
    let root_of_v = root_write_of_txg(&flow, V_OF_THE_F2_FLOWS.1);
    let WrittenContents::Bytes(root_bytes) = &mut flow.writes[root_of_v].contents else {
        panic!("根槽写是普通写");
    };
    root_bytes[ROOT_RECORD_CHECKSUM_FIELD_OFFSET] ^= 0x01;
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    let read_failed = consulted_oracle_item(CrashPointReplayOracleViolationKind::ReadFailed);
    let states_reporting_r_unreadable = (0..comparison.cpu_red_items_of_each_state.len())
        .filter(|ordinal| {
            comparison.cpu_red_items_of_each_state[*ordinal].contains(&read_failed)
                && comparison.cpu_consulted_reported_root_of_each_state[*ordinal]
                    == Some(R_OF_THE_F2_FLOWS)
                && consulted_recovery_lands_on(
                    &comparison,
                    *ordinal,
                    R_OF_THE_F2_FLOWS,
                    RECOVERY_WORD_CLASS_FAILED,
                )
        })
        .count();
    let states_falling_back_past_v = (0..comparison.cpu_red_items_of_each_state.len())
        .filter(|ordinal| {
            comparison.cpu_consulted_reported_root_of_each_state[*ordinal]
                == Some(V_MINUS_OF_THE_F2_FLOWS)
                && comparison.cpu_red_items_of_each_state[*ordinal].contains(
                    &consulted_oracle_item(
                        CrashPointReplayOracleViolationKind::RecoveredToAnOlderRoot,
                    ),
                )
        })
        .count();
    eprintln!(
        "CRASH_JUDGE_GPU_F2 flow={} states={} states_reporting_r_unreadable={states_reporting_r_unreadable} states_falling_back_past_v={states_falling_back_past_v}",
        flow.name, comparison.states
    );
    assert!(
        states_reporting_r_unreadable > 0,
        "流 {}：R 走读失败、V 的根读不出的状态上 CPU 看 journal 那一遍报读不出、报的根是 R {R_OF_THE_F2_FLOWS:?}",
        flow.name
    );
    assert_eq!(
        states_falling_back_past_v, 0,
        "流 {}：一个状态都不退到 V⁻ {V_MINUS_OF_THE_F2_FLOWS:?}（F2 只退到见证到的那一版 V 及之后、所选根之前）",
        flow.name
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

// ─── 第 39 批（删除线）：GPU 恢复内核的 F2 改成 F2_NEWEST 那一形（判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节
// K1 / K2 W4 H4 行、K1 V1 行与第三节第 2 条） ───

/// V 那次发布轮换写的系统配置槽：V 的根槽写之后、R 的根槽写之前的系统配置槽写（每次发布按单元、记录、根槽、系统配置轮换的次序落盘），每块盘一槽。
fn system_configuration_rotation_writes_of_v(flow: &RecordedFlow) -> Vec<usize> {
    let root_of_v = root_write_of_txg(flow, V_OF_THE_F2_FLOWS.1);
    let root_of_r = root_write_of_txg(flow, R_OF_THE_F2_FLOWS.1);
    let rotation: Vec<usize> = (root_of_v + 1..root_of_r)
        .filter(|index| flow.writes[*index].kind == StepKind::SystemConfigurationSlot)
        .collect();
    assert_eq!(
        rotation.len(),
        singlefs_core::recovery::PoolReader::device_identities(&flow.base).len(),
        "V 那次轮换每块盘写一槽：{rotation:?}"
    );
    rotation
}

/// 攻方 H4 的底（判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 K1 / K2 W4 H4 行；攻方
/// `research/prompts/m3-batch35-36-code-r1-opus-output.md` 第四节，攻方的 Pm、P、R 是这里的 V⁻、V、R）：F2 那两条流的底上，V 那次发布轮换写的系统配置槽
/// （每块盘一槽，是两槽里较新的那一槽）写成全零。R 的根槽落了、R 的轮换没落的状态上，每块盘读得出的只剩 V⁻ 那次轮换写的那一槽，
/// c_见证 落回 V⁻ 那次发布末条记录的计数器，系统配置没见证 R；V 已确认、根与单元都读得出。
fn record_two_overwrites_whose_second_tree_table_does_not_read_back_and_whose_first_rotation_reads_back_as_zeros(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let mut flow = record_two_overwrites_whose_second_tree_table_does_not_read_back(tag, name);
    for rotation_write in system_configuration_rotation_writes_of_v(&flow) {
        let slot_bytes = usize::try_from(flow.writes[rotation_write].length_in_bytes())
            .expect("系统配置槽写 4096 字节装得进 usize");
        flow.writes[rotation_write].contents = WrittenContents::Bytes(vec![0u8; slot_bytes]);
    }
    flow
}

/// 看 journal 那一遍有几个状态经 F2 退到 `version`：CPU 报的根是它，oracle 判「落到更旧的根」（所选根 R 落了而走的是更旧的这一版）。
fn states_where_the_consulted_recovery_falls_back_to(
    comparison: &Comparison,
    version: (u32, u64),
) -> usize {
    let recovered_to_an_older_root =
        consulted_oracle_item(CrashPointReplayOracleViolationKind::RecoveredToAnOlderRoot);
    (0..comparison.cpu_red_items_of_each_state.len())
        .filter(|ordinal| {
            comparison.cpu_red_items_of_each_state[*ordinal].contains(&recovered_to_an_older_root)
                && comparison.cpu_consulted_reported_root_of_each_state[*ordinal] == Some(version)
        })
        .count()
}

/// F2 的四条流共用的判法：CPU 看 journal 那一遍至少一个状态退到 `landed_version`（读出它的内容）、一个状态都不退到 `version_not_landed_on`，
/// GPU 每个状态逐位相同、落到同一条根。本机没有 NVIDIA 独显适配器时报「本次未跑」返回。
fn assert_the_gpu_judge_agrees_with_the_cpu_falling_back_to(
    test_name: &str,
    flow: &RecordedFlow,
    landed_version: (u32, u64),
    version_not_landed_on: (u32, u64),
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, flow, expand_up_to()) else {
        return;
    };
    let recovered_to_an_older_root =
        consulted_oracle_item(CrashPointReplayOracleViolationKind::RecoveredToAnOlderRoot);
    let states_falling_back_to_the_landed_version = (0..comparison
        .cpu_red_items_of_each_state
        .len())
        .filter(|ordinal| {
            comparison.cpu_red_items_of_each_state[*ordinal].contains(&recovered_to_an_older_root)
                && comparison.cpu_consulted_reported_root_of_each_state[*ordinal]
                    == Some(landed_version)
                && consulted_recovery_lands_on(
                    &comparison,
                    *ordinal,
                    landed_version,
                    RECOVERY_WORD_CLASS_FILE_READ,
                )
        })
        .count();
    let states_falling_back_to_the_other_version =
        states_where_the_consulted_recovery_falls_back_to(&comparison, version_not_landed_on);
    eprintln!(
        "CRASH_JUDGE_GPU_F2 flow={} states={} states_falling_back_to_{landed_version:?}={states_falling_back_to_the_landed_version} states_falling_back_to_{version_not_landed_on:?}={states_falling_back_to_the_other_version}",
        flow.name, comparison.states
    );
    assert!(
        states_falling_back_to_the_landed_version > 0,
        "流 {}：R 走读失败、系统配置没见证 R 的状态上，CPU 看 journal 那一遍退到 {landed_version:?}、读出它的内容",
        flow.name
    );
    assert_eq!(
        states_falling_back_to_the_other_version, 0,
        "流 {}：一个状态都不退到 {version_not_landed_on:?}",
        flow.name
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// 攻方 H4（底见 [`record_two_overwrites_whose_second_tree_table_does_not_read_back_and_whose_first_rotation_reads_back_as_zeros`]）：
/// R 走读失败、c_见证 落回 V⁻ 的状态上，F2_NEWEST 退到 [V⁻, R) 里读得出、不被 R 的实例表抛弃的最新一条根——已确认、单元完好的 V；
/// F2_EXACT 只认 V⁻、把 V 越过去（攻方原样：三处都落 Pm）。CPU 看 journal 那一遍至少一个状态退到 V、一个状态都不退到 V⁻，GPU 逐位相同。
/// GPU 恢复内核读得出所选根的实例表也只认恰好相等的那条、或把下界换成见证上限（第 33 批 T 的写法）时 GPU 落 V⁻，这条红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_recovery_falls_back_to_the_newest_version_from_the_witnessed_one_when_the_newer_system_configuration_slots_read_back_as_zeros(
) {
    let flow = record_two_overwrites_whose_second_tree_table_does_not_read_back_and_whose_first_rotation_reads_back_as_zeros(
        "crash-judge-gpu-f2-newest-newer-rotation-zeroed",
        "two-overwrites-second-tree-table-unreadable-first-rotation-zeroed",
    );
    assert_the_gpu_judge_agrees_with_the_cpu_falling_back_to(
        "the_gpu_judge_agrees_with_the_cpu_that_a_recovery_falls_back_to_the_newest_version_from_the_witnessed_one_when_the_newer_system_configuration_slots_read_back_as_zeros",
        &flow,
        V_OF_THE_F2_FLOWS,
        V_MINUS_OF_THE_F2_FLOWS,
    );
}

/// 实例表那一片单元（码 3、记录类型 4）最前面插一行 (`instance`, `selected_root_txg`, W = 0)：原有的记录往后挪一条，记录数与声明长度各加一条
/// （单元自己的两道校验和由调用方重封）。
fn insert_an_instance_table_row_in_front(bytes: &mut [u8], instance: u32, selected_root_txg: u64) {
    let record_count = usize::from(u16::from_le_bytes(
        bytes[PACKED_UNIT_RECORD_COUNT_OFFSET..PACKED_UNIT_RECORD_COUNT_OFFSET + 2]
            .try_into()
            .expect("2 字节"),
    ));
    let records_end = PACKED_UNIT_RECORDS_START + record_count * INSTANCE_TABLE_RECORD_BYTES;
    bytes.copy_within(
        PACKED_UNIT_RECORDS_START..records_end,
        PACKED_UNIT_RECORDS_START + INSTANCE_TABLE_RECORD_BYTES,
    );
    let row = &mut bytes
        [PACKED_UNIT_RECORDS_START..PACKED_UNIT_RECORDS_START + INSTANCE_TABLE_RECORD_BYTES];
    row.fill(0);
    row[1..5].copy_from_slice(&instance.to_le_bytes());
    row[5..13].copy_from_slice(&selected_root_txg.to_le_bytes());
    let new_record_count = record_count + 1;
    bytes[PACKED_UNIT_RECORD_COUNT_OFFSET..PACKED_UNIT_RECORD_COUNT_OFFSET + 2].copy_from_slice(
        &u16::try_from(new_record_count)
            .expect("记录数装得进 2 字节")
            .to_le_bytes(),
    );
    bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
        &u16::try_from(new_record_count * INSTANCE_TABLE_RECORD_BYTES)
            .expect("声明长度装得进 2 字节")
            .to_le_bytes(),
    );
}

/// 实例表一片的字节数（码 3 打包记录单元，占两槽）。
const INSTANCE_TABLE_PAGE_BYTES: u64 = 2 * SLOT_BYTES;

/// 只让所选根 R 认一张抛弃 V 的实例表：V⁻、V、R 指着的是同一张单片实例表，它在基镜像里（流里没有写它的单元写，同一实例里不重写）。
/// 照 R 的实例表指针第 0 条位置条目从基镜像读出它（整单元 CRC 与指针里的相同），最前面插一行 (V⁻ 的实例, V⁻ 的 txg)、重封，
/// 写进两块盘基镜像里流从没写过、全零的 [`SLOT_NOTHING_IN_THE_FLOW_WROTE`]：实例表单元头只带身份 (0, 4, 片号, 0) 与诞生、写序，不带自己的落点，
/// 换一槽照样自证得过（读者按位置条目读、只比整单元 CRC）。R 的根槽写与 R 那次发布的 journal 记录新根段里的实例表指针（写者照抄根记录里那一条）
/// 两条位置条目的槽改指这一槽、整单元校验和换成新的，各自重封（记录的反向链一路传下去）。V⁻、V 的根记录与原来那张表一个字节都不动。
fn point_the_chosen_root_alone_at_an_instance_table_abandoning_v(flow: &mut RecordedFlow) {
    let root_record_pointer_field = ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET
        ..ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + NODE_POINTER_BYTES;
    let instance_table_pointer_of = |recorded_flow: &RecordedFlow, root_write: usize| {
        recorded_flow.writes[root_write]
            .bytes()
            .expect("根槽写带字节")[root_record_pointer_field.clone()]
        .to_vec()
    };
    let root_of_r = root_write_of_txg(flow, R_OF_THE_F2_FLOWS.1);
    let pointer_to_the_shared_table = instance_table_pointer_of(flow, root_of_r);
    assert_eq!(
        instance_table_pointer_of(flow, root_write_of_txg(flow, V_OF_THE_F2_FLOWS.1)),
        pointer_to_the_shared_table,
        "流 {}：V 与 R 的根记录指着同一张实例表",
        flow.name
    );
    let (device_of_the_first_location, slot_of_the_shared_table) =
        device_and_slot_of_a_pointer_location(
            &pointer_to_the_shared_table,
            POINTER_LOCATION_ENTRY_OFFSETS[0],
        );
    let shared_table_start = slot_of_the_shared_table * SLOT_BYTES;
    let shared_table_end = shared_table_start + INSTANCE_TABLE_PAGE_BYTES;
    assert!(
        flow.writes.iter().all(|write| write.device.0
            != device_of_the_first_location
            || write.offset.0 + write.length_in_bytes() <= shared_table_start
            || write.offset.0 >= shared_table_end),
        "流 {}：R 指着的实例表（盘 {device_of_the_first_location} 槽 {slot_of_the_shared_table}）流里没写过，在基镜像里",
        flow.name
    );
    let shared_table = singlefs_core::recovery::PoolReader::read(
        &flow.base,
        DeviceIdentity(device_of_the_first_location),
        singlefs_core::address::DeviceOffsetInBytes(shared_table_start),
        usize::try_from(INSTANCE_TABLE_PAGE_BYTES).expect("32768"),
    )
    .expect("基镜像里读得到 R 指着的实例表");
    assert_eq!(
        crc32c(&shared_table),
        read_u32_at(
            &pointer_to_the_shared_table,
            POINTER_LOCATION_ENTRY_OFFSETS[0] + LOCATION_ENTRY_UNIT_CHECKSUM_OFFSET
        ),
        "流 {}：基镜像里那一槽的整单元 CRC 与 R 的实例表指针里的相同",
        flow.name
    );
    assert!(
        unit_class(&shared_table) == 3
            && packed_record_type(&shared_table) == PACKED_RECORD_TYPE_INSTANCE_TABLE,
        "流 {}：R 指着的是一片实例表（码 3、记录类型 4）",
        flow.name
    );
    let mut table_of_r = shared_table;
    insert_an_instance_table_row_in_front(
        &mut table_of_r,
        V_MINUS_OF_THE_F2_FLOWS.0,
        V_MINUS_OF_THE_F2_FLOWS.1,
    );
    reseal_unit(&mut table_of_r);
    for device in devices_of_an_instance_table_pointer(&pointer_to_the_shared_table) {
        assert!(
            device == DeviceIdentity(0) || device == DeviceIdentity(1),
            "R 的实例表指针两条位置条目落在盘 0、盘 1 上（基镜像只在这两块盘上写）：盘 {}",
            device.0
        );
    }
    assert_nothing_in_the_flow_wrote_the_slot(flow, SLOT_NOTHING_IN_THE_FLOW_WROTE);
    write_into_the_base_on_both_devices(flow, SLOT_NOTHING_IN_THE_FLOW_WROTE, &table_of_r);
    let pointer_to_the_table_of_r = pointer_naming_a_page_at(
        &pointer_to_the_shared_table,
        SLOT_NOTHING_IN_THE_FLOW_WROTE,
        crc32c(&table_of_r),
    );
    rewrite_root_record_and_reseal(flow, root_of_r, &|bytes| {
        bytes[root_record_pointer_field.clone()].copy_from_slice(&pointer_to_the_table_of_r);
    });
    let journal_record_pointer_field = JOURNAL_NEW_ROOT_SEGMENT_OFFSET
        + JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER_OFFSET
        ..JOURNAL_NEW_ROOT_SEGMENT_OFFSET
            + JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER_OFFSET
            + NODE_POINTER_BYTES;
    let journal_records_of_r: Vec<usize> = (0..flow.writes.len())
        .filter(|write_index| {
            flow.writes[*write_index].kind == StepKind::JournalRecord
                && flow.writes[*write_index].bytes().is_some_and(|bytes| {
                    read_u32_at(bytes, JOURNAL_RECORD_INSTANCE_OFFSET) == R_OF_THE_F2_FLOWS.0
                        && read_u64_at(bytes, JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET)
                            == R_OF_THE_F2_FLOWS.1
                })
        })
        .collect();
    assert!(
        !journal_records_of_r.is_empty(),
        "流 {}：R 那次发布写了 journal 记录",
        flow.name
    );
    for record in journal_records_of_r {
        assert_eq!(
            &flow.writes[record].bytes().expect("记录写带字节")
                [journal_record_pointer_field.clone()],
            pointer_to_the_shared_table.as_slice(),
            "流 {}：R 那次发布的记录新根段里的实例表指针与 R 的根记录原来那一条相同",
            flow.name
        );
        rewrite_journal_record_and_reseal(flow, record, &|bytes| {
            bytes[journal_record_pointer_field.clone()].copy_from_slice(&pointer_to_the_table_of_r);
        });
    }
}

/// 所选根的实例表抛弃了区间里最新的那条根：H4 的底上，只让 R 认一张最前面多一行 (1, 3) 的实例表
/// （[`point_the_chosen_root_alone_at_an_instance_table_abandoning_v`]：改好的那一片写进基镜像里流从没写过的槽，R 的根记录与记录新根段改指它）——
/// 按 R 的实例表，实例 1 里 txg > 3 的根（V 与 R）在被抛弃的时间线上；V⁻、V 认的仍是原来那张表，V 自己照样有效。
/// R 走读失败、c_见证 落回 V⁻ 的状态上，F2_NEWEST 在 [V⁻, R) 里按 R 的实例表滤掉 V、落 V⁻。
/// CPU 看 journal 那一遍至少一个状态退到 V⁻、一个状态都不退到 V，GPU 逐位相同。GPU 恢复内核读得出所选根的实例表而不按它滤时落 V，这条红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_to_a_version_the_instance_table_of_the_chosen_root_abandons(
) {
    let mut flow = record_two_overwrites_whose_second_tree_table_does_not_read_back_and_whose_first_rotation_reads_back_as_zeros(
        "crash-judge-gpu-f2-newest-abandoned-by-the-chosen-instance-table",
        "two-overwrites-second-tree-table-unreadable-first-rotation-zeroed-instance-table-abandons-v",
    );
    point_the_chosen_root_alone_at_an_instance_table_abandoning_v(&mut flow);
    assert_the_gpu_judge_agrees_with_the_cpu_falling_back_to(
        "the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_to_a_version_the_instance_table_of_the_chosen_root_abandons",
        &flow,
        V_MINUS_OF_THE_F2_FLOWS,
        V_OF_THE_F2_FLOWS,
    );
}

/// 所选根的实例表读不出：H4 的底上，R 的根槽写里实例表指针两条位置条目的整单元校验和各翻一位、重封根记录（V⁻、V 的根记录不动，它们指着的表照样读得出）。
/// R 走读失败、c_见证 落回 V⁻ 的状态上，F2_NEWEST 只认与见证到的那一版恰好相等的那条根、落 V⁻。CPU 看 journal 那一遍至少一个状态退到 V⁻、
/// 一个状态都不退到 V，GPU 逐位相同。GPU 恢复内核读不出所选根的实例表也照取最新时落 V，这条红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_recovery_whose_chosen_root_instance_table_does_not_read_back_falls_back_only_to_the_witnessed_version(
) {
    let mut flow = record_two_overwrites_whose_second_tree_table_does_not_read_back_and_whose_first_rotation_reads_back_as_zeros(
        "crash-judge-gpu-f2-newest-chosen-instance-table-unreadable",
        "two-overwrites-second-tree-table-unreadable-first-rotation-zeroed-chosen-instance-table-unreadable",
    );
    let root_of_r = root_write_of_txg(&flow, R_OF_THE_F2_FLOWS.1);
    rewrite_root_record_and_reseal(&mut flow, root_of_r, &|bytes| {
        for checksum_offset in POINTER_LOCATION_CHECKSUM_OFFSETS {
            bytes[ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + checksum_offset] ^= 0x01;
        }
    });
    assert_the_gpu_judge_agrees_with_the_cpu_falling_back_to(
        "the_gpu_judge_agrees_with_the_cpu_that_a_recovery_whose_chosen_root_instance_table_does_not_read_back_falls_back_only_to_the_witnessed_version",
        &flow,
        V_MINUS_OF_THE_F2_FLOWS,
        V_OF_THE_F2_FLOWS,
    );
}

// ─── 第 45 批（删除线）：GPU 恢复内核的 F2 不越过一次末条读得出、根读不出的发布（攻方 `research/prompts/m3-batch37-43-code-r1-opus-output.md`
// 第二节 Y4，用户 2026-10-08 定「两形都修」） ───

/// 攻方 Y4 `Ru+cfg+Proot`（攻方的 Pm、P、R 是这里的 V⁻、V、R）：H4 的底上（[`record_two_overwrites_whose_second_tree_table_does_not_read_back_and_whose_first_rotation_reads_back_as_zeros`]：
/// R 的树表两份改坏、V 那次轮换写的系统配置槽写成全零），再把 V 的根槽写自证校验和字段翻一位（V 的根槽读不出）。R 走读失败、c_见证 落回 V⁻ 的状态上，
/// [V⁻, R) 里读得出、不被 R 的实例表抛弃的只有 V⁻；V⁻ 与 R 之间的 V 带末条标志的记录读得出、根读不出、不被 R 的实例表抛弃，退到 V⁻ 就越过已确认的 V——
/// CPU 看 journal 那一遍照报 R 读不出、一个状态都不退到 V⁻；GPU 恢复内核同判，每个状态逐位相同。GPU 不看这样一次发布时落 V⁻、CPU 报读不出，这条红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_past_a_version_whose_last_record_reads_while_its_root_does_not(
) {
    let test_name = "the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_past_a_version_whose_last_record_reads_while_its_root_does_not";
    let mut flow = record_two_overwrites_whose_second_tree_table_does_not_read_back_and_whose_first_rotation_reads_back_as_zeros(
        "crash-judge-gpu-f2-no-skip-of-a-version-whose-root-does-not-read-back",
        "two-overwrites-second-tree-table-unreadable-first-rotation-zeroed-first-root-unreadable",
    );
    let root_of_v = root_write_of_txg(&flow, V_OF_THE_F2_FLOWS.1);
    let WrittenContents::Bytes(root_bytes) = &mut flow.writes[root_of_v].contents else {
        panic!("根槽写是普通写");
    };
    root_bytes[ROOT_RECORD_CHECKSUM_FIELD_OFFSET] ^= 0x01;
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    let read_failed = consulted_oracle_item(CrashPointReplayOracleViolationKind::ReadFailed);
    let states_reporting_r_unreadable = (0..comparison.cpu_red_items_of_each_state.len())
        .filter(|ordinal| {
            comparison.cpu_red_items_of_each_state[*ordinal].contains(&read_failed)
                && comparison.cpu_consulted_reported_root_of_each_state[*ordinal]
                    == Some(R_OF_THE_F2_FLOWS)
                && consulted_recovery_lands_on(
                    &comparison,
                    *ordinal,
                    R_OF_THE_F2_FLOWS,
                    RECOVERY_WORD_CLASS_FAILED,
                )
        })
        .count();
    let states_falling_back_to_v_minus =
        states_where_the_consulted_recovery_falls_back_to(&comparison, V_MINUS_OF_THE_F2_FLOWS);
    eprintln!(
        "CRASH_JUDGE_GPU_F2 flow={} states={} states_reporting_r_unreadable={states_reporting_r_unreadable} states_falling_back_to_v_minus={states_falling_back_to_v_minus}",
        flow.name, comparison.states
    );
    assert!(
        states_reporting_r_unreadable > 0,
        "流 {}：R 走读失败、c_见证 落回 V⁻、V 的根读不出的状态上，CPU 看 journal 那一遍报读不出、报的根是 R {R_OF_THE_F2_FLOWS:?}",
        flow.name
    );
    assert_eq!(
        states_falling_back_to_v_minus, 0,
        "流 {}：一个状态都不退到 V⁻ {V_MINUS_OF_THE_F2_FLOWS:?}（V 末条读得出、根读不出，退过去就越过它）",
        flow.name
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// 系统配置槽里 journal tail 的偏移（`system_configuration.rs` 的 TAIL_OFFSET，8 字节）。
const SYSTEM_CONFIGURATION_JOURNAL_TAIL_OFFSET: usize = 469;
/// journal 记录的字节数（一个记录槽）。
const JOURNAL_RECORD_SLOT_BYTES: u64 = 4096;
/// journal 环的起点（`JOURNAL_RING_START_SLOT` × 16 KiB = 16 MiB）。
const JOURNAL_RING_START_BYTES: u64 = 1024 * 16384;
/// 伪造的那条记录放在同一块盘上流里最后一个记录槽之后第几个槽：离开流里写过的槽，仍在环里。
const RECORD_SLOTS_PAST_THE_LAST_RECORD_OF_THE_FLOW: u64 = 8;

/// c_见证 = 0 而环里有一条计数器 0 的记录（判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 K1 V1 行「建模差两处」之二，
/// 正推 `research/prompts/m3-batch35-36-code-r1-sonnet-output.md` V1 那一格）：
/// F2 设计那一退的底上，流里每一次系统配置槽写的 journal tail 改成 0、重封（c_见证 恒为 0）；流尾另起一段，写一条 V 那次发布末条记录的副本，
/// 计数器改成 0、重封，放在同一块盘上流里最后一个记录槽之后（写者写不出计数器 0 的记录，这是伪造的）。R 走读失败的状态上，那一段落没落，
/// CPU 与 GPU 都按 D23（journal 的角色与格式） 已定项 14「c_见证 = 0 … 都不退」不退、照报 R 读不出。
fn record_two_overwrites_whose_second_tree_table_does_not_read_back_with_a_witnessed_counter_of_zero_naming_v(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let mut flow = record_two_overwrites_whose_second_tree_table_does_not_read_back(tag, name);
    let system_configuration_writes = flow
        .writes
        .iter()
        .filter(|write| write.kind == StepKind::SystemConfigurationSlot)
        .count();
    for nth_from_the_end in 0..system_configuration_writes {
        rewrite_and_reseal(
            &mut flow,
            StepKind::SystemConfigurationSlot,
            nth_from_the_end,
            4096,
            SYSTEM_CONFIGURATION_CHECKSUM_FIELD,
            &|bytes| {
                bytes[SYSTEM_CONFIGURATION_JOURNAL_TAIL_OFFSET
                    ..SYSTEM_CONFIGURATION_JOURNAL_TAIL_OFFSET + 8]
                    .fill(0);
            },
        );
    }
    let is_the_last_record_of_v = |write: &RetainedWrite| {
        write.kind == StepKind::JournalRecord
            && write.bytes().is_some_and(|bytes| {
                u32::from_le_bytes(bytes[16..20].try_into().expect("4 字节")) == V_OF_THE_F2_FLOWS.0
                    && u64::from_le_bytes(bytes[26..34].try_into().expect("8 字节"))
                        == V_OF_THE_F2_FLOWS.1
                    && bytes[7] & 1 == 1
            })
    };
    let last_record_of_v = flow
        .writes
        .iter()
        .rposition(is_the_last_record_of_v)
        .expect("流里有 V 那次发布带末条标志的记录");
    let device = flow.writes[last_record_of_v].device;
    let mut forged_record = flow.writes[last_record_of_v]
        .bytes()
        .expect("记录写带字节")
        .to_vec();
    forged_record[JOURNAL_RECORD_COUNTER_OFFSET..JOURNAL_RECORD_COUNTER_OFFSET + 6].fill(0);
    reseal_journal_record(&mut forged_record);
    let last_record_slot_of_the_flow = flow
        .writes
        .iter()
        .filter(|write| write.kind == StepKind::JournalRecord && write.device == device)
        .map(|write| write.offset.0)
        .max()
        .expect("这块盘上有记录写");
    let forged_record_offset = last_record_slot_of_the_flow
        + RECORD_SLOTS_PAST_THE_LAST_RECORD_OF_THE_FLOW * JOURNAL_RECORD_SLOT_BYTES;
    assert!(
        forged_record_offset + JOURNAL_RECORD_SLOT_BYTES
            <= JOURNAL_RING_START_BYTES + geometry().journal_ring_bytes,
        "伪造的记录在环里"
    );
    append_a_segment_of_writes(
        &mut flow,
        vec![RetainedWrite {
            device,
            kind: StepKind::JournalRecord,
            is_force_unit_access: false,
            offset: singlefs_core::address::DeviceOffsetInBytes(forged_record_offset),
            contents: WrittenContents::Bytes(forged_record),
        }],
    );
    flow
}

/// c_见证 = 0 而环里有一条计数器 0 的记录（底见 [`record_two_overwrites_whose_second_tree_table_does_not_read_back_with_a_witnessed_counter_of_zero_naming_v`]）：
/// CPU 与 GPU 恢复内核都对 c_见证 = 0 设守卫、不退（D23（journal 的角色与格式） 已定项 14「c_见证 = 0 … 都不退」）。CPU 看 journal 那一遍至少一个状态
/// 照报 R 读不出、一个状态都不退到 V 或 V⁻，GPU 逐位相同。GPU 去掉守卫（找计数器 = 0 的那条记录、退到 V）时与 CPU 不一致，这条红；
/// CPU 去掉守卫时伪造那一段落了的状态退到 V，这条也红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_witnessed_counter_of_zero_does_not_fall_back_even_to_the_version_of_a_record_carrying_counter_zero(
) {
    let test_name = "the_gpu_judge_agrees_with_the_cpu_that_a_witnessed_counter_of_zero_does_not_fall_back_even_to_the_version_of_a_record_carrying_counter_zero";
    let flow = record_two_overwrites_whose_second_tree_table_does_not_read_back_with_a_witnessed_counter_of_zero_naming_v(
        "crash-judge-gpu-f2-witnessed-counter-zero",
        "two-overwrites-second-tree-table-unreadable-every-tail-zero-forged-record-of-counter-zero",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    let read_failed = consulted_oracle_item(CrashPointReplayOracleViolationKind::ReadFailed);
    let states_reporting_r_unreadable = (0..comparison.cpu_red_items_of_each_state.len())
        .filter(|ordinal| {
            comparison.cpu_red_items_of_each_state[*ordinal].contains(&read_failed)
                && comparison.cpu_consulted_reported_root_of_each_state[*ordinal]
                    == Some(R_OF_THE_F2_FLOWS)
                && consulted_recovery_lands_on(
                    &comparison,
                    *ordinal,
                    R_OF_THE_F2_FLOWS,
                    RECOVERY_WORD_CLASS_FAILED,
                )
        })
        .count();
    let states_falling_back_to_v =
        states_where_the_consulted_recovery_falls_back_to(&comparison, V_OF_THE_F2_FLOWS);
    let states_falling_back_to_v_minus =
        states_where_the_consulted_recovery_falls_back_to(&comparison, V_MINUS_OF_THE_F2_FLOWS);
    eprintln!(
        "CRASH_JUDGE_GPU_F2 flow={} states={} states_reporting_r_unreadable={states_reporting_r_unreadable} states_falling_back_to_v={states_falling_back_to_v} states_falling_back_to_v_minus={states_falling_back_to_v_minus}",
        flow.name, comparison.states
    );
    assert!(
        states_reporting_r_unreadable > 0,
        "流 {}：R 走读失败、c_见证 = 0 的状态上 CPU 看 journal 那一遍报读不出、报的根是 R {R_OF_THE_F2_FLOWS:?}",
        flow.name
    );
    assert_eq!(
        (states_falling_back_to_v, states_falling_back_to_v_minus),
        (0, 0),
        "流 {}：c_见证 = 0 不退，一个状态都不退到 V {V_OF_THE_F2_FLOWS:?} 或 V⁻ {V_MINUS_OF_THE_F2_FLOWS:?}",
        flow.name
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// extent 树的种类码（`singlefs_core::records::TREE_KIND_EXTENT`）。
const TREE_KIND_OF_THE_EXTENT_TREE: u16 = 1;

/// H-S1-fold 的 A 臂（判决 `research/prompts/m3-batch33s-code-r1-main-verification.md` 第二节 S1 行与第三节第 1 条；攻方
/// `research/prompts/m3-batch33s-code-r1-opus-output.md` 第一节 1.2 `A-newer-version-pointer-garbled`，录成流）：删除中间态的流上
/// 意图那一版（txg 5，extent 树照抄 txg 4 的）树表里 extent 树根指针第 0 条位置条目的整单元校验和异或几位（单元本身不动），重封那张树表、沿引用链补到根槽：
/// 意图那一版读 extent 树根时第 0 条那一份对不上、读回第 1 条那一份（I-3.5 的折叠表把那个落点改记成它自己）。
fn record_deletion_whose_intent_version_reads_the_shared_extent_root_from_its_second_copy(
    tag: &str,
) -> RecordedFlow {
    const TXG_OF_THE_INTENT: u64 = 5;
    let name = "deletion-intent-version-reads-the-shared-extent-root-from-its-second-copy";
    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
    let root_of_the_intent = root_write_of_txg(&flow, TXG_OF_THE_INTENT);
    let tree_table_copies =
        copies_of_the_tree_table_named_by_the_root_write(&flow, root_of_the_intent);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &tree_table_copies, &|bytes| {
        let checksum = only_tree_table_entry_of_kind(bytes, TREE_KIND_OF_THE_EXTENT_TREE)
            + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET
            + POINTER_LOCATION_CHECKSUM_OFFSETS[0];
        for byte in &mut bytes[checksum..checksum + 4] {
            *byte ^= 0x5a;
        }
    });
    flow
}

/// H-S1-fold A 臂：CPU 池级 checker 判 I-3.5 时按全部版本走完之后的折叠表取每一版的单元（攻方 F1），txg 4 那一版先走、经第 0 条读回的
/// 那个落点在意图那一版读回第 1 条之后同算一个单元，意图那一版不算新引用它，I-3.5 一个状态都不红；第 0 条那一份照判 I-2.1 红。GPU 走读内核
/// 本来就按最终的落点表认，每个状态逐位相同（CPU 每版走完当场折时多红 I-3.5，随根环次序翻面）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_an_intent_version_reading_the_shared_extent_root_from_its_second_copy_does_not_redden_the_appearance_rule(
) {
    let flow =
        record_deletion_whose_intent_version_reads_the_shared_extent_root_from_its_second_copy(
            "crash-judge-gpu-i35-intent-reads-the-shared-extent-root-from-its-second-copy",
        );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_that_an_intent_version_reading_the_shared_extent_root_from_its_second_copy_does_not_redden_the_appearance_rule",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-2.1")
            && !comparison.cpu_checker_red_invariants.contains("I-3.5"),
        "流 {} 上 CPU 红 I-2.1、一个状态都不红 I-3.5（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// 三个数据单元那条流（txg 4）下段叶第一条记录的数据指针：第 0 条位置条目的槽写成 `slot`、整单元校验和写成 `checksum`（盘不动），
/// 重封那片叶、沿引用链补到根。
fn redirect_the_first_location_of_the_first_lower_leaf_data_pointer(
    flow: &mut RecordedFlow,
    name: &str,
    slot: u64,
    checksum: u32,
) {
    let extent_lower_leaf = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && usize::from(index_node_key_width(bytes)) == EXTENT_KEY_BYTES
            && index_node_entry_width(bytes) == EXTENT_LOWER_LEAF_RECORD_BYTES
    };
    let leaf_copies = last_unit_copies(flow, name, &extent_lower_leaf, false);
    corrupt_units_and_reseal_up_to_the_root(flow, &leaf_copies, &|bytes| {
        let location = index_node_entries_start(EXTENT_KEY_BYTES)
            + EXTENT_KEY_BYTES
            + POINTER_LOCATION_ENTRY_OFFSETS[0];
        bytes[location + LOCATION_ENTRY_SLOT_OFFSET
            ..location + LOCATION_ENTRY_SLOT_OFFSET + LOCATION_ENTRY_SLOT_BYTES]
            .copy_from_slice(&slot.to_le_bytes()[..LOCATION_ENTRY_SLOT_BYTES]);
        bytes[location + LOCATION_ENTRY_UNIT_CHECKSUM_OFFSET
            ..location + LOCATION_ENTRY_UNIT_CHECKSUM_OFFSET + 4]
            .copy_from_slice(&checksum.to_le_bytes());
    });
}

/// 两块盘的基镜像在槽 `slot` 写进 `bytes`（流里谁都没写过它，每个状态都在）。
fn write_into_the_base_on_both_devices(flow: &mut RecordedFlow, slot: u64, bytes: &[u8]) {
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        flow.base.devices.get_mut(&device).expect("盘在池里").write(
            singlefs_core::address::DeviceOffsetInBytes(slot * SLOT_BYTES),
            bytes,
        );
    }
}

/// 拼读那三形下一槽（[`SLOT_NOTHING_IN_THE_FLOW_WROTE`] + 1）在基镜像里写的 16 KiB。
fn bytes_of_the_slot_after_the_gap() -> Vec<u8> {
    vec![0x5au8; usize::try_from(SLOT_BYTES).expect("16384")]
}

/// 「16 KiB 全零接下一槽那 16 KiB」那 32 KiB 的 CRC-32C：空段最后一槽按 32 KiB 读，CPU 的稀疏读者读出的就是它。
fn checksum_of_zeros_followed_by_the_slot_after_the_gap() -> u32 {
    let mut thirty_two_kibibytes = vec![0u8; usize::try_from(SLOT_BYTES).expect("16384")];
    thirty_two_kibibytes.extend_from_slice(&bytes_of_the_slot_after_the_gap());
    crc32c(&thirty_two_kibibytes)
}

/// H-S2-tail（判决 `research/prompts/m3-batch33s-code-r1-main-verification.md` 第二节 S2 行、第三节第 1 条；攻方 `research/prompts/m3-batch33s-code-r1-opus-output.md`
/// 第二节 2.2 `S2-crc-of-zeros-then-next-slot`，录成流）：三个数据单元的流上，基镜像在 [`SLOT_NOTHING_IN_THE_FLOW_WROTE`] + 1 写进 16 KiB 非零字节，
/// txg 4 那一版下段叶第一条数据指针第 0 条位置条目改指空段右端那一槽 [`SLOT_NOTHING_IN_THE_FLOW_WROTE`]、校验和写成「16 KiB 全零接下一槽」的 CRC。
fn record_three_unit_file_whose_first_data_pointer_names_the_last_slot_of_a_gap(
    tag: &str,
) -> RecordedFlow {
    let name = "three-unit-file-first-data-pointer-naming-the-last-slot-of-a-gap";
    let mut flow = record_new_pool_through_a_three_unit_file(tag, name);
    assert_nothing_in_the_flow_wrote_the_slot(&flow, SLOT_NOTHING_IN_THE_FLOW_WROTE);
    write_into_the_base_on_both_devices(
        &mut flow,
        SLOT_NOTHING_IN_THE_FLOW_WROTE + 1,
        &bytes_of_the_slot_after_the_gap(),
    );
    redirect_the_first_location_of_the_first_lower_leaf_data_pointer(
        &mut flow,
        name,
        SLOT_NOTHING_IN_THE_FLOW_WROTE,
        checksum_of_zeros_followed_by_the_slot_after_the_gap(),
    );
    flow
}

/// GPU 内核 read_at 拼读那一读的整段 CRC 对得上位置条目时记的判不了原因（`crash_judge_common.wgsl` 的 `UNDECIDABLE_SPLICED_READ_MATCHES`）。
const GPU_UNDECIDABLE_SPLICED_READ_MATCHES: u32 = 7;

/// H-S2-tail：CPU 的稀疏读者从空段右端那一槽读 32 KiB，读出「16 KiB 全零 + 下一槽」、CRC 对得上，第 0 条那一份当数据单元读、往下判它的头。
/// GPU 读者 `read_at` 拼出整段 CRC（全零 16 KiB 那一条的版本表第 8 字异或下一个落点那一版前 16 KiB 的 CRC），对得上之后往下要读的字节跨过落点末尾、
/// 表里没有，池级 checker 与恢复内核都记判不了（[`GPU_UNDECIDABLE_SPLICED_READ_MATCHES`]），照流水线那样回落 CPU 之后每个状态的红项相同；
/// 判了的状态逐位相同。拼读之前 GPU 守卫拒这一读、判第 0 条读不出，多红 I-2.1。
#[test]
fn the_gpu_judge_leaves_to_the_cpu_a_data_pointer_whose_first_location_names_the_last_slot_of_a_gap_with_the_checksum_of_zeros_and_the_next_slot(
) {
    assert_the_gpu_judge_leaves_to_the_cpu_only_states_undecidable_for(
        "the_gpu_judge_leaves_to_the_cpu_a_data_pointer_whose_first_location_names_the_last_slot_of_a_gap_with_the_checksum_of_zeros_and_the_next_slot",
        &record_three_unit_file_whose_first_data_pointer_names_the_last_slot_of_a_gap(
            "crash-judge-gpu-s2-tail-data-pointer-last-slot-of-a-gap",
        ),
        &[GPU_UNDECIDABLE_SPLICED_READ_MATCHES],
        // 恢复内核读第一个文件时同一处拼读记同一个原因（两个内核共用 `crash_judge_common.wgsl` 的 read_at 与这个常量）
        &[GPU_UNDECIDABLE_SPLICED_READ_MATCHES],
        &[GPU_UNDECIDABLE_SPLICED_READ_MATCHES],
    );
}

/// 正推「16 KiB 节点加下一槽」那一形（`research/prompts/m3-batch33s-code-r1-sonnet-output.md` K1-S2 末段，录成流）：三个数据单元的流上，基镜像在
/// [`SLOT_NOTHING_IN_THE_FLOW_WROTE`] 写进 txg 4 那片下段叶（码 2、16 KiB）的原样字节、下一槽写进 16 KiB 非零字节，下段叶第一条数据指针第 0 条位置条目
/// 改指那一槽、校验和写成「节点 16 KiB 接下一槽」那 32 KiB 的 CRC。
fn record_three_unit_file_whose_first_data_pointer_names_a_node_followed_by_a_written_slot(
    tag: &str,
) -> RecordedFlow {
    let name = "three-unit-file-first-data-pointer-naming-a-node-followed-by-a-written-slot";
    let mut flow = record_new_pool_through_a_three_unit_file(tag, name);
    assert_nothing_in_the_flow_wrote_the_slot(&flow, SLOT_NOTHING_IN_THE_FLOW_WROTE);
    let extent_lower_leaf = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && usize::from(index_node_key_width(bytes)) == EXTENT_KEY_BYTES
            && index_node_entry_width(bytes) == EXTENT_LOWER_LEAF_RECORD_BYTES
    };
    let node = flow.writes[last_unit_copies(&flow, name, &extent_lower_leaf, false)[0]]
        .bytes()
        .expect("单元写带字节")
        .to_vec();
    assert_eq!(
        u64::try_from(node.len()).expect("单元长装得进 u64"),
        SLOT_BYTES,
        "码 2 节点 16 KiB"
    );
    write_into_the_base_on_both_devices(&mut flow, SLOT_NOTHING_IN_THE_FLOW_WROTE, &node);
    write_into_the_base_on_both_devices(
        &mut flow,
        SLOT_NOTHING_IN_THE_FLOW_WROTE + 1,
        &bytes_of_the_slot_after_the_gap(),
    );
    let mut node_followed_by_the_next_slot = node;
    node_followed_by_the_next_slot.extend_from_slice(&bytes_of_the_slot_after_the_gap());
    redirect_the_first_location_of_the_first_lower_leaf_data_pointer(
        &mut flow,
        name,
        SLOT_NOTHING_IN_THE_FLOW_WROTE,
        crc32c(&node_followed_by_the_next_slot),
    );
    flow
}

/// 「16 KiB 节点加下一槽」：CPU 从那一槽读 32 KiB，CRC 对得上，第 0 条那一份当数据单元读（头是码 2，红）。GPU 不用拼：单元槽落点本来就是 32 KiB 长
/// （`crash_judge_tables` 的落点表，盘尾那一槽才 16 KiB），版本字节的后 16 KiB 就是基镜像里的下一槽，整段 CRC 照表比、往下照读，每个状态逐位相同，
/// 一个状态都不记判不了。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_data_pointer_names_a_node_with_the_checksum_of_the_node_and_the_next_slot(
) {
    let flow =
        record_three_unit_file_whose_first_data_pointer_names_a_node_followed_by_a_written_slot(
            "crash-judge-gpu-s2-node-followed-by-a-written-slot",
        );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_when_a_data_pointer_names_a_node_with_the_checksum_of_the_node_and_the_next_slot",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red > 0,
        "流 {}：第 0 条那一份当数据单元读、头是码 2，CPU 池级 checker 要有红",
        flow.name
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致或判不了（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// 正推「空槽加下一已写槽前 16 KiB」那一形（同上 K1-S2 末段；第 32 批攻方表里的 `zero16+next`，录成流）：删除中间态的流上墓碑树根第一条条目类型段 1 → 7、
/// 子指针第 1 条位置条目指 [`SLOT_NOTHING_IN_THE_FLOW_WROTE`]、整单元校验和写成「16 KiB 全零接下一槽」的 CRC，下一槽在两块盘的基镜像里写进 16 KiB 非零字节。
fn record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_naming_the_last_slot_of_a_gap_with_the_checksum_of_zeros_and_the_next_slot(
    tag: &str,
) -> RecordedFlow {
    let name =
        "deletion-tombstone-root-entry-type-seven-second-location-naming-the-last-slot-of-a-gap";
    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
    assert_nothing_in_the_flow_wrote_the_slot(&flow, SLOT_NOTHING_IN_THE_FLOW_WROTE);
    write_into_the_base_on_both_devices(
        &mut flow,
        SLOT_NOTHING_IN_THE_FLOW_WROTE + 1,
        &bytes_of_the_slot_after_the_gap(),
    );
    let checksum = checksum_of_zeros_followed_by_the_slot_after_the_gap();
    let root_copies = last_unit_copies(&flow, name, &is_the_tombstone_tree_root, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
        set_the_type_of_the_first_entry(bytes, UNREGISTERED_CONTAINER_TREE_ENTRY_TYPE);
        set_the_slot_and_checksum_of_a_location_in_the_first_entry(
            bytes,
            POINTER_LOCATION_ENTRY_OFFSETS[1],
            SLOT_NOTHING_IN_THE_FLOW_WROTE,
            checksum,
        );
    });
    flow
}

/// 「空槽加下一已写槽前 16 KiB」：CPU 头里类标签 0 不登记，按 32 KiB 读出「16 KiB 零 + 下一槽」、对得上，那一份 I-2.1 成立，红 I-9.23。
/// GPU F-α 那一处比 32 KiB 时拿 `read_at` 拼出的整段 CRC 比（只比 CRC、不往下读，不记判不了），每个状态逐位相同（拼读之前那一读读不到、
/// 多红 I-2.1、I-4.8、I-7.4）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_the_last_slot_of_a_gap_with_the_checksum_of_zeros_and_the_next_slot(
) {
    let flow = record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_naming_the_last_slot_of_a_gap_with_the_checksum_of_zeros_and_the_next_slot(
        "crash-judge-gpu-s2-tombstone-entry-type-seven-last-slot-of-a-gap",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_the_last_slot_of_a_gap_with_the_checksum_of_zeros_and_the_next_slot",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-9.23")
            && !comparison.cpu_checker_red_invariants.contains("I-2.1"),
        "流 {} 上 CPU 红 I-9.23、一个状态都不红 I-2.1（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致或判不了（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

// ───────────── 第 56 批 G2、G3、G6：容器号从 1 起、逐片加一、每片至少一条；旋钮值编进 GPU 扫描内核（I-9.21、I-9.22、I-9.36、I-9.37）─────────────

/// 打包记录单元头里的容器号（类身份段：出生树 43、打包记录类型 51、容器号 53、容器出生代 61）。
const PACKED_UNIT_CONTAINER_NUMBER_OFFSET_IN_THE_HEADER: usize = 53;

/// 只这一棵容器树装 `records_per_container` 条一片的旋钮，另两棵当没装（交给 GPU 扫描内核与 CPU 新入口的同一组旋钮值）。
fn knobs_of_the_lineage_tree(records_per_container: u64) -> ContainerCapacityKnobs {
    ContainerCapacityKnobs {
        tombstone: None,
        lineage: Some(LineageContainerCapacityKnob::new(records_per_container).expect("1..=1359")),
        retirement: None,
    }
}

fn knobs_of_the_retirement_tree(records_per_container: u64) -> ContainerCapacityKnobs {
    ContainerCapacityKnobs {
        tombstone: None,
        lineage: None,
        retirement: Some(
            RetirementContainerCapacityKnob::new(records_per_container).expect("1..=959"),
        ),
    }
}

fn knobs_of_the_tombstone_tree(records_per_container: u64) -> ContainerCapacityKnobs {
    ContainerCapacityKnobs {
        tombstone: Some(
            TombstoneContainerCapacityKnob::new(records_per_container).expect("1..=815"),
        ),
        lineage: None,
        retirement: None,
    }
}

/// 带旋钮值的那一条（墓碑树 I-9.22，谱系树与退役树 I-9.37）。
const KNOB_CAPACITY_INVARIANT_OF_THE_TOMBSTONE_TREE: &str =
    singlefs_checker::image::TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE;
const KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES: &str =
    singlefs_checker::image::LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE;

/// 旋钮下合法的流：GPU 扫描内核编进 `knobs`，每个状态的池级 checker 违例位与 CPU 新入口带同一组旋钮值逐位相同；
/// CPU 一个状态都不红 `knob_capacity_invariant`、至少一个状态上判过而成立（旋钮那一支真走到了）。
fn assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_legal_flow(
    test_name: &str,
    flow: &RecordedFlow,
    knobs: ContainerCapacityKnobs,
    knob_capacity_invariant: &'static str,
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) =
        compare_on_every_state_given_the_container_capacity_knobs(cards, flow, knobs)
    else {
        return;
    };
    assert!(
        !comparison
            .cpu_checker_red_invariants
            .contains(knob_capacity_invariant)
            && comparison
                .cpu_checker_held_invariants
                .contains(knob_capacity_invariant),
        "流 {} 上 CPU 新入口带旋钮值：{knob_capacity_invariant} 判过而成立、一个状态都不红（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}（GPU 扫描内核编进旋钮值）：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// 旋钮下坏的流：同上，CPU 新入口要红 `knob_capacity_invariant`，GPU 红在同样的状态、同样的位上。
fn assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
    test_name: &str,
    flow: &RecordedFlow,
    knobs: ContainerCapacityKnobs,
    knob_capacity_invariant: &'static str,
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) =
        compare_on_every_state_given_the_container_capacity_knobs(cards, flow, knobs)
    else {
        return;
    };
    assert!(
        comparison
            .cpu_checker_red_invariants
            .contains(knob_capacity_invariant),
        "流 {} 上 CPU 新入口带旋钮值要红 {knob_capacity_invariant}（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}（GPU 扫描内核编进旋钮值）：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

/// G6：退役树两片各 1 条（[`record_two_leaves_retired_with_one_retirement_record_per_container`]），GPU 扫描内核编进退役旋钮 1：
/// 旋钮下的合法流，I-9.37 两边都判过而成立、一个状态都不红（CPU 产品入口在这条流上红 I-9.36，另一条用例钉）。
#[test]
fn the_gpu_judge_given_the_retirement_knob_agrees_with_the_cpu_on_two_retirement_containers_of_one_record(
) {
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_legal_flow(
        "the_gpu_judge_given_the_retirement_knob_agrees_with_the_cpu_on_two_retirement_containers_of_one_record",
        &record_two_leaves_retired_with_one_retirement_record_per_container(
            "crash-judge-gpu-retirement-knob-two-containers",
        ),
        knobs_of_the_retirement_tree(1),
        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
    );
}

/// G6：谱系树两片各 1 条（[`record_two_rollbacks_with_one_lineage_record_per_container`]），GPU 扫描内核编进谱系旋钮 1：合法流。
#[test]
fn the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_on_two_lineage_containers_of_one_record(
) {
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_legal_flow(
        "the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_on_two_lineage_containers_of_one_record",
        &record_two_rollbacks_with_one_lineage_record_per_container(
            "crash-judge-gpu-lineage-knob-two-containers",
        ),
        knobs_of_the_lineage_tree(1),
        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
    );
}

/// G6：墓碑树两片各 1 条（[`record_two_empty_inodes_unlinked_with_one_tombstone_per_container`]），GPU 扫描内核编进墓碑旋钮 1：合法流，
/// I-9.22 两边都判过而成立（CPU 产品入口在这条流上红 I-9.21）。
#[test]
fn the_gpu_judge_given_the_tombstone_knob_agrees_with_the_cpu_on_two_tombstone_containers_of_one_record(
) {
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_legal_flow(
        "the_gpu_judge_given_the_tombstone_knob_agrees_with_the_cpu_on_two_tombstone_containers_of_one_record",
        &record_two_empty_inodes_unlinked_with_one_tombstone_per_container(
            "crash-judge-gpu-tombstone-knob-two-containers",
        ),
        knobs_of_the_tombstone_tree(1),
        KNOB_CAPACITY_INVARIANT_OF_THE_TOMBSTONE_TREE,
    );
}

/// 新池新建文件、建 1 个空 inode（txg 4），分配器装上谱系容器一片 `lineage_records_per_container` 条的旋钮，挂着回退到第一个文件那一版
/// （txg 3）`rollbacks` 次（同 [`record_two_rollbacks_with_one_lineage_record_per_container`]）。
fn record_rollbacks_under_the_lineage_knob(
    tag: &str,
    name: &'static str,
    lineage_records_per_container: u64,
    rollbacks: usize,
) -> RecordedFlow {
    let pool = pool_rolled_back_under_these_lineage_knobs(
        tag,
        &vec![lineage_records_per_container; rollbacks],
    );
    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
    recorded_flow_of_the_pool(&pool, name, versions)
}

/// 新池新建文件、建 1 个空 inode（txg 4），挂着回退到第一个文件那一版（txg 3），每次回退之前分配器换上谱系容器一片
/// `lineage_records_per_container_of_each_rollback` 里那一项条的旋钮（回退的发布路径照分配器上的开关取容量）：旋钮可以在中途换值（第 58 批 J3）。
fn pool_rolled_back_under_these_lineage_knobs(
    tag: &str,
    lineage_records_per_container_of_each_rollback: &[u64],
) -> BuiltPool {
    let mut pool = build_pool(tag);
    let first_file_version = pool.output.clone();
    common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    // 迭代上界：切片的长度。
    for lineage_records_per_container in lineage_records_per_container_of_each_rollback {
        pool.allocator
            .set_deletion_switches(singlefs_core::unlink::DeletionSwitches {
            batch_size: UnlinkBatchSize::OneExtentLowerLeaf,
            tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
            lineage_container_capacity: LineageContainerCapacity::CappedForTests(
                singlefs_core::lineage_tree::LineageRecordsPerContainerCappedForTests::new(
                    *lineage_records_per_container,
                )
                .expect("1..=1359"),
            ),
            retirement_container_capacity:
                singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
            container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
        });
        roll_back_to(&mut pool, &first_file_version);
    }
    pool
}

/// G3 ③ 在 GPU 上（I-9.37 多于旋钮值那一判）：谱系容器一片 2 条写出两条谱系记录（一片 2 条），GPU 与 CPU 新入口都给谱系旋钮 1：那片装了 2 条，
/// CPU 红 I-9.37，GPU 同判。
#[test]
fn the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_a_lineage_container_holds_more_records_than_the_knob(
) {
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_a_lineage_container_holds_more_records_than_the_knob",
        &record_rollbacks_under_the_lineage_knob(
            "crash-judge-gpu-lineage-container-past-the-knob",
            "two-rollbacks-two-lineage-records-in-one-container",
            2,
            2,
        ),
        knobs_of_the_lineage_tree(1),
        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
    );
}

fn is_a_container_of_record_type(bytes: &[u8], record_type: u16) -> bool {
    unit_class(bytes) == 3 && packed_record_type(bytes) == record_type
}

/// 容器树根（层级 1）第 `entry_index` 条条目身份引用里的容器号改成 `container_number`（分隔 key 与子指针不动）。
fn set_the_container_number_of_an_entry(
    bytes: &mut [u8],
    entry_index: usize,
    container_number: u64,
) {
    assert!(
        entry_index < usize::from(index_node_entry_count(bytes)),
        "根里有第 {entry_index} 条条目"
    );
    let field = first_entry_of_a_container_number_node()
        + entry_index * usize::from(CONTAINER_TREE_ENTRY_BYTES)
        + CONTAINER_TREE_ENTRY_CONTAINER_NUMBER_OFFSET;
    bytes[field..field + 8].copy_from_slice(&container_number.to_le_bytes());
}

/// 流里最后写的那片 `record_type` 容器（两份副本）头里的容器号与最后写的那棵树根（两份副本）里指它的那一条（第 `entry_index` 条）条目的身份容器号
/// 一起改成 `container_number`，各自重封到根（身份照旧与子头相符）。最后写的那片容器只被最后那几版的根指着、那几版的根就是最后写的那个根。
fn renumber_the_last_container_of_the_flow(
    flow: &mut RecordedFlow,
    record_type: u16,
    entry_index: usize,
    container_number: u64,
) {
    let containers = last_unit_copies(
        flow,
        "最后写的那片容器",
        &|bytes| is_a_container_of_record_type(bytes, record_type),
        false,
    );
    corrupt_units_and_reseal_up_to_the_root(flow, &containers, &|bytes| {
        bytes[PACKED_UNIT_CONTAINER_NUMBER_OFFSET_IN_THE_HEADER
            ..PACKED_UNIT_CONTAINER_NUMBER_OFFSET_IN_THE_HEADER + 8]
            .copy_from_slice(&container_number.to_le_bytes());
    });
    let roots = last_unit_copies(
        flow,
        "最后写的那棵容器树根",
        &|bytes| is_a_container_tree_root_over_containers_of_record_type(bytes, record_type),
        false,
    );
    corrupt_units_and_reseal_up_to_the_root(flow, &roots, &|bytes| {
        set_the_container_number_of_an_entry(bytes, entry_index, container_number);
    });
}

/// G3 ①（GPU 那一份）：谱系树两片各 1 条，第二片容器与根里指它的条目的容器号 2 → 1（两片同号，攻方 `mut-w-same`）。GPU 与 CPU 新入口都给谱系旋钮 1：
/// 记录数都在旋钮之内，CPU 只因「相邻两片加一」红 I-9.37，GPU 同判。
#[test]
fn the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_two_lineage_containers_share_a_number(
) {
    let mut flow = record_rollbacks_under_the_lineage_knob(
        "crash-judge-gpu-lineage-containers-share-a-number",
        "two-rollbacks-lineage-containers-numbered-one-and-one",
        1,
        2,
    );
    renumber_the_last_container_of_the_flow(&mut flow, PACKED_RECORD_TYPE_LINEAGE, 1, 1);
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_two_lineage_containers_share_a_number",
        &flow,
        knobs_of_the_lineage_tree(1),
        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
    );
}

/// G3 ①（GPU 那一份）：退役树两片各 1 条，第二片的容器号 2 → 1（两片同号）。GPU 与 CPU 新入口都给退役旋钮 1，CPU 红 I-9.37，GPU 同判。
#[test]
fn the_gpu_judge_given_the_retirement_knob_agrees_with_the_cpu_when_two_retirement_containers_share_a_number(
) {
    let mut flow = record_two_leaves_retired_with_one_retirement_record_per_container(
        "crash-judge-gpu-retirement-containers-share-a-number",
    );
    flow.name = "two-leaves-retired-retirement-containers-numbered-one-and-one";
    renumber_the_last_container_of_the_flow(&mut flow, PACKED_RECORD_TYPE_RETIREMENT, 1, 1);
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_given_the_retirement_knob_agrees_with_the_cpu_when_two_retirement_containers_share_a_number",
        &flow,
        knobs_of_the_retirement_tree(1),
        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
    );
}

/// G2（GPU 那一份，攻方 `mut-w-gap2`）：谱系树两片各 1 条，第二片的容器号 2 → 3（1、3）。CPU 新入口红 I-9.37（不是逐片加一），GPU 同判。
#[test]
fn the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_lineage_containers_skip_a_number()
{
    let mut flow = record_rollbacks_under_the_lineage_knob(
        "crash-judge-gpu-lineage-containers-skip-a-number",
        "two-rollbacks-lineage-containers-numbered-one-and-three",
        1,
        2,
    );
    renumber_the_last_container_of_the_flow(&mut flow, PACKED_RECORD_TYPE_LINEAGE, 1, 3);
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_lineage_containers_skip_a_number",
        &flow,
        knobs_of_the_lineage_tree(1),
        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
    );
}

/// G2（GPU 那一份，攻方 `mut-w-start2`）：谱系树两片各 1 条，流里每一片谱系容器头里的容器号与每个谱系树根里每条条目的身份容器号都加 1（2、3）：
/// 回退一次那一版的谱系树只一片、号 2，回退两次那一版两片 2、3。CPU 新入口红 I-9.37（第一片不是 1），GPU 同判。
#[test]
fn the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_lineage_containers_start_from_two()
{
    let mut flow = record_rollbacks_under_the_lineage_knob(
        "crash-judge-gpu-lineage-containers-start-from-two",
        "two-rollbacks-lineage-containers-numbered-two-and-three",
        1,
        2,
    );
    let every_lineage_container: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| {
            write.kind == StepKind::UnitWrite
                && write.bytes().is_some_and(|bytes| {
                    is_a_container_of_record_type(bytes, PACKED_RECORD_TYPE_LINEAGE)
                })
        })
        .map(|(index, _)| index)
        .collect();
    assert_eq!(
        every_lineage_container.len(),
        4,
        "两片谱系容器各写一次、每次两份副本"
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &every_lineage_container, &|bytes| {
        let container_number =
            read_u64_at(bytes, PACKED_UNIT_CONTAINER_NUMBER_OFFSET_IN_THE_HEADER);
        bytes[PACKED_UNIT_CONTAINER_NUMBER_OFFSET_IN_THE_HEADER
            ..PACKED_UNIT_CONTAINER_NUMBER_OFFSET_IN_THE_HEADER + 8]
            .copy_from_slice(&(container_number + 1).to_le_bytes());
    });
    let every_lineage_root: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| {
            write.kind == StepKind::UnitWrite && write.bytes().is_some_and(is_the_lineage_tree_root)
        })
        .map(|(index, _)| index)
        .collect();
    assert_eq!(
        every_lineage_root.len(),
        4,
        "两次回退各写一个谱系树根、每次两份副本"
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &every_lineage_root, &|bytes| {
        // 迭代上界：根里的条目数（这条流里至多 2）。
        for entry_index in 0..usize::from(index_node_entry_count(bytes)) {
            let field = first_entry_of_a_container_number_node()
                + entry_index * usize::from(CONTAINER_TREE_ENTRY_BYTES)
                + CONTAINER_TREE_ENTRY_CONTAINER_NUMBER_OFFSET;
            let container_number = read_u64_at(bytes, field);
            set_the_container_number_of_an_entry(bytes, entry_index, container_number + 1);
        }
    });
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_lineage_containers_start_from_two",
        &flow,
        knobs_of_the_lineage_tree(1),
        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
    );
}

/// G2（GPU 那一份，攻方 `mut-w-eager`）：谱系容器一片 1 条回退三次（三片各 1 条），最后写的那片清空（记录数与声明长度写 0、记录区写 0）：
/// 最右多一片 0 条。CPU 新入口红 I-9.37（每片至少一条），GPU 同判。
#[test]
fn the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_the_rightmost_lineage_container_is_empty(
) {
    let mut flow = record_rollbacks_under_the_lineage_knob(
        "crash-judge-gpu-lineage-rightmost-container-empty",
        "three-rollbacks-rightmost-lineage-container-empty",
        1,
        3,
    );
    let last_container = last_unit_copies(
        &flow,
        "最后写的那片谱系容器",
        &|bytes| is_a_container_of_record_type(bytes, PACKED_RECORD_TYPE_LINEAGE),
        false,
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &last_container, &|bytes| {
        assert_eq!(
            read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET),
            1,
            "最后写的那片谱系容器一条记录"
        );
        bytes[PACKED_UNIT_RECORD_COUNT_OFFSET..PACKED_UNIT_RECORD_COUNT_OFFSET + 2]
            .copy_from_slice(&0u16.to_le_bytes());
        bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2]
            .copy_from_slice(&0u16.to_le_bytes());
        bytes[PACKED_UNIT_RECORDS_START..].fill(0);
    });
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_given_the_lineage_knob_agrees_with_the_cpu_when_the_rightmost_lineage_container_is_empty",
        &flow,
        knobs_of_the_lineage_tree(1),
        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
    );
}

/// 甲二（每段原地写任意组合 × 单元写全不落或全落，判决 m3-crash-enumeration-merge-r2 第二节 L6-2）展开的新池新建文件那条流：
/// GPU 探针内核拆出的持久掩码与 CPU 判表（`crash_facts` 那一套拆法）、crash.rs 枚举（`judge_crash_state_range` 交给回调的镜像）逐序号相同，
/// 判器三个内核在每个状态上判出的对拍字与 CPU 判器逐状态相同。
#[test]
fn the_gpu_judge_decodes_and_judges_in_place_subsets_with_copy_on_write_none_or_all_like_the_cpu() {
    const TEST_NAME: &str =
        "the_gpu_judge_decodes_and_judges_in_place_subsets_with_copy_on_write_none_or_all_like_the_cpu";
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let flow = record_new_pool_file_creation(
        "crash-judge-gpu-in-place-subsets",
        "new-pool-in-place-subsets",
    );
    let expansion = singlefs_checker_tier::crash::quick_tier_expansion;
    let tables = build_judge_tables(&JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    })
    .expect("建得出表");
    assert!(
        tables.segment_expansions.iter().any(|segment_expansion| matches!(
            segment_expansion,
            singlefs_checker_tier::crash_facts::SegmentExpansionFact::InPlaceSubsetsWithCopyOnWriteNoneOrAll {
                copy_on_write_writes,
                ..
            } if copy_on_write_writes.len() >= 2
        )),
        "这条流里有带两次以上单元写的甲二段：GPU 按全量拆就拆出单元写只落一部分的状态"
    );
    // 判表先入库（对拍那一类的库）、崩溃点重放的流也先入库（崩溃点重放枚举那一类的库），两样都录完才看只录那一趟要不要当场返回
    let packed = packed_tables_through_the_library(TEST_NAME, &tables);
    let mut persisted_sets_of_the_crash_enumeration: Vec<Vec<bool>> = Vec::new();
    let judged = sample_crash_states_on_the_cpu(
        CrashPointReplayItem {
            name: TEST_NAME,
            setting_file: THIS_FILE,
        },
        &flow.base,
        &flow.writes,
        &flow.segments,
        flow.judged_root_index,
        &flow.versions,
        &expansion,
        0..tables.state_count,
        &mut |_ordinal, image, _judgement| {
            persisted_sets_of_the_crash_enumeration.push(image.persisted.clone())
        },
    );
    let (Some(packed), Some(judged)) = (packed, judged.judged()) else {
        return;
    };
    assert_eq!(judged, tables.state_count);
    for (ordinal, persisted) in persisted_sets_of_the_crash_enumeration.iter().enumerate() {
        assert_eq!(
            &tables.persisted_writes_of_state(u64::try_from(ordinal).expect("序号装得进 u64")),
            persisted,
            "序号 {ordinal}：判表那一套拆法与 crash.rs 枚举拆出的持久集合相同"
        );
    }
    let Some(lease) =
        cards.lease_card_or_report_not_run(TEST_NAME, mebibytes_a_judge_needs(&packed, 1))
    else {
        return;
    };
    let probe_words = singlefs_checker_tier::crash_judge_dispatch::persisted_masks_on_the_card(
        lease.card(),
        &packed,
        0..tables.state_count,
    );
    drop(lease);
    let enumerated_writes = usize::try_from(tables.enumerated_write_count).expect("装得进 usize");
    for (ordinal, words) in probe_words.iter().enumerate() {
        let (persisted, undecidable) =
            singlefs_checker_tier::crash_judge_gpu::persisted_set_of_the_probe_words(
                words,
                enumerated_writes,
            );
        assert_eq!(undecidable, 0, "序号 {ordinal}：探针拆得了");
        assert_eq!(
            persisted, persisted_sets_of_the_crash_enumeration[ordinal],
            "序号 {ordinal}：GPU 拆出的持久掩码与 CPU 相同"
        );
    }
    let Some(comparison) =
        compare_on_every_state_of_these_segments(cards, &flow, &flow.segments, &expansion)
    else {
        return;
    };
    assert!(
        comparison.mismatches.is_empty(),
        "甲二展开的 {} 个状态里 {} 个 GPU 与 CPU 的对拍字不同：\n{}",
        comparison.states,
        comparison.mismatches.len(),
        comparison
            .mismatches
            .iter()
            .filter(|line| !line.is_empty())
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// 33 次两态写的一段（判决 m3-crash-enumeration-merge-r2 第二节 L1-7，攻方模型 decode_alias.py 那一形）：从第 2³² 个状态起拆 8 个，
/// GPU 探针按 lo / hi 两个字算段内余数，拆出的掩码与 CPU 判表逐序号相同（第 2³² + 5 个是写 0、写 2、写 32 持久，不被截成低 32 位的 5）。
#[test]
fn the_gpu_judge_decodes_an_ordinal_of_a_thirty_three_write_segment_past_thirty_two_bits_like_the_cpu(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_decodes_an_ordinal_of_a_thirty_three_write_segment_past_thirty_two_bits_like_the_cpu";
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let writes = 33u32;
    let states = (1u64 << writes) - 1;
    let tables = singlefs_checker_tier::crash_judge_tables::JudgeTables {
        recorded_write_count: writes,
        enumerated_write_count: writes,
        is_tearable: vec![false; 33],
        torn_images: vec![],
        segments: vec![singlefs_checker_tier::crash_facts::SegmentFact {
            writes: (0..writes).collect(),
            states: 0..states,
            slice_start: None,
        }],
        segment_expansions: vec![
            singlefs_checker_tier::crash_facts::SegmentExpansionFact::EveryProperSubset,
        ],
        state_source:
            singlefs_checker_tier::crash_judge_tables::JudgeStateSource::SegmentExpansions,
        state_count: states + 1,
        devices: vec![0],
        device_bytes: 1 << 26,
        unit_area_start_in_bytes: 1 << 24,
        judged_root_index: 0,
        writes: vec![],
        locations: vec![],
        publishes: vec![],
        expected_versions: vec![],
    };
    // 探针也只拿读回的那一份表算；只录那一趟用例当场返回
    let Some(packed) = packed_tables_through_the_library(TEST_NAME, &tables) else {
        return;
    };
    let Some(lease) =
        cards.lease_card_or_report_not_run(TEST_NAME, mebibytes_a_judge_needs(&packed, 1))
    else {
        return;
    };
    let first = 1u64 << 32;
    let probe_words = singlefs_checker_tier::crash_judge_dispatch::persisted_masks_on_the_card(
        lease.card(),
        &packed,
        first..first + 8,
    );
    drop(lease);
    for (offset, words) in probe_words.iter().enumerate() {
        let ordinal = first + u64::try_from(offset).expect("装得进 u64");
        let (persisted, undecidable) =
            singlefs_checker_tier::crash_judge_gpu::persisted_set_of_the_probe_words(words, 33);
        assert_eq!(undecidable, 0);
        assert_eq!(
            persisted,
            tables.persisted_writes_of_state(ordinal),
            "序号 {ordinal}"
        );
    }
    let (fifth, _) = singlefs_checker_tier::crash_judge_gpu::persisted_set_of_the_probe_words(
        &probe_words[5],
        33,
    );
    let persisted_writes: Vec<usize> = fifth
        .iter()
        .enumerate()
        .filter(|(_, is_persisted)| **is_persisted)
        .map(|(write_index, _)| write_index)
        .collect();
    assert_eq!(persisted_writes, vec![0, 2, 32]);
}

/// 给定状态表（判决 m3-crash-enumeration-merge-r2 第二节 L6-3，随机历史两项上 GPU 要的那一种展开）：新池新建文件那条流全量展开的掩码，
/// 打乱次序（倒过来、隔一个取一个）再接上全部持久那一个（它在倒过来那一串的头一个里已经有了，清单里同一个状态出现两次也照判）做成清单，GPU 照清单判出的对拍字与 CPU 判器在同一张镜像上的逐状态相同。
#[test]
fn the_gpu_judge_on_given_persisted_sets_agrees_with_the_cpu_judge_on_every_state() {
    const TEST_NAME: &str =
        "the_gpu_judge_on_given_persisted_sets_agrees_with_the_cpu_judge_on_every_state";
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let flow =
        record_new_pool_file_creation("crash-judge-gpu-given-states", "new-pool-given-states");
    let limit = expand_up_to();
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= limit {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let judge_flow = JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    };
    let enumerated = build_judge_tables(&judge_flow).expect("建得出表");
    let mut given_ordinals: Vec<u64> = (0..enumerated.state_count).rev().step_by(2).collect();
    given_ordinals.push(enumerated.state_count - 1);
    let given: Vec<Vec<bool>> = given_ordinals
        .iter()
        .map(|ordinal| enumerated.persisted_writes_of_state(*ordinal))
        .collect();
    let tables =
        singlefs_checker_tier::crash_judge_tables::build_judge_tables_for_the_given_persisted_sets(
            &judge_flow,
            &[],
            given.clone(),
        )
        .expect("给定状态表建得出");
    assert_eq!(
        tables.state_count,
        u64::try_from(given.len()).expect("装得进 u64")
    );
    let Some(gpu) =
        singlefs_checker_tier::crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool(cards, TEST_NAME, &tables)
    else {
        return;
    };
    // CPU 参照一侧：流先入崩溃点重放枚举那一类的库、再从库里读回，按序号判（用户 2026-10-13 定：CPU 参照也从库读），
    // 清单第 i 个取它在全量展开里的序号那一份
    let Some(cpu_by_ordinal) = cpu_cross_check_words_read_back_from_the_library(
        &format!("{TEST_NAME}-cpu-reference"),
        &flow,
        &expansion,
        0..enumerated.state_count,
    ) else {
        return;
    };
    let mismatches: Vec<String> = given_ordinals
        .iter()
        .zip(&given)
        .zip(&gpu)
        .enumerate()
        .filter_map(|(position, ((ordinal, persisted), gpu_words))| {
            let (cpu_persisted, cpu_cross_check_words) = &cpu_by_ordinal[ordinal];
            assert_eq!(
                cpu_persisted, persisted,
                "清单第 {position} 个：CPU 从库里读回判的序号 {ordinal} 与清单那一个是同一个状态"
            );
            let gpu_cross_check_words = cross_check_words_of_gpu_verdict(gpu_words);
            (!cross_check_words_agree_in_the_cross_check(
                &gpu_cross_check_words,
                cpu_cross_check_words,
            ))
            .then(|| {
                format!(
                    "清单第 {position} 个（序号 {ordinal}）：CPU {cpu_cross_check_words:?}；GPU {gpu_cross_check_words:?}"
                )
            })
        })
        .collect();
    assert_eq!(gpu.len(), given.len(), "GPU 判了清单里的每一个");
    assert!(
        mismatches.is_empty(),
        "{} 个点名的状态 GPU 与 CPU 不同：\n{}",
        mismatches.len(),
        mismatches
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// CPU 判器经崩溃点重放枚举那一类的库判这条流在 `ordinals` 上的每个状态（流先入库、再从库里读回判，
/// `crash_state_enumeration::sample_crash_states_on_the_cpu_with_threads_against_the_version_table`），交回按序号的持久集合与对拍字；
/// 只录那一趟交 `None`。线程数照 [`CPU_THREADS_ENVIRONMENT_VARIABLE`]。
fn cpu_cross_check_words_read_back_from_the_library(
    item_name: &str,
    flow: &RecordedFlow,
    expansion: &(dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion + Sync),
    ordinals: std::ops::Range<u64>,
) -> Option<BTreeMap<u64, (Vec<bool>, CrossCheckWords)>> {
    let judged_by_ordinal: std::sync::Mutex<BTreeMap<u64, (Vec<bool>, CrossCheckWords)>> =
        std::sync::Mutex::new(BTreeMap::new());
    let judged = sample_crash_states_on_the_cpu_with_threads_against_the_version_table(
        CrashPointReplayItem {
            name: item_name,
            setting_file: THIS_FILE,
        },
        &flow.base,
        &flow.writes,
        &flow.segments,
        flow.judged_root_index,
        CrashPointReplayVersionTable::new(&flow.versions, &[]),
        expansion,
        ordinals.clone(),
        cpu_threads_from_the_value_of_the_variable(
            process_environment_variable(CPU_THREADS_ENVIRONMENT_VARIABLE).as_deref(),
            process_environment_variable(THREAD_CAP_ENVIRONMENT_VARIABLE).as_deref(),
        ),
        &|ordinal, image, judgement| {
            judged_by_ordinal.lock().expect("判定线程不 panic").insert(
                ordinal,
                (
                    image.persisted.clone(),
                    cross_check_words_of_cpu_judgement(judgement),
                ),
            );
        },
    )
    .judged()?;
    let judged_by_ordinal = judged_by_ordinal.into_inner().expect("判定线程不 panic");
    assert_eq!(
        u64::try_from(judged_by_ordinal.len()).expect("装得进 u64"),
        ordinals.end - ordinals.start,
        "CPU 从库里读回之后判了点名序号段的每个状态（交回 {judged} 个）"
    );
    Some(judged_by_ordinal)
}

/// 判表核段长（判决 m3-crash-enumeration-merge-r2 第二节 L1-7「判表打包核段长」）：建表时每段按它的展开方式现算的状态数要与枚举计划的序号区间相同。
/// 第二次问展开方式时改了口（枚举计划时每段都不展开、序号区间都是空的，建判表那一份拆法时换成甲二、每段至少一个状态）的流，
/// 建表交「段的状态数对不上」，不 panic、不建出一张拆错的表。
// crash-case-check:not-a-crash-case 只建一张判表，不判任何崩溃状态
#[test]
fn a_judge_flow_whose_expansion_changes_between_two_asks_is_refused_for_the_segment_state_count() {
    let flow = record_new_pool_file_creation(
        "crash-judge-gpu-expansion-changes",
        "new-pool-expansion-changes",
    );
    let asks = std::cell::Cell::new(0usize);
    let segment_count = flow.segments.len();
    let expansion = |_segment_index: usize, _segment: &[usize]| {
        asks.set(asks.get() + 1);
        if asks.get() <= segment_count {
            CrashPointReplaySegmentExpansion::NotExpanded
        } else {
            CrashPointReplaySegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll
        }
    };
    let refusal = build_judge_tables(&JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    })
    .expect_err("段的状态数对不上，不建表");
    assert!(
        matches!(
            refusal,
            singlefs_checker_tier::crash_judge_tables::JudgeTablesRefusal::SegmentStateCountNotRepresentable { .. }
        ),
        "{refusal:?}"
    );
}

// ───────────── 第 57 批 H1、H2（判决 `research/prompts/m3-batch56-code-r1-main-verification.md` 第三节）：旋钮下非最右那片装满旋钮值；产品支墓碑那一个调用点 ─────────────

/// H1（GPU 那一份，攻方 `k2_knob_bad_images` 那一形）：旋钮 1 写出的两片各 1 条，GPU 扫描内核与 CPU 新入口都给这一棵旋钮 2：每片都不超旋钮值、
/// 容器号从 1 逐片加一，只「不是最右的那片装满旋钮值」不成立，CPU 红这一棵的那一条（I-9.22 / I-9.37），GPU 红在同样的状态、同样的位上。
#[test]
fn the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_two_tombstone_containers_written_under_the_knob_one(
) {
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_two_tombstone_containers_written_under_the_knob_one",
        &record_two_empty_inodes_unlinked_with_one_tombstone_per_container(
            "crash-judge-gpu-tombstone-knob-two-over-containers-of-one-record",
        ),
        knobs_of_the_tombstone_tree(2),
        KNOB_CAPACITY_INVARIANT_OF_THE_TOMBSTONE_TREE,
    );
}

#[test]
fn the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_two_lineage_containers_written_under_the_knob_one(
) {
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_two_lineage_containers_written_under_the_knob_one",
        &record_two_rollbacks_with_one_lineage_record_per_container(
            "crash-judge-gpu-lineage-knob-two-over-containers-of-one-record",
        ),
        knobs_of_the_lineage_tree(2),
        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
    );
}

#[test]
fn the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_two_retirement_containers_written_under_the_knob_one(
) {
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_two_retirement_containers_written_under_the_knob_one",
        &record_two_leaves_retired_with_one_retirement_record_per_container(
            "crash-judge-gpu-retirement-knob-two-over-containers-of-one-record",
        ),
        knobs_of_the_retirement_tree(2),
        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
    );
}

/// H2 产品支墓碑那一个调用点要的流：新池新建文件、再建 816 个空 inode（一次发布），照产品容量逐个 unlink 前 815 个到底（墓碑树一片装满 815 条），
/// 再 unlink 第 816 个（意图那次开第二片、1 条）：墓碑树 [815, 1]，产品镜像合法。起点镜像是 mkfs 到第 816 个 unlink 之前的整段录制流施加出来的那一份，
/// 写表只录第 816 个 unlink 那几次发布（第 58 批 J4：前 815 次 unlink 约 1630 次发布，整条流从 mkfs 录起时它们的写也进枚举写表、
/// 超过判器持久掩码的宽，建表交 `EnumeratedWritesPastTheMaskWidth`；之前的段在原来那一造法里本就整段持久、不展开，崩溃状态与这里相同）。
fn record_a_second_tombstone_container_opened_after_a_full_one_at_product_capacity(
    tag: &str,
) -> RecordedFlow {
    let tombstones_filling_one_container = singlefs_format::TOMBSTONE_CONTAINER_RECORDS;
    let mut pool = pool_with_new_inodes(tag, tombstones_filling_one_container + 1);
    // 迭代上界：815 个号。
    for inode in FIRST_FILE.0 + 1..=FIRST_FILE.0 + tombstones_filling_one_container {
        unlink_an_empty_inode_to_the_end_under_the_capacities(
            &mut pool,
            inode,
            ContainerTreeCapacities::FROM_THE_FORMAT,
        );
    }
    let operations_before_the_last_unlink = pool.stream.operation_count();
    unlink_an_empty_inode_to_the_end_under_the_capacities(
        &mut pool,
        FIRST_FILE.0 + tombstones_filling_one_container + 1,
        ContainerTreeCapacities::FROM_THE_FORMAT,
    );
    let tombstone_record_counts: Vec<usize> = pool
        .output
        .tombstone_containers
        .iter()
        .map(|container| container.contents.records.len())
        .collect();
    assert_eq!(
        tombstone_record_counts,
        [
            usize::try_from(tombstones_filling_one_container).expect("装得进 usize"),
            1
        ],
        "墓碑树第一片装满 815、第二片 1 条"
    );
    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
    let operations = pool.retained_operations();
    let mut base = pool.memory_pool_after_mkfs();
    base.apply(&operations[pool.mkfs_operation_count..operations_before_the_last_unlink]);
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[operations_before_the_last_unlink..],
        &geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("第 816 个 unlink 的发布有根槽写");
    RecordedFlow {
        name: "product-tombstone-containers-after-a-full-one",
        base,
        writes,
        segments,
        judged_root_index,
        versions,
    }
}

/// H2（GPU 产品支墓碑那一个调用点，攻方 `mut-w-gap2` 那一形）：产品容量下墓碑树 [815, 1]，第二片容器与根里指它的条目的容器号 2 → 3（1、3）：
/// 第一片装满、容量那句成立，CPU 产品入口只因「相邻两片加一」红 I-9.21，GPU 红在同样的状态、同样的位上。写表只录第 816 个 unlink 那几次发布
/// （之前的录制流施加成起点镜像，第 58 批 J4），每段照展开上限展开；GPU 扫描内核产品支不调三判（`d_judge_tombstone_container_capacity` 里那一个调用点）
/// 就在这里与 CPU 不一致。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_product_tombstone_containers_after_a_full_one_skip_a_number(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_product_tombstone_containers_after_a_full_one_skip_a_number";
    let mut flow = record_a_second_tombstone_container_opened_after_a_full_one_at_product_capacity(
        "crash-judge-gpu-i921-product-containers-skip-a-number",
    );
    renumber_the_last_container_of_the_flow(&mut flow, PACKED_RECORD_TYPE_TOMBSTONE, 1, 3);
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-9.21"),
        "流 {} 上 CPU 池级 checker 要红 I-9.21（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

// ───────────── 第 58 批 J3（判决 `research/prompts/m3-batch57-code-r1-main-verification.md` 第三节；两条腿 K4）：旋钮支三片、中间那片少一条 ─────────────

/// 写的时候逐条换的旋钮值：2、2、1、1 写出 [2, 1, 1]（攻方 `research/prompts/m3-batch57-code-r1-opus-model/probe-crate/tests/k4_middle_short_container.rs` 那一形）。
/// 写侧每一步都照当时的旋钮满了才关；第 57 批 H1 那几条流都只有两片容器，两片时「只判叶序第一片」与正确写法等价，三片时分得开。
const KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_MIDDLE_CONTAINER: [u64; 4] = [2, 2, 1, 1];
/// 按叶序每片的记录数。
const RECORD_COUNTS_OF_A_SHORT_MIDDLE_CONTAINER: [usize; 3] = [2, 1, 1];
/// 判的时候编进 GPU 扫描内核、交给 CPU 新入口的旋钮值：容器 2 不是最右那片、只装 1 条，不是旋钮值。
const KNOB_RECORDS_JUDGING_A_SHORT_MIDDLE_CONTAINER: u64 = 2;

/// 只墓碑树压到 `records_per_container` 条一片的写入口（另两棵与节点照产品）。
fn capacities_of_the_tombstone_container_knob(
    records_per_container: u64,
) -> ContainerTreeCapacities {
    ContainerTreeCapacities {
        tombstone_container: TombstoneContainerCapacity::CappedForTests(
            TombstoneRecordsPerContainerCappedForTests::new(records_per_container)
                .expect("1..=815"),
        ),
        lineage_container: LineageContainerCapacity::FromTheContainerFormat,
        retirement_container:
            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
        node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

/// 只退役树压到 `records_per_container` 条一片的写入口（另两棵与节点照产品）。
fn capacities_of_the_retirement_container_knob(
    records_per_container: u64,
) -> ContainerTreeCapacities {
    ContainerTreeCapacities {
        tombstone_container: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container: LineageContainerCapacity::FromTheContainerFormat,
        retirement_container:
            singlefs_core::retirement_tree::RetirementContainerCapacity::CappedForTests(
                singlefs_core::retirement_tree::RetirementRecordsPerContainerCappedForTests::new(
                    records_per_container,
                )
                .expect("1..=959"),
            ),
        node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

/// 墓碑树那一形：新池新建文件、建 4 个空 inode（2–5 号），逐个 unlink 到底，写入口按
/// [`KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_MIDDLE_CONTAINER`] 逐个换墓碑旋钮：墓碑树 [2, 1, 1]。
fn record_tombstone_containers_of_two_one_and_one_records_under_changing_knobs(
    tag: &str,
) -> RecordedFlow {
    let mut pool = pool_with_new_inodes(
        tag,
        u64::try_from(KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_MIDDLE_CONTAINER.len())
            .expect("装得进 u64"),
    );
    // 迭代上界：4 个号。
    for (inode, records_per_container) in
        (FIRST_FILE.0 + 1..).zip(KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_MIDDLE_CONTAINER)
    {
        unlink_an_empty_inode_to_the_end_under_the_capacities(
            &mut pool,
            inode,
            capacities_of_the_tombstone_container_knob(records_per_container),
        );
    }
    let tombstone_record_counts: Vec<usize> = pool
        .output
        .tombstone_containers
        .iter()
        .map(|container| container.contents.records.len())
        .collect();
    assert_eq!(
        tombstone_record_counts, RECORD_COUNTS_OF_A_SHORT_MIDDLE_CONTAINER,
        "墓碑树按叶序每片的记录数"
    );
    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
    recorded_flow_of_the_pool(
        &pool,
        "tombstone-containers-of-two-one-and-one-records-under-changing-knobs",
        versions,
    )
}

/// 谱系树那一形：新池新建文件、建 1 个空 inode，挂着回退到第一个文件那一版 4 次，每次回退之前按
/// [`KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_MIDDLE_CONTAINER`] 换谱系旋钮：谱系树 [2, 1, 1]。
fn record_lineage_containers_of_two_one_and_one_records_under_changing_knobs(
    tag: &str,
) -> RecordedFlow {
    let pool = pool_rolled_back_under_these_lineage_knobs(
        tag,
        &KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_MIDDLE_CONTAINER,
    );
    let lineage_record_counts: Vec<usize> = pool
        .output
        .lineage_containers
        .iter()
        .map(|container| container.contents.records.len())
        .collect();
    assert_eq!(
        lineage_record_counts, RECORD_COUNTS_OF_A_SHORT_MIDDLE_CONTAINER,
        "谱系树按叶序每片的记录数"
    );
    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
    recorded_flow_of_the_pool(
        &pool,
        "lineage-containers-of-two-one-and-one-records-under-changing-knobs",
        versions,
    )
}

/// 退役树那一形：新池新建文件之外再建 233 个空 inode（叶 [1..=233]、[234]），unlink 234 到底（最右那片叶删空、退役一条），
/// 之后每次建 1 个（开新叶）、unlink 它到底（又退役一条），共退役 4 次，写入口按 [`KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_MIDDLE_CONTAINER`]
/// 逐次换退役旋钮：退役树 [2, 1, 1]。
fn record_retirement_containers_of_two_one_and_one_records_under_changing_knobs(
    tag: &str,
) -> RecordedFlow {
    let mut pool = pool_with_new_inodes(tag, INODE_RECORDS_PER_LEAF);
    // 迭代上界：4 次退役。第 0 次删的是建出来的最后一个号（独占最右那片叶），之后每次先建一个再删它。
    for (retirement_index, records_per_container) in
        KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_MIDDLE_CONTAINER
            .into_iter()
            .enumerate()
    {
        if retirement_index > 0 {
            common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
        }
        let inode = pool.output.inode_number_watermark() - 1;
        unlink_an_empty_inode_to_the_end_under_the_capacities(
            &mut pool,
            inode,
            capacities_of_the_retirement_container_knob(records_per_container),
        );
    }
    let retirement_record_counts: Vec<usize> = pool
        .output
        .retirement_containers
        .iter()
        .map(|container| container.contents.records.len())
        .collect();
    assert_eq!(
        retirement_record_counts, RECORD_COUNTS_OF_A_SHORT_MIDDLE_CONTAINER,
        "退役树按叶序每片的记录数"
    );
    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
    recorded_flow_of_the_pool(
        &pool,
        "retirement-containers-of-two-one-and-one-records-under-changing-knobs",
        versions,
    )
}

/// J3（GPU 那一份）：墓碑树 [2, 1, 1]，GPU 扫描内核与 CPU 新入口都给墓碑旋钮 2：每片都不超旋钮值、容器号从 1 逐片加一，
/// 只「不是最右的容器 2 装满旋钮值」不成立，CPU 红 I-9.22，GPU 红在同样的状态、同样的位上（GPU 只判叶序第一片就在这里与 CPU 不一致）。
#[test]
fn the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_tombstone_containers_of_two_one_and_one_records_written_under_changing_knobs(
) {
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_tombstone_containers_of_two_one_and_one_records_written_under_changing_knobs",
        &record_tombstone_containers_of_two_one_and_one_records_under_changing_knobs(
            "crash-judge-gpu-tombstone-containers-of-two-one-and-one-records",
        ),
        knobs_of_the_tombstone_tree(KNOB_RECORDS_JUDGING_A_SHORT_MIDDLE_CONTAINER),
        KNOB_CAPACITY_INVARIANT_OF_THE_TOMBSTONE_TREE,
    );
}

/// J3（GPU 那一份）：谱系树 [2, 1, 1]，两边都给谱系旋钮 2，CPU 红 I-9.37，GPU 同判。
#[test]
fn the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_lineage_containers_of_two_one_and_one_records_written_under_changing_knobs(
) {
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_lineage_containers_of_two_one_and_one_records_written_under_changing_knobs",
        &record_lineage_containers_of_two_one_and_one_records_under_changing_knobs(
            "crash-judge-gpu-lineage-containers-of-two-one-and-one-records",
        ),
        knobs_of_the_lineage_tree(KNOB_RECORDS_JUDGING_A_SHORT_MIDDLE_CONTAINER),
        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
    );
}

/// J3（GPU 那一份）：退役树 [2, 1, 1]，两边都给退役旋钮 2，CPU 红 I-9.37，GPU 同判。
#[test]
fn the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_retirement_containers_of_two_one_and_one_records_written_under_changing_knobs(
) {
    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_retirement_containers_of_two_one_and_one_records_written_under_changing_knobs",
        &record_retirement_containers_of_two_one_and_one_records_under_changing_knobs(
            "crash-judge-gpu-retirement-containers-of-two-one-and-one-records",
        ),
        knobs_of_the_retirement_tree(KNOB_RECORDS_JUDGING_A_SHORT_MIDDLE_CONTAINER),
        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
    );
}

// ───────────── 第 59 批 L1（判决 `research/prompts/m3-batch58-code-r1-main-verification.md` 第三节 L1；两条腿 K1-3）：少一条的那片之后还有满的一片、根环转过一圈 ─────────────
// ───────────── 第 60 批 N1（判决 `research/prompts/m3-batch59-code-r1-main-verification.md` 第三节 N1；攻方 M1-4、正推 M1-1）：少一条的那片离最右三片 ─────────────

/// 只一棵容器树逐条换旋钮写、再把根环转过一圈的那几张盘怎么造：写第 k 条记录时这一棵一片几条，根环转过一圈之后这一棵按叶序每片的记录数。
struct ContainersWrittenUnderChangingKnobsBeforeTheRootRingTurnsOver {
    records_per_container_of_each_record: &'static [u64],
    record_counts_in_leaf_order: &'static [usize],
}

/// 少一条的那片之后还有一片满的（第 59 批 L1）：旋钮 2、2、1、1、2、1 写出 [2, 1, 2, 1]（`crates/singlefs-harness/tests/checker_known_bad_images.rs` 的
/// `A_SHORT_CONTAINER_BEFORE_A_FULL_ONE` 那一形）。第 58 批 J3 那三条 [2, 1, 1] 上少一条的那片只会落在倒数第二位，GPU 扫描内核
/// 「只判倒数第二片」在它们上面与正确判法逐位相同；这里少一条的容器 2 之后还有一片满的。
const A_SHORT_CONTAINER_BEFORE_A_FULL_ONE:
    ContainersWrittenUnderChangingKnobsBeforeTheRootRingTurnsOver =
    ContainersWrittenUnderChangingKnobsBeforeTheRootRingTurnsOver {
        records_per_container_of_each_record: &[2, 2, 1, 1, 2, 1],
        record_counts_in_leaf_order: &[2, 1, 2, 1],
    };

/// 少一条的那片离最右三片（第 60 批 N1）：旋钮 2、2、1、1、2、2、2、1 写出 [2, 1, 2, 2, 1]（`crates/singlefs-harness/tests/checker_known_bad_images.rs` 的
/// `A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST` 那一形，第 59 批 L2）。上面 [2, 1, 2, 1] 与第 58 批 J3 那几条上少一条的那片离最右
/// 都不超过两片，GPU 扫描内核「只判最右之前两片」（攻方 `research/prompts/m3-batch59-code-r1-opus-model/` 的 `w-last-two`）在它们上面与正确判法逐位相同；
/// 这里容器 2 在倒数第三片之前。写的中途 [2, 1, 1]、[2, 1, 2]、[2, 1, 2, 1] 那几版里容器 2 离最右不超过两片，留在候选集里时那一形也红，所以要转出去。
const A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST:
    ContainersWrittenUnderChangingKnobsBeforeTheRootRingTurnsOver =
    ContainersWrittenUnderChangingKnobsBeforeTheRootRingTurnsOver {
        records_per_container_of_each_record: &[2, 2, 1, 1, 2, 2, 2, 1],
        record_counts_in_leaf_order: &[2, 1, 2, 2, 1],
    };

/// 第 59 批 L1、第 60 批 N1 那几条流写的是哪一棵容器树（另两棵照产品）。
#[derive(Clone, Copy, Debug)]
enum ContainerTreeWrittenBeforeTheRootRingTurnsOver {
    Tombstone,
    Lineage,
    Retirement,
}

impl ContainerTreeWrittenBeforeTheRootRingTurnsOver {
    /// 新池新建文件之后，第 k 条记录写的时候只这一棵压到 `records_per_container_of_each_record[k]` 条一片：墓碑树逐个 unlink 空 inode 到底
    /// （同 [`record_tombstone_containers_of_two_one_and_one_records_under_changing_knobs`]），谱系树挂着回退到第一个文件那一版
    /// （[`pool_rolled_back_under_these_lineage_knobs`]），退役树每次退役独占最右那片叶的号（同
    /// [`record_retirement_containers_of_two_one_and_one_records_under_changing_knobs`]）。
    fn pool_written_under_these_knobs(
        self,
        tag: &str,
        records_per_container_of_each_record: &[u64],
    ) -> BuiltPool {
        match self {
            Self::Tombstone => {
                let mut pool = pool_with_new_inodes(
                    tag,
                    u64::try_from(records_per_container_of_each_record.len()).expect("装得进 u64"),
                );
                // 迭代上界：切片的长度。
                for (inode, records_per_container) in
                    (FIRST_FILE.0 + 1..).zip(records_per_container_of_each_record)
                {
                    unlink_an_empty_inode_to_the_end_under_the_capacities(
                        &mut pool,
                        inode,
                        capacities_of_the_tombstone_container_knob(*records_per_container),
                    );
                }
                pool
            }
            Self::Lineage => pool_rolled_back_under_these_lineage_knobs(
                tag,
                records_per_container_of_each_record,
            ),
            Self::Retirement => {
                let mut pool = pool_with_new_inodes(tag, INODE_RECORDS_PER_LEAF);
                // 迭代上界：切片的长度。第 0 次删的是建出来的最后一个号（独占最右那片叶），之后每次先建一个再删它。
                for (retirement_index, records_per_container) in
                    records_per_container_of_each_record.iter().enumerate()
                {
                    if retirement_index > 0 {
                        common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(
                            &mut pool, 1,
                        );
                    }
                    let inode = pool.output.inode_number_watermark() - 1;
                    unlink_an_empty_inode_to_the_end_under_the_capacities(
                        &mut pool,
                        inode,
                        capacities_of_the_retirement_container_knob(*records_per_container),
                    );
                }
                pool
            }
        }
    }

    /// 这一版这一棵按叶序每片的记录数。
    fn record_counts_of_each_container(self, pool: &BuiltPool) -> Vec<usize> {
        match self {
            Self::Tombstone => pool
                .output
                .tombstone_containers
                .iter()
                .map(|container| container.contents.records.len())
                .collect(),
            Self::Lineage => pool
                .output
                .lineage_containers
                .iter()
                .map(|container| container.contents.records.len())
                .collect(),
            Self::Retirement => pool
                .output
                .retirement_containers
                .iter()
                .map(|container| container.contents.records.len())
                .collect(),
        }
    }

    /// 判的时候编进 GPU 扫描内核、交给 CPU 新入口的旋钮值：只这一棵装 `records_per_container` 条一片。
    fn knobs_judging(self, records_per_container: u64) -> ContainerCapacityKnobs {
        match self {
            Self::Tombstone => knobs_of_the_tombstone_tree(records_per_container),
            Self::Lineage => knobs_of_the_lineage_tree(records_per_container),
            Self::Retirement => knobs_of_the_retirement_tree(records_per_container),
        }
    }

    /// 这一棵带旋钮值的那一条（墓碑树 I-9.22，谱系树与退役树 I-9.37）。
    fn knob_capacity_invariant(self) -> &'static str {
        match self {
            Self::Tombstone => KNOB_CAPACITY_INVARIANT_OF_THE_TOMBSTONE_TREE,
            Self::Lineage | Self::Retirement => {
                KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES
            }
        }
    }
}

/// 只 `tree` 这一棵按 `layout` 逐条换旋钮写出三片以上的容器，再建 30 次空 inode（[`turn_the_root_ring_over_by_creating_inodes`]）把根环转过一圈：
/// 写这棵树的那几版、连同写到一半时少一条的那片离最右更近的那几版，都转出写完之后各版的回退候选集，那几版的状态上照判写到一半的那一形，
/// 写完之后的状态上候选集里只剩 `layout.record_counts_in_leaf_order` 那一形（与 harness `checker_known_bad_images.rs` 里同名那一形的三条同形）。
/// 流从 mkfs 之后录起，被判的是最后一次根槽写。
fn record_containers_written_under_changing_knobs_with_the_root_ring_turned_over(
    tree: ContainerTreeWrittenBeforeTheRootRingTurnsOver,
    layout: &ContainersWrittenUnderChangingKnobsBeforeTheRootRingTurnsOver,
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let mut pool =
        tree.pool_written_under_these_knobs(tag, layout.records_per_container_of_each_record);
    turn_the_root_ring_over_by_creating_inodes(&mut pool);
    assert_eq!(
        tree.record_counts_of_each_container(&pool),
        layout.record_counts_in_leaf_order,
        "{tree:?} 按叶序每片的记录数（根环转过一圈之后）"
    );
    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
    recorded_flow_of_the_pool(&pool, name, versions)
}

/// L1、N1：`tree` 这一棵按 `layout` 写、根环转过一圈，GPU 扫描内核与 CPU 新入口都给这一棵旋钮 2：每片都不超旋钮值、容器号从 1 逐片加一、
/// 每片至少一条，只「不是最右的容器 2 装了 1 条，不是旋钮值 2」不成立。CPU 在这条流的全部状态上红过的不变量恰好是这一棵的那一条
/// （I-9.22 / I-9.37；另两棵照产品、不红，别的不变量在合法写出来的盘上也不红），GPU 红在同样的状态、同样的位上。
/// [2, 1, 2, 1] 上 GPU 扫描内核只判倒数第二片（第 3 片，装满 2 条）、[2, 1, 2, 2, 1] 上只判最右之前两片（第 3、4 片，都装满 2 条），
/// 在根环转过之后的状态上照绿，就在那里与 CPU 不一致。
fn assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_that_containers_written_under_changing_knobs_with_the_root_ring_turned_over_redden_only_the_knob_capacity_invariant(
    test_name: &str,
    tree: ContainerTreeWrittenBeforeTheRootRingTurnsOver,
    layout: &ContainersWrittenUnderChangingKnobsBeforeTheRootRingTurnsOver,
    tag: &str,
    name: &'static str,
) {
    let flow = record_containers_written_under_changing_knobs_with_the_root_ring_turned_over(
        tree, layout, tag, name,
    );
    let knob_capacity_invariant = tree.knob_capacity_invariant();
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state_given_the_container_capacity_knobs(
        cards,
        &flow,
        tree.knobs_judging(KNOB_RECORDS_JUDGING_A_SHORT_MIDDLE_CONTAINER),
    ) else {
        return;
    };
    assert_eq!(
        comparison.cpu_checker_red_invariants,
        std::collections::BTreeSet::from([knob_capacity_invariant]),
        "流 {} 上 CPU 新入口带旋钮值红过的不变量恰好是 {knob_capacity_invariant}",
        flow.name
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}（GPU 扫描内核编进旋钮值）：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
}

#[test]
fn the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_tombstone_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over(
) {
    assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_that_containers_written_under_changing_knobs_with_the_root_ring_turned_over_redden_only_the_knob_capacity_invariant(
        "the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_tombstone_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over",
        ContainerTreeWrittenBeforeTheRootRingTurnsOver::Tombstone,
        &A_SHORT_CONTAINER_BEFORE_A_FULL_ONE,
        "crash-judge-gpu-tombstone-containers-of-two-one-two-and-one-records-ring-turned",
        "tombstone-containers-of-two-one-two-and-one-records-with-the-root-ring-turned-over",
    );
}

#[test]
fn the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_lineage_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over(
) {
    assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_that_containers_written_under_changing_knobs_with_the_root_ring_turned_over_redden_only_the_knob_capacity_invariant(
        "the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_lineage_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over",
        ContainerTreeWrittenBeforeTheRootRingTurnsOver::Lineage,
        &A_SHORT_CONTAINER_BEFORE_A_FULL_ONE,
        "crash-judge-gpu-lineage-containers-of-two-one-two-and-one-records-ring-turned",
        "lineage-containers-of-two-one-two-and-one-records-with-the-root-ring-turned-over",
    );
}

#[test]
fn the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_retirement_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over(
) {
    assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_that_containers_written_under_changing_knobs_with_the_root_ring_turned_over_redden_only_the_knob_capacity_invariant(
        "the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_retirement_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over",
        ContainerTreeWrittenBeforeTheRootRingTurnsOver::Retirement,
        &A_SHORT_CONTAINER_BEFORE_A_FULL_ONE,
        "crash-judge-gpu-retirement-containers-of-two-one-two-and-one-records-ring-turned",
        "retirement-containers-of-two-one-two-and-one-records-with-the-root-ring-turned-over",
    );
}

#[test]
fn the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_tombstone_containers_of_two_one_two_two_and_one_records_with_the_root_ring_turned_over(
) {
    assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_that_containers_written_under_changing_knobs_with_the_root_ring_turned_over_redden_only_the_knob_capacity_invariant(
        "the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_tombstone_containers_of_two_one_two_two_and_one_records_with_the_root_ring_turned_over",
        ContainerTreeWrittenBeforeTheRootRingTurnsOver::Tombstone,
        &A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST,
        "crash-judge-gpu-tombstone-containers-of-two-one-two-two-and-one-records-ring-turned",
        "tombstone-containers-of-two-one-two-two-and-one-records-with-the-root-ring-turned-over",
    );
}

#[test]
fn the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_lineage_containers_of_two_one_two_two_and_one_records_with_the_root_ring_turned_over(
) {
    assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_that_containers_written_under_changing_knobs_with_the_root_ring_turned_over_redden_only_the_knob_capacity_invariant(
        "the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_lineage_containers_of_two_one_two_two_and_one_records_with_the_root_ring_turned_over",
        ContainerTreeWrittenBeforeTheRootRingTurnsOver::Lineage,
        &A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST,
        "crash-judge-gpu-lineage-containers-of-two-one-two-two-and-one-records-ring-turned",
        "lineage-containers-of-two-one-two-two-and-one-records-with-the-root-ring-turned-over",
    );
}

#[test]
fn the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_retirement_containers_of_two_one_two_two_and_one_records_with_the_root_ring_turned_over(
) {
    assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_that_containers_written_under_changing_knobs_with_the_root_ring_turned_over_redden_only_the_knob_capacity_invariant(
        "the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_retirement_containers_of_two_one_two_two_and_one_records_with_the_root_ring_turned_over",
        ContainerTreeWrittenBeforeTheRootRingTurnsOver::Retirement,
        &A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST,
        "crash-judge-gpu-retirement-containers-of-two-one-two-two-and-one-records-ring-turned",
        "retirement-containers-of-two-one-two-two-and-one-records-with-the-root-ring-turned-over",
    );
}

// ───────────── 第59批 E14：GPU 扫描内核 I-1.8 两句与走读内核位置项校验和那张表各一个敏感的取样点 ─────────────

/// I-1.8 ①「同一组的成员载荷相同」的一个敏感取样点：三个数据单元那条流（[`record_new_pool_through_a_three_unit_file`]）上，
/// 最后一个数据单元只改盘号小的那一份副本的载荷首字节、重封到根（与 `new-pool-data-unit-copies-differ` 同一形）。CPU 判 I-1.8 红。
/// GPU 扫描内核收扫描项的次序是落点表的 (盘, 偏移) 升序：码 1 的扫描项在一块盘上不止一个时，被改的那个单元的两份副本在收的次序里
/// 隔着别的数据单元，只有按归并键排过序才挨在一起、归成一组；新池新建文件那条流上码 1 只有一个数据单元，两份副本按收的次序本来就挨着，
/// 排没排序看不出来（第59批 E13 那一行变异在那条流上没红）。
fn record_three_unit_file_with_one_copy_of_its_last_data_unit_carrying_another_payload(
    tag: &str,
) -> RecordedFlow {
    let name = "three-unit-file-one-copy-of-the-last-data-unit-carries-another-payload";
    let mut flow = record_new_pool_through_a_three_unit_file(tag, name);
    let is_a_data_unit = |bytes: &[u8]| unit_class(bytes) == 1;
    let targets = last_unit_copies(&flow, name, &is_a_data_unit, true);
    let changed_device = flow.writes[targets[0]].device;
    let changed_offset = flow.writes[targets[0]].offset;
    // 收的次序里两份副本之间夹着别的数据单元：被改的那一盘上偏移更大的，或另一块盘上偏移更小的
    let data_units_between_the_two_copies = flow
        .writes
        .iter()
        .filter(|write| {
            write.kind == StepKind::UnitWrite
                && write.bytes().is_some_and(is_a_data_unit)
                && ((write.device == changed_device && write.offset.0 > changed_offset.0)
                    || (write.device != changed_device && write.offset.0 < changed_offset.0))
        })
        .count();
    assert!(
        data_units_between_the_two_copies >= 1,
        "流 {name}：按 (盘, 偏移) 收的次序里，被改的数据单元的两份副本之间要夹着别的数据单元（这条流要的就是这一点）"
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
        bytes[134] ^= 0xFF;
    });
    flow
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_one_copy_of_a_data_unit_among_several_carries_another_payload(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_one_copy_of_a_data_unit_among_several_carries_another_payload",
        &record_three_unit_file_with_one_copy_of_its_last_data_unit_carrying_another_payload(
            "crash-judge-gpu-i18-among-three-data-units",
        ),
        "I-1.8",
    );
}

/// 码 3 单元头里的出生序号（4 字节，偏移 103）：在码 3 的归并键 [93, 107) 那一段里，不在全序键（诞生代号 [73, 81)、实例与事务号 [93, 103)）里。
const PACKED_UNIT_BIRTH_SEQUENCE_OFFSET: usize = 103;

/// I-1.8 ②「同 key 的各组两两全序键不等」那一支：新池新建文件的流上 inode 叶容器只改盘号大的那一份副本的出生序号（加 1）、重封到根。
/// 两份副本的归并键不同、各成一组，对象 key 与全序键都相同：CPU 判 I-1.8 第二句红（两份载荷相同，第一句成立）。
/// GPU 扫描内核按换过次序的归并键（对象 key、全序键、其余）排，这两组挨着，相邻两组的公共前缀到出生序号那一段才断（第 57 个字节），
/// 比相邻两组的那一句要认出「对象 key 连着全序键都相同」。
fn record_new_pool_with_the_second_copy_of_the_inode_leaf_carrying_another_birth_sequence(
    tag: &str,
) -> RecordedFlow {
    let name = "new-pool-second-copy-of-the-inode-leaf-carries-another-birth-sequence";
    let mut flow = record_new_pool_file_creation(tag, name);
    let copies = last_unit_copies(&flow, name, &is_an_inode_leaf_container, false);
    let second_copy = *copies
        .iter()
        .max_by_key(|index| flow.writes[**index].device.0)
        .expect("一个单元两份副本");
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &[second_copy], &|bytes| {
        let birth_sequence = read_u32_at(bytes, PACKED_UNIT_BIRTH_SEQUENCE_OFFSET);
        bytes[PACKED_UNIT_BIRTH_SEQUENCE_OFFSET..PACKED_UNIT_BIRTH_SEQUENCE_OFFSET + 4]
            .copy_from_slice(&birth_sequence.wrapping_add(1).to_le_bytes());
    });
    flow
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_two_copies_of_an_inode_leaf_split_into_two_groups_of_one_total_order_key(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_two_copies_of_an_inode_leaf_split_into_two_groups_of_one_total_order_key",
        &record_new_pool_with_the_second_copy_of_the_inode_leaf_carrying_another_birth_sequence(
            "crash-judge-gpu-i18-two-groups-of-one-total-order-key",
        ),
        "I-1.8",
    );
}

/// 分配记录叶：key 宽 10（设备 4、槽号 6）、条目 20（key、跨度 2 含已释放标志、分配代或释放代 8）。
const ALLOCATION_LEAF_RECORD_BYTES: usize = 20;
/// 记账树里「defer 待释放」那一项（D5（快照 / 空间记账机制） 已定项 4 第 5 项，`singlefs_core::records::STATISTIC_DEFER_QUEUE_BYTES`）。
const STATISTIC_DEFER_QUEUE_BYTES: u16 = 5;

/// 隔离的那个数据单元的两份副本读回来什么样（[`record_three_unit_file_through_its_second_deletion_batch_with_the_released_data_unit_quarantined`]）。
#[derive(Clone, Copy, Debug)]
enum CopiesOfTheQuarantinedDataUnit {
    /// 盘号大的那一份载荷首字节翻转、不重封：与每一条指着它的位置条目都对不上，这个单元整组豁免。
    TheSecondReadsBackOtherBytes,
    /// 两份都是原样；带着指它的位置条目的几片单元（下段叶、映射叶）按写的次序轮流把盘 0、盘 1 那一条的校验和写错：每一份都对得上
    /// 另一片里指着它的那一条，不豁免。
    EachMatchesOneOfTheLocationEntriesNamingIt,
}

/// 隔离豁免那一支（I-3.1 / I-3.11 的隔离的记录，D19（块指针的结构与宽度预算） 已定项 5；池级 checker `quarantined_slots_exempted_per_device`，
/// GPU 扫描内核 `c_quarantine_exempted`、读走读内核主走读记的位置项校验和那张表）：
/// 三个数据单元的文件发完意图（txg 5）与按批大小 1 的前两批（txg 6 放掉第三个单元、txg 7 放掉第二个单元）。txg 7 那一版改成「写者把放掉的
/// 第二个单元隔离了」：分配记录叶里那个单元两块盘上的记录改回第一批那一版的样子（仍分配、分配代），记账叶里两块盘的 defer 待释放各减一个单元，
/// 重封到根。最新根不引用它、更早的候选根（txg 4、5、6）引用它，判 I-3.11 那一遍它是一条已分配而没有根引用的记录，豁免不豁免看 `copies`：
/// 豁免时 I-3.11 成立，不豁免时 I-3.11 红（已分配多一个单元）。I-3.1 那一遍走的是全部候选根，这个单元有根引用，不进隔离。
fn record_three_unit_file_through_its_second_deletion_batch_with_the_released_data_unit_quarantined(
    tag: &str,
    copies: CopiesOfTheQuarantinedDataUnit,
) -> RecordedFlow {
    let name = match copies {
        CopiesOfTheQuarantinedDataUnit::TheSecondReadsBackOtherBytes => {
            "second-deletion-batch-quarantined-data-unit-second-copy-reads-back-other-bytes"
        }
        CopiesOfTheQuarantinedDataUnit::EachMatchesOneOfTheLocationEntriesNamingIt => {
            "second-deletion-batch-quarantined-data-unit-copies-each-match-one-location-entry"
        }
    };
    let mut pool = pool_with_a_three_unit_file(tag);
    publish_the_intent(&mut pool);
    let after_the_first_batch = publish_the_next_step_in_batches_of_one(&mut pool);
    let after_the_second_batch = publish_the_next_step_in_batches_of_one(&mut pool);
    let released_by_the_second_batch: Vec<_> = after_the_first_batch
        .data_pointers
        .iter()
        .filter(|pointer| !after_the_second_batch.data_pointers.contains(pointer))
        .copied()
        .collect();
    assert_eq!(
        released_by_the_second_batch.len(),
        1,
        "按批大小 1 的第二批放掉一个数据单元（第一批之后 {} 个、第二批之后 {} 个）",
        after_the_first_batch.data_pointers.len(),
        after_the_second_batch.data_pointers.len()
    );
    let quarantined = released_by_the_second_batch[0];
    let mut flow = recorded_flow_of_the_pool(
        &pool,
        name,
        versions_with_the_first_file_before_its_deletion(),
    );

    // 分配记录：第二批那一版两块盘上那一条（已释放、释放代）改回第一批那一版的（仍分配、分配代）
    let record_of = |records: &[singlefs_core::allocator::AllocationRecord],
                     location: &singlefs_core::pointer::LocationEntry| {
        *records
            .iter()
            .find(|record| record.device == location.device && record.slot == location.slot)
            .unwrap_or_else(|| panic!("那一版的分配记录里有 {location:?} 那一条"))
    };
    let record_replacements: Vec<(Vec<u8>, Vec<u8>)> = quarantined
        .locations
        .iter()
        .map(|location| {
            let released = record_of(&after_the_second_batch.allocation_records, location);
            let allocated = record_of(&after_the_first_batch.allocation_records, location);
            assert!(
                released.is_released && !allocated.is_released,
                "第二批那一版这一条已释放、第一批那一版仍分配：{released:?} / {allocated:?}"
            );
            (released.to_bytes(), allocated.to_bytes())
        })
        .collect();
    let records_of_an_allocation_leaf = |bytes: &[u8]| -> Vec<usize> {
        if unit_class(bytes) != 2
            || usize::from(index_node_key_width(bytes)) != ALLOCATION_RECORD_TREE_KEY_BYTES
            || usize::from(index_node_entry_width(bytes)) != ALLOCATION_LEAF_RECORD_BYTES
            || bytes[50] != 0
        {
            return Vec::new();
        }
        let entry_count = usize::from(read_u16_at(
            bytes,
            index_node_entry_count_offset(ALLOCATION_RECORD_TREE_KEY_BYTES),
        ));
        (0..entry_count)
            .map(|index| {
                index_node_entries_start(ALLOCATION_RECORD_TREE_KEY_BYTES)
                    + index * ALLOCATION_LEAF_RECORD_BYTES
            })
            .collect()
    };
    let leaves_with_the_released_records: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| {
            write.kind == StepKind::UnitWrite
                && write.bytes().is_some_and(|bytes| {
                    records_of_an_allocation_leaf(bytes).iter().any(|record| {
                        record_replacements.iter().any(|(released, _)| {
                            bytes[*record..*record + ALLOCATION_LEAF_RECORD_BYTES] == released[..]
                        })
                    })
                })
        })
        .map(|(index, _)| index)
        .collect();
    assert!(
        !leaves_with_the_released_records.is_empty(),
        "流 {name} 里第二批那一版的分配记录叶写着那两条已释放的记录"
    );
    corrupt_units_and_reseal_up_to_the_root(
        &mut flow,
        &leaves_with_the_released_records,
        &|bytes| {
            for record in records_of_an_allocation_leaf(bytes) {
                for (released, allocated) in &record_replacements {
                    if bytes[record..record + ALLOCATION_LEAF_RECORD_BYTES] == released[..] {
                        bytes[record..record + ALLOCATION_LEAF_RECORD_BYTES]
                            .copy_from_slice(allocated);
                    }
                }
            }
        },
    );

    // 记账：第二批那一版两块盘的 defer 待释放各减一个单元
    let unit_bytes = flow
        .writes
        .iter()
        .find(|write| {
            write.kind == StepKind::UnitWrite
                && write.device == quarantined.locations[0].device
                && write.offset == quarantined.locations[0].slot.to_device_offset()
        })
        .expect("流里写过被隔离的那个数据单元")
        .length_in_bytes();
    let devices_of_the_unit: Vec<u32> = quarantined
        .locations
        .iter()
        .map(|location| location.device.0)
        .collect();
    let accounting_leaf = last_unit_copies(&flow, name, &is_an_accounting_leaf, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &accounting_leaf, &|bytes| {
        let entry_count = usize::from(read_u16_at(
            bytes,
            index_node_entry_count_offset(ACCOUNTING_KEY_BYTES),
        ));
        for device in &devices_of_the_unit {
            let rows: Vec<usize> = (0..entry_count)
                .map(|index| {
                    index_node_entries_start(ACCOUNTING_KEY_BYTES) + index * ACCOUNTING_ROW_BYTES
                })
                .filter(|row| {
                    read_u16_at(bytes, *row) == STATISTIC_DEFER_QUEUE_BYTES
                        && read_u32_at(bytes, row + ACCOUNTING_ROW_DEVICE_OFFSET) == *device
                })
                .collect();
            assert_eq!(rows.len(), 1, "记账叶里盘 {device} 的 defer 待释放恰好一行");
            let value = read_u64_at(bytes, rows[0] + ACCOUNTING_ROW_VALUE_OFFSET);
            assert!(
                value >= unit_bytes,
                "盘 {device} 的 defer 待释放 {value} 字节里算着被放掉的那个单元（{unit_bytes} 字节）"
            );
            bytes[rows[0] + ACCOUNTING_ROW_VALUE_OFFSET..rows[0] + ACCOUNTING_ROW_VALUE_OFFSET + 8]
                .copy_from_slice(&(value - unit_bytes).to_le_bytes());
        }
    });

    match copies {
        CopiesOfTheQuarantinedDataUnit::TheSecondReadsBackOtherBytes => {
            let second = quarantined.locations[1];
            let second_copy_writes: Vec<usize> = flow
                .writes
                .iter()
                .enumerate()
                .filter(|(_, write)| {
                    write.kind == StepKind::UnitWrite
                        && write.device == second.device
                        && write.offset == second.slot.to_device_offset()
                })
                .map(|(index, _)| index)
                .collect();
            assert_eq!(
                second_copy_writes.len(),
                1,
                "被隔离的那个数据单元盘 {} 那一份流里只写过一次",
                second.device.0
            );
            let WrittenContents::Bytes(bytes) = &mut flow.writes[second_copy_writes[0]].contents
            else {
                panic!("单元写是普通写");
            };
            bytes[134] ^= 0xFF;
        }
        CopiesOfTheQuarantinedDataUnit::EachMatchesOneOfTheLocationEntriesNamingIt => {
            let entry_bytes = |location: &singlefs_core::pointer::LocationEntry| -> Vec<u8> {
                let mut entry = Vec::with_capacity(14);
                entry.extend_from_slice(&location.device.0.to_le_bytes());
                entry.extend_from_slice(&location.slot.0.to_le_bytes()[..6]);
                entry.extend_from_slice(&location.unit_checksum.to_le_bytes());
                entry
            };
            let entries_naming_it = [
                entry_bytes(&quarantined.locations[0]),
                entry_bytes(&quarantined.locations[1]),
            ];
            // 带着指它的位置条目的单元（下段叶里的数据指针、映射叶里的映射条目，走读都把那条位置条目的校验和记下），按第一次写的次序，
            // 每个单元的几份副本。每一片都写错一条：哪一片先被走到，它写错的那一盘只记一个校验和的话就只剩错的那一个
            let mut referrers: Vec<(u64, Vec<usize>)> = Vec::new();
            for (index, write) in flow.writes.iter().enumerate() {
                let Some(bytes) = write.bytes() else {
                    continue;
                };
                if write.kind != StepKind::UnitWrite
                    || !entries_naming_it.iter().any(|entry| {
                        bytes
                            .windows(entry.len())
                            .any(|window| window == &entry[..])
                    })
                {
                    continue;
                }
                match referrers
                    .iter_mut()
                    .find(|(offset, _)| *offset == write.offset.0)
                {
                    Some((_, copies_of_the_referrer)) => copies_of_the_referrer.push(index),
                    None => referrers.push((write.offset.0, vec![index])),
                }
            }
            assert!(
                referrers.len() >= 2,
                "流 {name} 里至少两片单元带着指被隔离的那个数据单元的位置条目（第一批之前那一版与第一批那一版各有下段叶），只找到 {}",
                referrers.len()
            );
            for (position, (_, copies_of_the_referrer)) in referrers.iter().enumerate() {
                let stale = &entries_naming_it[position % 2];
                corrupt_units_and_reseal_up_to_the_root(
                    &mut flow,
                    copies_of_the_referrer,
                    &|bytes| {
                        let occurrences: Vec<usize> = (0..=bytes.len() - stale.len())
                            .filter(|start| bytes[*start..*start + stale.len()] == stale[..])
                            .collect();
                        assert_eq!(
                            occurrences.len(),
                            1,
                            "这片单元里指着那一份的位置条目恰好一条"
                        );
                        let checksum = occurrences[0] + 10;
                        let written_wrong = read_u32_at(bytes, checksum) ^ 0xFFFF_FFFF;
                        bytes[checksum..checksum + 4].copy_from_slice(&written_wrong.to_le_bytes());
                    },
                );
            }
        }
    }
    flow
}

/// 豁免那一形：被隔离的单元盘 1 那一份对不上任何一条指着它的位置条目，整组豁免，第二批那一版上 I-3.11 照样成立
/// （不豁免的话已分配多一个单元、红）。CPU 一个状态都不红 I-3.11，GPU 每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_quarantined_data_unit_whose_second_copy_reads_back_other_bytes_is_exempted(
) {
    let test_name = "the_gpu_judge_agrees_with_the_cpu_that_a_quarantined_data_unit_whose_second_copy_reads_back_other_bytes_is_exempted";
    let flow =
        record_three_unit_file_through_its_second_deletion_batch_with_the_released_data_unit_quarantined(
            "crash-judge-gpu-quarantine-second-copy-reads-back-other-bytes",
            CopiesOfTheQuarantinedDataUnit::TheSecondReadsBackOtherBytes,
        );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
    assert!(
        comparison.cpu_checker_held_invariants.contains("I-3.11")
            && !comparison.cpu_checker_red_invariants.contains("I-3.11"),
        "流 {} 上 CPU 池级 checker 判过 I-3.11、一个状态都不红（被隔离的单元豁免了；成立过的 {:?}，红过的 {:?}）",
        flow.name,
        comparison.cpu_checker_held_invariants,
        comparison.cpu_checker_red_invariants
    );
}

/// 不豁免那一形：被隔离的单元两份都对得上某一条指着它的位置条目（带着这两条的每一片单元各写错一条），不豁免，CPU 判 I-3.11 红。
/// 位置项校验和那张表要把同 (盘, 槽) 的每一个不同的校验和都记下：只记一个的话，先走到的那一片写错的那一盘只剩错的那一个，那一份就对不上、整组豁免。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_quarantined_data_unit_whose_copies_each_match_one_location_entry_is_not_exempted(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_that_a_quarantined_data_unit_whose_copies_each_match_one_location_entry_is_not_exempted",
        &record_three_unit_file_through_its_second_deletion_batch_with_the_released_data_unit_quarantined(
            "crash-judge-gpu-quarantine-copies-each-match-one-location-entry",
            CopiesOfTheQuarantinedDataUnit::EachMatchesOneOfTheLocationEntriesNamingIt,
        ),
        "I-3.11",
    );
}

/// 分片的段（C790）：新池新建文件那条流照甲二展开，挑状态最多的那一段只判它域里后一半（片起点 = 段的态数的一半）。判表按片起点加片内序号拆出的
/// 持久掩码与不分片的判表在同一个段内编号上拆出的逐个相同（CPU 一侧，不要卡）；GPU 探针从片起点的各位起加片内序号、逐位进位拆出的掩码
/// 与 CPU 判表逐序号相同、一个都不判不了；GPU 判器判这张表的每个状态，对拍字与 CPU 判器逐状态相同。
// crash-case-check:not-a-crash-case 只判一张判表里一段分片之后的那几十个状态（新池新建文件那条流），单跑几秒
#[test]
fn the_gpu_judge_decodes_and_judges_a_sliced_segment_from_its_slice_start_like_the_cpu() {
    const TEST_NAME: &str =
        "the_gpu_judge_decodes_and_judges_a_sliced_segment_from_its_slice_start_like_the_cpu";
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let flow =
        record_new_pool_file_creation("crash-judge-gpu-sliced-segment", "new-pool-sliced-segment");
    let expansion = singlefs_checker_tier::crash::quick_tier_expansion;
    let judge_flow = JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    };
    let whole = build_judge_tables(&judge_flow).expect("建得出表");
    let (sliced_segment, states_of_the_segment) = whole
        .segments
        .iter()
        .enumerate()
        .map(|(segment_index, segment)| (segment_index, segment.states.end - segment.states.start))
        .max_by_key(|(_segment_index, states)| *states)
        .expect("流里至少一段");
    assert!(
        states_of_the_segment >= 2,
        "挑的那一段至少两个态，切得出后一半"
    );
    let slice_start = states_of_the_segment / 2;
    let slice_states = states_of_the_segment - slice_start;
    let mut segment_slices = singlefs_checker_tier::crash::no_segment_slices(flow.segments.len());
    segment_slices[sliced_segment] =
        Some(singlefs_checker_tier::crash::CrashPointReplaySegmentSlice {
            start_within_the_domain: singlefs_checker_tier::crash::StateCount1280Bits::from_u64(
                slice_start,
            ),
            state_count: slice_states,
        });
    let sliced = singlefs_checker_tier::crash_judge_tables::build_judge_tables_against_the_version_table_with_slices(
        &judge_flow,
        &[],
        &segment_slices,
    )
    .expect("分片的判表建得出");
    let start_of_the_slice_in_the_flow = sliced.segments[sliced_segment].states.start;
    let start_of_the_segment_in_the_whole_flow = whole.segments[sliced_segment].states.start;
    for ordinal_within_the_slice in 0..slice_states {
        assert_eq!(
            sliced.persisted_writes_of_state(
                start_of_the_slice_in_the_flow + ordinal_within_the_slice
            ),
            whole.persisted_writes_of_state(
                start_of_the_segment_in_the_whole_flow + slice_start + ordinal_within_the_slice
            ),
            "片内第 {ordinal_within_the_slice} 个是段内编号 {} 的那个态",
            slice_start + ordinal_within_the_slice
        );
    }
    let Some(packed) = packed_tables_through_the_library(TEST_NAME, &sliced) else {
        return;
    };
    let Some(lease) =
        cards.lease_card_or_report_not_run(TEST_NAME, mebibytes_a_judge_needs(&packed, 1))
    else {
        return;
    };
    let ordinals_of_the_slice =
        start_of_the_slice_in_the_flow..start_of_the_slice_in_the_flow + slice_states;
    let probe_words = singlefs_checker_tier::crash_judge_dispatch::persisted_masks_on_the_card(
        lease.card(),
        &packed,
        ordinals_of_the_slice.clone(),
    );
    drop(lease);
    let enumerated_writes = usize::try_from(sliced.enumerated_write_count).expect("装得进 usize");
    for (offset, words) in probe_words.iter().enumerate() {
        let ordinal = ordinals_of_the_slice.start + u64::try_from(offset).expect("装得进 u64");
        let (persisted, undecidable) =
            singlefs_checker_tier::crash_judge_gpu::persisted_set_of_the_probe_words(
                words,
                enumerated_writes,
            );
        assert_eq!(undecidable, 0, "序号 {ordinal}：探针拆得了");
        assert_eq!(
            persisted,
            sliced.persisted_writes_of_state(ordinal),
            "序号 {ordinal}：GPU 从片起点的各位起拆出的持久掩码与 CPU 相同"
        );
    }
    let Some(gpu) =
        singlefs_checker_tier::crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool(
            cards, TEST_NAME, &sliced,
        )
    else {
        return;
    };
    // CPU 参照一侧：不分片的那条流先入崩溃点重放枚举那一类的库、再从库里读回，判它的每个状态（用户 2026-10-13 定：CPU 参照也从库读）。
    // 这张表的序号换到不分片的编号上：切的那一段之前的段两边一样；那一段里片内第 k 个是段内编号 片起点 + k 的那个态（上面逐个核过持久掩码）；
    // 之后的段整体往后挪片起点那么多个
    assert_eq!(
        start_of_the_slice_in_the_flow, start_of_the_segment_in_the_whole_flow,
        "切的那一段之前的段两边一样，那一段在两张表里起在同一个序号"
    );
    let ordinal_in_the_whole_flow = |ordinal: u64| {
        if ordinal < start_of_the_slice_in_the_flow {
            ordinal
        } else {
            ordinal + slice_start
        }
    };
    let Some(cpu_by_ordinal) = cpu_cross_check_words_read_back_from_the_library(
        &format!("{TEST_NAME}-cpu-reference"),
        &flow,
        &expansion,
        0..whole.state_count,
    ) else {
        return;
    };
    let mismatches: Vec<String> = gpu
        .iter()
        .enumerate()
        .filter_map(|(ordinal, gpu_words)| {
            let ordinal = u64::try_from(ordinal).expect("装得进 u64");
            let whole_ordinal = ordinal_in_the_whole_flow(ordinal);
            let (cpu_persisted, cpu_cross_check_words) = &cpu_by_ordinal[&whole_ordinal];
            assert_eq!(
                *cpu_persisted,
                sliced.persisted_writes_of_state(ordinal),
                "序号 {ordinal}：CPU 从库里读回判的不分片序号 {whole_ordinal} 与这张表的那一个是同一个状态"
            );
            let gpu_cross_check_words = cross_check_words_of_gpu_verdict(gpu_words);
            (!cross_check_words_agree_in_the_cross_check(
                &gpu_cross_check_words,
                cpu_cross_check_words,
            ))
            .then(|| {
                format!(
                    "序号 {ordinal}：CPU {cpu_cross_check_words:?}；GPU {gpu_cross_check_words:?}"
                )
            })
        })
        .collect();
    assert_eq!(
        u64::try_from(gpu.len()).expect("装得进 u64"),
        sliced.state_count,
        "GPU 判了这张表的每个状态"
    );
    assert!(
        mismatches.is_empty(),
        "分片之后的 {} 个状态里 {} 个 GPU 与 CPU 的对拍字不同：\n{}",
        sliced.state_count,
        mismatches.len(),
        mismatches
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// 里程碑四快照、搬迁与 discard 那几条流发布用的实例代号：都在建池那个进程里接着发。
const INSTANCE_OF_THE_SNAPSHOT_AND_RELOCATION_FLOWS: InstanceGeneration = InstanceGeneration(1);

/// 快照下覆盖写的内容：与第一个文件（`common::file_content`）不同的一份。
fn content_overwritten_under_the_snapshot() -> Vec<u8> {
    (0..2700_u32)
        .map(|index| u8::try_from((index * 13 + 5) % 239).expect("小于 256"))
        .collect()
}

/// 一次发布交回的那一版在版本表里那一行：(实例代号, checkpoint_txg) 取它的根，内容由调用方按条款给。
fn published_version_of(output: &TransactionOutput, content: Vec<u8>) -> PublishedVersion {
    PublishedVersion {
        instance: output.root.instance,
        checkpoint_txg: output.root.checkpoint_txg,
        content,
    }
}

/// 同一个进程里接着 `pool.output` 建一个快照，池接到它那一版上，交回那一版。
fn create_a_snapshot_and_advance(pool: &mut BuiltPool) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let created = publish_create_snapshot(
        &mut writer,
        &mut pool.allocator,
        &previous,
        INSTANCE_OF_THE_SNAPSHOT_AND_RELOCATION_FLOWS,
    )
    .expect("建快照");
    pool.output = created.clone();
    created
}

/// 建的那个快照的树号（码 14 条目点名它）。
///
/// # Panics
/// 这一版的树表里码 14 条目不是恰好一条：这几条流只建一个快照。
fn the_only_snapshot_of(version: &TransactionOutput) -> TreeIdentifier {
    let snapshots: Vec<TreeIdentifier> = version
        .tree_table_entries
        .iter()
        .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .map(|entry| entry.tree)
        .collect();
    let [snapshot] = snapshots.as_slice() else {
        panic!("这一版的树表里恰好一条码 14 条目，实际 {snapshots:?}");
    };
    *snapshot
}

/// 同一个进程里接着 `pool.output` 在快照下覆盖写一次（内容 [`content_overwritten_under_the_snapshot`]），池接到它那一版上，交回那一版。
fn overwrite_under_the_snapshot_and_advance(pool: &mut BuiltPool) -> TransactionOutput {
    let previous = pool.output.clone();
    let overwritten = publish_overwrite_in_process(
        pool,
        &previous,
        &content_overwritten_under_the_snapshot(),
        FIXED_WRITE_TIME_SECONDS + 60,
        INSTANCE_OF_THE_SNAPSHOT_AND_RELOCATION_FLOWS,
    )
    .expect("快照下覆盖写");
    pool.output = overwritten.clone();
    overwritten
}

/// 删快照：意图那次发布，再按产品批大小走空销毁中的快照，池接到最后一版上；交回意图那一版与之后的每一批。
fn destroy_the_snapshot_and_advance(
    pool: &mut BuiltPool,
    snapshot: TreeIdentifier,
) -> Vec<TransactionOutput> {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let intent = publish_snapshot_destruction_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        snapshot,
        INSTANCE_OF_THE_SNAPSHOT_AND_RELOCATION_FLOWS,
    )
    .expect("删快照的意图");
    let mut current = intent.clone();
    let walk = walk_the_deletion_queue_until_it_is_empty(
        &mut writer,
        &mut pool.allocator,
        &mut current,
        UnlinkBatchSize::OneExtentLowerLeaf,
        INSTANCE_OF_THE_SNAPSHOT_AND_RELOCATION_FLOWS,
    );
    walk.stopped_by.expect("销毁中的快照走空走到底");
    pool.output = current;
    std::iter::once(intent)
        .chain(walk.steps_published)
        .collect()
}

/// 快照、deadlist 与删快照的流：新池新建文件（txg 3）、建快照（txg 4：树表多一对码 14 / 15 条目）、快照下覆盖写（txg 5：换下的
/// 数据单元、extent 树、inode 叶容器与根进 deadlist、映射条目留存）、删快照的意图（码 14 条目 flags 位 0）与之后走空的每一批
/// （最后一批删掉那一对条目）。每一版都带文件：建快照那一版是第一个文件的内容，覆盖写那一版起是覆盖写的内容。
///
/// # Panics
/// 覆盖写那一版的 deadlist 是空的、或删到最后那一版的树表里还有码 14 条目：那这条流就没走到要罩的那几种形态。
fn record_new_pool_through_a_snapshot_its_deadlist_and_its_destruction(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let mut pool = build_pool(tag);
    let mut versions = vec![published_version_of(&pool.output, file_content())];
    let created = create_a_snapshot_and_advance(&mut pool);
    versions.push(published_version_of(&created, file_content()));
    let snapshot = the_only_snapshot_of(&created);
    let overwritten = overwrite_under_the_snapshot_and_advance(&mut pool);
    assert!(
        !overwritten.deadlist_entries.is_empty(),
        "快照下覆盖写换下的用户可见单元进 deadlist"
    );
    versions.push(published_version_of(
        &overwritten,
        content_overwritten_under_the_snapshot(),
    ));
    for destroyed in destroy_the_snapshot_and_advance(&mut pool, snapshot) {
        versions.push(published_version_of(
            &destroyed,
            content_overwritten_under_the_snapshot(),
        ));
    }
    assert!(
        pool.output
            .tree_table_entries
            .iter()
            .all(|entry| entry.kind != TREE_KIND_SNAPSHOT_INODE_ROOT),
        "删到最后那一版的树表里没有码 14 条目"
    );
    recorded_flow_of_the_pool(&pool, name, versions)
}

/// 快照、deadlist 与删快照那条流的每个状态上 GPU 与 CPU 整份相同（红位、池级 checker 的违例位与判到位、两遍恢复落到的根与结局）：
/// 走读内核按快照树走（码 14 / 15 条目的树、I-9.16 ③ 成对、I-9.27、I-9.25 跳过销毁中的快照），引用与快照那一截判 I-3.5 ③ ④ 与逆向、
/// I-3.6、I-3.7（判决 m4-snapshot-r1 第三节 S5，D13（验证路线） 已定项 15）。CPU 池级 checker 在这条流上判过 I-3.5、I-3.6、I-3.7 而成立。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_snapshot_its_deadlist_and_its_destruction()
{
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_snapshot_its_deadlist_and_its_destruction";
    let flow = record_new_pool_through_a_snapshot_its_deadlist_and_its_destruction(
        "crash-judge-gpu-snapshot-deadlist-destruction",
        "snapshot-deadlist-and-destruction",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
    for invariant in ["I-3.5", "I-3.6", "I-3.7"] {
        assert!(
            comparison.cpu_checker_held_invariants.contains(invariant),
            "快照、deadlist 与删快照的流上 CPU 池级 checker 判过 {invariant} 而成立（成立过的：{:?}）",
            comparison.cpu_checker_held_invariants
        );
    }
}

/// 同一个进程里接着 `pool.output` 发整理的一批（搬 `planned` 里那几个单元），池接到它那一版上，交回那一版。
fn relocate_and_advance(pool: &mut BuiltPool, planned: &[PlannedRelocation]) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let relocated = publish_relocation_batch(
        &mut writer,
        &mut pool.allocator,
        &previous,
        planned,
        INSTANCE_OF_THE_SNAPSHOT_AND_RELOCATION_FLOWS,
    )
    .expect("整理的一批");
    pool.output = relocated.clone();
    relocated
}

/// 搬 `version` 里 `identity` 这个角色的那一项计划：映射 key 与它按映射位置的源落点。
///
/// # Panics
/// 这个角色不进映射、或在这一版的映射里查不到。
fn planned_relocation_of_the_role(
    version: &TransactionOutput,
    identity: TransactionUnit,
) -> PlannedRelocation {
    let (_, mapping_key) = version
        .mapped_units
        .iter()
        .find(|(role, _)| *role == identity)
        .expect("这个角色进映射");
    let locations = match central_mapping_locations_in_the_version(version, mapping_key) {
        MappingLookup::Found(locations) => locations,
        MappingLookup::NodeMalformedOrNoEntryWithThisKey
        | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => {
            panic!("{identity:?} 在这一版的映射里查得到")
        }
    };
    PlannedRelocation {
        mapping_key: mapping_key.clone(),
        source: Placement {
            slot: locations[0].slot,
            span: identity.span_slots(),
        },
    }
}

/// 搬迁的流：新池新建文件（txg 3）之后整理的一批搬第一个文件唯一的数据单元（txg 4：原样字节写到目的地、映射条目换到目的地、
/// 源落点进 defer；extent 树里那条指针的位置条目不动，成了过期的提示）。两版都是第一个文件的内容。
fn record_new_pool_with_the_data_unit_relocated(tag: &str, name: &'static str) -> RecordedFlow {
    let mut pool = build_pool(tag);
    let created = pool.output.clone();
    let planned =
        planned_relocation_of_the_role(&created, TransactionUnit::Data(DataUnitIndexInFile::FIRST));
    let batch = relocate_and_advance(&mut pool, std::slice::from_ref(&planned));
    assert_eq!(
        batch.units_relocated_by_this_publish.len(),
        1,
        "这一批搬了一个单元"
    );
    recorded_flow_of_the_pool(
        &pool,
        name,
        vec![
            published_version_of(&created, file_content()),
            published_version_of(&batch, file_content()),
        ],
    )
}

/// 搬迁的流（数据单元搬走、提示过期）每个状态上 GPU 与 CPU 整份相同：两个检查内核把码 1 / 2 / 3 的位置条目当提示——先按提示读、
/// 两份都对不上再按映射位置读，引用记映射位置（判决 m4-gc-r1 第三节 G3）；恢复内核与 core 的冷走读同样经映射回退。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_data_unit_relocation() {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_data_unit_relocation";
    let flow = record_new_pool_with_the_data_unit_relocated(
        "crash-judge-gpu-data-unit-relocation",
        "data-unit-relocation",
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_every_state_of_the_flow(TEST_NAME, &flow);
}

/// 快照钉住的单元被搬的流：新池新建文件（txg 3）、建快照（txg 4）、快照下覆盖写（txg 5，旧数据单元只被快照引用、映射条目留存）、
/// 整理的一批搬那个旧数据单元（txg 6：留存的映射条目换到目的地，快照树里的提示过期）。覆盖写那一版起是覆盖写的内容。
fn record_new_pool_with_a_unit_kept_for_the_snapshot_relocated(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let mut pool = build_pool(tag);
    let mut versions = vec![published_version_of(&pool.output, file_content())];
    let created = create_a_snapshot_and_advance(&mut pool);
    versions.push(published_version_of(&created, file_content()));
    let overwritten = overwrite_under_the_snapshot_and_advance(&mut pool);
    versions.push(published_version_of(
        &overwritten,
        content_overwritten_under_the_snapshot(),
    ));
    let (kept_key, kept_locations) = overwritten
        .mapping_entries_kept_for_the_snapshots
        .iter()
        .find(|(key, _)| key.first() == Some(&UNIT_CLASS_DATA))
        .cloned()
        .expect("旧数据单元的映射条目留存着");
    let planned = PlannedRelocation {
        mapping_key: kept_key,
        source: Placement {
            slot: kept_locations[0].slot,
            span: TransactionUnit::Data(DataUnitIndexInFile::FIRST).span_slots(),
        },
    };
    let batch = relocate_and_advance(&mut pool, std::slice::from_ref(&planned));
    assert_eq!(
        batch
            .units_relocated_by_this_publish
            .iter()
            .map(|relocated| relocated.holder)
            .collect::<Vec<_>>(),
        vec![RelocatedUnitHolder::KeptForTheSnapshots],
        "搬的是只被快照引用的那一个"
    );
    versions.push(published_version_of(
        &batch,
        content_overwritten_under_the_snapshot(),
    ));
    recorded_flow_of_the_pool(&pool, name, versions)
}

/// 快照钉住的单元被搬的流每个状态上 GPU 与 CPU 整份相同：走读内核走快照树时同样先按提示读、再按映射位置读，
/// 引用与快照那一截按映射位置认单元（判决 m4-gc-r1 第三节 G3「被快照引用的块照样能搬」）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_relocation_of_a_unit_kept_for_the_snapshot(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_relocation_of_a_unit_kept_for_the_snapshot";
    let flow = record_new_pool_with_a_unit_kept_for_the_snapshot_relocated(
        "crash-judge-gpu-relocation-of-a-unit-kept-for-the-snapshot",
        "relocation-of-a-unit-kept-for-the-snapshot",
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_every_state_of_the_flow(TEST_NAME, &flow);
}

/// discard 的流：新池新建文件的流尾另起一段，放一次盘 0 上罩住第一个文件数据单元那一份的整段 discard（64 槽对齐、64 槽长，
/// 种类 `StepKind::Discard`，写表里按「范围写零」记）。这一段取 D13（验证路线） 已定项 4 的三态：没落、整段是 0、撕裂。
fn record_new_pool_with_the_segment_of_the_data_unit_discarded_on_device_zero(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(
        tag,
        "new-pool-segment-of-the-data-unit-discarded-on-device-zero",
    );
    let data_unit_slot = flow
        .writes
        .iter()
        .find(|write| {
            write.kind == StepKind::UnitWrite
                && write.device == DeviceIdentity(0)
                && write
                    .bytes()
                    .is_some_and(|bytes| unit_class(bytes) == UNIT_CLASS_DATA)
        })
        .expect("新池新建文件在盘 0 上写一份数据单元")
        .offset
        .0
        / SLOT_BYTES;
    let segment_start = data_unit_slot - data_unit_slot % singlefs_format::CLUSTER_SEGMENT_SLOTS;
    append_a_segment_of_writes(
        &mut flow,
        vec![RetainedWrite {
            device: DeviceIdentity(0),
            kind: StepKind::Discard,
            is_force_unit_access: false,
            offset: singlefs_core::address::DeviceOffsetInBytes(segment_start * SLOT_BYTES),
            contents: WrittenContents::Zeros {
                length: singlefs_format::CLUSTER_SEGMENT_SLOTS * SLOT_BYTES,
            },
        }],
    );
    flow
}

/// discard 的流建得出判表（第一版不再拒 `StepKind::Discard`）、写表里那一次 discard 不占落点，每个状态上 GPU 与 CPU 整份相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_discard_over_the_segment_of_a_data_unit() {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_discard_over_the_segment_of_a_data_unit";
    let flow = record_new_pool_with_the_segment_of_the_data_unit_discarded_on_device_zero(
        "crash-judge-gpu-discard-over-a-data-unit",
    );
    let tables = judge_tables_of_the_flow(&flow);
    let discard = tables.writes.last().expect("写表里至少有那一次 discard");
    assert_eq!(discard.kind, StepKind::Discard, "流尾那一次写是 discard");
    assert_eq!(discard.location, None, "discard 罩多个落点，不占一个落点");
    assert_the_gpu_judge_agrees_with_the_cpu_on_every_state_of_the_flow(TEST_NAME, &flow);
}

// ───────── 里程碑四自建 GC 新判的那几条上 GPU（判决 m4-gc-r1 第三节 G1、G3 的 N2–N6）─────────

/// 同一个进程里接着 `pool.output` 发整理的一次发布（搬 `planned` 里那几个单元、对意图做 `intent_step`），池接到它那一版上，交回那一版。
fn compaction_step_and_advance(
    pool: &mut BuiltPool,
    planned: &[PlannedRelocation],
    intent_step: CompactionIntentStep,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let published = publish_compaction_step(
        &mut writer,
        &mut pool.allocator,
        &previous,
        planned,
        intent_step,
        INSTANCE_OF_THE_SNAPSHOT_AND_RELOCATION_FLOWS,
    )
    .expect("整理的一次发布");
    pool.output = published.clone();
    published
}

/// 整理意图的流：新池新建文件（txg 3）、签发整理意图（txg 4：logged_ops 写一条，R 是第一个文件数据单元所在的那一段，c0 取新建文件那一版的 txg；
/// 分配器这一侧同时装上 R 排他）、整理的一批搬那个数据单元（txg 5：意图照抄）、丢意图（txg 6：logged_ops 删掉那一条，树表条目根指针回到全零）。
/// 四版都是第一个文件的内容。
fn record_new_pool_through_a_compaction_intent_a_relocation_and_its_drop(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let mut pool = build_pool(tag);
    let created = pool.output.clone();
    let data_unit_slot = created
        .unit(TransactionUnit::Data(DataUnitIndexInFile::FIRST))
        .slot;
    let region = CompactionRegion::of_one_cluster_segment(
        DeviceIdentity(0),
        SlotNumber(data_unit_slot.0 - data_unit_slot.0 % singlefs_format::CLUSTER_SEGMENT_SLOTS),
    )
    .expect("段起点是 64 的倍数、装得进 6 字节");
    let intent = CompactionIntent {
        region,
        allocation_generation_at_issue: created.root.checkpoint_txg,
        empty_cluster_segments_at_issue: 0,
    };
    pool.allocator.compaction_mut().issue(intent);
    let issued = compaction_step_and_advance(&mut pool, &[], CompactionIntentStep::Issue(intent));
    assert_eq!(
        issued.logged_operations.len(),
        1,
        "签发那一版的 logged_ops 里一条意图"
    );
    let planned =
        planned_relocation_of_the_role(&issued, TransactionUnit::Data(DataUnitIndexInFile::FIRST));
    let batch = compaction_step_and_advance(
        &mut pool,
        std::slice::from_ref(&planned),
        CompactionIntentStep::CarryTheIntent,
    );
    assert_eq!(
        batch.units_relocated_by_this_publish.len(),
        1,
        "这一批搬了一个单元"
    );
    let dropped = compaction_step_and_advance(&mut pool, &[], CompactionIntentStep::Drop);
    pool.allocator.compaction_mut().withdraw_the_live_intent();
    assert!(
        dropped.logged_operations.is_empty(),
        "丢意图那一版的 logged_ops 空了"
    );
    let versions = [&created, &issued, &batch, &dropped]
        .into_iter()
        .map(|version| published_version_of(version, file_content()))
        .collect();
    recorded_flow_of_the_pool(&pool, name, versions)
}

/// 整理意图那条流的每个状态上 GPU 与 CPU 整份相同（红位、池级 checker 的违例位与判到位、两遍恢复落到的根与结局）：走读内核与引用集合那一遍
/// 把 logged_ops 树当多层码 2 树走、节点记进引用，恢复内核把 logged_ops 根数进映射条目数，扫描内核判 I-3.16，引用与快照那一截之后判 I-4.4、I-5.6、
/// I-3.15、I-1.14（D13（验证路线） 已定项 15）。CPU 池级 checker 在这条流上判过那五条而成立。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_compaction_intent_a_relocation_and_its_drop(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_compaction_intent_a_relocation_and_its_drop";
    let flow = record_new_pool_through_a_compaction_intent_a_relocation_and_its_drop(
        "crash-judge-gpu-compaction-intent-relocation-drop",
        "compaction-intent-relocation-and-drop",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        mismatch_listing(&comparison)
    );
    for invariant in ["I-4.4", "I-5.6", "I-3.15", "I-3.16", "I-1.14"] {
        assert!(
            comparison.cpu_checker_held_invariants.contains(invariant),
            "整理意图那条流上 CPU 池级 checker 判过 {invariant} 而成立（成立过的：{:?}）",
            comparison.cpu_checker_held_invariants
        );
    }
}

/// 流里每一次命中 `select` 的单元写的下标（至少一个单元的两份副本）。
fn every_unit_write_where(
    flow: &RecordedFlow,
    what: &str,
    select: &dyn Fn(&[u8]) -> bool,
) -> Vec<usize> {
    let matching: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::UnitWrite && write.bytes().is_some_and(select))
        .map(|(index, _)| index)
        .collect();
    assert!(
        matching.len() >= 2,
        "流 {} 里命中「{what}」的单元写至少两条（一个单元的两份副本），只有 {} 条",
        flow.name,
        matching.len()
    );
    matching
}

/// 码 2 节点的条目从头的末尾再过 29 字节起（同 `only_tree_table_entry_of_kind` 的 131 = 86 + 2 × 8 + 29）。
fn first_index_node_entry_offset(bytes: &[u8]) -> usize {
    86 + 2 * usize::from(index_node_key_width(bytes)) + 29
}

/// logged_ops 条目里 value 的起点（key 10 之后）与 value 里 c0、补零两段的偏移（判决 G1 字段表；walk.rs 的
/// `COMPACTION_INTENT_ALLOCATION_GENERATION_AT_ISSUE_OFFSET_IN_THE_VALUE`、`COMPACTION_INTENT_PADDING_OFFSET_IN_THE_VALUE`）。
const LOGGED_OPERATION_VALUE_OFFSET_IN_THE_ENTRY: usize = 10;
const COMPACTION_INTENT_ALLOCATION_GENERATION_AT_ISSUE_OFFSET_IN_THE_VALUE: usize = 12;
const COMPACTION_INTENT_PADDING_OFFSET_IN_THE_VALUE: usize = 28;

/// 整理意图那条流上改坏 logged_ops 根（码 2、key 宽 10、条目宽 44）的每一份——签发那次写的与之后照抄时写的都改——里第一条条目，重封到根。
fn record_the_compaction_intent_flow_with_its_logged_operation_changed(
    tag: &str,
    name: &'static str,
    mutate_the_first_entry: &dyn Fn(&mut [u8]),
) -> RecordedFlow {
    let mut flow = record_new_pool_through_a_compaction_intent_a_relocation_and_its_drop(tag, name);
    let logged_operations_root = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 10
            && index_node_entry_width(bytes) == 44
    };
    let targets = every_unit_write_where(&flow, "logged_ops 根", &logged_operations_root);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
        let entry = first_index_node_entry_offset(bytes);
        mutate_the_first_entry(&mut bytes[entry..entry + 44]);
    });
    flow
}

/// I-4.4（意图只描述目标态，N4）：整理意图 value 末尾的补零第一个字节写 1（「再搬 N 个」这样的增量只能落在补零里），每个状态 GPU 与 CPU 整份相同、
/// CPU 红 I-4.4。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_compaction_intent_carries_an_increment_in_its_padding()
{
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_a_compaction_intent_carries_an_increment_in_its_padding";
    let flow = record_the_compaction_intent_flow_with_its_logged_operation_changed(
        "crash-judge-gpu-compaction-intent-padding",
        "compaction-intent-with-an-increment-in-its-padding",
        &|entry| {
            entry[LOGGED_OPERATION_VALUE_OFFSET_IN_THE_ENTRY
                + COMPACTION_INTENT_PADDING_OFFSET_IN_THE_VALUE] = 1;
        },
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(TEST_NAME, &flow, "I-4.4");
}

/// I-5.6（活整理意图的区域里没有签发之后分配的记录，N3）：整理意图的 c0 改成 0，R 里新建文件那一版分配的数据单元的记录分配代大于它，
/// 每个状态 GPU 与 CPU 整份相同、CPU 红 I-5.6。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_region_of_a_compaction_intent_holds_records_allocated_after_its_c0(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_the_region_of_a_compaction_intent_holds_records_allocated_after_its_c0";
    let flow = record_the_compaction_intent_flow_with_its_logged_operation_changed(
        "crash-judge-gpu-compaction-intent-c0-zero",
        "compaction-intent-with-c0-zero",
        &|entry| {
            let at = LOGGED_OPERATION_VALUE_OFFSET_IN_THE_ENTRY
                + COMPACTION_INTENT_ALLOCATION_GENERATION_AT_ISSUE_OFFSET_IN_THE_VALUE;
            entry[at..at + 8].copy_from_slice(&0u64.to_le_bytes());
        },
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(TEST_NAME, &flow, "I-5.6");
}

/// 没人分配也没写副本的一个槽（新池新建文件之后，同 `checker_compaction_intent_and_relocation_must_red_worlds.rs` 的 `SLOT_NOBODY_ALLOCATED`）。
const SLOT_NOBODY_ALLOCATED: u64 = 50_400;

/// 中央映射叶条目 55 字节里两条位置条目的槽号（6 字节）在条目里的偏移：key 27 之后，各条 盘 4 + 槽 6 + 整单元校验和 4。
const MAPPING_ENTRY_LOCATION_SLOT_OFFSETS: [usize; 2] = [27 + 4, 27 + 14 + 4];

/// I-3.15（N2 反查）「只改映射不写副本」：新池新建文件之后，中央映射叶里数据单元那一条的两条位置条目改指一个没人分配、也没写副本的槽，
/// 重封到根。源槽那条未释放记录罩住的数据单元在映射里那把相符的 key 落在别处：每个状态 GPU 与 CPU 整份相同、CPU 红 I-3.15。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_mapping_entry_moved_without_a_copy() {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_a_mapping_entry_moved_without_a_copy";
    let mut flow = record_new_pool_file_creation(
        "crash-judge-gpu-mapping-entry-moved-without-a-copy",
        "new-pool-mapping-entry-moved-without-a-copy",
    );
    let data_unit_slot = flow
        .writes
        .iter()
        .find(|write| {
            write.kind == StepKind::UnitWrite
                && write.device == DeviceIdentity(0)
                && write.bytes().is_some_and(|bytes| unit_class(bytes) == 1)
        })
        .expect("新池新建文件在盘 0 上写一份数据单元")
        .offset
        .0
        / singlefs_format::SLOT_BYTES;
    let mapping_leaf = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 27
            && index_node_entry_width(bytes) == 55
    };
    let targets = last_unit_copies(&flow, "中央映射叶", &mapping_leaf, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
        let header_end = 86 + 2 * usize::from(index_node_key_width(bytes));
        let count = usize::from(u16::from_le_bytes(
            bytes[header_end - 4..header_end - 2].try_into().expect("2"),
        ));
        let entry = (0..count)
            .map(|position| first_index_node_entry_offset(bytes) + position * 55)
            .find(|entry| {
                let at = entry + MAPPING_ENTRY_LOCATION_SLOT_OFFSETS[0];
                let mut slot = [0u8; 8];
                slot[..6].copy_from_slice(&bytes[at..at + 6]);
                u64::from_le_bytes(slot) == data_unit_slot
            })
            .expect("映射叶里有数据单元那一条");
        for offset in MAPPING_ENTRY_LOCATION_SLOT_OFFSETS {
            bytes[entry + offset..entry + offset + 6]
                .copy_from_slice(&SLOT_NOBODY_ALLOCATED.to_le_bytes()[..6]);
        }
    });
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(TEST_NAME, &flow, "I-3.15");
}

/// runs 那一统计量的标签（D5（快照 / 空间记账机制） 已定项 4 第 10 项；walk.rs 的 `STATISTIC_FRAGMENTATION_RUNS`）。记账行里设备、值两段的偏移
/// 用这份文件里记账那一节的 `ACCOUNTING_ROW_DEVICE_OFFSET`、`ACCOUNTING_ROW_VALUE_OFFSET`。
const STATISTIC_FRAGMENTATION_RUNS: u16 = 10;

/// I-3.16（N5 = C413）「增量少加一次」：新池新建文件那一版记账叶（根兼叶）里盘 0 的 runs 那一行减一，重封到根。全扫重算的 runs 与它对不上：
/// 每个状态 GPU 与 CPU 整份相同、CPU 红 I-3.16。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_runs_row_of_a_device_is_one_short() {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_the_runs_row_of_a_device_is_one_short";
    let accounting_leaf = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 22
            && index_node_entry_width(bytes) == 34
    };
    let flow = record_new_pool_with_a_resealed_field(
        "crash-judge-gpu-runs-row-one-short",
        "new-pool-runs-row-of-device-zero-one-short",
        &accounting_leaf,
        false,
        &|bytes| {
            let header_end = 86 + 2 * usize::from(index_node_key_width(bytes));
            let count = usize::from(u16::from_le_bytes(
                bytes[header_end - 4..header_end - 2].try_into().expect("2"),
            ));
            let row = (0..count)
                .map(|position| first_index_node_entry_offset(bytes) + position * 34)
                .find(|row| {
                    u16::from_le_bytes(bytes[*row..*row + 2].try_into().expect("2"))
                        == STATISTIC_FRAGMENTATION_RUNS
                        && u32::from_le_bytes(
                            bytes[row + ACCOUNTING_ROW_DEVICE_OFFSET
                                ..row + ACCOUNTING_ROW_DEVICE_OFFSET + 4]
                                .try_into()
                                .expect("4"),
                        ) == 0
                })
                .expect("记账叶里有盘 0 的 runs 那一行");
            let value_at = row + ACCOUNTING_ROW_VALUE_OFFSET;
            let runs = u64::from_le_bytes(bytes[value_at..value_at + 8].try_into().expect("8"));
            bytes[value_at..value_at + 8].copy_from_slice(
                &runs
                    .checked_sub(1)
                    .expect("新池新建文件那一版盘 0 的 runs 至少 1")
                    .to_le_bytes(),
            );
        },
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(TEST_NAME, &flow, "I-3.16");
}

/// 数据单元载荷起点（码 1 明文头 105 字节之后）往后第 100 个字节：改它不碰头里的任何字段。
const A_PAYLOAD_BYTE_OF_THE_DATA_UNIT: usize = 105 + 100;

/// I-1.14（N6）：搬迁的流上搬到目的地的那一份数据单元（流里最后一次码 1 单元写，两份副本）载荷改一个字节、重封，映射条目的校验和跟着补。
/// 数据单元那把 key 在新建文件那一版指源槽那一份、在这一批指目的槽那一份，两份字节不同：每个状态 GPU 与 CPU 整份相同、CPU 红 I-1.14。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_byte_changed_in_the_relocated_copy() {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_a_byte_changed_in_the_relocated_copy";
    let mut flow = record_new_pool_with_the_data_unit_relocated(
        "crash-judge-gpu-relocated-copy-changed",
        "relocated-copy-with-a-byte-changed",
    );
    let data_unit = |bytes: &[u8]| unit_class(bytes) == 1;
    let targets = last_unit_copies(&flow, "搬到目的地的那份数据单元", &data_unit, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
        bytes[A_PAYLOAD_BYTE_OF_THE_DATA_UNIT] ^= 0xff;
    });
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(TEST_NAME, &flow, "I-1.14");
}

// ───────────── 截断那一批：I-4.6 改判、I-4.10、I-9.38、I-4.4 收截断条目、上段叶条目标签 3、4（判决 m4-trunc-r1 第三节 K5、K7，第六节 S1、S2）─────────────
// GPU 恢复内核照 core 读截断中间态与洞（判决 K2、K5 的读者那一半与第六节 S5：size 之外挂着的单元不读、洞读 0），这几条流都整份比：
// 红位、池级 checker、两遍恢复的结局与落到的根（`Comparison::mismatches` 与 `recovery_mismatches`）。

/// 截断与洞那条流走到哪一步停（[`record_new_pool_through_truncations_and_holes`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum TruncationsAndHolesFlowStop {
    /// 三个单元的文件（txg 4）：下段根、标签 1，三条下段叶记录偏移 0、P、2P（P 是净荷容量）。
    AfterTheThreeUnitFile,
    /// 缩到一个半净荷那次意图（txg 5）：单元 1 重写成半个净荷，单元 2 在 size 之外挂着、截断条目点名第一个文件。
    AfterTheIntentOfTheFirstTruncation,
    /// 缩完（最后一批放掉单元 2、删截断条目）再变长到三个半净荷（txg 7）：单元 1 补零重写成一整个净荷，之后到 size 是洞，标签 3。
    AfterTheHoleBehindALowerSegment,
    /// 走到底：再缩到一个净荷（意图 + 各批）、变长到两个半净荷（一个单元之后是洞，标签 3 或 4）、缩到 0（意图 + 放 0 个单元的最后一批）。
    AtTheEnd,
}

/// 第一个文件改成 `new_size` 字节那一版读出的内容：`current` 截到 `new_size`，或在它后面补 0 到 `new_size`（变长的洞读 0）。
fn content_after_the_size_change(current: &[u8], new_size: usize) -> Vec<u8> {
    let mut content = current[..new_size.min(current.len())].to_vec();
    content.resize(new_size, 0);
    content
}

/// 截断的各批（每批一个单元）一直发到截断条目删掉为止（最后一批放完剩下的、同一次发布删条目，可以 0 个单元，判决 K2、K11），
/// 每一版的内容都是 `content`（意图那次已把 size 写成目标）。
fn publish_the_truncation_batches_until_the_entry_is_gone(
    pool: &mut BuiltPool,
    versions: &mut Vec<PublishedVersion>,
    content: &[u8],
) {
    const MOST_BATCHES_OF_ONE_ON_THIS_FLOW: usize = 4;
    for _batch_index in 0..MOST_BATCHES_OF_ONE_ON_THIS_FLOW {
        if common_truncation::truncation_entry_of(&pool.output).is_none() {
            return;
        }
        let published = common_truncation::publish_the_next_truncation_step_and_advance(
            pool,
            common_truncation::batches_of_one(),
        );
        versions.push(PublishedVersion {
            instance: published.root.instance,
            checkpoint_txg: published.root.checkpoint_txg,
            content: content.to_vec(),
        });
    }
    assert!(
        common_truncation::truncation_entry_of(&pool.output).is_none(),
        "这条流上每次截断至多放 {MOST_BATCHES_OF_ONE_ON_THIS_FLOW} 批就删掉截断条目"
    );
}

/// 第一个文件改成 `new_size` 字节（截断时只发意图那一次），版本表记下这一版，交回这一版读出的内容。
fn change_the_size_and_note_the_version(
    pool: &mut BuiltPool,
    versions: &mut Vec<PublishedVersion>,
    current: &[u8],
    new_size: usize,
) -> Vec<u8> {
    let published = common_truncation::change_the_size_of_the_first_file_and_advance(
        pool,
        u64::try_from(new_size).expect("装得进 u64"),
    );
    let content = content_after_the_size_change(current, new_size);
    versions.push(PublishedVersion {
        instance: published.root.instance,
        checkpoint_txg: published.root.checkpoint_txg,
        content: content.clone(),
    });
    content
}

/// 截断与洞的流（`common_truncation` 的搭建，批大小 1）：新池新建文件（txg 3）、第一个文件顺序写成三个数据单元（txg 4，两个半净荷）、
/// 缩到一个半净荷（意图 txg 5，最后一批 txg 6）、变长到三个半净荷（txg 7，标签 3）、缩到一个净荷（意图 + 最后一批：洞整个截掉、换标签写在意图那次，
/// 判决第六节 S1）、变长到两个半净荷（一个单元之后是洞）、缩到 0（意图把单元 0 重写成声明长度 0，最后一批放 0 个单元）。版本表照每一版读出的内容记。
fn record_new_pool_through_truncations_and_holes(
    tag: &str,
    name: &'static str,
    stop: TruncationsAndHolesFlowStop,
) -> RecordedFlow {
    let payload = singlefs_core::unit::data_unit_payload_capacity();
    let mut pool = pool_with_a_three_unit_file(tag);
    let mut versions = versions_with_the_first_file_before_its_deletion();
    let mut content = content_of_three_data_units();
    if stop >= TruncationsAndHolesFlowStop::AfterTheIntentOfTheFirstTruncation {
        content = change_the_size_and_note_the_version(
            &mut pool,
            &mut versions,
            &content,
            payload + payload / 2,
        );
    }
    if stop >= TruncationsAndHolesFlowStop::AfterTheHoleBehindALowerSegment {
        publish_the_truncation_batches_until_the_entry_is_gone(&mut pool, &mut versions, &content);
        content = change_the_size_and_note_the_version(
            &mut pool,
            &mut versions,
            &content,
            3 * payload + payload / 2,
        );
        assert_eq!(
            common_truncation::upper_leaf_tag_of_the_first_file(&pool.output),
            3,
            "两个单元之后到三个半净荷是洞：下段根、标签 3"
        );
    }
    if stop >= TruncationsAndHolesFlowStop::AtTheEnd {
        content = change_the_size_and_note_the_version(&mut pool, &mut versions, &content, payload);
        publish_the_truncation_batches_until_the_entry_is_gone(&mut pool, &mut versions, &content);
        content = change_the_size_and_note_the_version(
            &mut pool,
            &mut versions,
            &content,
            2 * payload + payload / 2,
        );
        assert_eq!(
            common_truncation::upper_leaf_tag_of_the_first_file(&pool.output),
            4,
            "缩到一个净荷之后只剩单元 0、内联（只有一个单元的文件不建下段），变长到两个半净荷之后到 size 是洞：标签 4"
        );
        content = change_the_size_and_note_the_version(&mut pool, &mut versions, &content, 0);
        publish_the_truncation_batches_until_the_entry_is_gone(&mut pool, &mut versions, &content);
    }
    recorded_flow_of_the_pool(&pool, name, versions)
}

/// 截断那几条流上 GPU 判器与 CPU 逐状态整份相同（[`assert_the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_truncation_flow`]），
/// CPU 池级 checker 在这条流上红 `invariant`。本机没有 NVIDIA 独显适配器时报「本次未跑」返回。
fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_flow_reddening(
    test_name: &str,
    flow: &RecordedFlow,
    invariant: &'static str,
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains(invariant),
        "流 {} 上 CPU 池级 checker 要红 {invariant}（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_truncation_flow(flow, &comparison);
}

/// 截断那几条流上每个状态整份相同：对拍字、红位、池级 checker 违例位逐位相同、没有判不了的（[`Comparison::mismatches`] 是空的），
/// 两遍恢复的结局与看 journal 那一遍落到的根相同（[`Comparison::recovery_mismatches`] 是 0；它也算进 `mismatches`，单列一句是为了红的时候
/// 一眼看出是恢复内核那一半），池级 checker 那一半另核一遍（[`Comparison::checker_mismatches`] 是空的）。
fn assert_the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_truncation_flow(
    flow: &RecordedFlow,
    comparison: &Comparison,
) {
    let first_lines = |lines: &[String]| {
        lines
            .iter()
            .filter(|line| !line.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(
        comparison.recovery_mismatches,
        0,
        "流 {}：{} 个状态 GPU 与 CPU 两遍恢复的结局或看 journal 那一遍落到的根不同（不一致的前几个）：\n{}",
        flow.name,
        comparison.recovery_mismatches,
        first_lines(&comparison.mismatches)
    );
    assert!(
        comparison.checker_mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 的池级 checker 不一致（前几个）：\n{}",
        flow.name,
        comparison.checker_mismatches.len(),
        first_lines(&comparison.checker_mismatches)
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        first_lines(&comparison.mismatches)
    );
}

/// 截断与洞那条合法流走到底：每个状态 GPU 与 CPU 整份相同（两遍恢复读截断中间态与洞、池级 checker）；CPU 池级 checker 在这条流上判过 I-4.6、
/// I-4.10、I-9.38、I-4.4（截断条目）与 I-9.39（截断第二轮：每条记录判一次，截庚合进来之前记录的截断字段都是 0、照判成立）而成立、一个状态都不红。GPU 两个池级 checker 内核都认标签 3、4（不认的话走读内核红 I-1.1、扫描内核读不全
/// extent 树，判到位对不上）；恢复内核认标签 3、4、单元数按标签对 size、size 之外挂着的单元不读、洞按全零比（不照的话截断中间态与有洞的那几版
/// 两遍恢复报走读失败或内容不同，CPU 读出文件）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_truncations_and_holes() {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_on_every_state_of_truncations_and_holes";
    let flow = record_new_pool_through_truncations_and_holes(
        "crash-judge-gpu-truncations-and-holes",
        "truncations-and-holes",
        TruncationsAndHolesFlowStop::AtTheEnd,
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert_the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_truncation_flow(
        &flow,
        &comparison,
    );
    for invariant in ["I-4.6", "I-4.10", "I-9.38", "I-4.4", "I-9.39"] {
        assert!(
            comparison.cpu_checker_held_invariants.contains(invariant),
            "截断与洞的流上 CPU 池级 checker 判过 {invariant} 而成立（成立过的：{:?}）",
            comparison.cpu_checker_held_invariants
        );
        assert!(
            !comparison.cpu_checker_red_invariants.contains(invariant),
            "截断与洞的流上 CPU 池级 checker 不红 {invariant}（红过的：{:?}）",
            comparison.cpu_checker_red_invariants
        );
    }
}

/// 截断条目 value 里 inode 号、对象出生代、目标 size 之后的补零（判决 K1：value 34 = 8 + 8 + 8 + 补零 10）的偏移与对象出生代的偏移。
const TRUNCATION_INTENT_OBJECT_BIRTH_OFFSET_IN_THE_VALUE: usize = 8;
const TRUNCATION_INTENT_PADDING_OFFSET_IN_THE_VALUE: usize = 24;
/// logged_ops 条目的种类码：2 是截断（判决 K1）。
const LOGGED_OPERATION_KIND_TRUNCATION: u16 = 2;

/// 缩到一个半净荷那次意图之后的流上，改坏 logged_ops 根（码 2、key 宽 10、条目宽 44）最后写的那一份（两份副本）里第一条条目——那条截断条目，重封到根。
fn record_the_first_truncation_intent_with_its_logged_operation_changed(
    tag: &str,
    name: &'static str,
    mutate_the_truncation_entry: &dyn Fn(&mut [u8]),
) -> RecordedFlow {
    let mut flow = record_new_pool_through_truncations_and_holes(
        tag,
        name,
        TruncationsAndHolesFlowStop::AfterTheIntentOfTheFirstTruncation,
    );
    let logged_operations_root = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 10
            && index_node_entry_width(bytes) == 44
    };
    let targets = last_unit_copies(
        &flow,
        "截断意图那次写的 logged_ops 根",
        &logged_operations_root,
        false,
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
        let entry = first_index_node_entry_offset(bytes);
        assert_eq!(
            read_u16_at(bytes, entry),
            LOGGED_OPERATION_KIND_TRUNCATION,
            "logged_ops 根里第一条是截断条目"
        );
        mutate_the_truncation_entry(&mut bytes[entry..entry + 44]);
    });
    flow
}

/// I-4.4（截断条目只描述目标态，判决 K7 T-1）：截断条目补零的第一个字节写 1（「再截 N 个」这样的增量只能落在补零里），
/// 每个状态 GPU 与 CPU 整份相同（两遍恢复与池级 checker）、CPU 红 I-4.4。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_truncation_intent_carries_an_increment_in_its_padding()
{
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_a_truncation_intent_carries_an_increment_in_its_padding";
    let flow = record_the_first_truncation_intent_with_its_logged_operation_changed(
        "crash-judge-gpu-truncation-intent-padding",
        "truncation-intent-with-an-increment-in-its-padding",
        &|entry| {
            entry[LOGGED_OPERATION_VALUE_OFFSET_IN_THE_ENTRY
                + TRUNCATION_INTENT_PADDING_OFFSET_IN_THE_VALUE] = 1;
        },
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_flow_reddening(
        TEST_NAME, &flow, "I-4.4",
    );
}

/// I-4.10（截断中间态有账，判决 K7 T-2）：截断条目的对象出生代加一，它不再对得上第一个文件的记录，size 之外挂着的单元 2 没有账
/// （I-4.4 与 I-4.6 跟着红），每个状态 GPU 与 CPU 整份相同（两遍恢复与池级 checker）、CPU 红 I-4.10。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_truncation_intent_names_another_object_birth() {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_the_truncation_intent_names_another_object_birth";
    let flow = record_the_first_truncation_intent_with_its_logged_operation_changed(
        "crash-judge-gpu-truncation-intent-object-birth",
        "truncation-intent-naming-another-object-birth",
        &|entry| {
            let at = LOGGED_OPERATION_VALUE_OFFSET_IN_THE_ENTRY
                + TRUNCATION_INTENT_OBJECT_BIRTH_OFFSET_IN_THE_VALUE;
            let object_birth = read_u64_at(entry, at);
            entry[at..at + 8].copy_from_slice(&(object_birth + 1).to_le_bytes());
        },
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_flow_reddening(
        TEST_NAME, &flow, "I-4.10",
    );
}

/// I-9.38（数据单元声明长度对 size，判决 K7 T-4）：缩到一个半净荷那次意图重写的尾单元（单元 1，声明长度半个净荷）声明长度多一（多出来的那个字节是补零，
/// 单元照样解得开），每个状态 GPU 与 CPU 整份相同（落到意图那一版时两遍恢复都在这个单元上判声明长度不符、走读失败；池级 checker）、CPU 红 I-9.38。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_data_unit_within_the_size_declares_one_byte_more() {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_a_data_unit_within_the_size_declares_one_byte_more";
    let mut flow = record_new_pool_through_truncations_and_holes(
        "crash-judge-gpu-truncation-tail-declared-length",
        "truncation-tail-declaring-one-byte-more",
        TruncationsAndHolesFlowStop::AfterTheIntentOfTheFirstTruncation,
    );
    let data_unit = |bytes: &[u8]| unit_class(bytes) == 1;
    let targets = last_unit_copies(&flow, "截断意图重写的尾单元", &data_unit, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
        let payload =
            u16::try_from(singlefs_core::unit::data_unit_payload_capacity()).expect("32634");
        let declared_length = read_u16_at(bytes, UNIT_DECLARED_LENGTH_OFFSET);
        assert_eq!(declared_length, payload / 2, "尾单元的声明长度是半个净荷");
        bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2]
            .copy_from_slice(&(declared_length + 1).to_le_bytes());
    });
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_flow_reddening(
        TEST_NAME, &flow, "I-9.38",
    );
}

/// I-4.6 ①（存在的单元从 0 起连号，判决 K7 T-3）：三个单元的文件那一版下段叶里偏移 2P 那条记录的 key 偏移改成 3P（单元 2 跳过一个净荷），
/// 每个状态 GPU 与 CPU 整份相同（落到三个单元那一版时两遍恢复都判单元不连号、走读失败；池级 checker）、CPU 红 I-4.6。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_data_units_of_a_file_skip_an_offset() {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_the_data_units_of_a_file_skip_an_offset";
    let mut flow = record_new_pool_through_truncations_and_holes(
        "crash-judge-gpu-data-units-skipping-an-offset",
        "data-units-skipping-an-offset",
        TruncationsAndHolesFlowStop::AfterTheThreeUnitFile,
    );
    let extent_lower_leaf = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && usize::from(index_node_key_width(bytes)) == EXTENT_KEY_BYTES
            && index_node_entry_width(bytes) == EXTENT_LOWER_LEAF_RECORD_BYTES
    };
    let targets = last_unit_copies(&flow, "三个单元那一版的下段叶", &extent_lower_leaf, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
        let payload =
            u64::try_from(singlefs_core::unit::data_unit_payload_capacity()).expect("32634");
        let records_start = index_node_entries_start(EXTENT_KEY_BYTES);
        let record_count = usize::from(read_u16_at(
            bytes,
            index_node_entry_count_offset(EXTENT_KEY_BYTES),
        ));
        let record_bytes = usize::from(EXTENT_LOWER_LEAF_RECORD_BYTES);
        let offset_field = (0..record_count)
            .map(|record_index| {
                records_start + record_index * record_bytes + EXTENT_KEY_OFFSET_OF_THE_FILE_OFFSET
            })
            .find(|offset_field| read_u64_at(bytes, *offset_field) == 2 * payload)
            .expect("下段叶里有偏移 2P 那条记录");
        bytes[offset_field..offset_field + 8].copy_from_slice(&(3 * payload).to_le_bytes());
    });
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_flow_reddening(
        TEST_NAME, &flow, "I-4.6",
    );
}

/// extent key 里文件偏移那一段（locality 8、inode 8 之后）。
const EXTENT_KEY_OFFSET_OF_THE_FILE_OFFSET: usize = 16;
/// 上段叶条目标签 3、1（判决 K5：3 是下段根、之后到 size 是洞；1 是下段根、之后没有洞）。
const EXTENT_UPPER_LEAF_TAG_LOWER_SEGMENT_ROOT_FOLLOWED_BY_A_HOLE: u8 = 3;

/// I-4.6 ②（单元数对标签，判决 K7 T-3 与第六节 S1）：变长出洞那一版上段叶里第一个文件那一条的标签 3 改写成 1——两个单元、size 要四个、没有截断条目，
/// 标签 1 要不少于四个，每个状态 GPU 与 CPU 整份相同（落到变长出洞那一版时两遍恢复都判单元数对不上 size、走读失败；池级 checker）、CPU 红 I-4.6。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_hole_tag_is_rewritten_as_no_hole() {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_a_hole_tag_is_rewritten_as_no_hole";
    let mut flow = record_new_pool_through_truncations_and_holes(
        "crash-judge-gpu-hole-tag-rewritten-as-no-hole",
        "hole-tag-rewritten-as-no-hole",
        TruncationsAndHolesFlowStop::AfterTheHoleBehindALowerSegment,
    );
    let targets = last_unit_copies(
        &flow,
        "变长出洞那一版的上段叶",
        &is_an_extent_upper_leaf,
        false,
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
        let tag = pointer_of_the_first_extent_upper_leaf_entry(
            bytes,
            EXTENT_UPPER_LEAF_TAG_LOWER_SEGMENT_ROOT_FOLLOWED_BY_A_HOLE,
        ) - 1;
        bytes[tag] = EXTENT_UPPER_LEAF_TAG_LOWER_SEGMENT_ROOT;
    });
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_flow_reddening(
        TEST_NAME, &flow, "I-4.6",
    );
}

/// 洞的内容按期望那一版逐单元比（判决 m4-trunc-r1 第六节 S5：内核读到洞时现算全零的载荷区 CRC，与判表从版本表给的期望比）：变长出洞那一版
/// （两个单元、size 三个半净荷、标签 3）的期望内容在洞里（单元 2 那一段的中间）塞一个非零字节，盘上一个字都不改。读出的洞是 0，落到这一版的状态上
/// CPU 看 journal 那一遍红「内容不同」；每个状态 GPU 与 CPU 整份相同——GPU 不比洞（或比洞时不照期望那一版）的话，这些状态上 GPU 判内容相同、对拍红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_expected_content_has_a_nonzero_byte_in_the_hole() {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_the_expected_content_has_a_nonzero_byte_in_the_hole";
    let payload = singlefs_core::unit::data_unit_payload_capacity();
    let mut flow = record_new_pool_through_truncations_and_holes(
        "crash-judge-gpu-nonzero-byte-in-the-hole",
        "nonzero-byte-in-the-hole-of-the-expected-content",
        TruncationsAndHolesFlowStop::AfterTheHoleBehindALowerSegment,
    );
    let version_with_the_hole = flow
        .versions
        .last_mut()
        .expect("变长出洞那一版是版本表里最后一版");
    assert_eq!(
        version_with_the_hole.content.len(),
        3 * payload + payload / 2,
        "变长出洞那一版的期望内容是三个半净荷"
    );
    let byte_in_the_hole = 2 * payload + payload / 2;
    assert_eq!(
        version_with_the_hole.content[byte_in_the_hole], 0,
        "洞里的期望内容是 0"
    );
    version_with_the_hole.content[byte_in_the_hole] = 1;
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    let wrong_content = consulted_oracle_item(CrashPointReplayOracleViolationKind::WrongContent);
    assert!(
        comparison
            .cpu_red_items_of_each_state
            .iter()
            .any(|red_items| red_items.contains(&wrong_content)),
        "流 {} 上落到变长出洞那一版的状态 CPU 看 journal 那一遍红「内容不同」",
        flow.name
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_truncation_flow(
        &flow,
        &comparison,
    );
}

// ───────────── 截断第二轮：截断字段、I-9.7 拆段、I-9.39、incompat 位 2（判决 m4-trunc-r2 第三节 R1、R6 与第六节用户定案）─────────────
// 记录与系统配置都按字节改：核心层写字段与 mkfs 写位 2 的那一半（截庚）还没合进来时，这几条流照样造得出。

/// inode 记录里截断字段两段与两段恒零区（判决 R1）：T 在偏移 120、N_w 在偏移 128；截断字段之前那一段预留的最后一个字节在 119，之后那一段在 136–139。
const INODE_RECORD_TRUNCATION_FIELD_TXG_OFFSET: usize = 120;
const INODE_RECORD_TRUNCATION_FIELD_SIZE_OFFSET: usize = 128;
const INODE_RECORD_LAST_RESERVED_BYTE_BEFORE_THE_TRUNCATION_FIELD: usize = 119;
const INODE_RECORD_LAST_RESERVED_BYTE_AFTER_THE_TRUNCATION_FIELD: usize = 139;
/// 系统配置槽里 incompat 位图第一个字节的偏移（magic 4 + 格式版本 2）与这一字节里认得的两位：位 1 布局身份、位 2 inode 记录带截断字段
/// （D15（格式冻结政策） 已定项 4）；位 0 退役、位 3 没分出去。
const FIRST_INCOMPAT_BYTE_OFFSET: usize = 6;
const INCOMPAT_FIRST_SSD_LINE_BIT: u8 = 0x02;
const INCOMPAT_INODE_RECORD_TRUNCATION_FIELD_BIT: u8 = 0x04;
const INCOMPAT_RETIRED_BIT_ZERO: u8 = 0x01;
const INCOMPAT_UNASSIGNED_BIT_THREE: u8 = 0x08;
/// 三个单元的文件那次顺序写的 checkpoint_txg（新池新建文件 txg 3 之后的那一次）：三个单元的单元头诞生代号都是它。
const CHECKPOINT_TXG_OF_THE_THREE_UNIT_FILE: u64 = 4;
/// 新池新建文件那一版的 checkpoint_txg（取号、两次暖机之后）：它唯一那个单元的诞生代号。
const CHECKPOINT_TXG_OF_THE_NEW_POOL_FILE: u64 = 3;
/// 缩到一个半净荷那次截断意图的 checkpoint_txg。
const CHECKPOINT_TXG_OF_THE_FIRST_TRUNCATION_INTENT: u64 = 5;

/// 截断字段的两段写成 (T, N_w)。
fn write_the_truncation_field(
    record: &mut [u8],
    truncation_field_txg: u64,
    truncation_field_size_in_bytes: u64,
) {
    record[INODE_RECORD_TRUNCATION_FIELD_TXG_OFFSET..INODE_RECORD_TRUNCATION_FIELD_TXG_OFFSET + 8]
        .copy_from_slice(&truncation_field_txg.to_le_bytes());
    record
        [INODE_RECORD_TRUNCATION_FIELD_SIZE_OFFSET..INODE_RECORD_TRUNCATION_FIELD_SIZE_OFFSET + 8]
        .copy_from_slice(&truncation_field_size_in_bytes.to_le_bytes());
}

/// 流里最后写的那片 inode 叶容器（两份副本）上第一个文件的记录照 `change` 改、重封到根。
fn change_the_record_of_the_first_file_in_the_last_inode_leaf(
    flow: &mut RecordedFlow,
    change: &dyn Fn(&mut [u8]),
) {
    let targets = last_unit_copies(
        flow,
        "最后写的那片 inode 叶容器",
        &is_an_inode_leaf_container,
        false,
    );
    corrupt_units_and_reseal_up_to_the_root(flow, &targets, &|bytes| {
        let record = offset_of_the_inode_record(bytes, FIRST_FILE.0);
        change(&mut bytes[record..record + INODE_RECORD_BYTES]);
    });
}

/// 一个系统配置槽（magic 对得上的那几份；全零的槽不碰）incompat 第一个字节照 `change` 改、重封整槽校验和。
fn change_the_first_incompat_byte_of_a_system_configuration_slot(
    slot: &mut [u8],
    change: &dyn Fn(u8) -> u8,
) {
    if &slot[..4] != b"SFSB" {
        return;
    }
    slot[FIRST_INCOMPAT_BYTE_OFFSET] = change(slot[FIRST_INCOMPAT_BYTE_OFFSET]);
    let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(
        slot,
        4096,
        SYSTEM_CONFIGURATION_CHECKSUM_FIELD,
    );
    slot[SYSTEM_CONFIGURATION_CHECKSUM_FIELD..SYSTEM_CONFIGURATION_CHECKSUM_FIELD + 32]
        .copy_from_slice(&digest);
}

/// 改一条流的哪几处系统配置槽。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SystemConfigurationSlotsToChange {
    /// 基镜像里 mkfs 写下的两槽，与流里每一次系统配置槽写：每个状态上择到哪一槽都是改过的那一种。
    TheBaseImageSlotsAndEveryWriteInTheFlow,
    /// 只改流里的系统配置槽写，基镜像里 mkfs 写下的两槽不动：判表照它们取落点几何（`crash_judge_tables` 的
    /// `JudgeTablesRefusal::BaseSystemConfigurationUnreadable`：基线两槽都解不开就建不出表）。
    OnlyTheWritesInTheFlow,
}

/// `devices` 里每块盘上 `slots_to_change` 点名的那几处系统配置槽 incompat 第一个字节照 `change` 改，交回改了流里几次系统配置槽写。
fn change_the_first_incompat_byte_of_every_system_configuration_slot_on(
    flow: &mut RecordedFlow,
    devices: &[DeviceIdentity],
    slots_to_change: SystemConfigurationSlotsToChange,
    change: &dyn Fn(u8) -> u8,
) -> usize {
    let spacing = u64::from(geometry().fixed_structure_slot_spacing);
    match slots_to_change {
        SystemConfigurationSlotsToChange::TheBaseImageSlotsAndEveryWriteInTheFlow => {
            for (device_identity, device) in &mut flow.base.devices {
                if !devices.contains(device_identity) {
                    continue;
                }
                for slot_offset in [0, spacing] {
                    let mut slot = device.read(
                        singlefs_core::address::DeviceOffsetInBytes(slot_offset),
                        4096,
                    );
                    change_the_first_incompat_byte_of_a_system_configuration_slot(
                        &mut slot, change,
                    );
                    device.write(
                        singlefs_core::address::DeviceOffsetInBytes(slot_offset),
                        &slot,
                    );
                }
            }
        }
        SystemConfigurationSlotsToChange::OnlyTheWritesInTheFlow => {}
    }
    let mut writes_changed = 0;
    for write in &mut flow.writes {
        if write.kind != StepKind::SystemConfigurationSlot || !devices.contains(&write.device) {
            continue;
        }
        match &mut write.contents {
            WrittenContents::Bytes(bytes) => {
                change_the_first_incompat_byte_of_a_system_configuration_slot(bytes, change);
                writes_changed += 1;
            }
            WrittenContents::Zeros { .. } => panic!("系统配置槽写是普通写"),
        }
    }
    writes_changed
}

/// 两块盘。
const BOTH_DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

/// 每一槽都带 incompat 位 2（第一个字节 0x02 → 0x06）。
fn set_incompat_bit_two_on_every_system_configuration_slot(flow: &mut RecordedFlow) {
    change_the_first_incompat_byte_of_every_system_configuration_slot_on(
        flow,
        &BOTH_DEVICES,
        SystemConfigurationSlotsToChange::TheBaseImageSlotsAndEveryWriteInTheFlow,
        &|first_byte| first_byte | INCOMPAT_INODE_RECORD_TRUNCATION_FIELD_BIT,
    );
}

/// 每一槽都清掉 incompat 位 2（第一个字节 0x06 → 0x02）：mkfs 与写者写系统配置都带位 2（截庚），要一个不带位 2 的池就得在录下的流上清。
fn clear_incompat_bit_two_on_every_system_configuration_slot(flow: &mut RecordedFlow) {
    change_the_first_incompat_byte_of_every_system_configuration_slot_on(
        flow,
        &BOTH_DEVICES,
        SystemConfigurationSlotsToChange::TheBaseImageSlotsAndEveryWriteInTheFlow,
        &|first_byte| first_byte & !INCOMPAT_INODE_RECORD_TRUNCATION_FIELD_BIT,
    );
}

/// 三个单元的文件那一版（txg 4）停下的截断与洞流，第一个文件的记录照 `change` 改。
fn record_the_three_unit_file_with_its_record_changed(
    tag: &str,
    name: &'static str,
    change: &dyn Fn(&mut [u8]),
) -> RecordedFlow {
    let mut flow = record_new_pool_through_truncations_and_holes(
        tag,
        name,
        TruncationsAndHolesFlowStop::AfterTheThreeUnitFile,
    );
    change_the_record_of_the_first_file_in_the_last_inode_leaf(&mut flow, change);
    flow
}

/// 截断字段这几条流上 GPU 判器与 CPU 逐状态比：CPU 池级 checker 红的恰是 `expected_red`、判过而成立的里有 `expected_held`；
/// 池级 checker 那一半（违例位、判到位）每个状态逐位相同，再比两遍恢复与整份对拍字。恢复那一半在核心层的读者认截断字段与 incompat 位 2
/// （截庚）合进来之前对不上：那之前 core 把带截断字段的记录判不可用、把 0x06 判成不认得的位，GPU 恢复内核已经照判决收它们。
fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_field_flow(
    test_name: &str,
    flow: &RecordedFlow,
    expected_red: &[&'static str],
    expected_held: &[&'static str],
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, flow, expand_up_to()) else {
        return;
    };
    let expected_red: std::collections::BTreeSet<&'static str> =
        expected_red.iter().copied().collect();
    assert_eq!(
        comparison.cpu_checker_red_invariants, expected_red,
        "流 {} 上 CPU 池级 checker 红的那几条",
        flow.name
    );
    for invariant in expected_held {
        assert!(
            comparison.cpu_checker_held_invariants.contains(invariant),
            "流 {} 上 CPU 池级 checker 判过 {invariant} 而成立（成立过的：{:?}）",
            flow.name,
            comparison.cpu_checker_held_invariants
        );
    }
    assert_the_pool_checker_half_then_the_recovery_half_agree(flow, &comparison);
}

/// 池级 checker 那一半先比（截庚合进来之前就该对上），再比恢复那一半与整份。
fn assert_the_pool_checker_half_then_the_recovery_half_agree(
    flow: &RecordedFlow,
    comparison: &Comparison,
) {
    let first_lines = |lines: &[String]| {
        lines
            .iter()
            .filter(|line| !line.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert!(
        comparison.checker_mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 的池级 checker 不一致（前几个）：\n{}",
        flow.name,
        comparison.checker_mismatches.len(),
        first_lines(&comparison.checker_mismatches)
    );
    assert_eq!(
        comparison.recovery_mismatches,
        0,
        "流 {}：{} 个状态 GPU 与 CPU 两遍恢复的结局或落到的根不同（核心层读者认截断字段与 incompat 位 2 的那一份合进来之前这一句红是预期的）：\n{}",
        flow.name,
        comparison.recovery_mismatches,
        first_lines(&comparison.mismatches)
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        first_lines(&comparison.mismatches)
    );
}

/// I-9.7 截断字段之前那一段预留（偏移 108–119）的最后一个字节写 1：CPU 只红 I-9.7，两遍恢复都把这条记录判不可用；每个状态整份相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_reserved_bytes_before_the_truncation_field_are_not_zero(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_the_reserved_bytes_before_the_truncation_field_are_not_zero";
    let flow = record_the_three_unit_file_with_its_record_changed(
        "crash-judge-gpu-reserved-byte-before-the-truncation-field",
        "reserved-byte-before-the-truncation-field",
        &|record| record[INODE_RECORD_LAST_RESERVED_BYTE_BEFORE_THE_TRUNCATION_FIELD] = 1,
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_field_flow(
        TEST_NAME,
        &flow,
        &["I-9.7"],
        &[],
    );
}

/// I-9.7 截断字段之后那一段预留（偏移 136–139）的最后一个字节写 1：CPU 只红 I-9.7；每个状态整份相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_reserved_bytes_after_the_truncation_field_are_not_zero(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_the_reserved_bytes_after_the_truncation_field_are_not_zero";
    let flow = record_the_three_unit_file_with_its_record_changed(
        "crash-judge-gpu-reserved-byte-after-the-truncation-field",
        "reserved-byte-after-the-truncation-field",
        &|record| record[INODE_RECORD_LAST_RESERVED_BYTE_AFTER_THE_TRUNCATION_FIELD] = 1,
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_field_flow(
        TEST_NAME,
        &flow,
        &["I-9.7"],
        &[],
    );
}

/// I-9.7 带 incompat 位 2 的池上截断字段 T = 0 而 N_w = 5（没写过的字段两段都是 0）：CPU 只红 I-9.7，两遍恢复都把这条记录判不可用。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_truncation_field_size_comes_without_its_txg() {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_a_truncation_field_size_comes_without_its_txg";
    let mut flow = record_the_three_unit_file_with_its_record_changed(
        "crash-judge-gpu-truncation-field-size-without-txg",
        "truncation-field-size-without-txg",
        &|record| write_the_truncation_field(record, 0, 5),
    );
    set_incompat_bit_two_on_every_system_configuration_slot(&mut flow);
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_field_flow(
        TEST_NAME,
        &flow,
        &["I-9.7"],
        &[],
    );
}

/// I-9.7 不带 incompat 位 2 的池上截断字段写成 (三个单元那一版的 txg, 0)：截断字段必须全零，CPU 只红 I-9.7（单元恰在 txg T 写下，I-9.39 成立），
/// 两遍恢复照核心层判这条记录在这个池里不收。录下的流每一槽都带位 2（mkfs 与写者都写 0x06），先在每一槽上清掉它才是不带位 2 的池。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_truncation_field_is_written_without_incompat_bit_two() {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_a_truncation_field_is_written_without_incompat_bit_two";
    let mut flow = record_the_three_unit_file_with_its_record_changed(
        "crash-judge-gpu-truncation-field-without-the-bit",
        "truncation-field-without-incompat-bit-two",
        &|record| write_the_truncation_field(record, CHECKPOINT_TXG_OF_THE_THREE_UNIT_FILE, 0),
    );
    clear_incompat_bit_two_on_every_system_configuration_slot(&mut flow);
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_field_flow(
        TEST_NAME,
        &flow,
        &["I-9.7"],
        &["I-9.39"],
    );
}

/// I-9.7 两块盘里只有盘 0 的系统配置带 incompat 位 2，记录的截断字段 (三个单元那一版的 txg, 0)：各盘择到的槽都带才算带（截辛推的读法第 4 条，
/// 主 agent 认了），按不带判，CPU 只红 I-9.7。录下的流两块盘每一槽都带位 2，先全清、再只给盘 0 置上。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_only_one_device_carries_incompat_bit_two() {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_only_one_device_carries_incompat_bit_two";
    let mut flow = record_the_three_unit_file_with_its_record_changed(
        "crash-judge-gpu-incompat-bit-two-on-one-device",
        "incompat-bit-two-on-one-device",
        &|record| write_the_truncation_field(record, CHECKPOINT_TXG_OF_THE_THREE_UNIT_FILE, 0),
    );
    clear_incompat_bit_two_on_every_system_configuration_slot(&mut flow);
    change_the_first_incompat_byte_of_every_system_configuration_slot_on(
        &mut flow,
        &[DeviceIdentity(0)],
        SystemConfigurationSlotsToChange::TheBaseImageSlotsAndEveryWriteInTheFlow,
        &|first_byte| first_byte | INCOMPAT_INODE_RECORD_TRUNCATION_FIELD_BIT,
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_field_flow(
        TEST_NAME,
        &flow,
        &["I-9.7"],
        &[],
    );
}

/// I-9.39 的必红世界：带 incompat 位 2，三个单元（txg 4 写下）那一版的记录截断字段写成 (5, 0)：⌈0 ÷ 净荷⌉.max(1) = 1，单元 1、2 在截断字段之外、
/// size 之内，诞生代号 4 早于 T——CPU 只红 I-9.39（I-1.13 报不适用，GPU 不比它）；GPU 扫描内核逐位跟上。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_live_data_unit_beyond_the_truncation_field_is_born_before_it(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_a_live_data_unit_beyond_the_truncation_field_is_born_before_it";
    let mut flow = record_the_three_unit_file_with_its_record_changed(
        "crash-judge-gpu-live-unit-born-before-the-truncation-field",
        "live-unit-born-before-the-truncation-field",
        &|record| write_the_truncation_field(record, CHECKPOINT_TXG_OF_THE_THREE_UNIT_FILE + 1, 0),
    );
    set_incompat_bit_two_on_every_system_configuration_slot(&mut flow);
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_field_flow(
        TEST_NAME,
        &flow,
        &["I-9.39"],
        &[],
    );
}

/// 合法世界：带 incompat 位 2，三个单元那一版的记录截断字段写成 (4, 0)，单元 1、2 在截断字段之外而恰在 txg T 写下、不早于它：CPU 一条都不红，
/// I-9.7 与 I-9.39 判过而成立。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_live_data_units_are_born_at_the_truncation_field_txg() {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_live_data_units_are_born_at_the_truncation_field_txg";
    let mut flow = record_the_three_unit_file_with_its_record_changed(
        "crash-judge-gpu-live-units-born-at-the-truncation-field-txg",
        "live-units-born-at-the-truncation-field-txg",
        &|record| write_the_truncation_field(record, CHECKPOINT_TXG_OF_THE_THREE_UNIT_FILE, 0),
    );
    set_incompat_bit_two_on_every_system_configuration_slot(&mut flow);
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_field_flow(
        TEST_NAME,
        &flow,
        &[],
        &["I-9.7", "I-9.39"],
    );
}

/// 合法世界：带 incompat 位 2，新池新建文件（一个单元，txg 3 写下）的记录截断字段写成 (4, 0)：⌈0 ÷ 净荷⌉.max(1) = 1，单元 0 不在截断字段之外，
/// 诞生代号早于 T 也是活的——CPU 一条都不红、I-9.39 判过而成立（去掉 .max(1) 就判死单元 0）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_truncation_field_to_size_zero_keeps_the_first_data_unit(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_a_truncation_field_to_size_zero_keeps_the_first_data_unit";
    let mut flow = record_new_pool_file_creation(
        "crash-judge-gpu-truncation-field-to-size-zero",
        "truncation-field-to-size-zero-keeping-the-first-data-unit",
    );
    change_the_record_of_the_first_file_in_the_last_inode_leaf(&mut flow, &|record| {
        write_the_truncation_field(record, CHECKPOINT_TXG_OF_THE_NEW_POOL_FILE + 1, 0);
    });
    set_incompat_bit_two_on_every_system_configuration_slot(&mut flow);
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_field_flow(
        TEST_NAME,
        &flow,
        &[],
        &["I-9.39"],
    );
}

/// 合法世界：带 incompat 位 2，缩到一个半净荷那次意图（txg 5）的记录照用户定案写上截断字段 (5, 一个半净荷)：留下两个单元，单元 2 在 size 之外还挂着、
/// 诞生代号 4 早于 T——它是截断中间态等着放的单元、不是活单元，I-9.39 不判它（「活单元」取 size 之内的读法），CPU 一条都不红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_truncation_intent_version_carries_its_truncation_field(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_the_truncation_intent_version_carries_its_truncation_field";
    let payload = u64::try_from(singlefs_core::unit::data_unit_payload_capacity()).expect("32634");
    let mut flow = record_new_pool_through_truncations_and_holes(
        "crash-judge-gpu-truncation-intent-with-its-field",
        "truncation-intent-with-its-truncation-field",
        TruncationsAndHolesFlowStop::AfterTheIntentOfTheFirstTruncation,
    );
    change_the_record_of_the_first_file_in_the_last_inode_leaf(&mut flow, &|record| {
        write_the_truncation_field(
            record,
            CHECKPOINT_TXG_OF_THE_FIRST_TRUNCATION_INTENT,
            payload + payload / 2,
        );
    });
    set_incompat_bit_two_on_every_system_configuration_slot(&mut flow);
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_field_flow(
        TEST_NAME,
        &flow,
        &[],
        &["I-9.39"],
    );
}

/// 合法世界：每一槽都带 incompat 位 2（0x06），记录不带截断字段：两个池级 checker 内核与恢复内核都收这一槽，CPU 一条都不红、I-9.7 判过而成立。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_every_system_configuration_slot_carries_incompat_bit_two()
{
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_every_system_configuration_slot_carries_incompat_bit_two";
    let mut flow = record_new_pool_through_truncations_and_holes(
        "crash-judge-gpu-incompat-bit-two-on-every-slot",
        "incompat-bit-two-on-every-slot",
        TruncationsAndHolesFlowStop::AfterTheThreeUnitFile,
    );
    set_incompat_bit_two_on_every_system_configuration_slot(&mut flow);
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_field_flow(
        TEST_NAME,
        &flow,
        &[],
        &["I-9.7"],
    );
}

/// 系统配置 incompat 第一个字节换成 `first_incompat_byte` 的三种不认得的写法：流里每一次系统配置槽写都换，基镜像里 mkfs 写下的两槽不动
/// （判表照它们取落点几何；那两槽也换就建不出表、GPU 不上卡）。一块盘的两槽都被换过的写盖过之后，那块盘上择不到系统配置，
/// 两块盘都这样的状态上恢复失败、CPU 判器红；每个状态 GPU 与 CPU 整份相同，GPU 两个池级 checker 内核与恢复内核认 incompat 位的那一判在卡上走到。
/// 核心层收与不收 0x04、0x0E、0x07 不随截庚变（这三种在哪一版读者都不认）。
fn assert_the_gpu_judge_agrees_with_the_cpu_when_every_slot_carries_an_unrecognized_first_incompat_byte(
    test_name: &str,
    tag: &str,
    name: &'static str,
    first_incompat_byte: u8,
) {
    let mut flow = record_new_pool_through_truncations_and_holes(
        tag,
        name,
        TruncationsAndHolesFlowStop::AfterTheThreeUnitFile,
    );
    let system_configuration_slot_writes_changed =
        change_the_first_incompat_byte_of_every_system_configuration_slot_on(
            &mut flow,
            &BOTH_DEVICES,
            SystemConfigurationSlotsToChange::OnlyTheWritesInTheFlow,
            &|_first_byte| first_incompat_byte,
        );
    assert!(
        system_configuration_slot_writes_changed > 0,
        "流 {} 里有系统配置槽写（每次发布轮换系统配置）：没有就一个状态都换不到不认得的槽",
        flow.name
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_red > 0,
        "流 {}：每一槽都带不认得的 incompat 位（第一个字节 {first_incompat_byte:#04x}），CPU 判器在某些状态上红",
        flow.name
    );
    assert_the_pool_checker_half_then_the_recovery_half_agree(&flow, &comparison);
}

/// incompat 第一个字节 0x04：只带位 2、不带布局身份那一位 1，两边都不认。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_every_system_configuration_slot_carries_incompat_bit_two_without_bit_one(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_when_every_slot_carries_an_unrecognized_first_incompat_byte(
        "the_gpu_judge_agrees_with_the_cpu_when_every_system_configuration_slot_carries_incompat_bit_two_without_bit_one",
        "crash-judge-gpu-incompat-0x04",
        "incompat-first-byte-0x04",
        INCOMPAT_INODE_RECORD_TRUNCATION_FIELD_BIT,
    );
}

/// incompat 第一个字节 0x0E：位 1、位 2 之外还带没分出去的位 3。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_every_system_configuration_slot_carries_an_unassigned_incompat_bit(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_when_every_slot_carries_an_unrecognized_first_incompat_byte(
        "the_gpu_judge_agrees_with_the_cpu_when_every_system_configuration_slot_carries_an_unassigned_incompat_bit",
        "crash-judge-gpu-incompat-0x0e",
        "incompat-first-byte-0x0e",
        INCOMPAT_FIRST_SSD_LINE_BIT | INCOMPAT_INODE_RECORD_TRUNCATION_FIELD_BIT | INCOMPAT_UNASSIGNED_BIT_THREE,
    );
}

/// incompat 第一个字节 0x07：位 1、位 2 之外还带退役的位 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_every_system_configuration_slot_carries_the_retired_incompat_bit(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_when_every_slot_carries_an_unrecognized_first_incompat_byte(
        "the_gpu_judge_agrees_with_the_cpu_when_every_system_configuration_slot_carries_the_retired_incompat_bit",
        "crash-judge-gpu-incompat-0x07",
        "incompat-first-byte-0x07",
        INCOMPAT_RETIRED_BIT_ZERO | INCOMPAT_FIRST_SSD_LINE_BIT | INCOMPAT_INODE_RECORD_TRUNCATION_FIELD_BIT,
    );
}

/// 系统配置槽里格式版本那两字节（小端）的偏移：紧跟 magic 之后（池级 checker 的 `FORMAT_VERSION_OFFSET`、核心层恢复读格式版本是同一处）。
const SYSTEM_CONFIGURATION_FORMAT_VERSION_OFFSET: usize = 4;

/// 流里每一次系统配置槽写的格式版本改成 `format_version`、incompat 第一个字节改成 `first_incompat_byte`，重封整槽校验和，交回改了几次。
/// 基镜像里 mkfs 写下的两槽不动：判表照它们取落点几何（那两槽也改就建不出表、GPU 不上卡，同
/// [`SystemConfigurationSlotsToChange::OnlyTheWritesInTheFlow`]）。
fn rewrite_the_format_version_and_the_first_incompat_byte_of_every_system_configuration_slot_write(
    flow: &mut RecordedFlow,
    format_version: u16,
    first_incompat_byte: u8,
) -> usize {
    let system_configuration_slot_writes = flow
        .writes
        .iter()
        .filter(|write| write.kind == StepKind::SystemConfigurationSlot)
        .count();
    for nth_from_the_end in 0..system_configuration_slot_writes {
        rewrite_and_reseal(
            flow,
            StepKind::SystemConfigurationSlot,
            nth_from_the_end,
            4096,
            SYSTEM_CONFIGURATION_CHECKSUM_FIELD,
            &|bytes| {
                assert_eq!(&bytes[..4], b"SFSB", "流里的系统配置槽写是带 magic 的整槽");
                let format_version_bytes = SYSTEM_CONFIGURATION_FORMAT_VERSION_OFFSET
                    ..SYSTEM_CONFIGURATION_FORMAT_VERSION_OFFSET + 2;
                assert_eq!(
                    bytes[format_version_bytes.clone()],
                    singlefs_core::system_configuration::FORMAT_VERSION.to_le_bytes(),
                    "写者写下的格式版本是这一版读者认的那一版"
                );
                bytes[format_version_bytes].copy_from_slice(&format_version.to_le_bytes());
                bytes[FIRST_INCOMPAT_BYTE_OFFSET] = first_incompat_byte;
            },
        );
    }
    system_configuration_slot_writes
}

/// C663「格式版本先」（用户 2026-10-20 定）：新池新建文件流里每一次系统配置槽写 incompat 第一个字节写 0x0E（位 1、位 2 之外带没分出去的位 3）、
/// 格式版本写 2，重封整槽校验和——一槽同时带不认得的 incompat 位与不是这一版读者写的格式版本。核心层恢复先判格式版本、整池拒
/// （`SystemConfigurationValueRefused`，失败点 6），不走 incompat 位不认得那一路（失败点 2：那一路这一槽不可择、换一槽换一盘接着择）；
/// 池级 checker 读这一槽同样先判格式版本、读成带不收之值，红 I-7.13。改过的槽落了的状态上两遍恢复都整池拒，没有一个状态报 incompat 位不认得；
/// GPU 恢复内核与两个池级 checker 内核逐态跟上，每个状态整份相同。
/// 判别力（`crates/mutations.tsv`）：GPU 恢复内核在 incompat 位不认得那一臂里不先判格式版本，改过的槽落了的状态上 GPU 把它当不可择的槽、
/// 换槽接着择，两遍恢复的失败点与落到的根跟 CPU 不同，这条红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_format_version_not_recognized_refuses_the_pool_before_an_unrecognized_incompat_bit(
) {
    use singlefs_checker_tier::crash_judge_gpu::{
        CONSULTED_PASS_FAILURE_POINT_WORD,
        GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_INCOMPAT_BITS_NOT_RECOGNIZED,
        GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_VALUE_REFUSED, IGNORED_PASS_FAILURE_POINT_WORD,
    };
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_that_a_format_version_not_recognized_refuses_the_pool_before_an_unrecognized_incompat_bit";
    let mut flow = record_new_pool_file_creation(
        "crash-judge-gpu-format-version-2-with-incompat-0x0e",
        "new-pool-system-configuration-format-version-2-and-incompat-first-byte-0x0e",
    );
    let system_configuration_slot_writes_rewritten =
        rewrite_the_format_version_and_the_first_incompat_byte_of_every_system_configuration_slot_write(
            &mut flow,
            singlefs_core::system_configuration::FORMAT_VERSION + 1,
            INCOMPAT_FIRST_SSD_LINE_BIT
                | INCOMPAT_INODE_RECORD_TRUNCATION_FIELD_BIT
                | INCOMPAT_UNASSIGNED_BIT_THREE,
        );
    assert!(
        system_configuration_slot_writes_rewritten > 0,
        "流 {} 里有系统配置槽写（每次发布轮换系统配置）：没有就一个状态都落不到改过的槽",
        flow.name
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-7.13"),
        "流 {} 上 CPU 池级 checker 先判格式版本、把改过的槽读成带不收之值，红 I-7.13（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert_the_pool_checker_half_then_the_recovery_half_agree(&flow, &comparison);
    // 两边逐态整份相同（上一句），下面两个数 GPU 与 CPU 一样
    let states_whose_both_passes_refuse_the_pool_for_the_format_version = comparison
        .gpu_words_of_each_state
        .iter()
        .filter(|words| {
            words[CONSULTED_PASS_FAILURE_POINT_WORD]
                == GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_VALUE_REFUSED
                && words[IGNORED_PASS_FAILURE_POINT_WORD]
                    == GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_VALUE_REFUSED
        })
        .count();
    let states_naming_an_unrecognized_incompat_bit = comparison
        .gpu_words_of_each_state
        .iter()
        .filter(|words| {
            words[CONSULTED_PASS_FAILURE_POINT_WORD]
                == GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_INCOMPAT_BITS_NOT_RECOGNIZED
                || words[IGNORED_PASS_FAILURE_POINT_WORD]
                    == GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_INCOMPAT_BITS_NOT_RECOGNIZED
        })
        .count();
    assert!(
        states_whose_both_passes_refuse_the_pool_for_the_format_version > 0,
        "流 {}：改过的槽落了的状态上两遍恢复都按格式版本整池拒（失败点 {GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_VALUE_REFUSED}），实际 0 个状态",
        flow.name
    );
    assert_eq!(
        states_naming_an_unrecognized_incompat_bit, 0,
        "流 {}：格式版本先于 incompat 位判，没有一个状态报 incompat 位不认得（失败点 {GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_INCOMPAT_BITS_NOT_RECOGNIZED}）",
        flow.name
    );
}
