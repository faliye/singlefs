# m3-close-b20-code-r1 判决：C971 乙入判不出与 C973 第 0 代根判据的代码轮（2026-11-07）

正文 `research/prompts/_m3-close-b20-code-r1-body.md`；腿报告：正推 `research/prompts/m3-close-b20-code-r1-sonnet-output.md`，攻方 `research/prompts/m3-close-b20-code-r1-opus-output.md`（sha256 213727c9…）。开工快照目录：`research/prompts/m3-close-b20-code-r1-snapshot/`。

## 这一轮被判过的文件

- crates/singlefs-core/src/mount.rs：C971 乙入判不出与重读判定
- crates/singlefs-core/src/instance_table_sweep.rs：C973 第 0 代根清扫判据
- crates/singlefs-core/src/recovery.rs：R2 系统配置槽判定与别池槽处理

## 一、核查

- 快照倒推（任务 10231）：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-b20-code-r1-judge -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py m3-close-b20-code-r1 --out /home/fy5090/.local/share/singlefs/work/m3-close-b20-code-r1-judge/m3-close-b20-code-r1-tree --repository /home/fy5090/code/singlefs`，「快照 m3-close-b20-code-r1：清单点名 8 份，倒推出 8 份，sha256 对上 8 份」。
- 引文核对：`python3 scripts/agents/cite-check.py research/prompts/m3-close-b20-code-r1-sonnet-output.md research/prompts/m3-close-b20-code-r1-opus-output.md --root /home/fy5090/.local/share/singlefs/work/m3-close-b20-code-r1-judge/m3-close-b20-code-r1-tree --background research/prompts/_m3-close-b20-code-r1-background.md`，「✓ 核了 22 处引文、139 处裸行号，对上 12 处，没判 149 处」，没有一处对不上；没判的是快照外的文件（`recovery.rs` 等）与裸行号，下面用到的逐处现读。
- 主 agent 现读：`crates/singlefs-core/src/mount.rs` 第 9326–9340 行（第一遍判据为假直接放行；否则 `before_rereading()`、整份 `read_stage` 重读一遍，只按重读那一遍的 `witnesses_a_publish_newer_than_the_selected_version()` 判拒）；`crates/singlefs-core/src/recovery.rs` 第 5148–5160 行（fsid 与本池不同、解析得过的槽不进 `self_verified`、也不计读错）；`crates/singlefs-core/src/instance_table_sweep.rs` 第 605–614 行（判据是 `(GENESIS_CHECKPOINT_TXG.0 + 1..=highest_txg_of_the_pass.0).take(ring_length_in_slots).any(|txg| target_for_publish(CheckpointTxg(txg), slots_per_region) == ring_slot)`）。
- 模型复跑：攻方模型拷进 `/home/fy5090/.local/share/singlefs/work/m3-close-b20-code-r1-judge/model`（sha256 与腿交的相同：db33449f…、b783e090…），`--quick`（任务 10232）与全量 `tools/rerun.sh`（任务 10233）。全量输出 `rerun-1/out/full.out`：S = 4、5、8、16 每档每个模式只有 `consistent`、`stop:*` 与 `deflation-changes-verdict` 三类栏，`hidden`、`equality`、`other-mismatch` 0 处，例「name=genesis_criterion S=8 mode=single consistent=63596 deflation-changes-verdict=24 stop:genesis-after-reread=11208 stop:genesis-still=28206 stop:not-verified=1559321 stop:zero=4341」。模型第 140–147 行：`deflation-changes-verdict` 数的是「按重读之后的最大 txg 重判，结局会变」的盘面，比的是另一种写法；今天代码的结局只看 `consistent` / `hidden` / `other-mismatch`。模型是攻方照代码手抄的，不是独立校验路子。
- 检查单第 7 格：`bash scripts/crash/crash-point-replay.sh --list-items`（任务 10230）列得出 `read-faulted-states-never-hold-the-witness-criterion` 与 `random-histories-and-fault-injection-given-states` 两项。

## 二、逐格判决

| 格 | 判 | 依据 |
|---|---|---|
| C971 认的两种坏槽、重读读的是系统配置槽、「没有一块盘两槽都自证过」与 R2 同一个量、拒在取号之前、取号 F2 逐盘核只用 R2 | **成立**（正推；攻方面一、面二同） | 正推报告 C971 节，`mount.rs:9239–9241` 对 `recovery.rs:5194–5196`、调用点 `mount.rs:9720` |
| C971 重读粒度 | **打中（两条腿，主 agent 现读坐实）**：实现整个读阶段重读一次、只按重读那一遍判；D22（单元原子性怎么合成） 已定项 8 射程第 185 行写「重读那一槽一次」。用户定的原句（`research/prompts/e205-preregistration.md` 第 46 行「重读之后仍是这两种之一的槽…」）没说只重读那一槽，「那一槽」是 2026-11-07 写回时多写的。差异只在多拒一侧：第一遍好的槽在重读时瞬时读坏 | `mount.rs:9333–9336`；攻方面三、正推 F2 |
| C971 别池的槽（fsid 不同、校验和过） | **打中（正推 F1，主 agent 现读坐实）**：登记第 47 行「「自证过」…整槽校验和过、fsid 与本池相同」，C707 那一行写实现把这种槽当「读得出而自证不过」；`recovery.rs:5151–5157` 对乙这一判把它略过，于是「没有一块盘两槽都自证过、没有撕裂 / 全零 / 读错槽、有别池槽」那一形照条款该拒、实现不拒 | 正推 F1；`.claude/kb/checks-owed.md` 第 424 行 C707 |
| C971 拒因成员注释 `mount.rs:399–401`、`rereads.read_stage` 记法 | 轻（正推 F3、F4）：注释没跟上乙这一拒，改注释；记法并进下一批核 | 正推报告 |
| C973 调读者同一个落点核、错位读重读一次仍不过不删、第 0 代根判据走同一条重读路 | **成立**（正推；攻方面四甲、乙、面五同；模型全量 `hidden` 0） | 上面模型读数 |
| C973 条款写法（`.claude/kb/checks-owed.md` 第 613 行「这一遍读出的环里已有落点公式会写进这一槽且 txg 更大的根」） | **打中（两条腿）**：照字面不可能成立（过了落点核的根只住它自己那一槽，而这一槽读出的是第 0 代根）；代码用的是上界式（`instance_table_sweep.rs:611–613`）。代码对，改条款那一句 | 攻方面四丁、正推 F5 |
| C973 丢了写进槽 0 的根槽写（说谎的设备吞掉 txg 1 的根槽写） | 只影响活性：清扫每轮都停、最多停一圈（3S 次发布），只少删不多删；写进 C973 射程 | 攻方面四戊；模型 `clean` 模式 `stop:genesis-still` 非零、`hidden` 0 |
| C973 上界 `..=` 与 `..` | 等价（攻方面四丙，模型 `equality` 0）：不进变异表 | 模型读数 |
| 检查单八格 | 成立（两条腿）；第 1 格旁注：清扫新加的 `.expect`（`instance_table_sweep.rs:507–511`）实现员报告没列，正推核过理由成立；第 2 格旁注：`RootRingSlot` 的区域号、槽号用裸整数，这一批之前就这样，记进下一批核 | 两份报告 |

## 三、交用户的

无岔路。重读粒度照用户原句改 kb，不改代码。

## 四、被攻过零轮的

第五节的三处改法（别池槽并进乙、重读粒度用例、C973 条款改写）。C973 判据「没打中」只有攻方一次抽样加主 agent 复跑它手抄的模型，下一次代码轮再攻这一批与第五节的改法，算第二份样本。

## 五、去向

| 事 | 去向 |
|---|---|
| C971 别池的槽 | 并进下一批 `implementation-writer`：乙那一判把 fsid 与本池不同而校验和过的槽算「读得出而自证不过」（照登记第 47 行与 C707），配用例与变异行；`mount.rs:399–401` 注释改成现状；`rereads.read_stage` 记法核一遍 |
| C971 重读粒度 | 写回批：D22 已定项 8 射程「重读那一槽一次」改成「整个读阶段重读一次、按重读那一遍判」（三语）；同一批实现员补一条用例钉住「第一遍好的槽重读时读坏照拒」 |
| C973 条款写法 | 写回批：`.claude/kb/checks-owed.md` C973 那一行判据分句改成上界式（环里这一遍读出的最大 txg 到没到发布路径写进这一槽的那个 txg），射程补「丢了写进槽 0 的根槽写时清扫最多停一圈、只少删」（三语） |
| `RootRingSlot` 裸整数 | 下一批核，不在这一轮 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D22（单元原子性怎么合成） 已定项 8 | 支撑 | 2026-11-07 改了：射程「重读那一槽一次」与实现不符，写回批改成整个读阶段重读一次 |
| D16（发布语义） 已定项 6 | 不影响 | 2026-11-07 不受影响：清扫读根环调落点核的口径不动，C973 判据写法改在欠账表 |
