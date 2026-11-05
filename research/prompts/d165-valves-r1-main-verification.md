# d165-valves-r1：派发闸里只放过一次的「…已判」放行行——第一轮判决（2026-11-01）

## 一、这一轮的材料与证据

- 正文：`research/prompts/_d165-valves-r1-body.md`；开工快照：`research/prompts/d165-valves-r1-snapshot/files-sha256.txt`（8 份）。
- 正推腿（Sonnet，格 F）：`research/prompts/d165-valves-r1-sonnet-output.md`（约 283 行），草稿 `~/.local/share/singlefs/work/d165-valves-r1-sonnet/`（两份只读扫描脚本、三份探针提示）。
- 攻方腿（Opus，格 A）：`research/prompts/d165-valves-r1-opus-output.md`（sha256 10e5341a…），模型 `research/prompts/d165-valves-r1-opus-model/`（8 份探针提示、3 份 grep 原样输出）。
- 倒推开工树与核引文（原样；倒推经 task-server 交，`--cwd` 给本机草稿目录）：

```
python3 scripts/agents/snapshot-tree.py d165-valves-r1 --out <草稿目录>/d165-valves-r1-tree --repository <仓根>
python3 scripts/agents/cite-check.py research/prompts/d165-valves-r1-sonnet-output.md research/prompts/d165-valves-r1-opus-output.md --root <草稿目录>/d165-valves-r1-tree --background research/prompts/_d165-valves-r1-body.md
```

- 倒推结果（任务 2478）：「✓ 快照 d165-valves-r1：清单点名 8 份，倒推出 8 份，sha256 对上 8 份」；写判决前 `sha256sum -c` 快照清单 8 份都 OK，腿跑着时快照里的文件没被改过。
- 引文核对：对开工树「✓ 核了 74 处引文、55 处裸行号，对上 51 处，没判 78 处」——没判的多是快照之外的文件（`scripts/gate/admission.py`、`scripts/kb/kb-spec-check.py`、`scripts/kb/stale-candidates.py`、`.claude/hooks/handback-guard.sh` 等）在开工树里没有；快照之外的文件这一轮没人改过，对主工作区再核一遍「✓ 核了 74 处引文、55 处裸行号，对上 74 处，没判 55 处（其中裸行号没有引文可比 54 处）」，剩下 1 处 `main-agent.md:100` 是只写了文件名，主 agent 现核 `.claude/main-agent.md` 第 100 行就是写「规格检查已判」的派发表那一行。没有对不上的。
- 主 agent 复跑（都经 task-server）：
  - 攻方腿 8 条派发闸 `--check-prompt` 探针（任务 2464–2471）：P1 退 2、快照段点名 m4-gpugc-code-r2 与「.claude/kb/milestone/04-fourth-txn.md（出现在派发提示」；P1b 退 0、没有快照段；P2 退 2、点名「crates/mutations.tsv（出现在派发提示」；P2b 退 0；P3 退 2、点名 d165-valves-r1 与「.claude/agent-common.md（出现在派发提示」；P4 退 2、有「认不出一份规格」；P4b 退 0；P5 退 2、有「认不出一份规格」。8 条与腿报告预写的逐条相同。
  - 正推腿三条探针（任务 2475–2477）：p8a 没有「认不出一份规格」，p8b、p8c 各有 1 处，与腿跑前写下的预测相同。
  - 正推腿两份只读扫描（任务 2473、2474，`--cwd` 本机草稿目录，读本机会话记录）：「Counter({'快照冲突已判': 2679, '规格检查已判': 320, '限额窗口已判': 152, '准入已判': 5, '写者锁已判': 5})」，「通用已判」不出现；写回员派发 321 次，「179 ('valve', 'no-verdict-file', …)」「141 ('valve', 'verdict-file', …)」「1 ('no-valve', …)」，即 320 次带「规格检查已判」。与腿报告的数相同。这些数只覆盖本机这一批会话记录。 <!-- account-quota:allow 历史派发闸放行行与规则讨论 -->
- 主 agent 现查的源码：`.claude/hooks/runner-dispatch-guard.sh`（f11bbf5d）第 1267–1279 行 ⑦ 的 `lifted` 要先有一次同族派发、且那次没再撞限额；第 1291 行 ⑧ 开头一句就是放行行；第 2020–2021 行锁是别的会话派的时，`launches`、`ended` 整个换成那份记录里的。`scripts/gate/apply-writer-patch.py`（057be868）自证第 684–685 行写死「改 lib.rs 的补丁与只追加变异行的补丁都拒」。 <!-- account-quota:allow 历史派发闸放行行与规则讨论 -->
- 核不动的：正推腿 1.1(C) 账号信号的稳定性、6.1 声明漏报率、⑨ 那 5 条放行行里键当时漏没漏输入；攻方腿 A2（主 agent 与子 agent 是否同一额度池）、A13（子 agent 两次写会话记录之间的最长间隔）。都按线索。

## 二、逐条判决

「采」是主 agent 现查或复跑坐实的；「线索」是腿读出来、主 agent 没量到的；所有改法被攻过零轮。

| 放行行 | 用了几次（本机会话记录） | 判 | 依据 |
|---|---|---|---|
| ⑦ 限额窗口已判 | 152 | 机制问题，采 | `lifted` 只认窗口之后又派出去、没再撞的同族派发（第 1273 行），而那次派发本身被第 1276 行拒——不写放行行就永远解不开，卡到重置时刻（攻方 A1，读码坐实）。正推腿的改法（读主会话与子 agent 记录里同族成功的回合、加无工具的探测类型）只解得开主 agent 自己那一族（攻方 A2，线索）  <!-- account-quota:allow 历史派发闸放行行与规则讨论 --> |
| ⑧ 规格检查已判 | 320 / 321 次写回员派发 | 机制问题，采 | 放行行在 `kb_scribe_spec_verdict` 第一句（第 1291 行），骨架默认带（`scripts/agents/dispatch-prompt.py` 第 216 行）、主 agent 定义要求每次写（`.claude/main-agent.md` 第 100 行），等于 ⑧ 对写回员恒不判。⑧ 认不出「写回员自己起草规格」这种正常派发：续做的「承接：」报告（P4）、判决是 `records/` 里的用户定案（P5）、判决形状对又点名别的 `.md`（p8b）、判决写别的形状（p8c）都拒在「认不出一份规格」 |
| ⑨ 准入已判 | 5 | 部分写法、部分机制，采 | 退 2（没登记实验键）、退 3（前提没齐）是写法问题：照拒绝说明接准入、补前提；退 2 而判法没跑起来是工具坏了，修工具。退 77（键与上次产物头相同）在派发一侧拒的是「派设计员写登记」这一步，本身不跑东西；5 条理由都是「判据变了」「被测实现变了」（任务 2473 的理由行），重跑理由在键外时只能靠放行行（攻方 A10、A11，读码；键漏没漏输入核不动） |
| ⑯ 写者锁已判 | 5 | 机制问题两处，采；去掉放行行不卡死，采 | 锁是别的会话派的时，本会话停掉（TaskStop）的没并进 `ended`（第 2020–2021 行）；派发过闸后被权限判定拒掉、锁留下，要等 10 分钟（正推 4.2，理由行「那次派发过了派发闸，随后被自动权限判定拒掉」）。别的情形有 TaskStop、3 小时、换目录三条出路（攻方 A12） |
| ⑫ 通用已判 | 0 | 写法问题，采 | 漏写类型、clerk / Explore 做得了的活，改写法；会话记录零实例，clerk 已带 WebFetch、WebSearch（攻方 A14） |
| ⑥ 快照冲突已判 | 2679 | 机制问题，采 | 按「路径出现在提示里」判，不按「要改」判：「不碰」一行（P1、P2）、只读引用（P3）里点名的文件同样拒；打补丁一侧只追加变异行也拒（`apply-writer-patch.py` 自证第 684–685 行），与派发闸 ⑤「`crates/mutations.tsv` 只追加不算撞」相反（攻方 A5）。改成「声明的要改文件集」判，写回员自己起草规格时派发那一刻看不到目标（攻方 A6，读码）；实现员的「交补丁」现在只是约定，写范围表放行它写主工作区的 `crates/**`（正推 6.2 的前提，线索） |

六种之外同形的单次放行口（攻方 C1–C4，正推第 7 节）：`scripts/kb/kb-spec-check.py` 的 `--allow-uncommitted`（不要理由）、`SINGLEFS_EXPERIMENT_RERUN_REASON`（理由进产物头）、`scripts/kb/stale-candidates.py` 的 `--allow-dirty-worktree`（理由进输出头）、派发闸 ⑪ 的「只修锚点」「只补落产物」声明（第 2432 行按整份提示的子串认）、`apply-writer-patch.py` 的 `--snapshot-ruled`。采为线索：存在是 grep 坐实的（`research/prompts/d165-valves-r1-opus-model/` 下 grep 原样输出），是不是单次放宽、各自该怎么改，这一轮没判。`--force`、复用式 `--skip` 不放过判定（攻方 C6），不算。

常设豁免表：现有两张被当范本的表头自己写着「逐次豁免」「这次不走三方」，门禁只比路径与哈希、不核理由（攻方 B1，采）；⑦ ⑧ ⑨ ⑯ 被拒的都是一次性事件，按事件取键的豁免行用一次就作废、按类取键又罩住以后的真情形（攻方 B2，采为判据）。所以这一轮的改法一律走「改判法」，不新立常设豁免表。

## 三、怎么办（改法都被攻过零轮）

| 闸 | 改法 | 落点 |
|---|---|---|
| ⑦ | 二选一，交用户（岔路一）：（甲）删掉限额窗口那一支，撞了限额的派发自己失败、失败通知就是信号；（乙）照正推腿改 `lifted`（读成功回合）加探测类型，A2 那一族解不开的另想 | `runner-dispatch-guard.sh` 第 1265–1287 行 `model_verdict`（限额窗口那一支第 1267–1279 行）  <!-- account-quota:allow 历史派发闸放行行与规则讨论 --> |
| ⑧ | 只认行首「规格：」标签行点名的文件当候选规格、逐份交 `kb-spec-check.py`；没有「规格：」行的当写回员自己起草（定义第 1 步它自己跑规格检查到绿），派发时不判；骨架与主 agent 定义删「规格检查已判」，骨架改打「规格：」「判决：」两行 | `kb_scribe_spec_verdict`；`scripts/agents/dispatch-prompt.py` 第 216 行；`.claude/main-agent.md` 第 100 行；`.claude/agent-common.md` 第 95 行 |
| ⑨ | 派发一侧删退 77 那一支（派设计员写登记不跑东西），退 2、退 3 照拒、不留放行行；跑的那一侧 `SINGLEFS_EXPERIMENT_RERUN_REASON` 归六种之外那一批 | `designer_admission_verdict` |
| ⑯ | 修两处：别的会话派的锁并入本会话 `ended`；从会话记录里认出过了闸又被权限判定拒掉的派发、不等 10 分钟；删放行行 | `writer_lock_holder_state`、第 2058 行 |
| ⑫ | 删放行行，闸照拒，出路只留 clerk / Explore | 第 2716–2720 行 |
| ⑥ | 交用户（岔路二）：（甲）派发一侧只判声明的要改文件（实现员「要动的 crates 文件」、写回员加「要写回的文件：」一行），剔「不碰」与只读引用；（乙）挪到写的那一刻，由写闸按真目标判（攻方推的，A6、A3、A4 都修得到）。两种都把 `crates/mutations.tsv` 的只追加照派发闸 ⑤ 不算撞，打补丁一侧同改；`--snapshot-ruled` 去掉 | `lib_snapshot_conflicts.py`；`write-guard.sh`（乙）；`apply-writer-patch.py` |

连带：6 种放行行各自的自证格、弄坏开关（`valve-reason-ignored`、`spec-check-skipped`、`writer-lock-valve-ignored`、`general-purpose-allowed`、`snapshot-valve-ignored`、`snapshot-valve-loose` 这类）照改法删或改；派发闸文件头 ⑥ ⑦ ⑧ ⑨ ⑫ ⑯ 的说明照改。六种之外那几处单次放行口记进新账本一百六十五节，另开一题。

交用户：

| 岔路 | 候选 | 依据 |
|---|---|---|
| 一：⑦ 限额窗口 | 甲 删掉那一支 / 乙 改判法 | 甲：窗口判错的代价是卡住到重置时刻（A1），漏判的代价是一次失败派发（攻方「没打中的形状」末条）；乙：A2 那一族照卡，没量  <!-- account-quota:allow 历史派发闸放行行与规则讨论 --> |
| 二：⑥ 快照冲突 | 甲 派发一侧按声明判 / 乙 挪到写的那一刻 | 甲：写回员自己起草时声明不全会漏（A6）；乙：改写闸，A5 那种合理例外按改动种类另判 |

## 回看决策

不涉及决策：这一轮判的是派发闸与打补丁脚本的放行行，`.claude/kb/decisions/` 里没有一条压在它们上面。

## 这一轮被判过的文件

- `.claude/hooks/runner-dispatch-guard.sh`
- `.claude/hooks/lib_snapshot_conflicts.py`
- `.claude/hooks/lib_writer_lock.py`
- `scripts/agents/dispatch-prompt.py`
- `scripts/gate/apply-writer-patch.py`
- `.claude/main-agent.md`
- `.claude/agent-common.md`

## 判决之后（2026-11-01）

用户答「三、怎么办」的弹窗：⑦ 那一问答「已经没有限额窗口了 现在走的统一都是task-server」；⑥ 那一问答「已经没有派发闸了 现在统一走的 task-server」；「何时改」那一问答「你查看目前项目的现状 重新审视你的修正 重新看哪些不需要改了 哪些需要废弃。 整个任务的分配和启动已经被task-server接管了。」随后又定「脚本不得绑定一台机器，必须所有机器都可以跑」。 <!-- account-quota:allow 历史派发闸放行行与规则讨论 -->

主 agent 现查与向做统一任务服务的会话核实（只问现状）的结果：task-server 只管任务（命令）的排队、挑机器、额度、卡、分片与结局，子 agent 的派发（Agent 工具）不经它；`.claude/settings.json` 的 PreToolUse `Agent|Task` 此刻仍注册 `.claude/hooks/runner-dispatch-guard.sh`，当天还拒过两边的派发；task-server 里没有限额窗口的概念。派发闸整份或部分退役，那个会话去问用户。 <!-- account-quota:allow 历史派发闸放行行与规则讨论 -->

照此重审「三、怎么办」：

| 闸 | 重审后 |
|---|---|
| ⑦ ⑥ ⑧ ⑨ ⑫ ⑯ | 不做。做统一任务服务的会话同日转述用户答它：派发闸「不动」，限额窗口那一支也不动；两条岔路撤，「三、怎么办」那张表里改派发闸的一律不做  <!-- account-quota:allow 历史派发闸放行行与规则讨论 --> |
| 打补丁脚本 `--snapshot-ruled` 与六种之外的同形放行口 | 不随派发闸走，照旧记在新账本一百六十五节，另开一题 |
| 腿的扫描脚本硬写家目录（`scan_valves_final.py`、`scribe_count.py`） | 违「脚本不得绑定一台机器」，只是腿的草稿、不进仓；这一轮的读数照旧作数（任务 2473、2474 在本机读的） |
