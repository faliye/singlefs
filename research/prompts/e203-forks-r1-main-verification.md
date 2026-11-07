# e203-forks-r1 判决：准入之前扫类的三条岔路（B1、B8、B4）拿 E203 的数判，第一轮（2026-11-07）

<!-- doc-lint:not-numbers B1 B4 B8 B9 B10 B11 B12 B13 X2 Q3 Q4 Q5 Q6 Q8 Q9 Q10 F1 F2 F3 F5 F6 -->

## 一、材料与核查

- 正文 `research/prompts/_e203-forks-r1-body.md`，背景 `research/prompts/_e203-forks-r1-background.md`（材料任务 9628）。腿：正推 `research/prompts/e203-forks-r1-sonnet-output.md`、攻方 `research/prompts/e203-forks-r1-opus-output.md`。
- 快照倒推：`task-server/task submit --cwd ~/.local/share/singlefs/work/e203-forks-r1-judge -- python3 <仓根>/scripts/agents/snapshot-tree.py e203-forks-r1 --out ~/.local/share/singlefs/work/e203-forks-r1-judge/e203-forks-r1-tree --repository <仓根>`（任务 9713），「清单点名 2 份，倒推出 2 份，sha256 对上 2 份」。
- 引文核对：`python3 scripts/agents/cite-check.py research/prompts/e203-forks-r1-sonnet-output.md research/prompts/e203-forks-r1-opus-output.md --root ~/.local/share/singlefs/work/e203-forks-r1-judge/e203-forks-r1-tree --root ~/.local/share/singlefs/work/e203-runner-segment-2-r2 --background research/prompts/_e203-forks-r1-background.md`，「核了 74 处引文、111 处裸行号，对上 29 处，没判 156 处」。没判的多是冻结源树的裸行号；判决用到的逐处人工核，列在第二节。
- 攻方腿模型复跑：拷到 `~/.local/share/singlefs/work/e203-forks-r1-judge/model/`，任务 9708 退 0，末行 `name=rerun_done lines=64`；与攻方报告第九节逐项对上（`name=b8_without_mapped_units_queries` F、same_mount、overwrite `alpha=0.583 band=GrowsWithTheAxis`，其余 11 行 `band=DoesNotGrow`；`name=monotone_on_the_u_axis state=remounted kind=overwrite steps_where_the_total_decreased=2`；`name=release_find_calls` 8 行 `finds_over_counted_calls=2.000`；`name=s2_residual_on_the_u_axis state=remounted kind=overwrite alphas=0.746,0.682,0.691,0.703,0.699 all_five_at_least_half=True`）。
- 正推腿交的补丁与脚本（`~/.local/share/singlefs/work/e203-forks-r1-sonnet/patch/q6-rows.patch`、`tools/q6-rows-run.sh`）没跑：它算的是攻方模型已复跑出的同一件（逐行 B8 步数），去向见第三节。

## 二、逐格判

| 格 | 两腿 | 主 agent 现查 | 判 |
|---|---|---|---|
| 1 B1 能不能拿 X2 直接读数判翻面 | 两腿都判：改成「判」是放宽（登记 `research/prompts/e203-preregistration.md` 第 618、680 行：F6 打中 Q4 不判，「X2 量过之后照严的读法 Q4 不判」） | 读了第 680 行原文 | 打中。E203 的 F1 照登记记 unknown，不改。X2 上整份拷分配器一行 1.32–1.42 秒（量的）只作为岔路上的下界读数交用户，被攻过零轮 |
| 1 单调前提 | 两腿都判「扫类步数随记录数不减」不成立；只有调用方那一份整份拷由 U 定、与 u、D、F 无关 | 复跑 `steps_where_the_total_decreased=2`；冻结源树 `transaction.rs:12608` 那一行 `allocator.clone()`，其上注释写「第一版每次发布约 450 KiB 的拷贝」 | 打中。下界只能用整份拷一行，不能用扫类合计 |
| 2 B8 的 F2 打中归属 | 两腿都判：D、F 两轴的涨 97.5–99.3% 来自 s36 的查询项，登记同一行「做法」与「式子」不相容（第 283 行：做法「`mapped_units` 整表换成差量，按角色查走随一版维护的索引」，式子「每轮 dm + 查询次数 × lg(M)」）；正文点名的三行是模型按四轴之和挑的 X2 点（`scripts/experiment-apparatus/e203_admission_scan_cost.rs` 第 2650 行起），不是 D、F 高端 | 读了第 283 行与模型第 2645–2652 行；复跑扣掉 s36 查询项后 11 格不涨 | 打中，归「打中归错了判据」（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错」）：F2 打中的主要是登记式子的错。扣掉后 F 轴同一次挂载覆盖写仍 α=0.583（固定点 22 轮的乘数），这一格 B8 仍输 |
| 2 s25 计数 | 攻方：两处查找只 `allocator.rs:2211` 一处记调用数，8 格 `finds_over_counted_calls=2.000` | 复跑相符 | 打中，装置计数欠一半；③2 这一行是上界部分，不改「三·二全过」，B1 的这一行步数少记约一半 |
| 2 F3 | 攻方：计时产物无 `name=point` 行，Q3、Q10 一行没打，F3 `fired=false` 是没判 | `grep -c '^E203RESULT name=point '` 计时 base 产物 0、计数 1；任务 9595 日志 q3、q10 各 0 行 | 打中，装置缺陷：没读到数时报了「没打中」。按产物现算（攻方，推的）两道之间残差在 U 轴、重挂载五轮 α 0.68–0.75，F3 应打中 |
| 3 B4 读法 | 两腿都判站不住：式子判就是 D28（挂载期承诺量） 已定项 1 的 ENOSPC 准入，需求取自两道之间算出的 `settled.resolved` | 冻结源树 `transaction.rs:12884–12889`（需求 = `demand_of_the_roles_on_each_device(&settled.resolved.rewritten_roles, …)`）、`:12890`（删除类发布不判式子判） | 打中（覆盖写）。B4 只能当改条款交（D8（核心索引结构） 已定项 16 射程与 `.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」那一格），不能当读法；删除类发布不判式子判，登记读法 1 把那一段算进「准入之前」是多算。正文给的 4.3–404 秒用的是 Q5 判为不可信的外推，作废 |
| 4 漏掉的候选 | 正推 B9–B13，攻方三个，都零轮 | 未核（零轮） | 不判赢；进交用户表，标零轮 |

## 三、交用户表

| 项 | 内容 | 状态 |
|---|---|---|
| 装置修 | ① 计时报告从计数产物取点的轴值，Q3、Q10、F3 照判；没读到数时报 unknown，不报 false。② s25 第二处查找记调用数。改 `scripts/experiment-apparatus/e203_admission_scan_cost.rs` 与插计数的那一处，带单测与变异 | 交用户：修后①只要重出报告，②要重跑计数构建（18 个点） |
| 登记修订 | s36 的式子改成按差量问（与同一行做法一致），照 `evidence-discipline.md` 记一次输：原式子下 F2 打中照记 | 交用户 |
| B1 | 留不住：对 `fs-design.md` 运行时决策路径「代价不许随盘容量增长」本来就红（U 轴 α 0.998，量的）；X2 整份拷一行 1.32–1.42 秒（量的）。E203 的 F1 仍 unknown | 够判，交用户 |
| B8 | 修订 s36 之后只剩 F 轴同一次挂载覆盖写一格涨（固定点轮数乘数，推的）；收严形态（固定点轮数有上界）是新候选，零轮 | 修装置与登记后重判 |
| B4 | 只能当改条款交；两道之间那一段 X2 上每次覆盖写约 2.78 秒（量的），其中 99.999% 是整份拷两次（正推腿，推的） | 够判，交用户 |
| 新候选 | 只把整份拷分配器换成撤回日志（条款不动）；分块共享拷贝；B8 收严形态；留 B1 并在条款里明示整份拷分配器是例外；式子判吃最坏数 | 零轮，各要登记计数实验 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 16 | 不影响 | 2026-11-07 不受影响：这一轮只判岔路，B4 若被选才改射程，交用户 |
| D28（挂载期承诺量） 已定项 1 | 不影响 | 2026-11-07 不受影响：引来判式子判就是 ENOSPC 准入，条款不改 |
