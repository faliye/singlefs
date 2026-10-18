# 回扫员报的「要主 agent 现查」（逐条判去向）

## 乙二甲（E131–E155）
- 认领命令报草稿目录不在工作根下一层（第一波五个都这样）：第二波改成顶层目录；第一波的草稿目录没认领、只写了草稿，收尾时主 agent 自己删。
- E142 页写装置 `e142_new_pool_file_creation_dry_run.rs` 11081 行，现仓 11083（HEAD 也是 11083）→ 批 5 改成现数（写法改成给数的命令）。
- E139 页「影响的决策」三行长回看被压成只改措辞；E142 页约 40 行回看同样 → 逐行判阶段看那几行。
- C660 新旧题目含义不同，F213 带出的每一处要看说的是旧题目还是新状态 → 逐行判提示里点名。

## 乙二乙（E156–E164）
- E164 页第 158 行 W5 仍写「省不下」、没带第 197 行新加的限定；`experiments.md` 第 186 行 E164 索引行同样没带、也没提 D13 已定项 17 的支撑关系 → 逐行判阶段看；索引行归批 5。
- E156 页首与 `experiments.md` 第 178 行写「17 单测」、E156 第 187 行 `17 passed`、第 177 行 `19 passed` 三种口径并存 → 批 5 现查对应哪一次重跑。
- `git show 5d0b7e7c1667:…out 第 N 行`、`git show dbcc0d2f:…/probe_crash_states.rs` 指针没核 → 批 5 抽核。

## 乙一（实验变更史与 E1–E129）
- E94 第 7 行与第 100 行仍写 C34「到还没收口」，而 C34 已销账、E57 已改作废 → 逐行判 F115 盯住；若候选没罩到归批 5。
- 回看格重写丢了经过（E102 四条「2026-10-04 改了…」换成「不受影响：只改措辞」；E49 三条合成一条 10-15 指针）→ 批 5 查决策变更史里还在不在；不在的记进欠账或补回。
- 索引里「今天」钉成具体日期的几处（E145、E156、E163、E165、E170）那天的状态今天是否仍成立 → 批 5 现查。
- F121–F124（层 0、第一条流、invariants.md 路径、`../mutate.sh`）写成出候选的行，用来搜残余旧名：留着。

## 甲（决策）
- D5 第 77 行仍写「付出的是 key 数量上升（每个统计量 × 每一代各占一条）」，D8 与 D5 已定项 2 已对齐成整批换代（H12）→ 候选 F10 里有，逐行判。
- D18 已定项 11 的前提「独占打开池中过半的设备」C833 说没做，索引行仍「进度：已实现」、欠块没列 C833 → 进度值要不要降，交用户定（弹窗清单）。
- C635、C645 题面与 `.claude/main-agent.md` 第 10 行仍写「动了崩溃点重放或判器」，D13 第 17 行索引与第 256 行已改成按判器单元现算 → 候选 F36 里有；main-agent 那句归批 5（定义改动走豁免登记）。
- `--changes` 看不到纯新增的事实（进度列）→ 工具缺口，记进调度记录；别的片可能同样漏，批 3 合表时主 agent 按 `git diff --stat` 抽看纯新增多的载体。
- 「D13 已定项数 15 → 17」没单独写行 → 批 3 主 agent 补一行（检索词 `D13[^。]{0,12}15 项|十五项`）。

## 乙二丙（E165–E178）
- E170、E173、E176、E177、E178「影响的决策」表里基准版 6 个格「改了：依据段新引这个实验…」被换成「不受影响：只改措辞」，「改了」的记录没了（决策页仍引着）→ 批 5：与乙一报的 E102、E49 同类，查变更史，决定补回或记欠账。
- E165 页 restored 与 verdict 两行写 `git show 5d0b7e7c1667:…` 指针，30c245b0 已把产物放回 `research/results/`，指针与文件名并存 → 批 5 改成指文件名。
- 旧路径 `../mutate.sh` 在 E108（6 行）、E92（5 行）、records/2026-10-11-里程碑三待优化上下游协调.md（1 行）→ 批 5 改成 `scripts/mutation/mutate.sh`（路径不冻结）。

## 丙甲（欠账表前两段）
- 已还清表第 735–742 行 8 笔（C626、C627、C601、C607、C608、C620、C674、C676）「怎么还的」一格只写「见 Cnnn 节」占位句（10-11 治理批 12 抄调查员拟写句子时漏换）→ 批 5 主 agent 按现行证据重写那一格（证据：各笔对应的 crates/mutations.tsv 行或 checker 档用例，现查）。
- C169、C285、C421、C505 四行整行删掉（字词损坏检测器与本地模型腿删除），已还清表没留编号；`decisions-history.md` 第 8037、8038 行引 C285，`records/2026-09-23-三轮三方与四类没东西会红.md` 第 41 行引 C505 → 批 5：补进已还清表记「作废：对象已删」四行；引用处是历史与记录，不动。
- C629、C548 的挪动在 records 里没有对应一条 → 不追（记录不是必须逐笔写）。
- C842 在欠的表里一格开头写「已还这一半」，table-shape 格认不认没验 → 批 5 跑 doc-registries 的 table-shape 格看。
- 工具缺口：CARRIER_PATTERNS 不含不变量目录（已记进调度记录）。

## 丙乙（欠账第三段、不变量、布局、里程碑一二）
- 数打架：`milestone/02-second-txn.md:128`「已承诺预留仍是第一个文件版本那次的 98 386」，`layout/01-first-txn.md:304` 与 `layout/03-third-txn.md:22` 已改成 196 760 → 候选里应有（F633 一类），逐行判；批 5 以代码常量现查为准。
- E170、E171、E94、`prior-art.md` 还写旧文件名 `invariants.md` → 批 5 改成新目录（路径不冻结）。
- F609（去向列，184 行）命中多是新写的欠账行本身，收益低 → 批 3 合表时整行删掉，不出候选。

## 丁甲（定义与规则）
- `.claude/agents/implementation-writer.md` 第 41 行「受影响的层 0 流」旧名没改净 → 批 5（定义改动按用户 10-11 定的豁免登记）。
- 21 个实验页复跑块里的 `mutations/e<号>_….tsv` 相对路径目录不存在，表在 `scripts/mutation/tables/` → 批 5 全仓改（路径不冻结），先核 replay.sh 登记表读的是哪一列。
- prior-art 新增「十、快照下的死块记账」整节、三方材料检查新增退出码 7、8 是纯新增 → 不出候选，不处理。

## 丙丙（里程碑 03–05）
- 03 待优化第 15 行「写者锁那一半没做」，`lib_writer_lock.py` 与 agent-common 开工认锁已有 → 逐行判应抓到；批 5 现查后改（别的会话正在改 `.claude/hooks/lib_writer_lock.py`，未暂存，改之前看它状态）。
- `milestone/01-first-txn.md` 第 4 行、`02-second-txn.md` 第 8 行「文末历史版本记一行」与 changelog-format、history-carriers 不符 → 批 5 改。
- 五·8：`crates/singlefs-checker/src/walk.rs` 第 8510 行 C314 旧简称 → crates 注释，归下一批改 crates 的清单（不在这一轮）。
- 五·3 的 E179 编号两处题目不一致（项五收口记录「覆盖写释放回退复用的干跑」，代码欠账批「预演内存随删除规模」），`experiments/` 只到 178，`research/prompts/e179-preregistration.md` 未跟踪 → 批 5 现查 e179 登记是哪一个，另一处改掉或取新号。
- 03、04 带日期的「现查」现状快照 → 逐行判。
- C58 在 checks-owed 里的去向要跟 06 走 → 逐行判 / 批 5。

## 丁乙（README、记录、skill）
- `records/2026-10-14-全量前修红.md` 是 0 字节空文件，只有 111f7dc6 一次提交 → 批 5 问写它的会话（跑全量的那个）要内容或删掉；弹窗清单。
- `scripts/gate/lib_reuse_checks.py` 没在 `scripts/gate/README.md` 里提到 → scripts README 归 tooling-writer 下一批；记欠账行。
- `tooling.md` 说本地模型腿脚本已删，`decisions-history.md`、E34、多份 records 还把 `ask-local.sh` 当现有脚本 → 候选 F916 里有，逐行判。
- 反向核没记的事：unit-graph（F915）、harness 新模块（F954）、四张门禁登记表（F955）、`.claude/heavy-thread-default` 在 CLAUDE.md / tooling.md 没记（F945）→ 批 5 补。

## S05 逐行判
- `records/2026-10-11-崩溃放量两段与脚本审计.md:51` 说 `--start-over` 换成新库，门禁脚本文件头写「照键写回同一份长期库」→ 批 5：记录是当时的说法，核门禁脚本现状后判记录那句是不是事件句。
- `records/2026-10-13-复用键与全仓改名.md` 含非法 UTF-8 字节（偏移 395168）→ 批 5 修（别的会话未暂存改动在这份文件上，先看它状态）。
- E171 页第 11 行「现状」写 O3 用 E170 第 1 次重跑冻结的执行器，`13-验证路线.md:183` 与 E170 页写 E171 第 1 次重跑用 v2-487 → 批 5 现查哪个对。
- 13 条崩溃枚举用例：`checks-owed.md:369`、`milestone/03-third-txn.md:139` → 批 5（已在 fixes-main）。

## S02 逐行判
- E166 模型常量仍是退役树加进之前的 65548 / 12 个号，仓里没对应欠账 → 批 5 立账（同 C774 一类，装置没追平退役树）。
- milestone/02 各步「现状（日期）」段判为现状句（第 121 行），有争议 → 批 5 主 agent 判：照 kb-discipline 第 8 条删日期读一遍。

## S04 逐行判
- D13 已定项 16 进度标记 HEAD 是「已实现」、10-16 被改成「未实现」，S04 读代码判应为「已实现」→ 批 5：问改它的会话为什么降（可能是别的缺口），不自行翻。
- `implementation-writer.md:41`「受影响的层 0 流」是 `scripts/gate/apply-writer-patch.py:78` 认的字面串、已按行豁免 → 改要同改脚本与自证，归 tooling-writer 下一批；记欠账。
- `records/2026-10-13-复用键与全仓改名.md:864` 孤立字节 0xEF 让 `sweep-term.py --check` 退 2 → 门禁 term-renames 格此刻红；批 5 修那一个字节（HEAD 那版是好的，先看是哪个会话改的）。
- `term-renames.md:25`「层 0」档豁免类已没有对象 → 批 5 收口一句。

## S01 逐行判
- D18:321「同一时刻只有一个可写挂载——由过半独占打开保证」与 C833（没做）矛盾；D18 已定项 11 进度「已实现」→ 并进甲报的同一条，交用户。
- milestone/02 号文件第 337 行 增补 2 收口表「C370、C371 同一个没定义的前提」，C370 单位一半已定 → 批 5 改成只剩 C371 那一半。
- `records/2026-10-01-删除与并发事务.md` :137、:201、:218、:246、:343 收拢表写「在跑」「开着」「等…再派」，判了事件句（标题带日期）→ 主 agent 判：收拢表是那天的快照，留；:343「第二段仍没跑」仍真，不动。
- `write-frequency.tsv:48` 仍「随每次发布重写」→ 已记在 10-11 待办，归 tooling-writer 下一批。
- C833 的里程碑写法不一：事实表 F3 与代码欠账批记录「里程碑十一·2」，checks-owed C833「里程碑十的一·2」（milestone/10 第 40 行）→ 批 5 以 milestone 文件现查定，改记录或欠账行。
- I-7.5 已实现日期三种写法（10-01 / 10-02）→ 批 5 以变更史为准统一。

## S06 逐行判
- milestone/02 号文件第 78 行、:166 把已关账的 C335 列作「要它」「会碰到的决策点」（计划清单）→ 批 5：加「（已还清，并入 C597）」括注，不删。
- D23 第 392 行只写了 10-15 的「同一指针两份都更新」，没写 10-17 用户定的「比 fsid」→ 是别的会话的待写回（规格在 `~/.local/share/singlefs/work/c834-writeback-spec/`），不归这一轮；收尾时提醒。

## S10 逐行判
- E142 页首「11081 行」：页上 r25 一节写冻结版 11081 行、sha 前缀 352ac898，现行 11083 → 批 5：页首改成「以 r25 冻结版为准（11081 行，sha 352ac898…）；现行源码已有 2 行改动（暂存区）」或改成命令取数；主 agent 定写法。
- C774 简称「E142 装置没追平退役树」实际只剩「第 26 次登记」→ 批 5 改简称（relabel-item.py）；里程碑二 :393、:596「门禁改法之后待重跑」r25 之后 QEMU 门禁重没重跑仓里没记录 → 收尾交用户。
- `crates/singlefs-core/src/make_filesystem.rs:3` 注释 `12+1+1+1+4` → 代码注释，归下一批改 crates 的清单（已记在 10-11 崩溃放量记录 :79）。

## S08 逐行判
- E159 页 :42 的指针 `name=` 为空，指到的行是 `E7RESULT E159RESULT arm=…` 没有 `name=` → 批 5 改成那一行的写法；门禁判指针这一格还没立（C837 仍欠那一半）。
- `checks-owed.md:404`（C609）写「E165 钉表自己还没整行抄：C627」，C627 已还清 → 批 5 改。

## S09 逐行判
- E174 页第 41–46 行的 `run` 命令：装置文件头写 `run` 一律经 `device-code-key.py run` 起，没有 `DEVICE_CODE_KEY_DEVICE_BINARY` 会退 3 → 批 5 改那几条命令（与 F105 同一件事）。
- D16:29「装置的候选下界还没带系统配置里的 F」：E139 第 522 行说第 2 次重跑上已过时；S03 对 D16 已定项 1 定案段同句判了要改 → 并进 S03 那一处，决策正文改动归写回员、交三方或用户定。
- C532（checks-owed:355）写「判 exact 的 230 行」，现 237（HEAD 也是 237）→ 批 5 改成现数。

## S11 逐行判
- 「整行抄自产物」标题下已换成 `git show` 指针行的页有 13 页（S08 核过 360 条指针 359 条指得到）；`experiment-runner.md:43` 仍要求整行抄，规则、定义、门禁里 0 处登记指针写法 → 交用户：产物归档出工作区后的指针写法要不要写进定义与 `evidence-discipline`（上游）；标题的括注（S08、S11 各 20 与 17 行要改）主 agent 批 5 统一加「（产物已移出工作区的行写成一行指针）」。
- 候选没罩到的同类：E171 页 :291「整行抄自 s1 产物」、E172 页 :234、:251 → 批 5 同样加括注。
- C308 / E142 G4 行「字段表偏移列已与 D22 已定项 7 相同」，layout/01 字段表只有宽度列没有偏移列 → 批 5 把「偏移列」改成「宽度与行序」。

## S12 逐行判
- `experiments.md:190` E168 索引行「九份」产物：git 里只有 positive-controls 与 s1-history 两份，segment1.out 115 MB 被 .gitignore 忽略 → 批 5 改成实数并写明哪几份未入库。
- E165 页 `J6Qn`（79 处）保留大写、`j6` 小写 → 口径：`J6Qn` 是另一形，不动。
- `scripts/experiments/e142_…dry_run.rs:5657` 字符串「层0」→ 归改名那一线（crates/scripts 不在这一轮）。

## S14 逐行判
- C709（checks-owed:472）前置「不挡里程碑三」与去向「里程碑三二·5」矛盾，milestone/03 没点名 C709 → 交用户定一边。
- C635 去向「并入 C20」读反了（是 C20 并进 C635）→ 批 5 改成「含并入自 C20 的按套布局标记」。
- E34:209 命令指已删的 `ask-local.sh` → 批 5 补一句「脚本已删，取回法见 tooling.md」。
- `.claude/gate.d/doc-process-records.sh:57` 注释、`.claude/batch-scope:161-162` 还登记 `ask-local.sh` → 归 tooling-writer 下一批。

## S13 逐行判
- C692（checks-owed:753）还清方式末尾「另见 C692 节：GPU 恢复内核还写死 `< 10u`」节不存在，`crash_judge_common.wgsl:3455` 仍有 `first_device_count < 10u` → 批 5：另立一笔欠账（GPU 恢复内核盘数写死 10）、删那句尾巴。
- E142 页 G 表落点列 5 行按惯例判要改（加「已还清」）→ 批 5 照 G2、G7、G16 惯例加。
- M02:393、:596 与 M03:38 与 S10 重叠 → 批 5 合并改法时叠在同一行上。
- F501 检索词 `\bC(1)（` 撞 E158 的「H-C1（丙）」→ 已判不相干，不重出表。

## S15 逐行判
- C618 去向写里程碑十一，`10-tenth-txn.md` 出口 2 与一·3 把它当里程碑十自己要还（「提前到这里」，10-13 规划晚于去向 10-12）→ 批 5 以里程碑文件为准改去向成里程碑十。
- D8:207「挡着新池新建文件写的九样」把 C71、C117 当前置，无日期；那个里程碑已出口 → 决策正文，交写回员改成「当时挡着（里程碑一）」。
- E142 G7 行写「C284 已还清」，checks-owed:218 C284 仍开着 → 批 5 现查变更史定哪边对。
- C376 前置引「收口表第 16 行」实际讲 C334 → 批 5 改指第 6 行。

## S17 逐行判
- D9:300 已定项 14 射程「`PointerHead::read_from`…两条没有代码在判」与 C406 第 4 格「三判已落在 `PointerHeadAsRead::judged`」冲突，正文没 C406 字样不在候选 → 决策射程句，交写回员（与 :70、:306 同一处改）。
- checks-owed:238（C314）第 4 格「还清那天一起改：milestone/01 步 1 / 步 6 的 txg 与 jsn 起点…E142 重跑」是否已改过 → 批 5 现查，改过就删那一串。

## S16 逐行判
- E92 第 24、75、140 行「C88 … 未实现，需求未知 / 消费者未实现」：C88 新写法是择新平局按实例代号已有、只欠 checker 一侧；「时间线判别」指的是不是 checker 那一侧 → 批 5 读 E92 那三处定。
- E16:307「C6 还要求一个显式的在飞记录数上限」：C6 行没有这一条，只在 C42 行命中 → 批 5 改成指 C42。

## S18 逐行判
- D18:366「那一步仍开着，记在 C163」：C163 现行行已写输入端解掉、只剩会红的检查 → 决策正文，交写回员改成「只欠会红的检查（C163）」，核 D18 已定项 12。
- milestone/03 号文件第 106 行 五·2 作业点（改写 22 行登记行）能不能标已做：要对 `records/2026-09-28-里程碑三第十项清单.md`「疑似已还的 52 行」逐行比 → 批 5 主 agent 比；比不完就留着。

## S20 逐行判
- C20 已并进 C635 但 D12:96、:124 与 milestone/03 号文件第 148 行 仍把 C20 当开着的账挂着（C635 检索词搜不到）→ 批 5 改（命令：`grep -n 'C20（' .claude/kb/decisions/12-目标介质.md .claude/kb/milestone/03-third-txn.md .claude/kb/checks-owed.md`）。
- checks-owed:51（C56）「`grep -c '^pub const' crates/singlefs-format/src/lib.rs` 是 101」现 100（别的会话暂存删了 `NEW_POOL_FILE_CREATION_ACCOUNTING_ROWS`）→ 批 5 改成现数。
- `records/2026-09-16-subagent拆分提案.md:1061` 未定项「提交没有读 `refs/sop/staged-green` 的闸」要不要做 → 收尾交用户。

## S21 逐行判
- F600 里 records 与实验页约 150 行仍写旧路径 `.claude/kb/invariants.md`，按派发口径判了事件句 → 照 `.claude/rules/path-moves.md`「怎么做」路径不冻结：批 5 主 agent 机械换成新目录（说搬迁这件事本身的行保留，第 5 条），范围按候选表 F600 行逐行看。
- `invariants/03-journal.md:15` I-8.8 三处第一版口径改写前要对着 `recovery.rs` 恢复函数本体核。

## S22 逐行判
- C722 行的 GPU 内核展开数（334 332 / 392 197 / 340 071）与代码钉值（334 651 / 395 742 / 345 991，`crash_judge_dispatch.rs` 在暂存区）对不上，F619 事实表的新数也已过时 → 批 5 按代码现取。
- milestone/03 行 38（:218）「这 16 行」应改 12 行，C589、C590「怎么拦」是「暂无」「无」做不成会红检查 → 批 5 改数并括注。
- checks-owed:484「与 C660、C705、C717 同属…一族」C705 已作废 → 批 5 括注。

## S19 逐行判
- D16:127 与 milestone/01 号文件第 185 行「C199（冻它之前要先还）」：C199 已收口到只剩 I-8.3 那一格，「冻 journal 记录格式前先还 C199」还成不成立看 D15 → 决策依赖，交用户/三方；两处一起改。
- 欠账表 C102、C182、C216、C224、C271 五笔仍以「已还一半」开头（别的已改成「已有 / 仍欠」）→ 批 5 统一写法。
- checks-owed:566「欠检查清单里的条目全部未实现」：C61、C402、C403、C420、C508、C572 已实现待提交销账 → 等销账（别的会话）后再改；收尾提醒。

## S24 逐行判
- layout/03 号文件第 50 行、:54、:55、:56 段串与角色数仍标「推的，待干跑核」：E172 页写与登记逐个相等但读数是退役树加进之前的 → 撤不撤「推的」要现跑装置核，不归这一轮；记进欠账（E172 第 3 次跑之后回看 layout/03 那几格）。
- checks-owed:475（C714）引 layout/03 号文件第 59 行 现文 → 批 5 改 :59 时同改引文。

## S23 逐行判
- E142:189、:315「整条流 16777240 个两态状态还没枚举过」：C577 之后这条流有没有整轮枚举过 → 批 5 查崩溃判定库 / 跑全量的会话（它正在跑全量，第 6 条 unlink 之后还有 8 条；新池新建文件那条应已判完）。
- E142:21 页首整段是第七次跑的口径（25 行 gap、28 个结构、事务 21 条写等）→ 批 5 整段按 r25 核换，写回员做。
- milestone/01 号文件第 217 行、:224「现状（…）」段判了事件句 → 主 agent 判：带日期的现状段照 kb-discipline 第 8 条删日期读，是现状句；:217 验收数 262165 若已不是现行数要改。批 5 看。

## S25 逐行判
- 「区域扫描」：05 写「区域扫描（里程碑四出口 2）」、D27 写「D26 的区域扫描」，04 出口 2 与二·1 作业点行字面没有 → 里程碑规划的互相登记缺一边，收尾交用户（补 04 出口 2 或二·1）。
- `records/2026-10-13-代码欠账批.md:56` C833「里程碑十一·2（10-tenth-txn.md 第 40 行）」笔误应为「里程碑十的一·2」→ 记录里说仓现状的指路句，批 5 改（与 S01 报的同一处）。

## S27 逐行判
- milestone/03 号文件第 195 行 待优化第 15 行「写者锁那一半没做」（与丙丙报同）、第 17 行现状格开头「开着」指什么没写 → 批 5 改第 15 行；第 17 行留，补一句「脚本已建，还欠什么见欠账 C…」现查。
- checks-owed:432（C650）「带故障的那一族另记 C657」，C657 已还清（:717）→ 批 5 括注。
- 上游副本 VERSION 现 0.0.66（待优化第 14 行写 0.0.64）；`test-script-discipline.md` 已移回项目本地规则 → 判要改已在表里。
- S29：待优化表第 15、20 行现状格开头用「已关」（同表第 24 行的词）。

## S28 逐行判
- C603（checks-owed:400）去向写「里程碑三待优化表新增一行」，表里没有 → 批 5：给待优化表补一行（自检性能浪费），或改去向；主 agent 定补行。
- 待优化表第 1、6、10、11、12 行「现查旧状态 + 下游已改」并存的格 → 批 5 把前半句挂回「当时」（照 S28 改法）；第 12 行 `selftest-registry.tsv` 已认副本路径一并改。
- 五·4（:108）要补「六笔去向已写进各登记行」；五·8 只剩 walk.rs:8510（crates，归下一批）。

## S26 逐行判
- 03 号文件第 39 行「全域没跑」：10-12 全量 unlink 流 0 红但「观察者记本次未跑」，10-13 带观察者那趟没结局 → 批 5 问跑全量的会话（它现在的全量第 6 条就是 unlink），等它回再改。
- 03 号文件第 133 行「要用户拍板的只剩 C489」：第 122、123 行还有多条「交用户」→ 批 5 改成列全。
- C572（checks-owed:376）「待提交落地」不在候选但同样过时；C568、C750、C572 验收看着已满足要不要挪已还清 → 挪已还清要证红，按用户 10-11 定「现成变异行即证据」：批 5 核各笔证据行在 `crates/mutations.tsv` 的行名，有的挪。
- F731 事实表写「改动之后的全域没重跑」与全量 f 第 2 项（1,678,377,204 态 0 红，`records/2026-10-11-崩溃放量两段与脚本审计.md:56`）相左 → 以记录为准，S26 的四处要改成立。

## S30 逐行判
- `main-agent.md:7`、:52、:90 三处（合入验证按批的旧说法、代码轮少「只改测试代码或注释的不开」）→ 定义改动，按用户 10-11 定「豁免三方直接改」做，登记进 `.claude/agent-def-review-exempt`；:90 那一行主 agent 看了再定。
- records 里 6 处旧路径 `.claude/singlefs-ai-sop/rules/test-script-discipline.md` → 照 path-moves 批 5 机械换（说搬迁本身的行保留）。

## S32 逐行判
- 旧小节名「新立一条判据，当场拿它回扫已有的条目」的引用（checks-owed:60、135、158、202；D2:256）指到不存在的标题 → 批 5 统一换成现名「新立一条判据，记下要回扫的条目，随下一次阶段同步一批扫」。
- milestone/02 第 68、198 行（2104413）判事件句、:227 判要改 → 批 5 看三处一致性。

## S33 逐行判
- `main-agent.md:94`「把库目录换到这一次的暂存树、只现判改到的」与 commit-stages 第 2b 步「没有一版，不包暂存树」冲突 → 定义改动，豁免登记一并做；先读 commit-stages 2b 定哪边对。
- C789 改写句「池级 checker 那一遍按 10-13 定案改走两段」代码上落没落地没核 → 批 5 核 `crash_state_enumeration` 再写。
- `term-renames.md:81-82` 现名列「判法版本」成了链上旧名 → 改之前跑 `sweep-term.py --check`（先修那个 0xEF 字节）。
- `.claude/hooks/lib_heavy_tests.py:3653` 还点名已删的 `library-version-is-staged-tree` 格 → 归 tooling-writer 下一批。

## S34 逐行判
- `.claude/gate.d/stage-inputs.tsv:52`「不留进度文件（没有断点续跑）」、`.claude/gate.d/crash-point-replay-items.tsv:2` 头注「13 条 crash-case:」→ 登记表头注文字，批 5 主 agent 改（不动判法）。
- C564「断点续跑」一样算不算还清（进度文件已取消、枚举经判定库）→ 批 5 现读 C564 行定：按「打断重跑只补没判的块」有没有用例判。

## S36 逐行判
- C793（checks-owed:547）「真跑的自证仍欠」：`crash-point-replay-shard-run.sh` 文件头已有 `real-two-hosts-run` 一格、`selftest-registry.tsv:92` 登记 heavy-if-set；行里还有旧词「层 0」→ 批 5 现读 C793 后改。
- `Cargo.toml:22` 与 `decisions-history.md:6235` 仍写旧路径 `…/singlefs-ai-sop/rules/test-script-discipline.md` → Cargo.toml 是代码树注释（归下一批 crates/工具），变更史那一行照 path-moves 改。

## S35 逐行判
- `kb-scribe.md:33`、:42 写 `research/**/*.rs` 与脚本一致，毛病在 `lib-item-ref-status.py:75-76` 扫描范围仍是 `research/**/*.rs`（实验源码已在 `scripts/experiments/`）→ 批 5 立欠账（10-11 记录已记但 checks-owed 没有）；脚本归 tooling-writer，定义那两句等脚本改了再改。
- E142:309 局限第 9 条引第十四次跑的读数当现状 → 批 5 加「第十四次跑时」。
- `cd research` 六处（E173、E176、E177、E178 复跑块、E95:52、vm-harness.md:10）→ 批 5 改成 `cd scripts/experiments` 并核相对路径。

## 写回丁交回要主 agent 看的
- milestone/02 号文件第 362 行（收口表第 24 行）仍把 C335 列成「要它」、02 号文件第 368 行（第 30 行）仍「实跑 5 档」→ 批 5 收尾主 agent 补改（02 文件写回员已交回，可改）。
- 03 号文件第 218 行 清单里 C589、C590 前提不在，只加括注留在 12 行里 → 留。
- 待优化第 17 行「开着」：minimal-copy 没对应欠账行 → 留「开着」。

## 写回甲交回要主 agent 做的
- 丙交回后跑 `python3 scripts/kb/relabel-item.py` 把 C774 简称改成「E142 装置追平退役树之后剩第 26 次登记」（动全仓引用，含 milestone/02 号文件第 393 行、:596 三处）。
- C714 引 layout/03 号文件第 59 行 的引文：丙改了 :59 之后主 agent 同改 checks-owed 的 C714 行。
- 跑 doc-registries 的 table-shape 格看 C842「已还这一半」开头认不认。
- 新立欠账编号是 C852–C856（C847–C851 被别的会话的写回规格 `c834-writeback-spec` 占了，那份还没落盘）。

## S38（第二轮探针 P1/P3）
- 要改 8 行：agent-common.md:62 `KS_THREADS` → `E168_THREADS`（主 agent 改，定义类）；checks-owed:314 `mask_code(` → 上游现名 `mask_code_spans`（改前看上游）；:321 C457 函数名与取法；:426 C638 例子改名、数要重数；:446 C668 两处（白名单表已删——依据是别的会话暂存的 `crash_identity.rs` 重写，HEAD 里还在，改前先问那个会话）；:649 C12 测试已搬到 checker 档；D2:315 `FormatTimeGeometry` → `SystemImmutableSizes::slot_spacing_for`。

## R3 通读（CLAUDE.md、README、kb 总览）
- 要改 10（报告 read-R3.md 有改后句）：CLAUDE.md:112「两条流」→ 14 条用例、8 项放量；verification-build.md:5、:76（16 种）、:154、:301（166 行）；tooling.md:198；experiments.md:19「结论列 50 字以内」；invariants/README.md:36、:40-46、:48-52。
- 要人看 7：CLAUDE.md:112 的范围词（2026-09-26 的判断，固定脚本流已全量重写）与「候选集下界」指针 → 主 agent 改 CLAUDE.md 时一并核；verification-build.md:39 带日期事件句要不要补现值 → 补一句；experiments.md E1、E3、E5、E10 前置「欠 checker / 事务层 / 文件系统」→ 改成现状（有了 checker 与事务层，E5 仍欠加密、E10 欠后台整理）。

## R1 通读（规则 12 份）
- 要改 7（报告 read-R1.md）：mutation-sampling.md:7、:68 出处句（test-discipline 没有「编译期溢出记成无效变异」那一条）；fs-design.md:134（「用类型让非法状态无法表示」在 code-discipline.md 不在 machine-first.md）；format-evolution.md:68 少写 `scripts/experiments/`；implementation-first.md:13「research/ 下的装置」；implementation-workflow.md:11 变异表在 `scripts/mutation/tables/`；:68「两条流」→ 14 条用例里 7 条登记了 exhaustive。主 agent 改（规则类）。
- 要人看 4：`crates/singlefs-unit-graph/` 没被 implementation-workflow.md:10 与 verification.md:8 归类（门禁 `doc-process-records.sh:327` 把它当产品代码，Cargo.toml 写测试基础设施库）→ 交用户定归哪一侧；implementation-workflow.md:66「整套重设计归别的会话」（10-03）→ 交用户；verification.md:56 与 verification-build.md:299「轻型保留到事务功能做全」→ 留。

## R2 通读（定义 14 份、共用约束、主 agent 入口）
- 要改 8、要补 8、要人看 4（报告 read-R2.md 有改后句）。都是定义类：主 agent 自己改，连同批 5 改的几处一起交用户逐份豁免。重点：agent-common.md:56 轻型三条命令补 `SINGLEFS_RUN_SCOPE` 前缀、:46 副本多拷 `.claude/heavy-thread-default`、:71-88 补 run-scope-guard 与 timer-only-remind-guard；main-agent.md:7、:29、:90 合入后验证只跑改到的包（`--all-packages` 才全跑）、补「起验证命令带范围声明」；mutation-triage.md:33 与 experiment-runner.md:36「全部复用」收尾；three-way-attack.md:35 函数在 `crash_state_enumeration.rs`；implementation-writer.md:48 合表点名；tooling-writer.md:51 漏 checker-tier-broad-mutation；kb-scribe.md:22（与 S31 同）。
- 要主 agent 现查：`run-scope-guard.sh` 文件头第 32 行说 merge-and-verify「whole，还不认只跑改到的包」与 `merge-and-verify.sh` 新写法矛盾 → 钩子，归 tooling-writer 下一批。

## S39（第二轮探针 P3–P9）
- 要人看 2：milestone/02 号文件第 550 行「崩溃点重放每个恢复结果上跑一遍」、:574「还没做的两样」→ 写回员戊现查 `crash_enumeration_record_spill_over_stream.rs` 等后判，判不了的留。
- 要现查：milestone/02 号文件第 374 行「待代码三方与崩溃验证」；checks-owed:387 引 `doc-decisions.sh` 行号对不上（改成锚串）；milestone/01 号文件第 159 行 计数是否还在；milestone/02 号文件第 592 行 `publish_file_version`（候选外）；探针 P3 应把 `research/results/` 加进零命中范围（下一轮）。
- D22 已定项 14「进度：未实现」而 I-7.6 已实现（`image.rs:140`）→ 翻进度是派生现状同步，写回员戊改 decisions/22 第 24 行再由主 agent `--write`。

## R4 通读（skill、scripts README、里程碑三四、layout/03）
- 要改 9：主 agent 落（commit-stages:19、:21、:32；scripts/README.md:7、scripts/agents/README.md:3、scripts/experiments/README.md:9；milestone/03 号文件第 105 行、:124、:206）；要补：scripts/gate/README.md 表末加 `lib_reuse_checks.py`、`lkmm-selftest.sh`。
- 要人看 6：03 号文件第 38 行 清扫器两条用例「写好、没跑」判定库里有没有判定（只读判不出）→ 收尾问跑全量的会话；03 号文件第 111 行 五·7「等下一次提交时 checker 档实跑」→ 翻 10-14 调度记录定；03 号文件第 113 行 五·9 E150 简称已改 → 改成现简称；03 号文件第 208 行-216 待优化 28–36 行行号漂、别的会话正改 crates → 等那批提交后重核（记进欠账）；03 号文件第 217 行 磁盘清理数 → 留；staged-gate.md:17 引的两句红话门禁里查不到 → 主 agent 改成现行写法「本次未跑」。
- 待优化表引的 6 个仓外证据路径已不在 → 记录性质，留。
