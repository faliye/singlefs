# m3-close-b19-core-r1：core 一簇的设计与攻击面（第一轮，2026-10-29）

## 一、题面

里程碑「删除与多次 COW」收尾（调度记录 `records/2026-10-18-里程碑三收尾与近期欠账.md` 批 19）还剩 core 一侧四笔要先定条款或先攻，实现员停在条款上交回。用户 2026-10-29 定「合成一轮三方推论定」。这一轮对每一格给出：候选（各自的定义）、推荐哪一个、推荐的依据与它最容易被打穿的地方、定了之后实现要动哪几处（文件与函数）、会红的用例或变异长什么样。

被判的条款与代码（快照 `research/prompts/m3-close-b19-core-r1-snapshot/`）：`.claude/kb/checks-owed.md` 的 C773、C724、C745、C874 各行；D16（发布语义）、D23（journal 的角色与格式） 的正文；`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/root_record.rs`、`crates/singlefs-core/src/allocator.rs`；池级 checker `crates/singlefs-checker/src/walk.rs`。

## 二、实现今天的样子（主 agent 的观测，2026-10-29）

方案按 `crates/` 今天的实现来谈。主工作区已打上批 19 第一波的补丁（P2 的在打，没验证），下列行号是此刻主工作区的：

- `crates/singlefs-core/src/root_record.rs:151` `PinnedRingSlot::of_the_lowest_pin`：根记录的钉槽 p 在盘上存的是钉值（txg）。
- `crates/singlefs-checker/src/walk.rs:9170` `pinned_ring_slot_of_the_root`：照「0–3S 之内一个钉值恰一个槽」把 p 反推成槽，钉值 0 只反推得出区域 0 槽 0。
- `mount.rs:3944`、`:3987` 两处 `.ok_or(RecoveryFailure::NoValidRoot)?`（C724）；同文件 `:8346` 另一处 `choose_root(…).ok_or(RecoveryFailure::NoValidRoot)?`。
- `mount.rs:6023` `Some(_gate_not_met) => return Err(device),`（补写闸没过时丢了原因，C874）；`mount.rs:6077` `rewrite_gate_not_met_by_a_device_without_any_self_verified_system_configuration`。
- `crates/singlefs-core/src/allocator.rs:2731` `record_root_written_by_this_process`、`:2830` `record_root_written_by_this_process_rereading_slots_bad_at_mount`（C745 要攻的换类）；写故障注入 `InjectedFault::WriteFails` 在 `crates/singlefs-harness/src/fault_injection.rs`。
- 实现员的现查（线索，主 agent 只核了上面几处行号）：P2 第一波报告 `~/.local/share/singlefs/work/m3-close-b19-p2-wave1/result.md`「停下交主 agent 的设计问题」第 1 条（C773 第 0 代根那一形量到的 (t, p)）；逐笔剩下的活见作业包报告 `~/.local/share/singlefs/work/m3-close-b19-packages/result.md`「第 1 步：逐笔表」。

## 三、格

### R1　C773：第 0 代根那一形的回收门槛钉
实现员量到：第 0 代根留在区域 1 槽 0，这一槽挂载时读坏或读回全零时，写行那次的根写 (t, p) = (1, 1)；同一份盘面没有读故障时是 (0, 255)，读故障反把门槛抬高一档。照条款应钉在 0，可 p 存的是钉值、checker 把 0 反推成区域 0 槽 0，候选修法那行变异写出 p = 0 坐实冲突。候选：甲 p 改存槽号、与钉值脱钩（动格式、checker、GPU 走读内核、D16 已定项 1 条文）；乙 写者钉 txg 0、p 仍存公式槽，checker 把槽 0 的钉值另认成 0（写者与 checker 各改一处、D16 条文补一句）；丙 认成已知结局（看得到的后果只有 I-7.18 对合法历史判红，记进条文）。判三者哪个对、各自在哪些盘面上会错。

### R2　C724：准入抬 F 上限里同名的 NoValidRoot 拒因
`mount.rs:3944`、`:3987` 两处要么加进 D23 已定项 14 的拒因单子，要么写明合法盘面走不到。判走不走得到（给出走得到的盘面或走不到的论证），推荐哪一种，变异第 5240 行在推荐下怎么红。

### R3　C745：挂载期间写故障下的换类
攻挂载期间写故障：换类之后表里旧记录被拿去顶上，或有一条写根环槽而不经 `record_root_written_by_this_process`、又不冻结的路径。攻出来的给出可写成 harness 用例的形态（注入点、期望）；攻不出的写清攻了哪些路径。

### R4　C874：补写那一支的可观测与崩溃流
补写闸哪一件没过，运行时要看得出（`mount.rs:6023` 今天丢了原因）：给出可观测输出的形态（错误成员带原因、还是观测记录一行），与补写中途崩溃的枚举流要罩住哪几个崩溃点；观察者「只判整写不判撕裂」那行变异放在哪。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 正推腿（Sonnet） | R1、R2、R4 | 从条款与 `crates/` 今天的代码正推推荐候选，给出实现落点与会红的用例 |
| 攻方腿（Opus） | R1–R4 | 找推荐候选的反例；R3 主攻：在今天的代码里找挂载期间写故障下换类出错的路径 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数；引代码写开工快照里的路径与行号。
- 方案按 `crates/` 今天的实现来谈；要建小模型核的，模型放腿自己的目录（`~/.local/share/singlefs/work/<腿名>/`），不放会话共用的暂存目录，不在主工作区改。
- 不跑重型测试、不起 checker 档的 cargo、不碰 GPU；小模型与 harness 探针经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 4G 的上限。
- 报告写进 `research/prompts/m3-close-b19-core-r1-<模型>-output.md`，分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
