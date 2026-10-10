# 根环实现第 10z 批第七、八、九块：代码轮第一轮判决（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 G2 H1 H2 R C P Q -->

正文 `research/prompts/_m3-batch10z-ghi-code-r1-body.md`。两腿报告：
- 正推：`research/prompts/m3-batch10z-ghi-code-r1-sonnet-output.md`，sha256 f6af2978…89bc，308 行；
- 攻方：`research/prompts/m3-batch10z-ghi-code-r1-opus-output.md`，sha256 fed2f247…46d0，237 行。

## 一、核查

| 项 | 结论 |
|---|---|
| 开工快照 | 冻结副本 `~/.local/share/singlefs/work/m3-batch10z-ghi-code-r1/tree/` 对 `research/prompts/m3-batch10z-ghi-code-r1-snapshot/crates-sha256.txt` `sha256sum -c` 全过；kb 对 `kb-sha256.txt` 全过 |
| 引文 | `cite-check.py` 两份报告合跑，`--root` 冻结副本与仓根：`✓ 核了 68 处引文，对上 68 处，没判 0 处` |
| 复跑 | 草稿目录 `~/.local/share/singlefs/work/m3-batch10z-ghi-code-r1-main/`。攻方模型目录的 `rerun.sh` 本身跑不起来：`env "$@" cap …` 里 `cap` 是 shell 函数，每件退 127（`attack-rerun.log`「退出码不为 0 的件：26」）；主 agent 拷一份改成直接经内存包装（`rerun-attack-fixed.sh`），重跑「退出码不为 0 的件：0」，`summary-b1.txt`、`summary-b2.txt`、`summary-b3.txt`、`iso-b3.txt`、`k11.txt`、`mutants.txt` 六份全同。正推 K2-1 探针在冻结副本拷贝上照它的 README 重跑：`PROBE after seq write 1: R in ring=true violations=["I-7.4: 被抛弃的根（实例 1、txg 8）引用的单元已被重新分配或抹头…`，与 `logs/probe-rollback-revived.log` 同形 |
| 腿的数 | 都是腿的模型或副本上的数（正推那一形是入库装置上搭的探针），不进 kb |

被判的源文件：`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-checker-tier/src/on_device_modes.rs`、`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-harness/src/gpu_memory_ledger.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs`、`crates/singlefs-checker-tier/tests/gpu_unit_checks_a_returned_card_lease_leaves_no_buffer_on_the_card.rs`、`crates/singlefs-harness/tests/gpu_card_rounds_remember_a_stop_request_removed_mid_run.rs`、`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs`。

## 二、逐格判决

| 格 | 两腿 | 判 |
|---|---|---|
| K2-1 丙罩不罩得住（攻方一形） | **攻方量到、复跑坐实**：P = (1, 6) 与 R 之间还有一次发布 Q = (1, 7)，挂载那一刻 Q 轮换写的系统配置槽、Q 的根槽、Q 的末条记录、R 的账读回全零，F2 退回 P。丙那一支注释的前提「这条根建在所选那一版上」（冻结副本 `mount.rs` 第 2004 行）不成立：R 从 Q 继承的单元只记在 C 里 Q 那一槽（`q_in_c_r=0 q_in_c_all=26`），这个进程写掉 Q 那一槽时放出，R 那一槽晚一个 txg 才写掉，那次发布根落之前的段边界状态上 I-7.4 红、违例的是 R。`summary-b1.txt`：`arm=bing … histories=60 hit=9`、`arm=h2 … histories=60 hit=8`、`arm=chain … histories=60`（0 红）；`summary-b2.txt`：`arm=bing k=5 … histories=40 hit=8`、`arm=bing-persist … histories=40 hit=8`、`arm=chain`、`arm=chain-persist` 各 40 段 0 红 | **打中**（丙），违反 `.claude/kb/decisions/23-journal的角色与格式.md` 第 390 行与 I-7.4（近 K 代块未被复用）；攻方改法「丙并中间」只在它的模型上量过、被攻过零轮 |
| K2-1 丙罩不罩得住（正推一形） | **正推在入库代码上搭探针造出、复跑坐实**：管理员回退（向前发布）当 R、崩在轮换之前、R 的账读不出而记录齐、会话写到抬 F 时 R 还在环里：回退复活的单元在 P 的账里是已释放，丙 `!is_released` 滤掉，回退发布不写数据单元、记录不点名它们，池级 checker 在被抛弃的 R 上报 I-7.4；对照（含已释放的记录）过。复活只有 `resurrect_released_record` 一处调用（`mount.rs:7763`），别的发布种类读代码推 | **打中**（丙）；正推的对照「不滤已释放」与乙（第四块判决 K2-2 改法行：`cplusreleased` 隔离 370 个槽、600 段里 588 段被拒）同一类，代价没量 |
| K1-1 借之前 poll 那一串 | 攻方：列卡 → 锁外 poll → 销账，poll 之后、销账之前另一笔刚记成没等成就被一起销，账模型多给 1960 MiB（`k11.txt`：`step=after_b_takes_in ledger_room=23510 truth_room=21550 under=0 over=1960`），真卡上撞不撞得上推的；正推：没造出多给，残留只会少给 | **打中（窄）**，并进下一批：列卡时把借据号一起取走（攻方推的改法） |
| K1-2、K1-3 | 两腿：没找到比 `GpuCard` 克隆活得久的 wgpu 句柄（前提今天没有会红的检查）；只有已在展开中才不再抛 | 站住；前提记欠账 |
| K2-2 丙多护的那一截 | 两腿：放的时机同 H2；攻方每臂 90 段丙没有多拒一次；正推：受护扫描是落点数 × 回收记录数，4×10⁴ × 4×10³ 调试构建 1.1 秒 | 站住；耗时记下 |
| K2-3、K2-4 | 正推：有效根槽读坏照拒，调用方都改了；空条目坏盘记录那一支没造出反例（读码推） | 站住 |
| K3 第七块 | 两腿：没有下一张卡时不问叫停，三处产品路径都经打印那一半 | 站住 |
| K4-2 拿锁 | 正推：借之前取留着的设备与重起共用一把锁，实测 300 毫秒的重起里等 250 毫秒，窗口只在重起头几毫秒（推的），不判红不弃卡 | 记下（低危，用户「不压并发」的方向，并进下一批看能不能拆锁） |
| K5 判别力 | 攻方：K2-1 那一形没有用例；正推：丙的集合「多护已释放的 / 只护前一半 / 只护跨度 ≥ 2」三种照绿，叫停闭包传到下游换成永不叫停、借之前 poll 注释掉在扫源码复现里照绿 | 打中，并进下一批 |

## 三、交用户

H2 丙在第一轮代码轮里被打出两个洞（第二节 K2-1 两行），都是借来的那一份落点罩不到 R 经别的路继承来的单元。H2 → 丙 → 丙补洞，每一次补法都只量过一轮。交用户定：继续补丙（丙并中间加回退复活那一形的补法，后者代价没量），还是撤回 H2（抬 F 那一串表里没记着的被抛弃根照今天读盘、读不出就拒）。弹窗原话与用户定案记在第四节。

下一批（第 10z 批第十块）在用户定了 H2 之后派，连同 K1-1 列卡带借据号、K4-2 能不能拆锁、K5 各项补用例与变异。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 备料 | 2026-10-08 改了：丙两形违反 I-7.4，交用户重定第 37 行 H2 与丙那两句 |
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-08 不受影响：第 390 行与 I-7.4 照旧 |
| D28（挂载期承诺量） 已定项 1 | 备料 | 2026-10-08 不受影响：选哪一条都要回看第九项计数，定了再看 |
| D24（后台重活能不能卸给 GPU） 已定项 2 | 备料 | 2026-10-08 不受影响：判的是 G2 乙补法的实现与测试 |

## 四、用户定案

用户 2026-10-08 弹窗定 H2 丙「丙继续补洞」（主 agent 推荐的是「撤回 H2」，弹窗里交代了两形、两处补法的量与没量、撤回的白拒代价）。去向：
- 第 10z 批第十块（core）：丙并上攻方「丙并中间」，再补回退复活那一形；回退那一形的补法代价先量，隔离数或拒数与乙同一量级时停下交主 agent，不硬做。两处补法被攻过零轮，做完走代码轮。
- 第 10z 批第十一块（GPU）：K1-1 列卡带借据号、K4-2 能不能拆锁、K5 各项。
- D16（发布语义） 已定项 1 第 37 行等第十块交回、补法定形之后再写回。
