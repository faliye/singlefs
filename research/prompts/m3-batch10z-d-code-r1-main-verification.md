# 根环实现第 10z 批第四块：代码轮第一轮判决（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 H1 H2 R C P M7 M11 LR2 -->

正文 `research/prompts/_m3-batch10z-d-code-r1-body.md`。两腿报告：
- 正推：`research/prompts/m3-batch10z-d-code-r1-sonnet-output.md`，sha256 170dbe2a…de76，253 行；
- 攻方：`research/prompts/m3-batch10z-d-code-r1-opus-output.md`，sha256 831e6683…8c18f24，213 行。

## 一、核查

| 项 | 结论 |
|---|---|
| 开工快照 | 冻结副本 `~/.local/share/singlefs/work/m3-batch10z-d-code-r1/tree/` 对 `research/prompts/m3-batch10z-d-code-r1-snapshot/crates-sha256.txt` `sha256sum -c` 全过；kb 对 `kb-sha256.txt` 只有 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md` 对不上（主 agent 2026-10-08 写 E174 两段结果改过），两腿都没引它 |
| 主工作区被改 | 腿跑着时删除线第 45 批把 `crates/singlefs-core/src/mount.rs` 打进主工作区，腿引的 mount.rs 行号都按冻结副本 |
| 引文 | `cite-check.py` 两份报告合跑，`--root` 冻结副本：`✓ 核了 41 处引文，对上 28 处，没判 13 处`，没判的都引 kb；换仓根：kb 那 13 处对上，另 5 处对不上全是主工作区 mount.rs 的行号（第 45 批挪了） |
| 主 agent 现读 | `.claude/kb/decisions/23-journal的角色与格式.md` 第 390 行「被崩溃恢复抛弃的时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头」；`.claude/kb/invariants.md` 第 58 行 I-7.4（近 K 代块未被复用） 按「按实例表判仍然有效」定候选集，池级 checker 的被抛弃根那一半按最新根的实例表判抛弃 |
| 复跑 | 草稿目录 `~/.local/share/singlefs/work/m3-batch10z-d-code-r1-main/rerun.sh`，两份冻结副本拷贝并行，rc 全 0。攻方 `rerun.sh … a3 a4 a5 a6 x`：`summary-a4.txt 全同`、`summary-a5.txt 全同`、`summary-a6.txt 全同`、`summary-x1.txt 全同`、`summary-z1.txt 全同`；`summary-a3.txt 不同`，差的只是模型目录里多出的两行 `arm=h2 k=1 … red_at_some_step:98`，`scripts/spec-a3.tsv` 11 件里没有 k=1 的 h2 臂，这两行复跑不出、判决不引；其余各行逐字相同。正推 `k2d_probe` 的 `k11_`（`2 passed`）、`k23_`（`6 passed`）两组，`RESULT k11-admin-rollback fault_on=true: Err(CandidateJudgementStillUnreadableAfterOneReread(…))`、`RESULT K2-3 根槽挂载时读回全零(之后恢复): 参照落点数=22 挂载记着=None 挂后空闲=20` 与 `logs/` 里的相同 |
| 腿的数 | 都是腿的模型或副本上的数，不进 kb |

被判的源文件：`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs`、`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs`。

## 二、逐格判决

| 格 | 两腿 | 判 |
|---|---|---|
| K1-1 抬 F 每一处读根环 | 两腿：准入上限、抬 F 重算两处都换了，有效根槽读坏照拒，管理员回退没变。正推：公开函数 `rollback_floor_ceiling` 与 checker 档 `crates/singlefs-checker-tier/src/on_device_modes.rs` 第 336 行的调用方仍按旧口径拒（`fault_on=true: Err(RollbackFloorCeilingRootRingSlotStillBadAfterOneReread …)`），产品路径不走它；攻方同报这两处自算上限、今天的用法碰不到 | 站住；公开函数与 checker 档调用方那一形并进下一批，同口径 |
| K1-2 认同条件 | 两腿：差一整圈、只比 txg、两样都没记着三形都挡住或走不到。正推：C 在那一槽有空条目时借到 `Some(空)` 没人钉 | 站住；空条目那一形并进下一批补用例 |
| K1-3 e158 单测 | 正推：新断言钉的是「不读账、照记着的落点隔离」，原来钉的「注入确实打到抬 F 那一遍」没有断言，注入落空照绿 | 打中（判别力），并进下一批 |
| K2-1 H1 | 两腿：没造出被发出去的（攻方 3732 段、正推五形各 60 步加写满） | 没打中（各一次抽样） |
| K2-2 H2 罩不罩得住 | **攻方量到、复跑坐实**：R 只建一个 inode、与 P 共用文件的数据单元，挂载那一刻 R 的账读回全零而记录齐，C 两份账都不加，C 在 R 那一槽只有 R 记录点名的 22 个落点、没有 R 从 P 继承的单元；会话覆盖写释放了它们，抬 F 照借来的 C 算、回收、再分配，池级 checker 判 I-7.4 红。`summary-a4.txt`：`arm=h2 k=3 … red_at_some_step:53`、`arm=h2 k=5 … red_at_some_step:68`（各 150 段，故障不撤）；同批 `arm=today k=3 … step_refused=150 hit=0`。正推 C 里有 R 时 61/61 罩住、C 里没有 R 的形 13/13 红，是 D23 第 392 行声明的代价 | **打中（H2）**。R 是这次挂载自己写行抛弃的，D23 第 390 行主语字面写「被崩溃恢复抛弃」；I-7.4 与池级 checker 按实例表判抛弃，套得上——checker 即规范，按违反 I-7.4 判。用户 2026-10-08 定的 H2 字面许可这一形，交用户（第三节） |
| K2-2 改法（攻方，都只在它的模型上量过、被攻过零轮） | 甲（先读盘、读不出才借 C）：故障撤了 0 红，故障不撤 `arm=diskfirst k=3 … red_at_some_step:53`、`k=5 … :68`，与 h2 逐格同；乙（C 再并全部已释放落点）：`summary-a5.txt` `arm=cplusreleased k=3 … step_refused=300`，不可用；丙（挂载那一刻较新可读根账读不出而记录齐时，C 那一槽并上那一刻现行账里仍分配着的落点）：`summary-a6.txt` `arm=cplusmountaccount k=3 … histories=2113 … hit=0`、`k=5 … histories=300 step_refused=0 hit=0`，隔离均值 8.6 / 33.1（h2 4.1 / 15.8），会动 `C_NEWER_ROOTS` 与 D28（挂载期承诺量） 已定项 1 第九项的计数，别的用例红不红是推的 | 交用户（第三节） |
| K2-3 挂载时被抛弃根槽读回全零 | 两腿都造出（`summary-z1.txt` `fault=slot-zero-at-mount … red_at_some_step=48 red_final=42`；正推 `挂载记着=None 挂后空闲=20`），撤掉第四块的副本上同样，不是这一批带进来的 | 归 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到）：题面补「挂载那一刻根槽读回全零或读错」一形，下一份 kb 写回 |
| K3 判别力 | 正推：`loop-span1`（补隔离那一圈跨度改成 1）照绿，用例核「不空闲」只查每个落点的起点槽（`raise_floor_takes_…:417`）；H2 端到端用例的期望取自被测方法本身，九条 H2 变异全靠一条库单测拦。攻方：入库 H2 用例的 R 是覆盖写，造不出 K2-2 那一形 | 打中，补用例与变异 |

## 三、交用户与下一批

用户 2026-10-08 弹窗定 H2「改成丙」：H2 照旧借 C 那一圈，可写挂载那一刻较新可读根的账读不出而记录齐时，C 那一槽再并上那一刻现行账里仍分配着的落点（攻方改法丙，只在攻方模型上量过、被攻过零轮；弹窗里交代了甲在账一直读不出时与 H2 逐格同红、乙隔离约 370 个槽而 600 段里 588 段写入被拒，用户追问「甲乙有没有可能是算法的问题」，主 agent 读攻方臂补丁答：甲缺的是信息、乙过滤条件太宽，丙是乙收窄到「挂载那一刻仍分配着的」那一截）。实现进下一批（第 10z 批第九块），选丙要回看 D28（挂载期承诺量） 已定项 1 第九项，连同：
1. K1-1：公开 `rollback_floor_ceiling` 与 checker 档 `on_device_modes.rs` 第 336 行、e158 第 1992 行两处自算上限，照抬 F 那一串的新口径（被抛弃根的槽读坏不拒）；
2. K1-2：C 那一槽有空条目时的用例；
3. K1-3：e158 单测补「注入确实打到抬 F 那一遍」的断言；
4. K3：「不空闲」核每个落点的全部槽；H2 端到端用例的期望独立算；K2-2 那一形（R 与 P 共用会被释放的单元）做成会红的用例。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 备料 | 2026-10-08 改了：H2 那一形违反 I-7.4，交用户重定第 37 行 H2 那一句 |
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-08 不受影响：第 390 行与 I-7.4 照旧；R 套不套得上第 390 行主语按 checker 的判法（按实例表判抛弃）认，条款字面的主语要不要放宽另交写回时一并核 |
| D28（挂载期承诺量） 已定项 1 | 备料 | 2026-10-08 不受影响：攻方改法丙会动第九项的计数，选丙时再回看 |
