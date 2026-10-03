# m3-formal-code-r2 云端辩方（Sonnet）报告（2026-10-02）

腿：云端辩方。复核对象：代码轮第一轮判决 `research/prompts/m3-formal-code-r1-main-verification.md`（含「改法落地」一节）落地之后够不够、有没有改过头；替被改掉的旧写法（`newest_root_among` 取后一个、checker ③ 取 4096、合判推论不加 J1 那一行）找站得住的理由。用户定案（F1–F6、G1、G2、H1、W1、V1 改逐档）不改，只判写成代码哪里站不住。

读法：代码按冻结副本 `/tmp/claude-1000/m3-formal-code-r2/tree/` 读（`sha256sum -c --quiet` 对 `crates-sha256.txt` 在副本里无输出；仓里工作区只有 `crates/mutations.tsv` 一份对不上，见 D9）。引文行号都是那份文件自己的，写进报告前用 `grep -nF` 现取。只读码、`grep`、`python3` 数命中、一次 `crates-mutation-rows.py static-check`（静态，不编译）。没编译、没跑任何测试、没跑 Verus、没跑门禁。

## 辩方判定一览

| # | 判决条目 | 辩方判定 | 证据（细节在后面 D 段） |
|---|---|---|---|
| 1 | 第一轮第三节第 1 条：攻方用例（经 `choose_root` 钉「txg 为主」）进 harness 档 | 站得住：落地够 | D1：用例进了 `crates/singlefs-harness/tests/`，G1 一行点名它；用例的断言对「摊反」必红（读码推，没跑） |
| 2 | 同条：变异行「胶水摊反择新键」「胶水拿所选槽之外的根调重放」，以及「改法落地」一节余下的 `mounted_read.rs`、`mount.rs` 两处 | 站得住：落地够；够不着一处（摊入次序与下标没有变异行，判决没要） | D1：五行锚点各命中 1 次，替换文读码类型对得上，各有点名用例 |
| 3 | 第一轮第三节第 2 条：`newest_root_among` 改成与核同一择法（键相同取前一个） | 旧写法（取后一个）没有站得住的理由；改法站得住，没改过头 | D2：全部调用点按读根环次序给入参；合法历史里的同键根（mkfs 种子三条）内容逐字节相同，改前改后同值 |
| 4 | 同条的同形别臂：checker 择根 | 同样打中别的臂：判决只改了一臂 | D3：`walk.rs` 择根 `max_by_key` 同键取后一个，注释自称与 `choose_root` 同一个读法；合法历史里不分叉（同键同内容），同键异内容态没造出，推的 |
| 5 | 改法落地：合判推论不加「去掉 J1 前提」一行，J1 由 M2 推得出 | 站得住（旧写法 = 不加）；Verus 没跑，探针数核不动 | D4：引理 `lemma_replay_core_result_lands_no_earlier_than_the_chosen_root` 只凭 M2 证 J1；J1 同时是推论结论的一项；别处另有必红行罩 J1 的 ensures |
| 6 | C651：checker ③ 取 4096 → 逐档（checker / 实现 / GPU 三份） | 旧写法的理由由判决第一节 C 组数推翻，我不改；新代码三份逐档同构，站得住 | D5：候选范围、档距、核偏移、取世代号最大、平局取偏移小、遇不收之值先交，三处逐项对上（读码） |
| 7 | 同条的新暴露面：每区槽数 S 越界 | 够不着 + 同样打中别的臂：逐档里 checker 记「不可择」继续试，实现整池拒；新用例没有一条用 S 越界 | D6：槽 0、槽 1 上同一分歧早就在，逐档把它从 2 个偏移放大到 2033 个；合法历史里到不了（推的） |
| 8 | 11-11 推的「现有用例分不开 ② 借槽距与 ③ 逐档」 | 站得住，只在 checker 一臂；恢复一臂已有用例分开；GPU 一臂没有变异行 | D7：恢复侧用例在借来的 4096 之外放世代号 5 的旧槽，checker 侧用例没有 |
| 9 | W1：mkfs 每盘先把 [0, 根环基址) 整段写 0 | 站得住（读码）：发在最前、与系统配置两槽的写隔着原有那道屏障；层 0 钉值与崩溃放量没跑 | D8 |
| 10 | K4：E165 r3 装置新族对变异表的牵连 | 打中一处：冻结副本里第 1812 行锚点命中 2 次（第一轮 48 行都命中 1 次）；仓里工作区的那一行后来已改 | D9：`static-check` 原样输出 |
| 11 | K4：E165 r3 两段装置的新族与拆行没有让旧档输出变（执行员报 profile-check 逐字节同） | 没验：要编译与跑装置，是 checker 档 | 只读到新族只在 `--profile r3 --family root_choice` 时跑（文件头注释），没看到旧档路径被改的证据，也没看到被保住的证据 |

## D1　K1-b 落地够不够（判定 1、2）

- 点名的五行（冻结副本 `crates/mutations.tsv`）：「择根胶水把择新键两项对调（代码轮 m3-formal-code-r1 K1-b，攻方 G1）」（`crates/mutations.tsv:1868`）、「F 生效值择最新根平局取后者（代码轮 m3-formal-code-r1 K1-d）」（`crates/mutations.tsv:1870`）、「只读挂载胶水拿所选槽之外的根调重放（代码轮 m3-formal-code-r1 K1-b」（`crates/mutations.tsv:1966`）、「可写挂载读阶段胶水拿所选槽之外的根调重放（代码轮 m3-formal-code-r1 K1-b」（`crates/mutations.tsv:1967`）；第 1869 行是 `recover` 那一处（同批，名字以「恢复胶水拿所选槽之外的根调重放」起头）。
- 锚点：用 python 把原文的 `\n` 换成换行、在各自文件里数命中，再跑 `crates-mutation-rows.py static-check`（原样输出在 D9）：第 1868–1873、1966、1967 行都没有出现在它的 ANCHOR 红单里，即各命中恰好 1 次。
- 替换文能不能编（红是不是因为编不过）：读码核。1966 的替换文是 `crate::recovery::readable_roots_with_ring_slots(reader, …)[0].1`，`mounted_read.rs` 里 `reader` 是 `&dyn PoolReader`；1967 的 `cache` 是 `&ReadStageCache<'_, Reader>`，同一个函数里 `choose_root(cache, …)` 已把它当 `&dyn PoolReader` 传，所以 `readable_roots_with_ring_slots`（`Reader: PoolReader + ?Sized`）接得住。1871–1873 的替换文（`[0_u8; 16]`、`NodePointer::empty_root()`、`record.new_tree_table`）在 `recovery.rs` 同一函数里都在作用域内（`record` 在匹配臂里取出，`new_tree_table` 是 `NodePointer`）。没编，推的；`.claude/rules/mutation-sampling.md` 防的「别的原因红了被记成抓到」这一条留给门禁基线判。
- 用例对变异必红（读码）：`root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance.rs` 里 `mount_read_only` 与 `mount_writable` 两条各把新根写进读根环次序里排在后面的槽，断言 `chosen_root` 是新根、施加之后的根也是它；变异让重放从读根环次序里第一条自证过的根起走，水位之上没有记录时落点就是那一条，两条断言之一必破。
- 够不着的一处：第一轮腿报告里「胶水把摊入次序倒过来、下标取错一格」没有变异行（`every_root_ring_slot(&immutable.sizes)` 在变异表 0 命中）。判决第三节没把它列进改法，所以不算落地不够；它今天靠 D2 里那条同键用例间接罩着（倒序时 `choose_root` 会择到后面那条，该用例断言择到前面那条 `mkfs_root`），但没有一行变异点名。
- 推翻 1、2：门禁基线里五条点名用例有一条不是 1 passed，或改坏后有一条没红。

## D2　K1-d：旧写法（取后一个）没有站得住的理由，改法没改过头（判定 3）

替旧写法找过的理由与结果：

1. 「`max_by_key` 取后一个是习惯写法，不用改」：不成立。旧文档自己写「与 [`choose_root`] 同一个择法」，而 `choose_root` 键相同取最前（「最大的不止一个槽时取读根环次序里最前的那个。」（`crates/singlefs-core/src/recovery.rs:1251`））。两处各说各的，判决说改一边，改哪边判决留给实现员。
2. 「改文档、不改行为更稳（行为不动就不会错）」：行为差只出现在键完全相同的两条根之间。合法历史里同键根可达，而且每池都有：mkfs 把同一个 `root_slot` 的字节写进三个区域的槽 0（「for ring_slot in genesis_root_ring_slots() {」（`crates/singlefs-core/src/make_filesystem.rs:619`）），键相同、内容逐字节相同，取前取后交回的 `RootRecord` 相同。所以改行为在合法历史上同值，不会把任何合法态判错；同键异内容的两条根没造出来（合法历史里发布的 (txg, 实例代号) 不重，推的）。取舍站在「两处各说各的」这一边：改成同一个择法后，F 生效值判有效根读的表与恢复择的那条根指着的表在同键异内容态上是同一张（可写挂载那一路：「最新那条根（`choose_root` 现择）指着的实例表」（`crates/singlefs-core/src/mount.rs:1225`）），改文档则要把这个分叉写成条款。
3. 「调用点的入参次序不一定是读根环次序，改成取最前反而取错」：不成立。`newest_root_among` 的入参来自 `readable_roots_with_ring_slots`、`readable_roots_with_ring_slots_rereading_unreadable_slots_once` 与 `..._rereading_ring_slots_known_to_hold_a_root_once`，三者都按 `every_root_ring_slot` 的次序逐槽读、逐槽 `push`（重读成功的槽也在原位置 `push`），所以入参就是读根环次序；新写法把这一前提写进文档：「入参须按读根环次序排（`every_root_ring_slot`：读根环的每一处都按这个次序走）。」（`crates/singlefs-core/src/recovery.rs:1804`）。

改法本身：「.reduce(|newest_so_far, candidate| {」（`crates/singlefs-core/src/recovery.rs:1809`）接「> (newest_so_far.checkpoint_txg, newest_so_far.instance)」（`crates/singlefs-core/src/recovery.rs:1811`），严格大于才换，与 `choose_root_slot` 的平局取法一致；同一文件里 F 生效值的每盘择最新也是严格大于才换（`.or_insert(*root)` 后跟 `>` 比较，「.or_insert(*root);」（`crates/singlefs-core/src/recovery.rs:1841`）），所以 F 这一族现在三处同一个平局取法。没改过头：只动平局那一支。

罩得住吗：「F 生效值择最新根平局取后者（代码轮 m3-formal-code-r1 K1-d）」（`crates/mutations.tsv:1870`）把 `>` 换成 `>=`，点名的用例（`of_two_roots_with_the_same_choice_key_the_effective_floor_reads_the_instance_table_of_the_earlier_slot_as_choose_root_does`，`crates/singlefs-harness/tests/root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance.rs:394`）让后面那条根的实例表位置校验和被取反，取后者就读不出表、重读仍读不出而报错，断言「择新键完全相同取读根环次序里前面那条（D22 已定项 24、核的平局取法）」（`crates/singlefs-harness/tests/root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance.rs:436`）旁的 `effective_floor == Ok(…)` 必破。

推翻「旧写法没有理由」：找到一个调用点把入参排成别的次序（例如按键排序后再交），或一个条款写明同键取后一个。推翻「没改过头」：找到一个合法历史上 `newest_root_among` 改前改后交回不同根。

## D3　同形别臂：checker 择根取后一个，注释自称与 `choose_root` 同一个读法（判定 4）

- 「择根：按 (txg, 实例) 取最大（D22（单元原子性怎么合成） 已定项 7，与实现 `recovery::choose_root` 同一个读法）。」（`crates/singlefs-checker/src/walk.rs:6711`）下一句是「.max_by_key(|(_, (_, _, root))| (root.checkpoint_txg, root.instance))」（`crates/singlefs-checker/src/walk.rs:6715`）：`Iterator::max_by_key` 键相同交回最后一个（第一轮攻方量过：「max_by_key_picks=slot2」（`research/prompts/m3-formal-code-r1-opus-model/outputs/maxby.out:2`））。实现 `choose_root` 键相同取最前，所以同键异内容的两条根在池里同在时，checker 走读的「最新根」与实现恢复所选的不是同一条。这与第一轮 K1-d 是同一个形状（同键取后一个，文档说同一个读法），判决只改了 `newest_root_among` 一臂。
- 可达性与它的限度：同键的两条根在合法历史里同值（D2 第 2 点）；我没有造出同键异内容态。我也只 `grep` 了 `judge_root_ring_health` 一带有没有「同键两条根」的判据，没找到，没读全，所以「checker 另判同键重复」这一条核不动。判决 K1-d 写的「条款没写平局取哪个，写回 D22 已定项 24」若写成「取读根环次序里最前」，checker 这一处要同改，否则条款与 O2 一条路径（`.claude/kb/decisions/13-验证路线.md` 已定项 12「checker 即规范」与独立性那一对）不一致。
- 同类先例：第一轮 K1-d 本身；没找到别的先例。推翻 D3：checker 在同键异内容的根环上另有判据红掉它（那样取前取后都不影响整池结论），或条款写明 checker 取后一个。

## D4　合判推论不加「去掉 J1 前提」一行（判定 5）

替旧写法（不加）找到的理由，都指向「这一行写出来必不红」：

- J1 在这个推论里出现两次：作为前提，「replay_lands_no_earlier_than_the_chosen_root(」（`crates/singlefs-replay-prefix/src/lib.rs:3034`）；也作为结论 `recovery_replays_from_the_newest_self_verified_root` 七项合取里的一项（`crates/singlefs-replay-prefix/src/lib.rs:1119` 同一句）。结论里的这一项如果不能由别的前提推出，去掉前提就红；判决的探针说去掉后 56 verified、0 errors，原因是只凭 M2（`replay_follows_the_prefix_rules`）就推得出 J1。
- 读码上可支撑探针的一点：「proof fn lemma_replay_core_result_lands_no_earlier_than_the_chosen_root<」（`crates/singlefs-replay-prefix/src/lib.rs:2679`）的 `requires` 只有 M2 一条（`requires` 在第 2694 行、`replay_follows_the_prefix_rules(` 在第 2695 行、`ensures` 在第 2706 行，之间没有别的前提），体里只一处 `assert(above_the_chosen_root(…))`。它是「只给推论调」的引理（`lemma_…` 在第 2921、2932 行被 `corollary_the_landed_version_does_not_go_back` 调），不是 `corollary_recovery_replays_from_the_newest_self_verified_root`（第 2982 行）调的；后者体里没有调它，却照样证出结论，说明 J1 前提在后者里本来就不承重。这是读码推的，不是 Verus 的输出。
- 判决写「J1 由 M2 推得出」而不加行，没有放过一个罩得住的缺口：J1 的两条真正的出口各有必红行，「M3·ensures·replay_core 落点不早于所选根那一条取反」（`crates/singlefs-replay-prefix/verus-mutations.tsv:111`）与「M3·ensures·引理：施加了时落点改成早于所选根」（`crates/singlefs-replay-prefix/verus-mutations.tsv:141`）；合判推论自己的前提与结论各有行，「C650·前提·合判去掉择根那一条」（`crates/singlefs-replay-prefix/verus-mutations.tsv:160`）、「C650·前提·合判去掉 replay_core 的 M2 那一条」（`crates/singlefs-replay-prefix/verus-mutations.tsv:161`）、「C650·ensures·合判结论改成落点早于每一条自证过的根」（`crates/singlefs-replay-prefix/verus-mutations.tsv:162`）。
- 站得住的另一半是改法没引进新缺口：推论的注释还写「前提就是两个核交回的东西」（`crates/singlefs-replay-prefix/src/lib.rs:2978`），J1 本来就是 `replay_core` 的一条 ensures（第 3599–3600 行），与注释一致。
- 一句缺口（不改判，写给主 agent）：推论体里的注释「// 每一条自证过的根 ≤ 所选根（择根核）≤ 落点（J1）：择新序可传递。」（`crates/singlefs-replay-prefix/src/lib.rs:3072`）读起来像是在用 J1 前提，今天它由 M2 展开得到；`requires` 里留着多余的 J1 也不报错。这只影响读的人，不影响证明。
- 核不动：Verus 的 56 / 57 两个数是判决引的探针，我没跑（形式化验证是 checker 档，子 agent 不跑）。推翻 D4：在入库装置上把 J1 那一条从 `requires` 删掉后 `verus-proof-check.py` 报 `proof_errors` 大于 0（那就该有这一行）。

## D5　C651：checker ③ 取 4096 → 逐档，三份是不是同一个读法（判定 6）

旧写法（取 4096）的理由是 2026-09-14 用户收尾弹窗的甲，判决与用户已改成逐档，不由我改。我只查新代码三份（checker `image.rs`、恢复 `recovery.rs`、GPU 判器 `crash_judge_checker.wgsl`）逐项是不是同一个读法，逐项各取一行：

| 项 | checker | 恢复 | GPU |
|---|---|---|---|
| 候选范围与档距 | 「for candidate_spacing in (FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES」（`crates/singlefs-checker/src/image.rs:591`），上界「const LARGEST_SLOT_SPACING_THIS_READER_ACCEPTS: u64 =」（`crates/singlefs-checker/src/image.rs:569`） | 「for candidate_spacing in (FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES..=largest_spacing)」（`crates/singlefs-core/src/recovery.rs:977`），上界「region_start(0).0 - SYSTEM_CONFIGURATION_SLOT_BYTES」（`crates/singlefs-core/src/recovery.rs:720`） | 「for (var candidate = 4096u; candidate <= largest; candidate = candidate + 512u) {」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:215`），常量「const ROOT_RING_BASE_BYTES: u32 = 1048576u;」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:96`） |
| 核自述槽距等于所在偏移 | 「if geometry.slot_spacing != candidate_spacing {」（`crates/singlefs-checker/src/image.rs:597`） | 「if recorded_spacing != candidate_spacing {」（`crates/singlefs-core/src/recovery.rs:991`） | 「if (s.reading == 0u && s.spacing == candidate) {」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:220`） |
| 世代号严格更大才换（同号留偏移小的） | 「.is_none_or(|(accepted, _)| view.slot_generation > accepted.slot_generation)」（`crates/singlefs-checker/src/image.rs:602`） | 「system_configuration.quantities.slot_generation」（`crates/singlefs-core/src/recovery.rs:995`），下一行 `>` | 「if (!have_best || lt64(best.generation, s.generation)) {」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:221`） |
| 读到带不收之值的槽 | 「refused @ SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses { .. } => {」（`crates/singlefs-checker/src/image.rs:607`）直接交回 | 「match mountable_slot_or_refusal(」（`crates/singlefs-core/src/recovery.rs:980`）后的 `?` 整池拒 | 函数里 `if (s.reading == 1u) {` 直接交回（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:217`） |
| 只在全池没有可择的槽 0 时才走 | 「.or(spacing_recorded_by_the_first_valid_slot_zero)」（`crates/singlefs-checker/src/image.rs:652`） | 「.or(slot_spacing_recorded_by_the_first_mountable_slot_zero)」（`crates/singlefs-core/src/recovery.rs:1089`） | 「var spacing = first_spacing;」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:250`） |

三份在这五项上同构（读码）。判决第二节 V1 的 C 组数是副本装置上量的，进 kb 之前要在入库装置上重做（判决第五节第 3 条），不归我。推翻 D5：同一份坏镜像上三份交回不同的槽。

## D6　逐档把「S 越界」的分歧从 2 个偏移放大到 2033 个（判定 7）

- 恢复：S 越界整池拒，逐档里同一个判法：「Err(SystemConfigurationSlotRefusal::RootRingSlotsPerRegionOutOfRange(out_of_range)) => {」（`crates/singlefs-core/src/recovery.rs:936`）；逐档函数文档「整池拒，与槽 0、槽 1 同一个判法。」（`crates/singlefs-core/src/recovery.rs:966`）。
- checker 与 GPU：S 越界是「这一槽不可择」，不整池拒：「R、S 越界照旧是这一槽不可择。槽距先判：它是根槽宽的上界。」（`crates/singlefs-checker/src/image.rs:304`）；GPU 头注释「// 3 自证过而不可择（incompat 位不认得、R 或 S 越界；I-8.5 把它数进自证过的槽）」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:11`），判在「if (slots_per_region < 4u || slots_per_region > 16u) {」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:100`）。逐档里 checker 与 GPU 遇到它只记一笔「见过自证过而不可择」、接着试下一档（「SystemConfigurationSlotReading::SelfVerifiedButNotChoosable => {」（`crates/singlefs-checker/src/image.rs:610`）那一臂只置标志）。
- 后果（读码推）：候选偏移上有一份自证过、S 越界的槽时，恢复在那一档当场整池拒，checker 与 GPU 继续试后面的档，可能收到可用槽而不报红。槽 0、槽 1 两个偏移上这一对句子早就是这样（所以不是这一批引进的分歧），逐档把暴露的偏移从 2 个放到 2033 个，而且恢复里「先拒、后核自述槽距等于所在偏移」，一份自述槽距不等于所在偏移的槽也会触发拒。
- 新用例没有这一格：新用例文件里造「带不收之值」的槽用的是 journal 环起点字段，「SlotNumber(JOURNAL_RING_START_SLOT_THE_READER_REFUSES);」（`crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs:325`），造「不可择」用的是 incompat 位，「INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT;」（`crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs:374`）；S 越界在逐档里只有恢复一侧的旧用例（`the_search_keeps_the_incompat_and_slots_per_region_refusals_of_slot_one`，`crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs:689`），checker 一侧没有。
- 够不够：合法崩溃历史里到不了这一格（W1 之后 mkfs 把 [0, 根环基址) 清成 0，自证过的槽只剩本池两槽；S 越界又要整槽校验和过，只能是盘上字节被人造出来，推的），所以不是「会把合法历史判错」的洞，是「两个读者在人造坏盘上不同结局、用例没钉、条款没写」。判决第四节第 1 条要写进 D22 已定项 16 的那一句只写了「带不收之值」，没提 S。
- 同类先例：槽 0、槽 1 上同一对句子（上面 `recovery.rs:936` 与 `image.rs:304`）；它们的 kb 出处我没去查。
- 推翻 D6：造一份自证过、S 越界的槽放在逐档候选偏移上，两边交回同一个结局（那就是我读漏了某一处统一）；或条款写明 checker 一侧 S 越界在逐档里也整池拒。

## D7　11-11 推的「现有用例分不开 ② 借槽距与 ③ 逐档」（判定 8）

- ② 的两处：「.or(spacing_recorded_by_the_first_valid_slot_zero)」（`crates/singlefs-checker/src/image.rs:652`）、「.or(slot_spacing_recorded_by_the_first_mountable_slot_zero)」（`crates/singlefs-core/src/recovery.rs:1089`）。
- 恢复一臂已分开：用例 `a_device_whose_slot_zero_fails_borrows_the_spacing_another_slot_zero_records_instead_of_searching`（`crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs:592`）在盘 0 的 8192 处放世代号 5 的旧槽，「stale.quantities.slot_generation = 5;」（`crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs:597`），断言「盘 0 按盘 1 槽 0 记的 4096 找到本池的槽 1，不去试 8192」（`crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs:612`）。把 ② 删掉就落到逐档、择中世代号 5 的旧槽，断言红；`crates/mutations.tsv` 第 917 行点名的就是它。所以 11-11 的观察在恢复一臂不成立。
- checker 一臂分不开：第 979 行点名的用例只在盘 1 记 8192、盘 0 槽 1 在 8192，断言「盘 0 的槽 1 在盘 1 记的槽距 8192 处找到；盘 1 两槽都自证过」（`crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs:1263`），没有第二个自证过的槽；删 ② 后逐档在 8192 档读到同一槽，同值（读码推，没跑）。11-11 的「第三类」判断在这一臂成立。
- GPU 一臂没有变异行：在 `crates/` 下 `grep -c 'checker_slot_one_by_reading_every_spacing\|first_spacing' mutations.tsv` 输出 `0`。
- 零轮的补法（我自己提的，没被攻过）：checker 一侧加恢复那条用例的孪生（盘 1 槽 0 记 4096、盘 0 槽 0 坏、盘 0 的 8192 处放世代号更大的旧槽，断言 checker 择到 4096 处本池的槽 1）；GPU 一臂要「槽 0 无效加旧槽」的坏盘输入才分得开，今天层 0 的 io_min 8192 流没有这种输入。
- 推翻 D7：找到一条现有 checker 一侧用例在删 ② 后红。

## D8　W1：mkfs 清 [0, 根环基址) 的位置（判定 9）

- 「let root_ring_base = region_start(0);」（`crates/singlefs-core/src/make_filesystem.rs:555`）、「device.write_zeroes_at(DeviceOffsetInBytes(0), root_ring_base.0)?;」（`crates/singlefs-core/src/make_filesystem.rs:557`）：每盘一次调用，发在根环三区与 journal 环清零之前。
- 系统配置两槽的 Plain 写在后面：「for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {」（`crates/singlefs-core/src/make_filesystem.rs:651`）、「device.write_at(offset, &slot, WriteDurability::Plain)?;」（`crates/singlefs-core/src/make_filesystem.rs:655`）；清零与它们之间有第 576 行的屏障（「device.barrier()?;」（`crates/singlefs-core/src/make_filesystem.rs:576`），在清零与两个单元写之后、根 FUA 写之前），第 659 行是收尾屏障。所以清零不会在崩溃子集里反过来盖掉槽，与 11-5 注释写的「这两对都由原有的那道屏障……隔开」一致（读码）。
- 没验：层 0 里钉 mkfs 段步数的值（每盘多一步）、崩溃放量钉值：是 checker 档，没跑；实现员推「从 mkfs 之后起枚举、不变」，我读到的注释「mkfs 中途崩溃第一版没有条款」（`crates/singlefs-core/src/make_filesystem.rs:541`）与它同向，不是证据。
- 推翻 D8：层 0 的 mkfs 段或崩溃放量钉值在 mkfs 多一步之后变了。

## D9　K4：E165 r3 装置新增函数让第 1812 行变异锚点命中 2 次（判定 10）

对冻结副本跑 `python3 research/scripts/crates-mutation-rows.py static-check /tmp/claude-1000/m3-formal-code-r2/tree`（退出码 1，输出 19 行，存 `/tmp/claude-1000/m3-formal-code-r2-defense/static-check.out`，不入库：它是对冻结副本的静态判，复跑几秒）。与这一轮有关的一行原样：

```

```

- 命中的两处：「fn system_configuration_for_the_scan(ring: RingGeometry) -> SystemConfiguration {」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:1189`）里的第 1192 行，与 r3 第二段新加的「fn system_configuration_for_the_root_choice(domain: RootChoiceDomain) -> SystemConfiguration {」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:9507`）里的第 9510 行，两处都是「filesystem_identifier: ROOT_FILESYSTEM_IDENTIFIER,」后接「this_device: DeviceIdentity(0),」。第一轮冻结副本同一文件里这句我数到 1 次，所以是这一轮 L5 引进的。
- 后果：门禁 checker-tier-crates-mutation-replay 派活前与门禁 code-source-discipline 的 mutation-tables 格都按「原文恰好命中一次」判，红。
- 同一份输出里别的 ANCHOR 行不归这一轮：1369 指的 litmus 文件不在 `crates/` 里，1829–1830、1839–1842、1897–1898 是 `e168_delete_ring_window_borrow.rs` 的行（别的会话）。
- 仓里工作区的第 1812 行后来已改：原文换成带 `fn system_configuration_for_the_scan(...)` 函数头的写法，我用 python 在仓里数它命中 1 次（行 1189）；冻结副本里还是旧的两处命中。我没碰它。
- 我这一轮的限度：r3 两段新族的变异行（第 1831–1838、1899–1909 行）各命中 1 次是 static-check 没列红的推出来的，红不红归 checker 档。
- 推翻 D9：对冻结副本重跑 `static-check`，第 1812 行不在红单里。

## 要主 agent 跑的命令

没有必须的。可选：D4 的 Verus 探针（`requires` 只写 M2、`ensures` 写 J1 的空体 proof fn）在入库装置上经 `verus-proof-check.py` 重做一次，核判决引的 56 / 57 两个数。

## 没做什么

- 没编译、没跑任何测试（harness 档与 checker 档都没跑）、没跑 Verus、没跑 E165 / E170 装置、没跑门禁；变异「红不红」全是读码推，替换文能不能编也是读码。
- 没读 `research/prompts/m3-formal-code-r2-opus-output.md` 与它的模型目录（禁读清单）。
- 没判 K1 的 RefFS 包（L1）、K2 的挂载透明（L2）、K3 里层 0 与崩溃放量钉值、GPU 判器 wgsl 里别处的逐项对拍；这一轮我的格是第一轮改法落地与被改掉的旧写法。
- 没替主 agent 撤销或维持判决，没改用户定案（F1–F6、G1、G2、H1、W1、V1 改逐档）。D3、D6、D7 里的补法都是我提的零轮，没被攻过。
- 草稿目录 `/tmp/claude-1000/m3-formal-code-r2-defense/` 只有 `static-check.out` 与分段草稿 `seg3.md`、`seg4.md`；没建编译目录、没拷仓副本。
