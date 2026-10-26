# m3-close-g4-rewrite-r3 云端正推腿（Sonnet）报告（2026-10-22）

立场：正文分工表「云端正推」一行——从 `crates/` 今天的代码与附录条款正推 Y1–Y4。没读禁读清单里的三样；没编 Rust、没起 checker 档与任何重型测试；只跑了纯 Python 小模型和一个已编好的 `target/debug/unit-graph`（经 `capped.sh 31` 与 `run-with-memory-cap.sh`，1 秒级，只读源码树）。每条结论标：量的（我跑出来的）／读码（读到的行）／推的。

## 0 复跑与快照

模型目录 `research/prompts/m3-close-g4-rewrite-r3-sonnet-model/`，纯 Python（`python3 -I`），不读冻结副本、不起 cargo。复跑（`--frozen` 随便给一个已有目录，模型不读它）：

    bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-close-g4-rewrite-r3-sonnet-model/tools/rerun.sh --repo <仓根> --frozen <已有目录> --work <新的草稿目录>

我这一趟复跑的末三行（原样）：

    name=rerun_compare file=run_all.out sha256=6633604cd9c49bb583ae47c4954f407dca2aee50e9376dc554d7cd6b36e80487 same=yes
    name=rerun_compare file=second_crash_bases.out sha256=0bf245c8d771484b46f0800858cb2770e0c1c19abee8eb0b38866ed00f3a2f0e same=yes
    name=rerun_compare file=y3_greps.out informational=yes same=no

`y3_greps.out` 读的是仓此刻的样子（别的会话在改，我自己的模型目录与这份报告也在 `research/prompts` 里被它数到），只比不判。`tools/rerun.sh --selftest` 在我改过的 `rerun_steps` 上红（模板自证拷到别处时没有 `*.py`），这是模板自证的写法与我换的那一段不相容，不影响复跑。

| 文件 | sha256 |
|---|---|
| slot_flow_model.py | cfd61b18895782d37eb53f92e99c9e90b5dc9c652ca70c3fe0058d181033daab |
| enumerate_states.py | 31790ceb7f35ffd4c0e55e45937bd4be6473da91700e8b74d96c6f7f13b224c3 |
| protocol_histories.py | cd74bed609710cf2ccbe765f611779723bf1280ed995293691887d3b66eaf98d |
| closure_check.py | 872e703381d241a4ad03c37c6ed23a1eb3c6ac742cccd4d5a54d682b254b24cf |
| run_all.py | 2f10ce3e57018dcbe90a04c1a3a0dc57c3e6f3f387d96cac20ca5aa0761e57d7 |
| second_crash_bases.py | bbce2c642fd4dc67490511eb8a783bee37e6da485e65d96a8592e49b89588a65 |
| tools/rerun.sh | e48fd4d0f7e59dcef614de6c95609d0fa7a76a308f9401b831e637b5437f450c |
| tools/y3_greps.sh | 5b740ae8d459436c464be1c57bf91acaa5b63c5d2d111f49637c2ba64149532e |
| out/run_all.out | 6633604cd9c49bb583ae47c4954f407dca2aee50e9376dc554d7cd6b36e80487 |
| out/second_crash_bases.out | 0bf245c8d771484b46f0800858cb2770e0c1c19abee8eb0b38866ed00f3a2f0e |
| out/unit_graph_membership.out | 621ea14e16ec5a30d0cd116f8432c0ec7f83ef9de73e575c8f913ad0b9070c9b |
| out/y3_greps.out | 79e0401d3d068bff3f4999b1dd17aaec57c4d1df7b21009c0ed7d138e82e7032 |

模型怎么写的：按 `crates/singlefs-harness/src/segments.rs` 的切段规则（`SegmentClosingRule::after`，第 127 行）、`crates/singlefs-checker-tier/src/crash.rs` 的原地覆写第三态判据（`is_tearable_in_place_overwrite`，第 2261 行）与 `crates/singlefs-core/src/transaction.rs` 的系统配置槽写协议，把系统配置槽符号化（零 / 自证过带世代号 / 撕裂），枚举「前面的段全持久、当前段每次写取两态或三态、后面全没持久」。**不读也不复用第二轮攻方原型的代码**（我读了它 `toggles.patch` 只为知道 MUT-B1、MUT-B2 的语义）。模型自己的判别力：下面 Y4 里它把 MUT-B1、MUT-B2 的数逐位推出来，且对正确协议恰好零死盘（会红的对照）。

快照核对（仓根里跑，量的，原样）：

    $ cat research/prompts/m3-close-g4-rewrite-r3-snapshot/code-sha256.txt | sha256sum -c - 2>&1 | grep -c ": OK"
    7
    $ cat research/prompts/m3-close-g4-rewrite-r3-snapshot/kb-sha256.txt | sha256sum -c - 2>&1 | grep -E "FAILED|WARNING"
    .claude/kb/checks-owed.md: FAILED
    sha256sum: WARNING: 1 computed checksum did NOT match
    $ cat research/prompts/m3-close-g4-rewrite-r3-snapshot/kb-sha256.txt | sha256sum -c - 2>&1 | grep -c ": OK"
    11

代码七份全 OK，kb 十二份里除 `.claude/kb/checks-owed.md` 之外的十一份 OK；`checks-owed.md` 对不上（批 15 的 kb 写回改过它，第二轮判决已记同一情形），我引欠账表的行都是它此刻的行。

## Y1 第二轮「已定」的撑不撑得住

### Y1-a 甲-二 不做：站得住

- 读码：判官 `judge_writable_mount_refusal_of_a_crash_state(`（`crates/singlefs-harness/src/mount_transparency.rs:3001`）收的参数只有崩溃镜像、所选版本、拒因、盘、只读读出、读故障：「read_faults: ReadFaultsInTheCrashState,」（`crates/singlefs-harness/src/mount_transparency.rs:3007`）是最后一个，没有补写闸的输入。
- 读码：补写闸哪一件没过在核里就丢了。「只在核里分辨：调用方照旧拒成」（`crates/singlefs-core/src/mount.rs:5728`）；调用点「Some(_gate_not_met) => return Err(device),」（`crates/singlefs-core/src/mount.rs:5797`）把值丢掉，拒因只剩盘号。所以判官一侧「闸恒不过」与「闸一不过」的拒是同一个输入，甲-二 在判官一侧多罩为零（读码；第二轮攻方在两张盘面上量到判官结论逐字段相同，我没复跑它）。
- 读码，没跑：「补写闸恒不过」这一类改坏，现成的正向测试 `a_device_holding_the_version_whose_both_slots_were_zeroed_is_rewritten_by_the_acquisition_and_the_checker_stays_green`（`crates/singlefs-harness/tests/acquisition_rewrites_a_device_that_lost_both_system_configuration_slots_only_through_the_gates.rs:159`）会在「let mounted = outcome.expect("盘 1 过了补写闸：可写挂载做成");」（同文件第 164 行）上 panic。登记行是同一个可观测结局的近亲：`crates/mutations.tsv` 第 5010 行「丙' 取号那一刻的见证读不认过了补写闸的盘（照旧拒）」，第 5011–5015 行是五件闸各一行。
- 什么会推翻：判官签名里加了补写闸输入；或造得出一个「补写闸恒不过」的改坏，第 159 行那条与第 5010–5015 行全不红。

### Y1-b G4-丙 原形态出局的三处理由

原形态 = 第二轮正文写的「挂在崩溃注入第二截的崩溃镜像与第三截的二次崩溃镜像上，各判一次」的按盘面判法（`research/prompts/m3-close-g4-rewrite-r2-main-verification.md` 第二节 W2 行）。

| 理由 | 判定 | 依据 |
|---|---|---|
| 整写子集：崩溃注入看不见撕裂域 | 站得住，但只对「两槽落同一段」那一类坏实现成立 | 量的：`grep -c Tearable crates/singlefs-checker-tier/src/crash_injection.rs` 输出 `0`。读码：第二截、第三截的镜像都按录制流的写取持久位——「persisted: crash_point.persisted_within_the_segment.clone(),」（`crates/singlefs-checker-tier/src/crash_injection.rs:1598`）、「persisted: second.persisted_within_the_segment.clone(),」（`crates/singlefs-checker-tier/src/crash_injection.rs:4286`）。量的（我的模型，Y4）：两槽同段的坏实现单盘全废态整写域 0、撕裂域 16；但「同段写后再清另一槽」那一类整写域也有 6–8 态，崩溃注入摆不摆得到是抽样的事（推的）。推翻：`crash_injection.rs` 里出现撕裂镜像（`Tearable` 命中大于 0） |
| 一件事记两条 | 对「新判官挂在崩溃注入里」站得住；换挂崩溃点重放就不再同套，换成改池级 checker 会重现（Y2 表） | 读码：无读故障时拒绝臂为真只有两条路——某块盘一份本池自证过的都没有，或择不到系统配置（`crates/singlefs-harness/src/mount_transparency.rs:3073` 调的复算，函数体在第 2419–2443 行；见证判不出那一支要有槽报读错，无读故障时为假）；所以「臂报 ⟹ 按盘面判也报」，同一张被拒盘面在 `judge_the_remount_after_a_second_crash(`（`crates/singlefs-checker-tier/src/crash_injection.rs:4324`）这一处就记两条。推翻：拒绝臂在无读故障的盘面上为真而盘面每块盘都有自证过的槽 |
| 判槽自证与被判的实现共用核心层算法 | 站得住，今天的拒绝臂同样有（旧账，不是 G4-丙 才带进来） | 读码：「if let Ok(system_configuration) = SystemConfiguration::parse_slot(&bytes) {」（`crates/singlefs-harness/src/mount_transparency.rs:2374`），择系统配置引入核心层的函数：「choose_root, choose_system_configuration, chosen_system_configuration_slot_bytes,」（`crates/singlefs-harness/src/mount_transparency.rs:36`）；规则：「**运行时与 checker 必须用不同的算法。**」（`.claude/rules/fs-design.md:35`）。池级 checker 有独立读法：「pub fn system_configuration_redundancy_report(」（`crates/singlefs-checker/src/image.rs:1492`）。harness 已依赖 checker：「singlefs-checker = { path = "../singlefs-checker" }」（`crates/singlefs-harness/Cargo.toml:9`）。推翻：核心层与 checker 的槽解析其实是同一段代码（我没逐行比两份解析，推的） |

补充（读码）：拒绝臂不能整个换成 checker 的读法。它还要「有槽设备报读错」那一半（`system_configuration_witness_undecidable_on_the_crash_image`），而 checker 的槽读法把读不到并进不自证：「不自证：读不到、不足 4096 字节、magic 或整槽校验和不对」（`crates/singlefs-checker/src/image.rs:790`）。能换的只有「某块盘一份本池自证过的都没有」那一半（`crates/singlefs-harness/src/mount_transparency.rs:2406`），还要给 `&dyn PoolReader` 配一个 `ImageReader` 适配。

### Y1-c 第一轮的甲（测试盘面另坏一个单元）还要不要

- 要不要取决于拒绝臂留不留。那条红的测试 `a_remount_refused_on_a_second_crash_image_where_device_zero_has_both_system_configuration_slots_torn_is_an_illegal_refusal_after_the_read_stage`（`crates/singlefs-checker-tier/src/crash_injection.rs` 第 8959 行起是文档注释，函数在第 8968 行）靠「盘 0 上所选那一版的一个单元改一个字节」让补写闸闸一不过，拒绝臂才有机会被调到。
- 丙-置（拒绝臂留，Y2）：甲留。它是让拒绝臂在单元测试里可测的唯一盘面。
- 丙-I（拒绝臂必须撤，Y2）：甲随臂一起不要；测试改成直接断言池级 checker 在这张盘面上红 I-8.5（harness 档的坏镜像用例）。
- 推翻：补写闸的闸一在 harness 的造盘里改了判法，使「改一个单元」不再让它不过。

## Y2 丙-置 与 丙-I

### Y2-a 撕裂域里正确实现在 mkfs 返回之后留不留得下单盘全废态：不留

**写系统配置槽的地方一共六处**（读码；命令 `grep -n "write_system_configuration_slot\|RotateSystemConfigurationSlots\|\.to_slot()" crates/singlefs-core/src/*.rs`，去掉测试里的 `.to_slot()` 之后落在六个写者上）。逐类：

| 历史 | 代码 | 同一段里同一块盘的槽写数 | 判 |
|---|---|---|---|
| 取号写两轮（含补写顺带） | 「for _round in ACQUISITION_WRITE_ROUNDS {」（`crates/singlefs-core/src/transaction.rs:1235`），轮后「if let Err(cause) = pool.perform_commit_step(CommitStep::Barrier) {」（`crates/singlefs-core/src/transaction.rs:1254`） | 1 | 不留 |
| 每次发布末尾的轮换 | 根槽 FUA 之后「RotateSystemConfigurationSlots {」（`crates/singlefs-core/src/transaction.rs:1604`），再一道屏障（第 1608 行） | 1 | 不留 |
| 抬 F 那一串 | 「没有这一道，两次写同在一段里，崩溃时两槽可以一起撕坏。」（`crates/singlefs-core/src/transaction.rs:800`）；前后各一道屏障（第 801、827 行） | 1 | 不留 |
| 取号失败的回卷 | 「同一块盘传两次会在一段里把两槽都覆写，同段断电两槽一起坏（第二轮判决 T1 打中的那一形）。」（`crates/singlefs-core/src/transaction.rs:673`）；写前一道屏障（第 693 行）。只在注入写错误的历史里走，崩溃点重放不枚举它 | 1 | 不留（没跑，读码） |
| 离线修复 | 「for _slot_not_self_verifying in &not_self_verifying {」（`crates/singlefs-core/src/system_configuration_repair.rs:374`）：两槽都不自证时连写两槽、末尾一道屏障（第 404 行）；有一槽自证时拒绝覆写它（第 361 行 `SelfVerifiedSlotWouldBeOverwritten`） | 起点已死的盘 2，别的盘 ≤1 | 不新增死盘（继承） |
| mkfs | 「device.write_at(offset, &slot, WriteDurability::Plain)?;」（`crates/singlefs-core/src/make_filesystem.rs:732`），逐盘两槽，屏障在整个循环之后（第 736 行） | 2 | 末段留得下，不归这一问（Y2-b） |

写协议的不变式（读码）：世代号 = 这一刻盘上自证过的最大世代号加一，落槽 = 世代号模二（`crates/singlefs-core/src/transaction.rs:577` 起的 `write_system_configuration_slot`），所以每一写覆写的是不持有最大世代号的那一槽；覆写之前另一槽已经被屏障持久。同一段里一块盘至多一个槽写 ⟹ 另一槽在这一段的任何崩溃态里都是上一段落定的自证槽，死盘进不来。

**量的（我的模型，`out/run_all.out`）：**

    name=remount_correct segments=[2, 2, 14, 2, 1, 2, 12, 2, 1, 2] states=20519 whole=20499 torn=20 single_dead(whole/torn)=0/0 all_dead(whole/torn)=0/0
    name=closure_correct fixpoint_rounds=3 reachable_images=25 images_with_a_dead_device=0 dead_states_met=0
    name=second_crash_bases whole_domain_images_of_the_four_system_configuration_segments=4 all_inside_the_closure=True

- 第一行：第二轮攻方 remount 流的形（它自己印出来的段序列，单元写 14 与 12 两段我直接取它的，没推），整写 20499 加撕裂 20，死盘 0。
- 第二行：可达盘面闭包——从 mkfs 返回后的盘面出发，反复在**任意崩溃状态（含撕裂）**上再起「取号两轮加两次发布 / 只取号 / 会话内连两次发布 / 抬 F 再发布 / 取号再抬 F 再发布」五类流，直到不再出新盘面（3 轮收敛，25 张盘面），没有一张有死盘。二次、三次崩溃都在闭包里。
- 第三行：攻方「第三截那一形」八个基线里，整写域的第一次崩溃镜像在闭包里（八个基线在我的固定形状里每个都 20519 态、0 死盘；攻方报六个 20519 加两个 24626，段号 5 的两个流形状多一次发布，喂它印出的段序列后对上 24626，见 Y4）。
- 与攻方的 W1 一致：攻方四组历史都是零发现（只抽一次），我这条路是另一个方法（穷举闭包 + 协议不变式），也是零。

**模型会红的对照（校验路径本身）：** 同一个模型对坏协议报红——MUT-B2 单盘全废 16 态、两盘全废 1 态；MUT-B1 单盘全废 42 态；回卷同一块盘传两次那一形（B4）3 态（原样行在 Y4 末尾）。

**更强的一句（量的，穷举 729 条小流）：** `name=double_write_equivalence flows=729 (3_segments_each_device_0_1_or_2_slot_writes) dead_state_reachable_without_a_double_write=0 dead_state_reachable_with_a_double_write=665 double_write_without_any_dead_state=0`。健康起点、协议照上表、基镜像两槽非零时，**单盘全废态可达 ⟺ 某一段里有一块盘的系统配置槽写不止一个**。

**什么现象会推翻「不留」：** 一个我没列的系统配置槽写者（`grep` 六处之外的第七处）；录制出来的某条流里有一段含同一块盘两个系统配置槽写（`[system_configuration_slot×k` 的 k 大于盘数就是线索，登记表的种类串里有它）；`has_writes_since_barrier` 在某个路径上把本该发的屏障省掉了（我读了 `perform_commit_step` 的屏障分支，每个写路径末尾都有屏障，但没跑）。带读故障的历史、三块盘以上的池不在这一问里（`make_filesystem` 断言两块盘：「第一版跑 2 块盘（D2（RAID 条带策略） 已定项 9）」，`crates/singlefs-core/src/make_filesystem.rs:540`）。

### Y2-b mkfs 末段：留得下，而且同一张盘面分不出「中途」与「之后」

- 读码：mkfs 的槽写在一个循环里逐盘两槽（「device.write_at(offset, &slot, WriteDurability::Plain)?;」，`crates/singlefs-core/src/make_filesystem.rs:732`），屏障在循环之后（第 736 行），所以末段是四个写同一段。条文也认这个结局：「后一种里系统配置的冗余可以缺，缺新槽的那块盘的可写挂载按现行补写条款处置」（`.claude/kb/decisions/22-单元原子性怎么合成.md:181`）。
- 量的（模型）：

        name=mkfs_used_pool_single_dead_by_segment(index,domain)=[((0, 'torn'), 2), ((0, 'whole'), 2), ((5, 'torn'), 34), ((5, 'whole'), 6)]
        name=mkfs_fresh_pool_single_dead_by_segment(index,domain)=[((5, 'whole'), 6)]

  段序列和那条 mkfs 流的登记一致：「写表是 mkfs 那 27 步（段序列 `2+12+1+1+1+4`）」（`crates/singlefs-checker-tier/tests/crash_enumeration_make_filesystem_over_a_used_pool_stream.rs:7`）。末段（段号 5）单盘全废：新盘 6 态（整写域），用过的盘 6 加 34 = 40 态——与第一轮判决记的攻方数一致：「mkfs 末段 6 态与 40 态」（`research/prompts/m3-close-g4-rewrite-r1-main-verification.md:16`）。用过的盘上还有首段（段号 0，清 [0, 根环基址) 那一段，罩住两个系统配置槽）的 4 态：一块盘的清零落了或撕了、另一块盘的旧池槽还在，池级 checker 不分 fsid、旧槽照算自证过，所以也是单盘全废——第一轮判决记的数只到末段，这 4 态不在里面（我的模型推的：整段清零按原地覆写取三态；没跑那条流）。用过的盘合计 44 态，新盘 6 态。模型里清根环与 journal 环的整段清零按「不碰系统配置槽」处理，所以 `mkfs_used_pool` 那一行的总态数、全废态数不当作那条流的态数引。
- 量的（模型）：`name=post_mkfs_pool_with_one_device_lost images=8 of_them_equal_to_a_state_inside_the_mkfs_tail_segment=8 (by_domain=['torn', 'whole'])`。mkfs 返回之后、一块盘丢了两槽（清零或撕裂）的盘面共 8 张，**八张逐槽等于 mkfs 末段里的某个状态**（槽级抽象；末段之前的段全持久，其余结构与 mkfs 返回后的盘相同）。所以只看一张镜像的池级 checker 分不出「mkfs 中途」与「mkfs 之后」。
- 能分的只有三条路：
  1. 按流分（丙-置）：观察者挂在节点上，只挂 mkfs 之后的节点；记录了 mkfs 的崩溃点重放流我只找到那一条（`2+12+1+1+1+4`；`crash_judge_gpu_agrees_with_the_cpu_judge_on_a_make_filesystem_over_a_used_pool.rs` 是它的 GPU 对拍。`grep -ln "make_filesystem" crates/singlefs-checker-tier/tests/*.rs` 命中 18 个文件，多数只拿 mkfs 建基镜像，我没逐个核），不挂就是。分段靠登记，不靠盘面。
  2. 在镜像上找「mkfs 已返回」的证据（丙-I 能用的唯一镜像内办法）：mkfs 写的槽世代号全是 0，返回之后任何一次系统配置写都带世代号至少 1，于是「幸存的某个自证槽世代号至少 1」可当证据。量的：`name=first_mount_after_mkfs_correct single_dead_states=0 without_evidence_of_mkfs_return=0` 与 `name=first_mount_after_mkfs_MUT_B2 single_dead_states=16 without_evidence_of_mkfs_return=6`——mkfs 末段的态一个都不被它误红，但第一次可写挂载取号窗口里单盘全废 16 态有 6 态没有证据而漏报（推的前提：证据只看世代号，不看根环与 journal）。
  3. 不分，照 I-8.5 全池皆废的先例：条文本来就让 mkfs 没做完的盘面照判红，由已知红清单认下：「全池皆废判红（mkfs 没做完、写到系统配置之前失败的盘面也是全池皆废，照判红，故障注入的已知红清单认这一类，用户 2026-10-03 定）」（`crates/singlefs-checker/invariants/03-journal.md:12`）。丙-I 走这条路要改的认下处见 Y2-c。

### Y2-c 丙-置 与 丙-I 各自要改哪几处、判出什么

**丙-置（我的具体化，被攻过零轮）。** 一个只记计数的观察者（形状见「pub type ObserveACrashState<'observer> = dyn Fn(u64, &CrashImage<'_>, &RecoveryReport, &mut CrashPointReplayObserverCounts)」，`crates/singlefs-checker-tier/src/crash_amplification.rs:16746`），每个登记节点的每个状态调一次：

1. 读 `system_configuration_redundancy_report`（`crates/singlefs-checker/src/image.rs:1492`）两次——在 `image` 上、在 `image.base` 上（`CrashImage` 有 `base`：`crates/singlefs-harness/src/memory_pool.rs:463`，且实现了 `ImageReader`：第 780 行）。
2. 判「相对基镜像**新出现**的、一份自证过的槽都没有的盘」，且池里至少还有一块盘有自证槽；有就 panic（观察者的 panic 在录入线程里接住记红项：「看一段的核对线程自己 panic（观察者的 panic 在线程里接住，记成红项）」，`crates/singlefs-checker-tier/src/crash_amplification.rs:18700`），并 `counts.add` 记判过几个状态。全池皆废的态不重记（I-8.5 已红，Y2-d 丙-择）；起点就死的盘不记（相对基镜像，继承，见下）。
3. 判槽自证读的是池级 checker 的读法，不碰核心层的 `parse_slot`。

要改的文件（读码）：

| 处 | 动作 |
|---|---|
| `crates/singlefs-checker-tier/tests/common_crash_point_observers/mod.rs` | 加观察者 fn（旁边是 `observe_the_mount_continuation`，第 162 行；先例：只数撕裂槽的 `count_the_torn_system_configuration_slots`，`crates/singlefs-checker-tier/tests/crash_segments_per_device_and_torn_in_place_overwrites.rs:690`） |
| `crates/singlefs-checker-tier/src/judging_units_entries.rs` | `OBSERVER_ENTRIES`（第 134 行）加一行；登记行今天 56 条（`awk 'NR>=134' crates/singlefs-checker-tier/src/judging_units_entries.rs \| grep -c "observer_name:"` 输出 56）。没登记的观察者取键拒算 |
| `crates/singlefs-checker-tier/tests/common_crash_points/mod.rs` | 标准前缀的登记表（「pub static OBSERVERS_ON_THE_STANDARD_PREFIX」，第 282 行：取号加暖机、第一个文件、顺序写三个节点）与快照、整理、闭环各步的表各加一个成员 |
| 一条登记完整性用例（新） | 每个计划里含系统配置槽写段的节点都挂了它，记录 mkfs 的流除外；今天新加一条流漏登记没有任何红 |
| `crates/mutations.tsv` | 至少四行：观察者整条不判、只判整写域不判撕裂域、不取基镜像（继承的死盘也报）、判槽读法换成永远「有」。判别力靠手造写表（先例同上），不靠改核心层 |
| 不动 | `singlefs-core`、`singlefs-checker`、`mount_transparency.rs`、WGSL |

撕裂域在内（读码）：录入段枚举状态用的就是带撕裂镜像的表：「crate::crash::TornImageTablesOfTheReadBackFlow::of_the_read_back_flow(」（`crates/singlefs-checker-tier/src/crash_amplification.rs:18737`）；展开方式取计划的，被 ignore 的点整段不展开（第 465 行），观察者不能挂在 ignore 的点上。快档（甲二）对系统配置槽段与全量相同：这几段里只有原地写，没有单元写。

**判出什么（量的，模型）：** MUT-B1 型 42 态、MUT-B2 型 16 态（另 1 态两盘全废归 I-8.5）、回卷传两次那一形（B4）3 态；正确实现 0 态；从死盘起的补写流 3 态不报（相对基镜像）。

**继承的死盘（量的）：** `name=rewrite_flow_from_a_dead_device ... single_dead(whole/torn)=2/1`、`name=rewrite_flow_states_still_dead_before_the_second_round_persisted dead_states=3`：起点就有一块盘两槽全废时，正确实现的取号两轮在第二轮落盘之前的 3 个态里那块盘仍是死的。不取基镜像就是 3 条假红（每条这样的流），所以相对基镜像是观察者的一部分；它就是 G4-戊 在观察者里白送的那一半。

**丙-I（正文起的名，零轮）要改的几处（读码）：**

| 处 | 动作 |
|---|---|
| `crates/singlefs-checker/src/image.rs:1460` | `judge_some_device_carries_a_self_verified_system_configuration` 加逐盘判；旁路报告的文档（第 1479、1485 行）跟着改 |
| `crates/singlefs-checker/src/walk.rs:17329`、`:17357` | 两处调用与「一块盘都择不到」那一支的不适用说明（第 17327 行起） |
| `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:5223` | GPU 内核里同一个 I-8.5 块（`if (!some_slot_verifies_itself) {`）加逐盘判，并过 GPU 对 CPU 的对拍用例（今天 I-8.5 的两条在 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 第 2020、4689 行） |
| `crates/singlefs-checker/invariants/03-journal.md:12` | I-8.5 条文与状态格 |
| 测试翻面 | 「单盘全废不进违例通道：盘 1 两槽都翻坏」那条（`crates/singlefs-harness/tests/checker_known_bad_images.rs:8833`）断言 I-8.5 成立，要翻成红；`crates/mutations.tsv` 第 1616、1617、1773 行三条 I-8.5 变异要重写 |
| mkfs 流的期望 | 「"{invariant} 判违例的状态数（只有整池拒的状态上 I-8.5 红）：{:?}"」（`crates/singlefs-checker-tier/tests/crash_enumeration_make_filesystem_over_a_used_pool_stream.rs:272`）：用过的盘上 I-8.5 红的态数要加 44（Y2-b），快档、全量各一遍 |
| 故障注入的已知红识别 | 「起点段里 mkfs 自己把错交回来，录制流里一次系统配置槽写都没落盘」（`crates/singlefs-harness/src/fault_injection.rs:1462`）：mkfs 注入点落在槽写之间时（推的）盘面是单盘全废，今天不红、之后红，识别要放宽；`crates/mutations.tsv` 里认这一类的几行跟着重写 |
| 拒绝臂必须撤（否则同一盘面在崩溃注入里记两处） | `crates/singlefs-harness/src/mount_transparency.rs` 的枚举成员（第 2950 行）、臂（第 3066–3077 行）、复算（第 2406–2443 行），`crates/singlefs-checker-tier/src/crash_injection.rs` 的计数字段（第 596 行）与三处用法、第 8968 行的测试，E171 装置按成员做的穷举匹配（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:3529`，编译层耦合），`crates/mutations.tsv` 第 2406–2408 行三条改指新判法 |
| 判定库 | 三组判法单元的键全动（下表） |
| E171 的 O3 | `scripts/experiment-apparatus/e170_o3_spec_executor/system_configuration.py:160` 的说明写「单盘两槽都不可择不拒池（I-8.5「单盘全废不判违例」）」；冻结的实验装置不改，O2 与 O3 在单盘全废态上之后分歧，E171 里有多少这样的态我没读 |

**两个候选并排（盘面 X1 = 池里一块盘两槽都不自证、另一块完好，补写闸有一件不过，可写挂载在取号之前拒，拒因是「读阶段判完之后系统配置又读不出」；X2 = 同样的盘面但补写闸五件都过，挂得上并顺带补写；「条」= 崩溃注入里一次 `findings.settle` 记的一条发现、崩溃点重放里一个红项）：**

| 问 | 丙-置 | 丙-I |
|---|---|---|
| 判官住哪一档 | checker 档（观察者与登记在 `singlefs-checker-tier`，验收用例是重型）；判槽读 `singlefs-checker` | 池级 checker 本体（坏镜像用例在 harness 档，轻）加 GPU 内核（checker 档） |
| 分清 mkfs 中途与之后 | 按节点登记，记录 mkfs 的流不挂；不看盘面 | 盘面分不出（Y2-b 的 8 对 8）；证据法漏 16 态里的 6 态；认下法要改 mkfs 流期望 +44（用过的盘）与故障注入识别 |
| 挂在哪、判几次 | 登记节点的每个状态一次；标准前缀三个节点被多条流共享，块键相同的块库里只录一次 | 每条流每个状态一次（CPU 或 GPU 的池级 checker 那一格），崩溃注入三截里池级 checker 各再判一次 |
| 撕裂域 | 在（观察者拿的是带撕裂镜像的表） | 在（池级 checker 判每个状态含撕裂镜像） |
| 崩溃注入一侧 | 不覆盖：拒绝臂只罩被拒的态，补写闸放过的单盘全废盘面挂上了在注入里没人判；这类盘面的协议缺陷由重放里同一批写序的流判 | 覆盖：三截的池级 checker 都判（第一截：`crates/singlefs-checker-tier/src/crash_injection.rs:1693`；第三截：第 4347 行） |
| X1 各记几条 | 重放：观察者 1，池级 checker 0；注入：第一截 0，第二截拒绝臂 1，第三截拒绝臂 1 | 重放：I-8.5 红 1；注入：第一截 1，第二截 1 条发现带两个理由（池级与拒绝臂，`judge_the_writable_mount_on_the_crash_image` 把池级违例与拒绝并成一个 observation，第 3560 行），第三截 2 条（池级一条 `crates/singlefs-checker-tier/src/crash_injection.rs:4347`，拒绝臂一条 `:4324`）；撤臂后各截比这少臂的那一份 |
| X2 各记几条 | 重放：观察者 1；注入 0 | 重放：1；注入：第一截 1，第二截 0（挂载之后的池绿），第三截里继承着死盘的早期态各 1（Y2-c 的 3 态，推的） |
| C670 / Q3 | 0 / 0 | 0 / 0 |
| 判槽自证读谁 | 池级 checker 的读法（独立于核心层） | 池级 checker 本体；撤臂之后判官不再有共用核心层算法的那一臂 |
| 对已有判定库的单元键（量的，下表） | 新增一个观察者组，已有的不动 | 判镜像、给定状态、GPU 三组动，56 个观察者的结论版本全换键 |
| 登记完整性 | 要一条用例，否则新流漏登记没有红 | 不需要 |
| 与用户已定的关系 | 不动：I-8.5 照旧对着只读恢复 | 反转一半：定案原文「3. I-8.5 对着只读恢复，可写会拒的走旁路报告。」（`research/prompts/m3-inv-a-r1-main-verification.md:58`）——单盘全废只读恢复挂得上（「一块盘两槽全废，只读挂载照常挂上」，`crates/singlefs-harness/tests/system_configuration_choice_names_an_empty_pool_and_devices_that_lost_both_slots.rs:188`），丙-I 把它判成违例，要交用户 |

**判定库单元键（量的）：** 命令 `target/debug/unit-graph closure . <入口> --target lib --toolchain-line 'host: x86_64-unknown-linux-gnu'`（GPU 那条另加 `--cfg 'feature="gpu"'`），原样输出在 `out/unit_graph_membership.out`：

| 组 | 入口 | 闭包 | 含 I-8.5 判法 | 丙-I |
|---|---|---|---|---|
| 判镜像的定义 | `singlefs_checker_tier::crash::judge_crash_image` | 2487 个单元 | 含 `judge_some_device_carries_a_self_verified_system_configuration`（`crates/singlefs-checker/src/image.rs:1460-1477`）、`system_configuration_slot_readings`、`slot_verifies_itself`；不含 `system_configuration_redundancy_report`（0 命中） | 换键 |
| 判给定状态 | `singlefs_checker_tier::crash_given_states::judge_the_given_states_on_the_cpu` | 2567 个单元 | 含（1 命中） | 换键 |
| GPU 判器 | `singlefs_checker_tier::crash_judge_gpu::JudgeKernelSources::of_the_judge` | 1722 个单元（WGSL 1600） | 含 WGSL 函数 `judge_the_pool_from_its_system_configuration_slots`（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:5179-5418`） | 换键 |
| CPU 事实核对 | `singlefs_checker_tier::crash_facts::verify_states_from_facts` | 90 个单元 | 0 命中 | 不动 |

观察者的结论版本由池级 checker 那一组的键拼出：「let pool_checker_version = ConclusionVersion(*pool_checker_units.fingerprint());」（`crates/singlefs-checker-tier/src/crash_amplification.rs:17028`），池级 checker 那一组固定是判镜像的定义（「JudgingUnits::JudgementOfTheCrashImages」，同文件第 16948 行），所以丙-I 让所有观察者的结论换键；丙-置只要观察者**只调** `system_configuration_redundancy_report`、不改 `image.rs` 里已有的函数，已有的键一个不动。工具是 10 月 18 日编的二进制，读的是仓此刻的源码树（闭包按函数单元，没跑 `build.rs`，所以这是闭包成员的量，不是编进二进制的键；键换不换要提交时看）。若下一批本来就要改 `image.rs`、`walk.rs`（正文第二节），判镜像与给定状态两组的键本来就换，丙-I 在 CPU 一侧的边际是 0；GPU 内核与观察者各组的边际取决于那一批碰不碰 WGSL（推的）。

### Y2-d 丙-择、G4-戊、拒绝臂撤与留，以及丙-置 对丙-I 的取舍

**丙-择（择不到系统配置也记一条）：不做。** 择不到系统配置 = 没有一块盘交得出可择的槽，正确实现的写序里它只会是全池皆废（量的：`all_dead` 在 MUT-B2 里 1 态，`name=MUT_B2 ... all_dead(whole/torn)=0/1`），而全池皆废池级 checker 已判红：「全池皆废判红」（`crates/singlefs-checker/invariants/03-journal.md:12`）。观察者再记就是一个态记两条。另一支「自证过而不可择」（incompat 位不认得）I-8.5 报不适用而不是红（`crates/singlefs-checker/src/walk.rs:17323` 起那一段注释），但这要有写者写出不认得的位，正确实现写不出（推的）。推翻：出现 I-8.5 不红而择不到系统配置的崩溃态。

**G4-戊（第三截只判新增的无槽盘）：** 我把它读成「第三截只记相对第一次崩溃镜像新出现的无槽盘」。
- 在丙-置 的观察者里它是白送的：观察者拿到 `image.base`，相对基镜像判（Y2-c）。
- 在留着的拒绝臂上不做（零轮）：臂拿到的只有二次崩溃镜像（`RefusalOnTheCrashImage`），做戊要给它加「第一次崩溃镜像上已死的盘」一栏，判官签名与 E171 的调用点都要动；收益只是坏实现下少几条重复的第三截发现，正确实现根本没有继承的死盘。推翻：第三截出现正确实现下的继承死盘。

**G4 拒绝那一臂撤与留：留（丙-置 之下）；丙-I 之下必须撤。**
- 臂的触发条件（读码）：无读故障时拒因是「读阶段判完之后系统配置又读不出」只可能是单盘全废且补写闸有一件不过（Y1-b 第二行）。补写闸第一件要求这块盘带所选那一版的全部单元；发布把单元写到每一块盘（「CommitStep::WriteUnitToEveryDevice {」，`crates/singlefs-core/src/transaction.rs:485`），所以协议缺陷造出的单盘全废盘面通常过得了闸、挂上并补写，臂不触发——攻方在副本上量到的也是这个：「补写闸正常那一趟 42 张全挂上」（`research/prompts/m3-close-g4-rewrite-r2-opus-output.md:170`）。臂今天的探测力在协议缺陷上近乎零，价值在「闸不过的单盘全废盘面不许出现」这一句。
- 撤的代价（读码）：枚举成员、臂、复算、计数字段、测试、E171 装置里按成员的穷举匹配（编译层耦合）、三行变异（`crates/mutations.tsv` 第 2406–2408 行）改指新判法。留的代价：零。
- 留不制造重报：臂在崩溃注入，丙-置 的观察者在崩溃点重放，两个套件各记各的（Y2-c 表）。丙-I 让池级 checker 在崩溃注入里也红，同一张被拒盘面第三截记两条，这正是第二轮「一件事记两条」那条理由在新位置上的重现，所以丙-I 必须撤臂。
- 留着的旧账：臂判槽自证共用核心层 `parse_slot`（Y1-b 第三行）。能换的只是「某块盘一份本池自证过的都没有」那一半，改法见「自己提的改法」第 4 条。
- 推翻：崩溃注入的随机历史里出现补写闸放不过的单盘全废盘面且臂报了协议缺陷（臂的价值比我估的大）。

**丙-置 对丙-I：取丙-置。** 理由按分量排：
1. 性质不同（推的，但可验）：「没有哪块盘因为写序而新掉了全部系统配置槽」是**写序的性质**，不是**任何一张可恢复镜像都满足的性质**——D22 专门给两槽全废的盘开了补写（`.claude/kb/decisions/22-单元原子性怎么合成.md:181`），说明这样的镜像是条文接得住的合法输入。池级 checker 的 I-8.5 是镜像性质；写序性质归判写序的一侧（崩溃点重放的观察者）。
2. 盘面分不出中途与之后（Y2-b，量的）：丙-I 只能认下或用证据法，丙-置 按节点登记就分开。
3. 重报：丙-I 在崩溃注入第三截一张被拒盘面记两条（Y2-c 表），要连拒绝臂一起撤，E171 装置跟着改。
4. 键：丙-I 动三个判法组与全部观察者的结论版本（Y2-c 的量）；丙-置 只加一个观察者组。下一批本来要改 `image.rs` 时 CPU 一侧的这一条打折（推的）。
5. 用户定案：丙-I 反转一半 `research/prompts/m3-inv-a-r1-main-verification.md:58`，要交用户。

丙-置 自己的代价：覆盖面靠登记（要一条完整性用例）；崩溃注入里补写闸放过的单盘全废盘面仍然没人判（由重放里同一批写序的流判，不是同一批状态，推的）；两个套件的判官都要记账。

**推翻取舍的现象：** 用户愿意把「单盘全废」改成池级违例；或登记完整性用例做不出来（节点和流的对应关系不是机器能枚举的）；或下一批改的恰好是 WGSL 与 `check_pool_image`，使丙-I 的重判成本并进那一批。

## Y3 未定项清单：四处条文的改后候选，与全仓同一件事的其余处

下面都是**候选**（零轮，依丙-置；原句整句抄自文件，改动只在替换的那一段，行里其余原样不动）。三语：D13、D23 各有 `.en.md`、`.ja.md` 译文，同一处要同改——D13 英文、日文版第 72 行对得上，「由 G4 判」那一句在英文、日文版是第 185 行（「judged by G4 (crash injection phases 2, 3」，`.claude/kb/decisions/13-verification-route.en.md:185`）；D23 英文、日文版第 396 行（「also refuses when disk lacks any self-verified system configuration, user decision 2026-10-03」，`.claude/kb/decisions/23-journal-role-format.en.md:396`；日文「あるディスクで自証通過システム構成が皆無の場合も拒絶、ユーザー 2026-10-03 決定」，`.claude/kb/decisions/23-ジャーナルの役割と形式.ja.md:396`）。

### Y3-a D13 第 72 行（挂载透明的射程）

原句：「拒绝合法 ⇔ 拒因在 D23（journal 的角色与格式） 已定项 14 那张单子上，且照那一条的判据在崩溃镜像上自己复算成立，且盘上逐字节不变，且只读读出在允许集合里」（`.claude/kb/decisions/13-验证路线.md:72`）。

问题（读码）：「读阶段判完之后系统配置又读不出」这一拒因，判官复算的是「某块盘一份本池自证过的都没有」那一半（`crates/singlefs-harness/src/mount_transparency.rs:2406`），D22 补写闸五件它不复算，而 D22 已定项 8 说补写闸五件都过就不拒（「某块盘两槽全废（一份本池自证过的系统配置都没有）时，取号那一步补写它的两槽，要这几样都成立，有一样不成立就拒」，`.claude/kb/decisions/22-单元原子性怎么合成.md:181`）。「照那一条的判据复算」对这一拒因因此说过了头。

候选：在「且照那一条的判据在崩溃镜像上自己复算成立」（`.claude/kb/decisions/13-验证路线.md:72`）之后插入：

> （「读阶段判完之后系统配置又读不出」这一拒因，判官复算的是 D23（journal 的角色与格式） 已定项 14 取号那一步里「某块盘一份自证过的系统配置都没有」那一半，不复算 D22（单元原子性怎么合成） 已定项 8 的补写闸五件：判官收到的拒因不带哪一件没过，补写闸由 harness 档用例强制进入、逐件变异判；补写闸放过的单盘全废盘面挂上了不拒，归崩溃点重放里挂在节点上的观察者判，见已定项 6）

### Y3-b D13 第 187 行（「三条 oracle 都不判的两样」第一样）

原句：「「无读故障的崩溃态上每块盘至少一份自证过的系统配置」这一性质由 G4 判（崩溃注入第二、三截，与 C670（无读故障的崩溃态上可写挂载见证判据为真没有判官） 那条同接法）」与「mkfs 返回之前的崩溃态不在 G4 的射程，mkfs 逐盘两槽在同一个循环里写、之后才逐盘屏障，崩溃注入的候选段与 E171（三oracle合谋测试） 的基镜像都取在 mkfs 之后」（都在 `.claude/kb/decisions/13-验证路线.md:187`）。

候选（把这两句换成下面一段，后面「G4 与那条还有一处不同形……」那句原样留）：

> 「无读故障的崩溃态上每块盘至少一份自证过的系统配置」这一性质分两处判：一是崩溃点重放里挂在 mkfs 之后各节点上的观察者（撕裂态在内，每个状态判一次；读池级 checker 的 `system_configuration_redundancy_report`，数相对流的基镜像新出现的、一份自证过的系统配置都没有的盘，不读核心层的解析；全池皆废的态已由 I-8.5 判红，观察者不重记）；二是崩溃注入第二、三截里「读阶段判完之后系统配置又读不出」那一拒（G4，与 C670（无读故障的崩溃态上可写挂载见证判据为真没有判官） 那条同接法），只罩被拒的态，补写闸放过的盘面挂上了不拒，由前一处判。mkfs 返回之前的崩溃态不在这一性质的射程：mkfs 逐盘两槽在同一个循环里写、之后才逐盘屏障（D22（单元原子性怎么合成） 已定项 8），末段留得下一块盘两槽都自证不过，观察者不挂在记录 mkfs 的节点上，崩溃注入的候选段与 E171（三oracle合谋测试） 的基镜像都取在 mkfs 之后

第一节开头的括注（原文「（第一样现在有崩溃注入这一侧的判官）」）要补成「（第一样现在有崩溃注入与崩溃点重放两侧的判官）」。**丙-I 之下**这一段改成：这一性质由 I-8.5 判（池级 checker，各流各截），mkfs 末段与故障注入的 mkfs 中途由已知红清单认下，拒绝臂撤（G4 一句整个删，「不同形」那句随之删）。

### Y3-c D23 第 396 行（可写挂载读到的更新状态读不出）

原句：「这一步在某块盘一份自证过的系统配置都没有时也拒（用户 2026-10-03 定留着）」（`.claude/kb/decisions/23-journal的角色与格式.md:396`）。取号那一步实现已有例外：「撕裂的系统配置轮换（读得出、自证不过）不拒：取号写重写坏的那一槽。」（`crates/singlefs-core/src/transaction.rs:872`）与补写闸（`crates/singlefs-core/src/mount.rs:5797`），条文没带。

候选：把括注换成「（用户 2026-10-03 定留着；2026-10-19 开补写例外：这块盘过了补写闸五件的不拒，取号写两轮顺带补写它的两槽，五件与权威写法在 D22（单元原子性怎么合成） 已定项 8，实现落点 `crates/singlefs-core/src/mount.rs` 的 `devices_without_the_selected_version`）」。同一行里讲 Q3 的那一句（「挂载透明（`crates/singlefs-harness/src/mount_transparency.rs`）在镜像上复算「按最新那条根的实例表，环里一条不被抛弃的自证有效根都没有」，没注读故障时复算为真判不合法、注过读故障的态不判、实例表读不出时不按表滤」，同文件第 396 行）旁边补一句同形的 G4：「挂载透明在镜像上复算「某块盘一份本池自证过的系统配置都没有」（取号那一步的拒因），没注读故障时复算为真判不合法，射程到 mkfs 返回之后，补写闸放过的盘不拒、不在这一拒里」。

### Y3-d I-8.5 第 12 行

原句：「单盘全废不判违例，走旁路报告：另报哪块盘少于两槽自证过、可写挂载会拒。对着只读恢复判。」（`crates/singlefs-checker/invariants/03-journal.md:12`）。

问题（读码，两处陈旧）：一是补写——一块盘一份自证过的槽都没有时，可写挂载取号在补写闸五件都过时不拒（`crates/singlefs-core/src/mount.rs:5797` 之前的分支）；二是 R2——只有一槽自证过的盘照常可写挂载：「判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过。」（`crates/singlefs-core/src/recovery.rs:4399`）与「撕裂的系统配置轮换（读得出、自证不过）不拒：取号写重写坏的那一槽。」（`crates/singlefs-core/src/transaction.rs:872`），所以「少于两槽自证过 ⟹ 可写挂载会拒」今天不成立；`devices_whose_writable_mount_would_be_refused` 这个字段名本身也成了误称（推的：改名牵动 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 第 8867、8876 行两处断言，建议只改文档、名字留到实现批）。

候选：

> 单盘全废不判违例，走旁路报告：另报哪块盘一份自证过的槽都没有、哪块盘少于两槽自证过（一槽都没有的盘，可写挂载取号那一步按补写闸五件处置——五件都过的补写、有一件不过的拒，D22（单元原子性怎么合成） 已定项 8；只有一槽自证过的盘照常可写挂载，判不出取 R2，同一处）。对着只读恢复判；单盘全废在 mkfs 之后的写序里不该出现，那一半由崩溃点重放里挂在节点上的观察者判（D13（验证路线） 已定项 6），不进这一条的违例通道。

**丙-I 之下**这一行改成：单盘全废判红（mkfs 末段与故障注入的 mkfs 中途由已知红清单认下），旁路报告只留「少于两槽自证过」那一半；并把「对着只读恢复判」整句去掉，交用户（Y2-c 末行）。

### Y3-e 全仓同一件事的其余处（前两轮没点到的）

命令（脚本 `research/prompts/m3-close-g4-rewrite-r3-sonnet-model/tools/y3_greps.sh <仓根>`，对每个说法 `grep -rnI --exclude-dir=target --exclude-dir=.git -- '<说法>' <区域>` 数行数与文件数；`research/prompts` 区域是各轮冻结证据与本轮材料，数了不引）。非 `research/prompts` 区域的命中（行数/文件数，量的，我这一趟跑的原样在 `out/y3_greps.out`）：

| 说法 | `.claude/kb` | `crates` | `scripts` | `records` |
|---|---|---|---|---|
| 可写挂载会拒 | 1/1 | 5/4 | 0/0 | 1/1 |
| 单盘全废 | 3/3 | 8/4 | 5/5 | 1/1 |
| 两槽全废 | 2/2 | 20/10 | 0/0 | 5/3 |
| 两槽都自证不过 | 0/0 | 9/7 | 0/0 | 1/1 |
| 两槽都废 | 0/0 | 2/2 | 0/0 | 0/0 |
| 少于两槽自证 | 1/1 | 2/2 | 0/0 | 0/0 |
| 一份自证过的系统配置都没有 | 7/4 | 19/10 | 0/0 | 1/1 |
| 一份本池自证过的系统配置都没有 | 1/1 | 9/4 | 0/0 | 0/0 |
| 没有自证过的槽 | 0/0 | 2/2 | 1/1 | 0/0 |

kb 与 crates 里非测试、非变异表文件的命中逐行列在 `out/y3_greps.out` 末段（「== kb 与 crates 里非测试、非变异表文件的命中行」）。读过之后分三类：

| 类 | 处 | 状态 | 处置 |
|---|---|---|---|
| 陈旧、前两轮没点到 | `crates/singlefs-checker/src/image.rs:350-351`：「单盘全废、哪块盘少于两槽自证过（可写挂载会拒）」（常量 `SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION` 的文档） | 补写与 R2 之后不成立（Y3-d） | 实现批改注释 |
| 同上 | 测试文档：`crates/singlefs-harness/tests/checker_known_bad_images.rs:8833-8834`；`crates/singlefs-harness/tests/system_configuration_slot_with_a_foreign_node_size_is_not_chosen.rs:95` | 同上 | 实现批改注释 |
| 同上，正文已点名 | `crates/singlefs-checker/src/image.rs:1479`、`:1484-1485`（报告类型与 `devices_whose_writable_mount_would_be_refused` 的文档，第 1484 行写的是 R2 之前的判不出：「可写挂载读阶段读每块盘两槽，有一槽读不出或自证不过就判不出、按真，重读一次仍不成就拒可写」） | 同上，且 R2 一条正文没提 | 实现批改 |
| 同上，条文 | `crates/singlefs-checker/invariants/03-journal.md:12`（Y3-d） | 同上 | kb 写回批 |
| 历史记载，不改 | `.claude/kb/checks-owed.md:732`（C461 已还清行：「另报哪块盘少于两槽自证过、可写挂载会拒，I-8.5（tail 槽有效集非空） 取全池皆废才判红」）与它的译文行（`.claude/kb/checks-owed.en.md:727`、`.claude/kb/checks-owed.ja.md:727`，译文里这句写成「writable mount会拒」「書き込み可能マウント会拒」，中文关键词 grep 不到它）；C367 行（第 734 行，译文第 729 行）；`.claude/kb/decisions-history.md:11725`；`records/` 里的几处（`两槽全废` 5 行、`单盘全废` 1 行、`可写挂载会拒` 1 行） | 当时是真的 | 不改（改历史） |
| 仍对，若采丙-I 才要改 | `crates/singlefs-checker/src/walk.rs:17323`（「I-8.5（V-池）：全池皆废判红；一块盘交得出可择的系统配置就照常往下判（单盘全废走旁路报告，不进违例通道）。」） | 今天对 | 丙-置 不动 |
| 仍对 | `.claude/kb/decisions/22-单元原子性怎么合成.md:181`；`crates/singlefs-core/src/mount.rs` 里讲补写闸的 5727、5745、5754、5849、6914 行；`crates/singlefs-core/src/transaction.rs:879`、`:1085`；`crates/singlefs-core/src/system_configuration_repair.rs:43`；`crates/singlefs-checker-tier/src/crash_injection.rs:8959` 起的测试文档 | 说的都是补写，与现行一致 | 不动 |
| 拒绝臂的注释 | `crates/singlefs-harness/src/mount_transparency.rs:3066-3068` 与 `:2945` 起的枚举成员文档 | 第一轮已收窄到 mkfs 之后 | 丙-置 之下补一句「补写闸放过的盘面不拒，归观察者」 |
| 冻结的实验装置 | `scripts/experiment-apparatus/e170_*/system_configuration.py` 四份的说明写「I-8.5「单盘全废不判违例」」（如 `scripts/experiment-apparatus/e170_o3_spec_executor/system_configuration.py:160`） | 冻结 | 不改；丙-I 之后与 checker 分歧 |

没有一处说「单盘全废」要进违例通道。**没找到的**：译文里中文关键词 grep 不到的说法（上面 C461 那一行就是），我只按编号找了 `checks-owed` 一份；D13、D23 的 `.en.md`、`.ja.md` 译文我只按行号对到上面几处，没对每个英文、日文说法再 grep 一遍。

### Y3-f 补写那一支进不了崩溃测试、运行时看不出（欠账表第二轮新立那一行）还缺什么

现状：`.claude/kb/checks-owed.md:579` 的 C874 已经立了，原话：「补写那一支进不了崩溃测试，补写本身中途崩溃没有崩溃枚举流；补写闸哪一件没过，运行时看不出」，验收写「一条用例强制走补写那一支并在补写中途崩溃，枚举到的态都判；补写闸哪一件没过在运行时有可观测的输出，用例断言它」。取号时现查的下一个空号：文件里最大是 C878，C875–C877 没有行（命令 `grep -o '^| C[0-9]* ' .claude/kb/checks-owed.md | sed 's/[^0-9]//g' | sort -n | tail -3` 输出 873、874、878）；别的会话可能已占，取号前再查。

这一行加上第三轮的结果之后还缺（都是推的，零轮）：
1. **验收要写明相对基镜像判。** 补写流的起点就有一块盘两槽全废：量的 `name=rewrite_flow_from_a_dead_device ... single_dead(whole/torn)=2/1`、`dead_states=3`——正确实现的取号两轮在第二轮落盘之前，那块盘在 3 个态里仍是死的。观察者或任何判「新死盘」的判官不取基镜像，这条流每次多 3 条假红。
2. **补写闸五件判官不复算**要作为明说的「不判」记在这一行（Y3-a 的新句子写的就是它）；判它的是 harness 档五条强制进入的用例与 `crates/mutations.tsv` 第 5011–5015 行。
3. **补写流的造法**：基镜像 = 第一个文件之后的池把一块盘的两个系统配置槽清零（先例：「fn zero_both_system_configuration_slots」，`crates/singlefs-harness/tests/acquisition_rewrites_a_device_that_lost_both_system_configuration_slots_only_through_the_gates.rs:86`），录真实的取号两轮加补写，进崩溃点重放；不要手造写表（手造写表测不到取号写协议）。
4. **新立两行**（不是并进 C874）：（甲）观察者的登记完整性——新加一条含系统配置槽写段的流漏登记，今天没有任何红；（乙）拒绝臂判槽自证与核心层共用 `parse_slot`（`crates/singlefs-harness/src/mount_transparency.rs:2374`），违反「**运行时与 checker 必须用不同的算法。**」（`.claude/rules/fs-design.md:35`），改法见「自己提的改法」第 4 条。
5. **旁路报告的字段名与文档**（`devices_whose_writable_mount_would_be_refused` 误称，Y3-d）随下一批实现一起改，记一行免得只改注释漏了字段名。

## Y4 第二轮攻方的数：换路子核一次

路子：不跑攻方的原型，也不读它的 `src/main.rs`；只读它的 `toggles.patch` 知道两个坏实现是什么（「MUT-B1，盘表下标 0 那块盘写完新槽之后同一段里把另一槽清成全 0。」`research/prompts/m3-close-g4-rewrite-r2-opus-model/toggles.patch:7`；「MUT-B2，取号两轮之间少一道屏障」同文件第 32 行），自己的模型按切段规则、第三态判据、写协议把数推出来。输入里不是我推的只有一样：remount 流的段序列，直接取攻方产物的这一行——「segments=2:system_configuration_slot,2:system_configuration_slot,14:unit_write,2:journal_record,1:root_record_fua,2:system_configuration_slot,12:unit_write,2:journal_record,1:root_record_fua,2:system_configuration_slot」（`research/prompts/m3-close-g4-rewrite-r2-opus-model/out/remount.counts:2`），两个单元写段（14、12）是它的，别的段的写数与落盘位置是我的模型推的。

| 攻方的数 | 我的结果（`out/run_all.out`，量的） | 判 |
|---|---|---|
| 正确实现 remount 流 20519 = 整写 20499 + 撕裂 20 | `name=remount_correct ... states=20519 whole=20499 torn=20 single_dead(whole/torn)=0/0 all_dead(whole/torn)=0/0` | 对上 |
| MUT-B2：撕裂域 16 个单盘全废、1 个两盘全废；整写域 0（「撕裂域 16 态单盘全废、1 态两盘全废」，`research/prompts/m3-close-g4-rewrite-r2-opus-output.md:50`；表里「0 / 16 单盘全废」，同文件第 131 行） | `name=MUT_B2 segments=[4, 14, 2, 1, 2, 12, 2, 1, 2] states=20583 whole=20508 torn=75 single_dead(whole/torn)=0/16 all_dead(whole/torn)=0/1`；逐盘 `name=MUT_B2_dead_states_by_domain_and_device [(('torn', (0,)), 8), (('torn', (0, 1)), 1), (('torn', (1,)), 8)]` | 对上，逐盘 8 加 8 加 1 也对（手算：一块盘两个写都撕裂，另一块盘两写各三态共 9 种，扣掉两盘都撕的 1 种得 8） |
| MUT-B1：整写域 8 态、撕裂域 34 态（「MUT-B1 整写 8 + 撕裂 34；MUT-B2 撕裂 16」，同文件第 58 行；「8 / 34，全是盘 0，补写闸正常时 42 张全挂上」，第 130 行）；撕裂域总数 67 | `name=MUT_B1_base_built_with_the_mutation_like_the_prototype segments=[3, 3, 14, 2, 1, 3, 12, 2, 1, 3] states=20582 whole=20515 torn=67 single_dead(whole/torn)=8/34 all_dead(whole/torn)=0/0` | 对上，但**有条件**（下一条） |
| 条件：8/34 的拆法依赖原型把开关设在整个进程里，连造基镜像的那段历史也带着坏实现 | 基镜像也带开关：盘 0 的最后一槽被清零，第一个槽写（落在那一槽）的基镜像字节是零、同段前面没有带字节的写，不取三态——这是 `state_count=20582` 比 `20591` 少 9 的原因。基镜像干净：`name=MUT_B1_base_built_clean ... states=20591 whole=20515 torn=76 single_dead(whole/torn)=6/36` | 总数 42 不变（8+34 = 6+36），整写/撕裂的拆法随基镜像变；引「整写域 8 态」时要带这个条件（攻方报告里我没看到这个条件；它的段序列行 `state_count=20582 segments=3:system_configuration_slot,3:system_configuration_slot,14:unit_write` 是 `research/prompts/m3-close-g4-rewrite-r2-opus-model/out/remount-SINGLEFS_PROTO_MUT_B1.counts:2`，与我推的段序列 `[3, 3, 14, 2, 1, 3, 12, 2, 1, 3]` 一致） |
| 「W1 四组历史合计 172 366 态零发现」（正文第一节第 4 条，`research/prompts/_m3-close-g4-rewrite-r3-body.md`） | `second_crash_group_arithmetic 6*20519+2*24626 = 172366`、`four_groups_total 3*20519+172366 = 233923` | 172 366 是**第三截那一组**八个基线之和（「六个基线各 20499 / 20，两个基线各 24601 / 25，合计 172366」，`research/prompts/m3-close-g4-rewrite-r2-opus-output.md:74`），不是四组合计；四组之和是 233 923。数没错，标签错了 |
| 八个基线：六个 20519，两个 24626 | `second_crash_bases.out` 里八个基线我的固定形状都是 20519、0 死盘：段号 0、1、9 的六个对上。段号 5 的两个，攻方自己印出的段序列是 remount 形再加一个 12 个单元写的发布（「segments=2:system_configuration_slot,2:system_configuration_slot,14:unit_write,2:journal_record,1:root_record_fua,2:system_configuration_slot,12:unit_write,2:journal_record,1:root_record_fua,2:system_configuration_slot,12:unit_write,2:journal_record,1:root_record_fua,2:system_configuration_slot」，`research/prompts/m3-close-g4-rewrite-r2-opus-model/out/second_crash_base_4.counts:3`），把这条段序列喂给我的模型：`name=remount_plus_one_more_12_unit_publish_the_segment_list_the_attacker_printed_for_its_bases_at_segment_5 ... states=24626 whole=24601 torn=25` | 八个全对上；段序列是它的输入，不是我推的（为什么挂载在段号 5 的崩溃镜像上多一次发布，我没读，推的） |
| W1 零发现（正确实现在 remount、读故障那一步、冷启动、第三截那一形上 G4-丙 判出 0） | `name=closure_correct fixpoint_rounds=3 reachable_images=25 images_with_a_dead_device=0 dead_states_met=0`（任意崩溃状态上反复再挂载 / 抬 F / 发布的闭包） | 对上，换了方法（穷举闭包 + 协议不变式，不是抽样）；覆盖写、清扫、回退三条攻方没跑的流在我这里同属「每次发布一道轮换」，闭包里有，但它们的真实段序列我没推（Y2-a） |

攻方一句旁注的一半我的模型不支持：「或只有一块盘两槽同段的写序错，就只剩单盘全废的态」（`research/prompts/m3-close-g4-rewrite-r2-opus-output.md:166`）。量的：

    name=B3_only_device_zero_skips_its_barrier_after_round_one ... single_dead(whole/torn)=0/16 all_dead(whole/torn)=0/1
    name=B4_device_zero_writes_both_slots_in_one_segment_device_one_once segments=[3] states=27 whole=8 torn=19 single_dead(whole/torn)=0/3 all_dead(whole/torn)=0/0

按设备切段时，只吞一块盘的屏障（B3）仍会把两块盘的四个写并进同一段，两盘全废的 1 态还在，现行 I-8.5 报它；真漏的是「同一块盘在一段里写了两个槽、而另一块盘在那一段只写一次」（B4，攻方的 MUT-B1 也是这一类：多写一个清零槽）：现行 I-8.5 一态不报。回卷同一块盘传两次正是这一形（`crates/singlefs-core/src/transaction.rs:673`，第二轮判决 T1 当时靠审阅发现，不是 checker）。

**校验这条路子本身会红：** 模型对 MUT-B1、MUT-B2、B3、B4 都出死盘，对正确协议恰好 0（上表与 Y2-a）。它抓得到的坏实现我用「某段里一块盘两个槽写」这一个判别式穷举核过（Y2-a 的 729 条流）。

**什么现象会推翻我对数：** 攻方原型在入库装置上的复跑数与上表某行不同——先查两边的段序列与基镜像（我对 8/34 的解释就靠基镜像带不带开关）；或第三态判据 `is_tearable_in_place_overwrite`（`crates/singlefs-checker-tier/src/crash.rs:2261`）以外还有一条「同段更早的写罩住的范围」放宽可撕的口径（我读了三条判据，没读到别的）。

**我没复跑攻方原型**（整趟要几十分钟、是重型）：上表是换路子推出来的数，对上的是数，不是它的原型。入库装置上的复跑仍是判决要做的事。

## 自己提的改法（全部被攻过零轮）

1. **丙-置 的具体形**（Y2-c）：新观察者相对基镜像判「新出现的无槽盘」、池里至少还有一块盘有槽才记、不挂在记录 mkfs 的节点上、读 `system_configuration_redundancy_report`；加一条登记完整性用例（含系统配置槽写段的节点都挂了它）；`crates/mutations.tsv` 至少四行（整条不判、只判整写域、不取基镜像、读法永远「有」），判别力靠手造写表。验收：手造写表上 B1 型（42 态）、B2 型（16 态）、B4 型（3 态）都报，从死盘起的补写流 0 报，正确实现的真实流 0 报。推翻：登记完整性用例做不出（节点与流的对应不是机器可枚举的）；观察者在 ignore 的点上收不到撕裂态（`crates/singlefs-checker-tier/src/crash_amplification.rs:465`）而这些点恰好含系统配置槽写段。
2. **拒绝臂留、甲留、丙-择 不做、戊 不做**（Y2-d）。推翻见各条。
3. **条文候选**（Y3-a 到 Y3-d）与三语同改；新立欠账：登记完整性、拒绝臂共用 `parse_slot`、旁路报告字段名与文档（Y3-f）。
4. **拒绝臂判槽自证的一半改读池级 checker**（Y1-b 第三行的修法）：在 `crates/singlefs-harness/src/mount_transparency.rs` 里给 `&dyn PoolReader` 配一个实现 `ImageReader`（「pub trait ImageReader {」，`crates/singlefs-checker/src/image.rs:19`，要 `devices`、`device_bytes`、`read`、两个候选槽号，后两个交 `None`）的适配，`some_device_without_any_self_verified_system_configuration_slot_on_the_crash_image`（第 2406 行）改调 `system_configuration_redundancy_report(..).devices_without_a_self_verified_slot`。限制：checker 的读法不分 fsid（见 `crates/singlefs-checker/src/image.rs:1490` 的文档），崩溃镜像里没有别的池的槽所以够用，用到 harness 里手造外来盘的地方要先看；「判不出」那一半（槽报读错）checker 的读法分不出读错与自证不过，保留原样。不改拒绝臂的真值表，只换这一半的解析者。推翻：适配之后拒绝臂在 `writable_mount_refusal_on_a_crash_state_without_read_faults.rs` 的现有用例上结论变了。
5. **丙-流：一条写流上的静态检查**（观察者的轻量补充，不替它）：对每条录制出来的崩溃点重放流，断言「每一段里每块盘至多一个系统配置槽写」，记录 mkfs 的那条流除外。依据（量的，Y2-a）：健康起点、协议照现行写序时，**单盘全废态可达 ⟺ 某段里有一块盘的槽写不止一个**，729 条小流 665 条有重复写、665 条都有死盘，另 64 条没有重复写的都没有死盘。好处：不看镜像、不用每个节点登记（挂在录制出流的那一处，新流自动罩住）、不动任何判法单元的闭包（放在判法闭包之外的函数里，推的）、GPU 一路自动罩住。坏处：只罩「写序里多写」，罩不住奇偶写错槽（覆写了最新的那一槽，要先有一槽已坏才死盘）与继承的死盘；它不是判崩溃状态的判官。录制流里 `StepKind::SystemConfigurationSlot` 与设备都有（`crates/singlefs-harness/src/segments.rs` 的 `classify`），所以不需要新信息。推翻：某条真实流里合法地出现同一块盘一段两个槽写（那就是 mkfs 末段那一形，之外没找到）。
6. **丙-I 的最小形**（只在用户要把单盘全废改成池级违例时用，不推荐）：I-8.5 对每块盘判「至少一个槽自证过」；镜像分不出 mkfs 中途，所以认下——mkfs 流期望加 44（用过的盘）与 6（新盘）、故障注入识别放宽到「mkfs 报错之前系统配置槽没写全」；GPU 内核同改；撤拒绝臂（E171 装置跟着改）；`research/prompts/m3-inv-a-r1-main-verification.md:58` 的用户定案要重定。这一形我没有逐处量过代价，Y2-c 的表是读码。

## 要主 agent 跑的命令

我没有需要重型测试才能核的东西；下面都是轻量的，主 agent 判决时要引数就自己复跑一遍：

1. 模型复跑，见第 0 节（几秒）。比较 `out/` 里四份的 sha256。
2. 判定库键的闭包成员：命令见 Y2-c 的表（`target/debug/unit-graph closure . <入口> --target lib --toolchain-line 'host: x86_64-unknown-linux-gnu'`，GPU 一条另加 `--cfg 'feature="gpu"'`），经 `bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 3G nice -n 19` 跑，1 秒级，读完 grep `judge_some_device_carries_a_self_verified_system_configuration`。要和构建脚本写进 `OUT_DIR` 的键逐字相同，用 `unit-graph key` 加 `--toolchain-of-the-build`、`--lock-file Cargo.lock` 与构建时开的每个特性（`crates/singlefs-unit-graph/src/bin/unit_graph.rs` 文件头的说明），我没做。
3. 丙-I 要选时先读 E171 装置里 O2 与 O3 在单盘全废态上的比对（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs`），我没读。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Y1 甲-二 不做 | 一致（站得住） | 判官收不到补写闸，闸恒不过由现成正向测试与 5010–5015 行杀；推翻：判官签名里加闸 |
| Y1 原形态·整写子集 | 一致，限定 | 崩溃注入 `Tearable` 命中 0；对两槽同段那一类成立，B1 型整写域也有 6–8 态 |
| Y1 原形态·一件事记两条 | 一致，限定 | 对「新判官挂在崩溃注入」成立；丙-置 结构上避开，丙-I 会重现 |
| Y1 原形态·共用核心层算法 | 一致，且旧账 | 今天的拒绝臂同样共用 `parse_slot`；欠账新立 |
| Y1 第一轮的甲 | 条件 | 拒绝臂留则留（丙-置），撤则不要（丙-I） |
| Y2 撕裂域留不留单盘全废态 | 不留（量的 + 读码） | 可达闭包 25 张盘面 0 死盘；单盘全废可达 ⟺ 某段里一块盘写了不止一个槽 |
| Y2 分清 mkfs 中途与之后 | 盘面分不出（量的 8 对 8） | 只能按流登记（丙-置）或认下（丙-I，用过的盘 44 态、新盘 6 态） |
| Y2 丙-置 | 取 | 改一个观察者加登记，不动 core/checker/judge，已有键不动；撕裂域在内 |
| Y2 丙-I | 不取（交用户） | 动三个判法组与 56 个观察者的结论键、GPU 内核、mkfs 期望、故障注入识别，要撤拒绝臂，反转一半用户定案 |
| Y2 丙-择 | 不做 | 全池皆废 I-8.5 已红 |
| Y2 G4-戊 | 观察者里白送；拒绝臂上不做 | 观察者拿 `image.base`；臂上要改判官签名，收益只是少几条重复 |
| Y2 拒绝臂撤与留 | 留（丙-置） | 协议缺陷造的盘面过得了闸，臂几乎不触发，撤的代价却非零；丙-I 必须撤 |
| 自己提的丙-流（写流静态检查） | 零轮，补充不替代 | 单盘全废可达 ⟺ 某段里一块盘写了不止一个槽，所以一条「每段每块盘至多一个槽写」的检查等价于协议缺陷这一半，不用逐节点登记 |
| Y3 D13 72、187、D23 396、I-8.5 12 | 候选已写 | 核心：补写例外进 D23；判官只复算前一半；观察者接补写闸放过的盘面 |
| Y3 其余同一件事 | 前两轮没点到的陈旧处 | `image.rs:350-351`、两处测试文档、`image.rs:1484` 的 R2 之前说法；R2 使「少于两槽即拒」本身就不成立 |
| Y3 补写欠账 C874 | 已立，缺 5 样 | 相对基镜像、闸五件不判的明说、补写流造法、新立两行、字段名 |
| Y4 MUT-B2、MUT-B1、20519 | 对上（我的模型推出） | MUT-B1 的 8/34 依赖原型把开关设在整个进程，基镜像干净则 6/36，总数 42 不变 |
| Y4 172 366 | 数对、标签错 | 是第三截一组之和，四组之和 233 923；八个基线喂攻方印出的段序列全对上 |
| Y4 攻方旁注 | 一半不支持 | 只吞一块盘的屏障仍有两盘全废态；真漏的是一块盘一段多写一槽（B1、B4 型） |

## 没做什么

- 没读禁读清单里的三样；没替主 agent 采纳或出判决；本轮没有攻方腿，不判别的腿的格没碰。
- 没复跑第二轮攻方原型（重型、几十分钟），Y4 是换路子推的数；没在入库装置上重做任何数——主 agent 引这些数之前要在入库装置或复跑过的原型上坐实。
- 没编 Rust、没起 checker 档测试、没跑崩溃点重放；没起 `gate.sh`。没跑 `build.rs`，所以键换不换是闭包成员的量，不是编进二进制的键。
- 没读 E171 装置里 O2、O3 的比对，所以丙-I 对 E171 的影响面没量；没读 `fault_injection.rs` 的识别器全文，只读了已知红形态的枚举与说明。
- 带读故障的历史、三块盘以上的池不在模型里；覆盖写、清扫、回退三条流我只按「每次发布一道轮换」归类，真实段序列没推。
- 译文（`.en.md`、`.ja.md`）只按行号对到 D13、D23 的几处，没逐句核；`checks-owed` 译文里 C461 一行只按编号找了。
- 没改 `crates/`、kb、`crates/mutations.tsv`；没往欠账表写行；条文只写了候选，没写回。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/m3-close-g4-rewrite-r3-sonnet-output.md --root . --background research/prompts/_m3-close-g4-rewrite-r3-background.md` 退 0，末行原样：

    ✓ 核了 41 处引文、67 处裸行号，对上 41 处，没判 67 处（其中裸行号没有引文可比 67 处；逐处列在上面）

67 处裸行号我逐处用 `grep -nF` 或 `awk 'NR==行号'` 现取过（行号在文件里、内容与上下文说的一致），脚本只核得到行号在不在文件里。
