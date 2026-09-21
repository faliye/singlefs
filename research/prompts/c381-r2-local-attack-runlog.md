# 运行记录：c381-r2-local-attack（2026-09-20 UTC）

提示文件：`research/prompts/c381-r2-local-attack.md`（英文，覆盖正文第七节 K3 全部、K2 里丙与戊两行；三张事实表加五道问题，不用任何 markdown 强调，指路一律用事实编号 / P1–P8 / T1–T4 / 臂名，不写代码行号或文件行号）。
转述核对表：`research/prompts/c381-r2-local-attack-translation-audit.md`（15 条事实逐句核对，另加本轮自己的问题框架（失败点、阶段、两条臂定义）三行核对；6 处首稿有出入已改回或补回、5 处有意不译并写明为什么、3 处英文多出的限定词逐条写明为什么加）。
调用方式：`nice -n 19 bash research/scripts/ask-local.sh research/prompts/c381-r2-local-attack.md`，前台跑，未用 `setsid` / `&` / `disown`。
起跑前 `ps -o pid,etimes,args -u "$(id -u)"` 查过：命中 `gate.sh --staged`（另一个会话）与 `cargo test --all` / `cargo build --release`（另一个会话），未命中 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`，不是性能测量在跑，不需要等。

## 调用记录

| 次序 | 目标文件 | 退出码 | 词数（`wc -w`） | oov-check.py 判定 | corruption-check.py 判定 | 结论 |
|---|---|---|---|---|---|---|
| 1 | `c381-r2-local-attack-output-s1.md` | 3 | 0（0 字节空文件） | 未跑（网关报错在正文生成之前，脚本没有正文可检） | 未跑，同上 | 本次调用不算样本、也不算作废副本，见下 |

第 1 次调用的原样报错（stderr，完整摘抄前 400 字符之外全文过长，这里给完整 JSON 的结构性摘要，`message`/`type`/`code`/`diagnoses` 字段逐字保留，`partial_content` 因过长只保留其结构不再重复抄录一遍——原样 JSON 已经完整落在上面这条 Bash 调用的工具输出里，此处摘录用于运行记录自身的可读性）：

```
ask-local: 网关报错 {"message": "输出预算不足被截断（已用 0 tok）——该调的是 max_tokens，不是模型或 prompt", "type": "truncated", "code": "truncated", "session_id": "e5038d33-2910-478a-949d-64db28850bbf", "attempts": 3, "diagnoses": ["第1次：正文复读跑飞，已掐断上游（判定时正文 6504 字符）", "第2次：正文复读跑飞，已掐断上游（判定时正文 6001 字符）", "第3次：正文复读跑飞，已掐断上游（判定时正文 6000 字符）"], "partial_content": "（约 6000 字符的表1部分内容，覆盖到 P5 C6 附近，格式与提示要求的表1一致：失败点、条款、判词、引原文一句，未见明显字词损坏，但这份内容从未走过 corruption-check.py / oov-check.py，也从未完整落盘，不当样本用）"}
```

`c381-r2-local-attack-output-s1.md` 是重定向建出的 0 字节空文件（`ls -la` 与 `wc -c` 都确认为 0），不是脚本自己判红留下的 `-output-void*.md`（脚本的作废副本逻辑只在字词损坏闸判红、也就是退出码 5 那条分支里触发；这次是网关在正文生成阶段就报错退出，退出码 3，走的是另一条分支，从没到过字词损坏闸，也没有到过 `cat "$TXT"` 那一步，所以 stdout 是空的，`$dir/$base-output-void*.md` 也没有产生）。这一份与 `research/prompts/c381-r1-local-attack-runlog.md` 记录的第 2 次调用同一种失败（网关判定正文复读跑飞、掐断、退出码 3），但这一次是本轮第 1 次调用，不是第 2 次。

## 停止的理由

退出码 3，既不是 0 也不是 5。按派发提示第 4 条「退出码非 0 非 5、或网关不通：停下，报『本地腿缺席』」与 `.claude/rules/three-way-inference.md`「本地腿缺席时必须显式报告」一节，这里不重试、不换措辞重跑、也不把 `partial_content` 里那约 6000 字符的部分正文当作样本使用或分析——那是网关自己判定为「复读跑飞」而掐断的产物，从没有完整生成过，也从没有经过字词损坏检查。是否要在这个提示上继续调用、要不要改动提示措辞或分段拆问，由主 agent 决定；本条腿当场停止。

## 干净样本清点

零份干净样本。一次调用，退出码 3，触发停止条款，不计入「连续五次调用（判红作废的也算）拿不到两份就停」的计数——那一条只数退出码 5 的判红作废，这次是网关错误，走的是另一条更早的停止条款。

## 没做什么

- 不解读、不总结、不采纳 `partial_content` 里那部分未完整生成的答案内容，不判它是否打中 K3 或 K2——那既不是完整样本，也不是主 agent 的判断该由本地腿代做的事。
- 没有重跑第 2 次调用去争取一份干净样本：按停止条款，退出码非 0 非 5 时当场停，不重试、不改措辞。
- 没有跑云端攻方腿、云端辩方腿或核实员，不在这条腿的任务范围内。
- 没有对 `c381-r2-local-attack-output-s1.md` 这个 0 字节空文件做任何改名或归档处理（不同于 `c381-r1-local-attack-runlog.md` 记录的做法——那一次改名是主 agent 中途明确指示的，这一轮到停止为止还没有收到同类指示）。

## 追加（2026-09-20，按主 agent 指示拆成三份重跑）

拆分理由、拆法、逐字核对结果、两处结构性改动（框架说明、悬空引用修补、表2自足化改写）都写在 `research/prompts/c381-r2-local-attack-translation-audit.md` 的追加一节，这里不重复。原提示 `research/prompts/c381-r2-local-attack.md` 未再改动。

三份新提示：`research/prompts/c381-r2-local-attack-part1.md`（表1前半，P1–P4×C1–C6）、`-part2.md`（表1后半，P5–P8×C1–C6，另加P8/N_switch追问）、`-part3.md`（表2 K3判臂 + 表3 K2丙戊两行）。

每次调用前都用 `ps -o pid,etimes,pcpu,args -u "$(id -u)"` 查过 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`，全程零命中，判定不是性能测量在跑；命中的都是别的会话的 `cargo test`/`cargo build`/`gate.sh` 与其他 agent 进程，按共用约束「只有别的cargo、gate.sh在跑的，加nice -n 19照常跑」处理，全程 `nice -n 19` 跑 `ask-local.sh`。

### part1（P1–P4 × C1–C6）

| 次序 | 目标文件 | 退出码 | 词数 | oov-check.py | corruption-check.py | 结论 |
|---|---|---|---|---|---|---|
| 1 | `c381-r2-local-attack-part1-output-s1.md` | 0 | 682 | 绿，生词=4（checkpoint'，非拼接，所有格撇号被切开）拼接=0 | 绿，全部计数为0 | 干净，但格式上四个失败点（P1–P4）给出的六格判词逐字重复、且未显式标出是哪个P/哪个C（只按顺序排列），格式不合要求、不是字词损坏，按定义这类判断留给主agent |
| 2 | `c381-r2-local-attack-part1-output-s2.md` | 0 | 540 | 绿，生词=4（checkpoint's）拼接=0 | 绿，全部计数为0 | 干净，逐格标出「P_ C_」，与 s1 内容不完全一致（例如 C4–C6 列 s1 判「inside scope」、s2 判「outside scope」），两次判词不一致这件事本身不由本地腿下结论 |

两份都通过手工扫描（`\b(\w+)\s+\1\b` 相邻重复词、` [,.;:] ` 孤立标点两项，脚本核过，均 0 命中）。两份干净样本达标，part1 到此为止，不再抽样。

### part2（P5–P8 × C1–C6，另加P8/N_switch追问）

| 次序 | 目标文件 | 退出码 | 词数 | oov-check.py | corruption-check.py | 结论 |
|---|---|---|---|---|---|---|
| 1 | `c381-r2-local-attack-part2-output-s1.md` | 0 | 67 | 绿，生词=0 拼接=0 | 灰，「语料太小（cjk=0 words=67），判不了」 | 记带损坏：只答了问题2（约67词），问题1要求的24格表（P5–P8×C1–C6）整个缺失，不是字词内部损坏，但按「缺词、断句这类损坏……同样记带损坏」的精神，整份要求的内容大段缺席按最保守读法记带损坏，不计入干净样本 |
| 2 | `c381-r2-local-attack-part2-output-s2.md` | 0 | 830 | 绿，生词=4（checkpoint's）拼接=0 | 绿，全部计数为0 | 干净，问题1、问题2都完整作答，24格逐格标「P_ C_:」 |
| 3 | `c381-r2-local-attack-part2-output-s3.md` | 0 | 1442 | 绿，生词=5（checkpoint's advancement）拼接=0 | 绿，全部计数为0 | 干净，问题1、问题2都完整作答，24格逐格标「P_ C_」，每格都带独立的「This would be refuted by」句 |

三份都通过手工扫描（相邻重复词、孤立标点两项，脚本核过，均 0 命中）。第 1 次带损坏（内容大段缺席）、第 2、3 次干净，两份干净样本（s2、s3）达标，part2 到此为止，不再抽样。

### part3（表2 K3判臂 + 表3 K2丙戊两行）

| 次序 | 目标文件 | 退出码 | 词数 | oov-check.py | corruption-check.py | 结论 |
|---|---|---|---|---|---|---|
| 1 | `c381-r2-local-attack-part3-output-s1.md` | 3 | 0（0字节空文件） | 未跑（网关报错在正文生成之前） | 未跑，同上 | 本次调用不算样本、也不算作废副本 |

第 1 次调用的原样报错（stderr 完整摘抄）：

```
ask-local: 网关报错 {"message": "输出预算不足被截断（已用 0 tok）——该调的是 max_tokens，不是模型或 prompt", "type": "truncated", "code": "truncated", "session_id": "7e6fe358-9cdb-431d-ab02-ce3d45437425", "attempts": 3, "diagnoses": ["第1次：正文复读跑飞，已掐断上游（判定时正文 6000 字符）", "第2次：正文复读跑飞，已掐断上游（判定时正文 6004 字符）", "第3次：正文复读跑飞，已掐断上游（判定时正文 8028 字符）"], "partial_content": "（约8000字符，覆盖丙/戊两臂在P1–P3附近的表2内容，格式与提示要求一致，逐格给出四步答案，但从未完整生成、也从未经过corruption-check.py/oov-check.py，不当样本用）"}
```

`c381-r2-local-attack-part3-output-s1.md` 是重定向建出的 0 字节空文件（`ls -la`、`wc -c` 确认为 0），已按 c381-r1 与本轮 part 拆分前那次的同一惯例改名为 `c381-r2-local-attack-part3-output-void-gateway1.md`（`mv -n`，未删除）。

**停止的理由**：退出码 3，非 0 非 5。按派发提示「退出码非0非5、或网关不通：停下，报『本地腿缺席』」，这里不重试、不换措辞。这次 `partial_content` 里约 8000 字符覆盖到「arm TABLE, P3」附近，比原来整份提示第一次调用掐断的位置（约6000字符，停在表1 P5附近）更靠后，但仍然是网关判「正文复读跑飞」后强制掐断，不是完整正文。

**新发现，需要主 agent 决定下一步**：part3 单独一份（表2 16行×4小项 + 表3 8行×4小项 + 3道问题）本身仍然超出本地模型一次答得完的输出预算，与原整份提示同一种失败模式（退出码3、网关判「复读跑飞」）。这不是本条腿能自行解决的——是否要把 part3 再拆成两份（例如「part3a：表2」「part3b：表3」），或者精简表2任务里为了自足化而新增的第一步（重新判一次C1那一列，这一步是本轮追加通知里让我为了避免跨调用依赖而加的，见核对表追加一节），由主agent定。本条腿已按当前的三份切法把 part1、part2 跑完并各自拿到两份干净样本；part3 按当前切法零份干净样本，一次调用后触发停止条款，当场停止，不做进一步拆分或精简的决定。

## 干净样本清点（三份合计）

- part1：2 份干净（s1、s2）。
- part2：2 份干净（s2、s3），另有 1 次带损坏（s1，内容大段缺失，不计入干净数）。
- part3：0 份干净；1 次调用，退出码 3，触发停止条款，当场停止。

## 没做什么（追加部分）

- 未对 part3 做进一步拆分或改写去规避这次的输出预算超限——那是结构性改动，按共用约束需要主agent或协调者决定，不由本条腿自行拍板。
- 未采用 part3 第 1 次调用 `partial_content` 里约 8000 字符的未完成内容作为样本或分析依据。
- 未重跑 part3 第 2 次调用：按停止条款，退出码非 0 非 5 时当场停，不重试。
- 未对 part1、part2 两次判词不一致（例如 part1 的 s1/s2 在 C4–C6 列判词相反、part2 的 s1 完全跳过表1）做「哪次对」的判断，也未据此推断哪条臂打没打中——那些是主 agent 的事。

## 追加（2026-09-21，接替腿，续跑 part3a-1/part3a-2/part3b）

本段由接替腿写入。上一条本地攻方腿（子 agent aa1e6dabb11b999c7）2026-09-20T19:49:23.916Z 撞会话限额中断，中断点在刚发出 `nice -n 19 bash research/scripts/ask-local.sh research/prompts/c381-r2-local-attack-part3a-1.md > research/prompts/c381-r2-local-attack-part3a-1-output-s1.md` 这次调用之后（该次调用的产物已落盘，退出码 0，5665 字节，但上一条腿本身没来得及跑两道闸就中断）。交接摘要：`research/prompts/c381-r2-local-attack-handover.md`。

第一步核现场（本段接替腿做的）：先 `ps -o pid,etimes,pcpu,args -u "$(id -u)"` 查负载，`qemu-system`/`vm-bench.sh`/`e152-file-system-benchmark`/`fio` 零命中；随后对已落盘的 `c381-r2-local-attack-part3a-1-output-s1.md` 补跑 `oov-check.py`（绿，生词=0 拼接=0）与 `corruption-check.py`（绿，全部计数为0），另手工扫描相邻重复词与孤立标点（均0命中），通读全文确认8行×5字段（Scope/Justification/Stage/Justification/Action/Consistency/Justification，外加问题2的 no+refuted-by）齐全、无缺词断句，判定：干净，沿用为 part3a-1 的 s1，不重跑。

以下每次调用前都重新 `ps` 查一次负载，全程零命中 qemu-system/vm-bench.sh/e152/fio，全程 `nice -n 19` 跑；前台跑 `ask-local.sh`（未用 setsid/&/disown；工具因单次命令超过120秒自动把命令移入后台执行，跑完前台等待通知，非本条腿主动挂起）。

### part3a-1（表2的P1–P4×两臂，8行）

| 次序 | 目标文件 | 退出码 | 词数 | oov-check.py | corruption-check.py | 手工扫描 | 结论 |
|---|---|---|---|---|---|---|---|
| 1 | `c381-r2-local-attack-part3a-1-output-s1.md` | 0 | 918 | 绿，生词=0 拼接=0 | 绿，全部计数为0 | 相邻重复词0、孤立标点0 | 干净（上一条腿留下的产物，本条腿补跑两道闸后确认） |
| 2 | `c381-r2-local-attack-part3a-1-output-s2.md` | 0 | 1004 | 绿，生词=4（inapplicable，非拼接，词表外但成词） | 绿，全部计数为0 | 相邻重复词0、孤立标点0 | 干净 |

通读两份，8行×每行4小项（Scope/Stage/Action/Consistency，各带Justification）齐全，无缺词断句；已够两份干净样本，part3a-1 到此为止，不再抽样。

### part3a-2（表2的P5–P8×两臂，8行）

| 次序 | 目标文件 | 退出码 | 词数 | oov-check.py | corruption-check.py | 手工扫描 | 结论 |
|---|---|---|---|---|---|---|---|
| 1 | `c381-r2-local-attack-part3a-2-output-s1.md` | 0 | 1279 | 绿，生词=0 拼接=0 | 绿，全部计数为0 | 相邻重复词0、孤立标点0 | 干净 |
| 2 | `c381-r2-local-attack-part3a-2-output-s2.md` | 0 | 1356 | 绿，生词=0 拼接=0 | 绿，全部计数为0 | 相邻重复词0、孤立标点0 | 干净 |

通读两份，8行×每行4小项齐全，无缺词断句；已够两份干净样本，part3a-2 到此为止，不再抽样。

### part3b（表3，丙戊×T1–T4，8行）

| 次序 | 目标文件 | 退出码 | 词数 | oov-check.py | corruption-check.py | 手工扫描 | 结论 |
|---|---|---|---|---|---|---|---|
| 1 | `c381-r2-local-attack-part3b-output-s1.md` | 0 | 657 | 绿，生词=0 拼接=0 | 绿，全部计数为0 | 相邻重复词0、孤立标点0 | 干净 |
| 2 | `c381-r2-local-attack-part3b-output-s2.md` | 0 | 1291 | 绿，生词=5（attempt's recovery's，均为所有格撇号被切开，非拼接） | 绿，全部计数为0 | 相邻重复词0、孤立标点0 | 干净 |

通读两份，8行（Arm STOP/TABLE × T1–T4）× 每行4–5字段（Caller receives/Recovery/Match/Fact，各带refuted-by）齐全，无缺词断句；已够两份干净样本，part3b 到此为止，不再抽样；提示未撞输出预算，未再需要拆成 part3b-1/part3b-2。

## 干净样本清点（本条腿续做的三份合计）

- part3a-1：2 份干净（s1、s2）。
- part3a-2：2 份干净（s1、s2）。
- part3b：2 份干净（s1、s2）。

至此，分给本地攻方的全部格（K3：表1 P1–P8×C1–C6 已由 part1/part2 各拿到2份干净、表2 丙戊×P1–P8 已由 part3a-1/part3a-2 各拿到2份干净；K2 丙戊两行：表3 丙戊×T1–T4 已由 part3b 拿到2份干净）都已达到「至少两份干净样本」的要求，没有还空着的格。

## 没做什么（本段追加）

- 未解读、未采纳任何样本里对K3/K2判断的具体内容或对错（含part3a-1/a-2里丙戊两臂在consistency一栏判的consistent/conflict/not applicable、part3b里caller receive与recovery是否match）——那些是主agent的事。
- 未跑云端攻方腿、云端辩方腿或核实员，不在这条腿的任务范围内。
- 未对part1、part2历史遗留的两处不确定（part1两次C4–C6判词不一致、part2 s1只答追问题不计入干净数）做进一步核实或重跑——按现场核实，那两份已达标（各2份干净），派发提示写明「不重做」。
