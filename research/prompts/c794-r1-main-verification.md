# c794-r1 判决：C794（瞬时隔离的合法状态池级 checker 判 I-3.11 红）第一轮（2026-10-15）

被判的对象：`.claude/kb/checks-owed.md` C794 那一行的处置；涉及 `crates/singlefs-checker/src/walk.rs` 的 `quarantined_slots_exempted_per_device`、`crates/singlefs-checker/invariants/05-空间记账.md` I-3.11 与 I-3.1、`.claude/kb/decisions/19-块指针的结构与宽度预算.md` 已定项 5。这一轮不改 `crates/`。

<!-- doc-lint:not-numbers H1 H2 H3 H4 H5 -->

## 一、证据核对（主 agent）

| 证据 | 核法 | 结论 |
|---|---|---|
| 开工快照 | 冻结副本里 `sha256sum -c research/prompts/c794-r1-snapshot/crates-sha256.txt`；主工作区 `kb-sha256.txt` | 冻结副本 95 份全对上。kb 里 `.claude/kb/checks-owed.md` 对不上：主 agent 开轮之后改了 C600 那一行的「仍欠」（五·6 实测）；把那一处倒回去（会话草稿 `bak-2026-10-14/checks-owed.before-C600.md`）sha256 6e4466e1… 与快照同。两条腿都没引这份文件的行 |
| 攻方 `research/prompts/c794-r1-opus-output.md`（sha256 0ea04065…）、正推 `research/prompts/c794-r1-sonnet-output.md`（sha256 60a27493…） | `cite-check.py <两份> --root <仓根> --background research/prompts/_c794-r1-background.md` | 「✓ 核了 71 处引文、32 处裸行号，对上 71 处，没判 32 处（其中裸行号没有引文可比 26 处）」 |
| 两腿模型 | 拷进 `~/.local/share/singlefs/work/c794-r1-main-rerun/`，各经 `run-with-memory-cap.sh 24G` 跑 `tools/rerun.sh` | 正推「name=compare test=c794_transient_quarantine_and_forgotten_release_leave_the_same_bytes lines=4 same_as_model_out=true」「name=compare test=c794_after_the_root_ring_overwrites_the_root_of_the_quarantined_unit lines=96 same_as_model_out=true」「name=rerun_done mismatches=0」；攻方「RERUN-CHECK name=grid lines=260 identical=true」「RERUN-CHECK name=enum-control lines=1 identical=true」 |
| 已有钉子 | 主工作区现读 `crates/singlefs-harness/tests/common_images_for_the_checker_tier/release_checksum_quarantine.rs` 第 154–161 行 | 「那两份改回对得上」那份镜像期望 `.and_violated("I-3.1").and_violated("I-3.11")`：用户 2026-09-25「核出坏的才豁免」的那一面被有意钉成期望 |
| 攻方自报违规 | 报告「没做的和不合规的」 | librocksdb-sys 在 16G 包装下两次 oom 后，从主工作区 `target/release` 拷现成产物进自己的编译目录、裸跑一次 `cargo build -j 1` 查原因（定义「每条编译与测试命令经内存包装」那一条）。没影响产物：复跑在 24G 包装下逐行相同 |

## 二、逐格判

| 格 | 正推 | 攻方 | 判 |
|---|---|---|---|
| H1 现象与可达性 | 站得住：经 `publish_overwrite`、`raise_rollback_floor` 加手写故障设备造出，I-3.11 从那次发布起红，根出环或抬 F 之后 I-3.1 也红（量到 txg 44 还红）；随机故障注入每个注入点只坏一次调用，摆不出这一形 | 打中（量过）：经会话公开入口覆盖写，故障层只让被换下那一份两次读报错、盘上一字不动，I-3.11 当场红，环转过之后 I-3.1 也红（两条共用同一个豁免函数）；今天的取样撞不上（推的）；回退、清扫两条路推的同形 | 一致：现象成立、经公开入口加故障设备可达；今天的崩溃点重放与故障注入取样撞不上。C794 那一行与正文只写了 I-3.11，I-3.1 同红，登记行要补 |
| H2/H3 候选 | 保持现状：站得住，不丢检出，代价是碰过瞬时隔离的池永久红；盘上隔离位：站得住，只丢「写者谎报隔离」一种，动用户定案；一律豁免：站不住（漏释放镜像上一条不变量都不红）；曾被引用旁证：站不住（窗口内漏释放绿、环转过后瞬时隔离红回来） | 一律豁免：写者漏释放、假隔离两形 8 个时刻整份镜像 violated=[]；盘上标记：假隔离 violated=[]；「根环里还有根指着就豁免」：只绿到环转过为止；Z10 四个读法下都红（I-3.9 也红）；**H3 打中「判别子观测不到」（量过）**：瞬时隔离与写者漏释放两份镜像 8 个时刻逐扇区相同 | 两腿独立量到同一个事实：今天的格式下两份镜像逐字节相同（正推在数据单元一站、攻方 8 个时刻），只读盘的 checker 对它们只能给同一个判定。这一格打中所有不改格式的读法，按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错」的「打中不分辨臂」：不拿它在不改格式的候选之间判高下；在「改不改格式」这一层它分辨得出。一律豁免、曾被引用旁证两条各有真泄漏镜像使之绿，出局。留下保持现状与盘上隔离位两条 |
| H4 代价 | 保持现状：`crates/` 0 处，改 2 处文字（I-3.11 / I-3.1 措辞与 C794 登记行验收）；盘上隔离位：值宽不变，借分配记录跨度段次高位，读写两边掩码都要改（`allocator.rs:93`、`walk.rs:4505` 加 GPU 解码），`is_released` 触点上界 core 6 文件 41 处、checker 16 处、checker 档源 4 文件 35 处、harness 源 5 文件 18 处（推的上界，没动手改） | 没判 | 代价数采正推的（上界，推的）；进岔路单 |
| H5 会红的用例 | 盘上隔离位：样本是手写故障设备造的瞬时隔离镜像（必须由红变绿）与位清掉的同一份镜像（必须仍红，换成一律豁免变绿）；保持现状下 C794 现行验收「不许判 I-3.11 红」不可能满足，要改写 | 打中判据不够严：H5 没点名泄漏种类，拿 Z10 当「真泄漏镜像」时一律豁免过得了 | 采纳攻方：H5 的真泄漏样本要逐种点名（漏释放、悬空单元、活单元记成已释放三种各一份），任一处置下三种都要红 |

单次抽样：两条腿各抽一次。两条留下的候选各自没被打中，只抽了一次样，不当「没打中」用；它们站得住靠的是正推的正向推导与两腿各自量到的逐字节相同，交用户时写明。

## 三、处置

不收敛：保持现状与盘上隔离位都站得住，按跑前条款第三、四行写岔路单交用户（`research/prompts/c794-r1-forks.md`）；盘上隔离位动用户 2026-09-25「核出坏的才豁免」，单列。不开第二轮之前先等用户定。

不论用户选哪一条，都要做：C794 登记行补「I-3.1 同红」，H5 验收改成三种真泄漏逐种点名。攻方自提三条（豁免退路里比单元头诞生代号与分配记录的分配代；H5 逐种点名；保持现状时把这一红写明认成「完好单元被隔离：瞬时读坏或写者缺陷，镜像上分不开」）都标零轮，其中第二条已采纳进验收，第一、三条随岔路单交用户。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D19（块指针的结构与宽度预算） 已定项 5 | 备料 | 2026-10-15 不受影响：现行硬规则 1 照旧；盘上隔离位一条若被选，要改的是它「记录留在已分配」那一句的盘上表示，交用户 |
| D5（快照 / 空间记账机制） 已定项 4 | 不影响 | 2026-10-15 不受影响：记账行形态不变；隔离位一条只动分配记录跨度段的一位 |
