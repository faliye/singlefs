# 根环实现第 10o–10s 批：代码轮第一轮判决（2026-10-06）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 T1 T2 S1 S2 Q6 -->

正文 `research/prompts/_m3-batch10o-code-r1-body.md`。两腿报告：
- 正推：`research/prompts/m3-batch10o-code-r1-sonnet-output.md`，sha256 69fe0c27…6c9686，353 行；
- 攻方：`research/prompts/m3-batch10o-code-r1-opus-output.md`，sha256 f3b36bc5…a58830df，287 行。

## 一、核查

| 项 | 结论 |
|---|---|
| 开工快照 | 冻结副本 `~/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/` 对 `research/prompts/m3-batch10o-code-r1-snapshot/crates-sha256.txt` 316 份 `sha256sum -c` 全过。kb 快照里有两份被别的会话改了：`.claude/kb/invariants.md` 改的是 I-9.2、I-9.17、I-9.23 三行，`.claude/kb/checks-owed.md` 改的是 C739 那一行。腿引的 kb 行由 cite-check 对着冻结副本里同一份 kb 核，那四行都不在这一轮的格里 |
| 引文 | `cite-check.py` 两份报告合跑，`--root` 冻结副本、`--background` 背景材料：`✓ 核了 69 处引文，对上 68 处，没判 1 处`。没判那一处是正推第 254 行引的 `gpu-cards.env.example:17`，主 agent 现查冻结副本第 17 行，原样为 `GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED='local:0:24G peer:0:16G'`，与腿的引用对得上 |
| 腿在副本上量的数 | 都是副本上的数，不进 kb；判决只拿来分辨候选、交用户定 T2 |
| 主 agent 现核 | K2-1 读码坐实（第二节 K2 那一格）；K5 补核亲自做了（第二节 K5 那一格）；K4 的几格照两腿报告的代码行对过冻结副本 |

这一轮被判的源文件：
- `crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/mount.rs`（K1）；
- `crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker/src/image.rs`（K2）；
- `crates/singlefs-checker-tier/src/crash_injection.rs`、`crates/singlefs-checker-tier/src/crash_judge_gpu.rs`、`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl`（K2）；
- `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs`（K3、K4）；
- `crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-checker-tier/src/gpu_card_configuration.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs`、`crates/singlefs-checker-tier/src/lib.rs`、`crates/singlefs-harness/src/gpu_memory_ledger.rs`、`crates/singlefs-harness/src/lib.rs`（K4）。

## 二、逐格判决

| 格 | 两腿 | 判 |
|---|---|---|
| K1-1 T1 甲三形 | 正推：1818 段红 0；拿掉「顶上」那一步的对照 44/54 红，说明探针红得出来。攻方：K1 组 2448 段，其中 1860 段真顶上了，连 T2 两臂共 6804 段 0 红；10i 打中的两形都关掉了 | **没打中，两条腿一致**。两条腿各抽样一次，照规则记「没打中」 |
| K1-2 写掉那一槽就换类 | 正推：根环表换槽的入口只有 `record_root_written_by_this_process` 一处，留下旧记录的只有发布失败退回那一支（D16 已定）。攻方：读码，没造写故障 | 没打中（读码）。写故障那一面零轮，记进下一轮攻击面 |
| K1-3 T2「回退不拒」那一臂 | 两条腿都量了，方向相同。正推：推 3 步时今天的臂 522 段里拒 462，不拒臂拒 0；推 9 步 282 对 0；两臂判错都是 0、checker 红都是 0。攻方：今天的臂 2034 段拒 1122，不拒臂 2322 段 0 拒 0 红 | **交用户**（岔路单 `research/prompts/m3-batch10o-code-r1-forks.md` T2 那一行）。数是副本上的，用户若定「不拒」，实现批在入库装置上重做这组历史、逐字段坐实之后才进 kb |
| K1-4 留洞的环、H-b 下的 I-5.1 | 留洞的环：正推 372 段红 0，攻方 180 段顶上过、0 红。I-5.1：两条腿都判是那张坏镜像自己就该红 | 没打中；主 agent 之前那个推的判断，两条腿都认 |
| K1 线索：错位读 | 攻方：错位读时，新类会记下一条不住在这一槽的根。零轮改法是记录时核落点公式 | 这一形在故障模型之外，记欠账，不进这一批 |
| K2-1 I-3.14 四形同口径 | 正推：一致。攻方：打中口径差，推的、读码 | **打中，主 agent 读码坐实**：<br>- 实现 `crates/singlefs-core/src/instance_table.rs:190`–`193` 文档写明，「有下一片而位置指针头部有一段不是第一版的取值」也是 `None`，第 223–228 行经 `NodePointer::read_with_head_fields_judgement_from` 判；<br>- checker `crates/singlefs-checker/src/walk.rs:740`–`741` 只判有无下一片与指针全不全零；<br>- `.claude/kb/invariants.md` I-3.14 行与 checker 同口径。<br>结果是这一形实现读不出整张表，checker 却照收、照走。正推漏了这一样。改法二选一交用户：岔路单 K2-1 那一行 |
| K2-2 / K2-4 I-3.8 的不适用理由 | 攻方：第 1 片起解不开时，I-3.8 照前几片的行判。正推：只对 I-3.14 四形点名了理由，第 0 片读不出、身份不对、容器记录不合法三种走缺省理由 | 规则没说，窄。记欠账，下一轮再定 |
| K2-3 H-b 键 | 正推：CPU 与 GPU 同选法。攻方：没攻 | 没打中（一条腿） |
| K3-1 7 个函数 | 正推：7 个逐点对过，一致。攻方：读了 2 个 | 没打中 |
| K3-2 对拍能不能说明 | 两条腿都说单靠对拍不够。正推列出三类状态对拍够不着：「版 + 下一次挂载那一版」、「候选 ≥ 2 条带已释放」、「早退边界」 | 读码与 34-1 那行变异补上了，判 10q 没改判定。三类状态的对拍流记进下一批（对拍文件归删除线，转它） |
| K3-3 调用点单测 | 两条腿都说：超预算一定红；不在七个函数里的那几个多调一处，只靠展开钉值拦 | 没打中，限度照实记 |
| K4-1 容量大于真能用的 | 攻方，量过：设了额度变量之后，配置给这张卡的额度与 3/4 那一份就不作数了，容量 5632 → 20000。正推：池外有三处设备不进账；驱动开销 2560 只在 5090 上量过 | **打中**。毛病出在主 agent 给 10s 定的 ④，只取了「额度变量」与「清场后空闲减驱动开销」两者的较小值，漏了配置给卡的额度。改成三者取最小 |
| K4-2 跨进程账 | 正推：flock、判活、SIGKILL 约 1 秒回收都对。攻方，量过：被杀而没收尸的僵尸进程判成活着；两个进程同时首次起池时，一份驱动开销算了两次，少借 1980 MiB | 小毛病两处，并进下一批修 |
| K4-3 两向 | 攻方，量过：<br>- 账本里的「清场后空闲」只要进程链不断就一直沿用。外人后来占了卡，账会超出真空闲 9940 MiB；起池时卡被占着，卡空了以后反而判红或等满。<br>正推，推的：<br>- 后起的进程用实时空闲小于 2048 MiB 滤卡，订满时会把卡判成不可用；<br>- 没有排队次序；<br>- 先建设备、后登记，中间有窗口 | **打中，两个方向都违反用户原话**：「只防 OOM」与「不压并发」。主 agent 给 10s 定的 ⑤ 里「基线沿用」是零轮设计，这一轮被攻破。并进下一批修 |
| K4-4 借的量与扫源码 | 正推：借的量逐项对得上；扫源码检查可以被别名、字符串里的 `//`、非顶格 cfg 绕过。攻方：`crash_amplification_pipeline_records_checks_and_reuses.rs` 有一条只排计划的用例，经产品路径另起不记账的设备 | **打中**，并进下一批 |
| K5 #9 钉值 | 正推复核两半都对。攻方说进度文件那个值没有对应的核 | 主 agent 补核：<br>- 主工作区新进度文件 47 行，逐行 CRC-32C 0 处不符，整份 `1d92a7ad…`；<br>- 46 行片行各删一项 `I-3.14:1`，按 `line_with_checksum`（`crates/singlefs-checker-tier/src/layer0_progress.rs:480`）重算每行校验和，整份 SHA-256 等于旧钉值 `11977571…`。<br>攻方写的推翻条件成立，**K5 站住** |

## 三、下一批（根环线第 10t 批）

聚到同一簇：
1. K4-1：额度变量、配置给卡的额度、清场后空闲减驱动开销，三者取最小。
2. K4-2：僵尸进程判成死；同时首次起池时驱动开销只记一份。
3. K4-3：
   - 每次起池都现读空闲。账的容量按「现读空闲 + 活进程已借、还没实际占的那部分」算，不沿用旧基线；
   - 后起的进程不再拿实时空闲滤卡，改按账判；
   - 先登记、后建设备，堵上中间那个窗口。
   - 出口：攻方量到的两个方向都由用例钉住，红在点名的格。
4. K4-4：pipeline 那一条改走卡池；扫源码检查堵住别名、字符串里的 `//`、非顶格 cfg 三种绕法。
5. K2-1：照用户定的那一侧改 checker 与 GPU 读法，或者改 kb。walk.rs 与走读内核归删除线第 32 批的范围，动手前先和删除线对文件。

记欠账（下一份 kb 写回）：
- K2-2 / K2-4：I-3.8 的不适用理由；
- K1 那条错位读线索；
- K3-2 三类状态的对拍流，转删除线；
- K1-2 写故障那一面零轮。

## 四、交用户

见岔路单 `research/prompts/m3-batch10o-code-r1-forks.md`：T2、K2-1 两行。

## 五、用户定案

用户 2026-10-06 弹窗定（岔路单 `research/prompts/m3-batch10o-code-r1-forks.md`）：

| 岔路 | 定的 | 去向 |
|---|---|---|
| T2 回退判候选、算 F_生效 时这一类槽读坏重读仍坏 | 照今天拒 | 回退那一处不改；D16 已定项 1 第 37 行「「不拒」那一臂没量」改成量过、用户定照拒（下一份 kb 写回） |
| K2-1 链指针记录有下一片而位置指针头部不是第一版取值 | 甲：算 I-3.14 第五种形态，这一片解不开 | checker 主走读、只读读法、引用集合那一遍、GPU 走读 / 扫描 / 恢复内核照实现改，补坏镜像与对拍流、变异（下一批，walk.rs 与内核先与删除线对文件）；kb I-3.14 行与 D18 已定项 11 补这一形（下一份 kb 写回） |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 备料 | 2026-10-06 改了：用户定 T2 照今天拒，第 37 行「「不拒」那一臂没量」改成量过、处置不变（写回报告 `~/.local/share/singlefs/work/kbwb-10o-r1/report.md`） |
| D18（块里携带什么信息） 已定项 11 | 备料 | 2026-10-07 改了：用户定 K2-1 甲，「最后一条不是合法的链指针记录」补第五形（写回报告 `~/.local/share/singlefs/work/kbwb-i314-fifth/report.md`），实现随第 10u 批合入 |
