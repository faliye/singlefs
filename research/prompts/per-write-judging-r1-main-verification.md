# 崩溃状态逐写独立判：第一轮判决（2026-11-06）

正文：`research/prompts/_per-write-judging-r1-body.md`。两条腿：正推 `research/prompts/per-write-judging-r1-sonnet-output.md`，攻方 `research/prompts/per-write-judging-r1-opus-output.md`。

## 核查怎么做的

- 开工快照倒推：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/per-write-judging-r1-verify -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py per-write-judging-r1 --out /home/fy5090/.local/share/singlefs/work/per-write-judging-r1-verify/per-write-judging-r1-tree --repository /home/fy5090/code/singlefs`（任务 8688）。快照清单只点名 4 份 kb 文件，倒推出 2 份、sha256 对上 2 份；`crates/singlefs-checker/invariants/02-根环.md` 等 2 份倒推不出（快照之后有提交改过它们）。腿引到这两份的行，判决不采。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/per-write-judging-r1-sonnet-output.md research/prompts/per-write-judging-r1-opus-output.md --root /home/fy5090/code/singlefs --background research/prompts/_per-write-judging-r1-background.md`，末行「✓ 核了 75 处引文、85 处裸行号，对上 75 处，没判 85 处（其中裸行号没有引文可比 85 处；逐处列在上面）」。快照里没有 `crates/` 下的文件，所以引文对着主工作区此刻的树核；只给行号、没有引文的 85 处没有逐处人工核，判决不靠它们。
- 复跑：
  - 正推腿要的记录核对器四格：任务 8593，「test result: ok. 4 passed; 0 failed」。
  - 攻方腿的模型：主 agent 在新草稿目录整份复跑，任务 8686（前两趟 8598、8655 都是第一次撞内存上限、重排时被 `rerun.sh` 的「--work 已存在且非空」挡住），第 2 次尝试退 0。产物 `/home/fy5090/.local/share/singlefs/work/per-write-judging-r1-attack/rerun-work-3/out.txt` 55 行。

## 逐条判

| 腿的论点 | 判 | 依据 |
|---|---|---|
| 攻方①：「对所有满足判据的段等价」是从删除流三段与一个阳性对照外推过头 | 采 | 任务 8321 的探针只量了段 12、16、20（都是 UnitWrite 段、`over_an_earlier_write=0`），阳性对照那两份写是 `over_an_earlier_write=true`（`PROBE_WRITE label=planted segment=16 position=0 … over_an_earlier_write=true`），正是判据要排除的那一类，证不了判据收下的段上逐写判对 |
| 攻方②：判据依赖「被选中的根只能是上一版」「在飞单元靠诞生代号挡在外面」，前者靠崩溃点重放要验的持久顺序，后者对搬迁副本不成立 | 挂起，第二轮核 | 推的；transaction.rs:10564、walk.rs:17931 两处主 agent 没逐行核 |
| 攻方③：反例 A（副本写晚、落进下一个单元段）、反例 B（记录先写、点名的单元后写，候选甲漏红） | 核不动 | 复跑 8686 的原型把每一段都跳过了：`PROTO_SEGMENT_SKIPPED … reason=写数超过 19`，`PROTO_PLANNED_STATES total=0`。一个状态都没判，反例既没坐实也没推翻 |
| 攻方：六类破坏条件漏 5 类 | 挂起，第二轮核 | 逐类指到的 文件:行 是推的 |
| 正推：判据 D1–D5，D5 要 S=∅ 那一遍的读迹，省了就有活反例 | 挂起，第二轮攻 | 推的；没有一条量过 |
| 正推：判据内候选甲、乙在「红不红」「红项」两层都等价 | 不采为结论 | 只在三段上量过（8321），见攻方① |
| 正推：两份副本都盖时红项不等于两个单写红项的并，候选甲在红项层不够 | 采 | 任务 8321：`PROBE_PAIRS_NOT_UNION label=planted segment=16 pairs=1`，两份都盖时多出 `journal_*_oracle:read_failed` |
| 正推：补两类破坏条件（已准入单元的副本组、单元写与系统配置槽同段） | 挂起，第二轮核 | 推的 |
| 正推：观察者分三类，只记计数的要按谓词类加权 | 挂起，第二轮核 | 推的 |
| 正推：取样点只撑得起这三段，缺 8 类敏感点 | 采 | 与攻方①同一件事，两腿一致 |
| 正推要的记录核对器四格今天的判法 | 采 | 任务 8593 绿 |

## 结论

第一轮不立判据。量过的只有两件：删除流段 12、16、20 不动时逐态判定都与 S=∅ 相同（8321）；要保红项，候选甲不够，至少要按单元副本组判（8321 的 pairs=1）。「对所有满足判据的段等价」与两个反例都还没有量过的证据。

开第二轮，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第二轮分工（第一轮的修补本身 / 产生结论的方法 / 从未被看过的地方），至少一轮派辩方：

1. 攻方腿把原型改成在写数多的段上按子集取样判（单写、成对、只差一个、随机），在反例 A、B 那几条臂上真判状态，量出逐写判与全组合判的差；改完的命令交主 agent 复跑。
2. 辩方腿复核正推腿的判据 D1–D5 与候选乙，对着攻方①②与漏掉的破坏条件逐条答：哪些判据要收严、收严之后还能降到多少态。
3. 两腿都要用正推腿列的 8 类敏感取样点，至少 3 类写成能跑的命令。

## 交用户的表

| 项 | 状态 | 说明 |
|---|---|---|
| 逐写独立判的判据 | 被攻过一轮，没立 | 两个反例核不动，第二轮量 |
| 候选甲（逐写判新旧） | 红项层不够（量的） | 8321 的 pairs=1 |
| 候选乙（按单元副本组判） | 被攻过零轮 | 第二轮辩方复核 |

## 回看决策

不涉及决策：这一轮没有立判据，D13（验证路线）的判法没有改，任何决策分项都不写回。
