# c381-r2 核查员核对表（2026-09-21）

我交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 〇、判别力自证

待核引用（`c381-r2-opus-output.md` 第 127–129 行）：「条款给切换的所选根写了死（`.claude/kb/decisions/23-journal的角色与格式.md` 第 371 行，摘这一句的前半）」，引文「**实例切换** = 挂载内做一次恢复：取新实例代号、写行、重发在飞 checkpoint——所选根取被重发的那个在飞 checkpoint 所基于的根（旧实例发布过根时是它最后发布的根，没有时是这次挂载恢复重放之后的根，回退那次发布里是 R_old）」。

自证操作：把行号加 1（371→372），按第 2 步核：`awk 'NR==372' .claude/kb/decisions/23-journal的角色与格式.md` 原样输出为空行。空行与引文逐字不符 ⇒ **判 ✗**。核查方法能分辨错误行号，往下开始正式核查。

## 一、快照校验（覆盖三条腿共用的前提）

`sha256sum -c research/prompts/c381-r2-start-snapshot.sha256`：19 个文件里 18 个 `OK`，仅 `.claude/kb/checks-owed.md` `FAILED`——与派发提示里预告的已知差异完全一致（另一会话同期提交了新版、叠加了本轮 C427 一行）。往下所有对 kb / crates 的核查按“对现在的工作区核”处理（三条腿引用的 kb 决策文件、`crates/` 三个核心文件均在 18 个 OK 之列，与快照一致）。

## 二、Opus 攻方腿（`c381-r2-opus-output.md`）

### 二·一 产物与复跑

| 核了什么 | 结果 | 命令 / 依据 |
|---|---|---|
| 模型目录 16 个文件 sha256 与报告表（第 34–51 行）逐个比对 | ✓ 16/16 全同 | `sha256sum` 全部文件，见工具输出 |
| `/tmp/claude-1000/c381-r2-opus/logs/` 九份日志与 `c381-r2-opus-model/` 归档份 | ✓ 9/9 `diff -q` 无差异 | 逐份 `diff -q` |
| `c381-r2-arms-core.diff` 行数统计：`transaction.rs` +83/-0、`history.rs` +1/-0、`model_comparison.rs` +1/-0，`mount.rs`/`recovery.rs`/`allocator.rs` 不在 diff 里 | ✓ 与报告第 268 行逐字一致 | `awk` 按 `--- a/` 分段计数 `+`/`-` 行 |
| 六个 phase 段数合计 58 968 | ✓ | `grep 'runs=' 六份.log \| sed 's/.*runs=//' \| paste -sd+ \| bc` = 58968 |
| 每个 phase 单独总数：f=7056(=1764×4)、c=30744(=7686×4)、b=9828(=2457×4)、d=4704(=1176×4)、e=4116(=2058×2)、t=2520(=630×4)，均与报告第 233–238 行「每条臂的段数」表吻合 | ✓ 6/6 | 逐份 `grep 'runs=' phase_X.log \| ... \| bc` |
| 按臂统计「红」（分类字符串里含「红」子串）总段数：甲 2756、丙 2889、戊-A 3929、戊-B 1495，与报告第 248 行逐字一致 | ✓ | awk 按 `[前缀 臂 分类] runs=N` 解析、按 `index(分类,"红")` 过滤求和；未过滤直接用「非干净」求和会得到甲5046/丙5228/戊-A11648/戊-B10633，与报告不符——确认报告统计口径是「分类字符串含红」而非「非干净」 |
| 「258 处、差额恒 65536 字节」 | ✓ | 原样重跑报告给的命令：`grep -o '记账的已分配…' phase_*.log badsector.log \| sed … \| awk … \| sort -n \| uniq -c` → `258 65536`，逐字符匹配 |
| `grep -rn -e 探针写 -e probe_write .claude/kb/ crates/ --include='*.md' --include='*.rs'`：`.claude/kb/` 命中 11 行，按文件拆分 `decisions/23-journal的角色与格式.md`2、`decisions/02-RAID条带策略.md`1、`checks-owed.md`2、`decisions-history/2026-09.md`4、`experiments/104-…md`1、`milestone/02-second-txn.md`1，`crates/` 命中 0 | ✓ 与报告第 111 行逐字一致（含各文件命中数拆分） | 原样重跑该 grep 命令 |
| fullpool.log：47 次覆盖写之后 C 的写序号与「挂不上」分类 | ✓（换算后一致） | 报告称「C 的第 19 / 20 次写报错」，日志原样标签是「C 第18次写瞬时错」「C 第19次写瞬时错」——按同一份报告在别处（badsector 场景）「C 的第 1 次写」对应日志「第0次写」的换算关系（人类序数 = 日志下标 + 1）核对：18→19、19→20，与报告的「19/20」一致，**不是差异，是报告统一把日志的 0-based 下标转成 1-based 序数**；两处换算方向一致，判 ✓ |
| badsector.log：「前缀 base，坏在 C 的第 1 次写」与日志原样「坏扇区在 C 的第0次写」 | ✓（见上一行，同一换算约定） | `grep -n "第0次写" badsector.log`，配合上一行的换算关系核实一致 |
| 第九节六行取样计数（撕裂 3528、断电+撕裂 10248、屏障 9828、整块盘 4704、戊自己那几步 4116、断电撕裂 2520）与第六节已核实的每 phase 总数做算术交叉核（子故障型段数 = 总数 × 命中的故障型比例 × 参与的臂数） | ✓ 6/6 算术一致 | 例：phase_f 撕裂=2/4 故障型×4 臂×441(=1764/4)=3528；phase_c「撕裂+报错」=1/3×4×2562(=7686/3)=10248；其余四行分别等于对应 phase 的确认总数（各自的分母/分子在报告里给全），逐一算术核对，无法从日志分类里直接 grep 出按故障型拆分的计数（分类只按“前缀/臂/结局”记，不按故障型），故用算术交叉核，不是直接读数 |

### 二·二 kb 与源码引用（行号均去文件里现查，不从背景材料数）

| 核了什么 | 结果 | 命令 / 依据 |
|---|---|---|
| `.claude/kb/layout/01-first-txn.md` 第 46 行整行引文 | ✓ 逐字一致（含省略号处的合法省略） | `sed -n '46p'` |
| `.claude/kb/decisions/02-RAID条带策略.md:225`「可写设备数」口径引文 | ✓ 逐字一致 | `sed -n '225p'` |
| `.claude/kb/decisions/23-journal的角色与格式.md:365` 判别子/收敛论证引文（三处引用同一行） | ✓ 逐字一致 | `sed -n '365p'` |
| 同文件 `:371`「实例切换」定义引文 | ✓ 逐字一致（自证也用了这一行） | `sed -n '371p'` |
| 同文件 `:360`（注 3）「普通挂载与回退…切换不重扫」引文 | ✓ 逐字一致 | `sed -n '360p'` |
| `crates/singlefs-core/src/transaction.rs`：`:1237`(`publish_version`)、`:1265-1269`(allocator 换回)、`:1572`(`publish_admitted`)、`:2110`(落盘闭包起点)、`:514`(`publish_without_units`)、`:547`(落盘闭包起点)、`:925`(`PublishError`)、`:93`(`PoolWriter`) | ✓ 8/8 | 逐行 `sed -n` 核对函数名/内容与报告描述吻合 |
| `crates/singlefs-core/src/mount.rs`：`:1108`(`mount_writable`)、`:1185`(`mount_rollback`)、`:979`(`establish_instance`)、`:377`(`rebuilt_allocator`)、`:323`(`isolate_slots_referenced_only_by_abandoned_roots`)、`:871`(`refuse_publishes_before_acquisition_that_do_not_pass_admission`)、`:920`(注释含「只读区域归属表」)、`:1261`(注释「不施加 R_old 之后的任何记录」) | ✓ 8/8 | 逐行 `sed -n` |
| `crates/singlefs-harness/src/history.rs`（`publish_error_member` 函数存在）、`crates/singlefs-harness/src/model_comparison.rs:141`(`match error {`)、`:162`(`\| PublishError::BlockDevice(_) => …`) | ✓ | `grep -n` 定位 |
| `.claude/kb/milestone/02-second-txn.md:409`（增补 3 第 4 件题面）、`:426`（验收句） | ✓ 2/2 逐字一致 | `sed -n` |
| `checks-owed.md` 条目标题核对：C126（切换预留的最坏量没有口径）、C334（切换的所选根没有会红的检查）、C287（切换收养开放 checkpoint 的事务后再崩）、C329（写行那次发布之前推抬 F 的空发布没有检查）、C330（中间实例那一行的 T_pub 取所选根的 txg）、C331（择根倒挂压过已确认的写）、C380（环上有洞时释放代判不出唯一值） | ✓ 7/7 标题与报告括注逐字相符 | `grep -n "^| CXXX "` 现查当前 `checks-owed.md`（已知与快照不同，按当前工作区核） |

**Opus 腿小计**：核了 22 处，✓ 22 处，✗ 0 处（不含自证那一条，也不含主 agent 已核过的六格里的重复项，`transaction.rs:93`、`mount.rs:190/920` 因 Opus 又单独引了一次而重核，结果仍 ✓，未重复计入 22 处的计数中——按“未被主 agent 六格覆盖的引用”计数取 22）。

## 三、Sonnet 辩方腿（`c381-r2-sonnet-output.md`）

本轮无复跑、无模型，全部核的是文件内引用。

| 核了什么 | 结果 | 命令 / 依据 |
|---|---|---|
| `.claude/kb/invariants.md:123`（I-3.1 定义、checker 读法两处标注） | ✓ 逐字一致 | `sed -n '123p'` |
| `crates/singlefs-core/src/block_device.rs:245-246`、`:449-450`（两处 `ForceUnitAccess => sync_data()`） | ✓ 2/2 | `sed -n` |
| `research/prompts/_c381-r2-background.md:81`（反向接受条款、收口表第 23′ 行共用账那句） | ✓ 逐字一致，确系背景材料本身，未误标成 kb 行号 | `sed -n '81p'` |
| `.claude/kb/decisions/23-journal的角色与格式.md:25`（已定项 15 标题行）、`:394`（定案句）、`:396`（射程句） | ✓ 3/3 引文内容准确；`:396` 处引用在原句中间的分号处截断、换成句号收尾，未加省略号标记（内容不失真，但截断方式与另两处用“……”标记的做法不一致） | `sed -n` 逐行核对 |
| `c381-r1-main-verification.md:17` 引文 | **✗** | 报告引文开头写「四条腿一致：……」，实际第 17 行开头是「**本地两份一致**：甲 S3、丁 S3、乙 S2、丙 S2 的新规则 2「冲突」……」——把「本地两份一致」误写成「四条腿一致」，改变了被引句陈述的一致范围（本地两份抽样 vs 四条腿）；引文中段与结尾（「第五问……不来自条款」）逐字准确 |
| `c381-r1-main-verification.md:40`、`:42`、`:45` | ✓ 3/3 逐字一致 | `sed -n` |
| `_c381-r1-body.md:76`（carve-out 原文） | ✓ 逐字一致 | `sed -n '76p'` |
| `c381-r1-opus-output.md:258-262`（乙/乙′下标 16/17 完整动作集合的数字：0+826/8323、1514+1314/7920） | ✓ 数字逐一致 | `sed -n` |
| `c381-r1-opus-output.md:271-282`（no-remount 数据：甲 3409/0、乙 1669/774、乙′ 16/17 为 3409/0、18 为 1669/774） | ✓ 逐字一致，第 279 行确系乙′下标 18 那一行 | `sed -n` |
| `c381-r1-opus-output.md:284-296`（下标 17 明细：1514/1314/5092 三组） | **✗** | 报告称「这 2828 段全部带一次 `Remount`」（2828=1514+1314）；但引用范围原样第 296 行明确写「`crash=["I-3.1"]` 的那 **1514** 段都带一次序列里的 `Remount`」，另外 1314 段的 `actions=[]`（第 291 行），字面上没有 `Remount`。报告把「1514/2828 段的 actions 里有 Remount」扩成「2828 段全部带 Remount」，超出了被引数据字面支持的范围，报告没有说明这一步扩展的依据（例如“检查 after 状态本身隐含一次重开”这类桥接假设未写出） |
| `.claude/kb/decisions/16-发布语义.md:101`、`:169`（Sonnet 自己在第三节表格里再次引用，主 agent 已核过这两行） | ✓（重复核，结果一致） | `sed -n` |

**Sonnet 腿小计**：核了 13 处，✓ 11 处，✗ 2 处（`c381-r1-main-verification.md:17` 误引「四条腿一致」；`c381-r1-opus-output.md:284-296` 的「2828 段全部带 Remount」超出引用数据字面支持范围）。

## 四、本地攻方腿

### 四·一 六份干净样本：判词与提示里写死的事实表核对

⚠️ part1 两份样本判定相反（s1 判 C4–C6 在射程、s2 判不在）是主 agent 已知的既有事实，照记不判谁对，不在下表重复。以下核六份指定的干净样本（part3a-1、part3a-2、part3b 各两份）。

| 样本 | 核了什么 | 结果 |
|---|---|---|
| part3a-1（表2，P1–P4×两臂）s1 vs s2 | 逐格核对 8 行×4 小项与 `part3a-1.md` 给定的事实 4、T1–T4 定义、两臂定义 | s1、s2 对 Arm STOP/TABLE 在 P1、P2、P4 的 scope/stage/action/consistency 判词一致；**P3 的 stage 判词不一致**：s1 判 T1（「journal record write failure occurring before the second barrier」），s2 判 T2（「journal record already sent but root slot FUA not succeeded」）。两份样本对同一失败点的阶段归属给出不同答案，按「一条腿只抽一次样不算一次观测」，此处如实记不稳定，不判哪份对 |
| part3a-2（表2，P5–P8×两臂）s1 vs s2 | 逐格核对 8 行 | Arm STOP/TABLE 在 P5、P6 的判词一致；**P7、P8 的 scope 判词不一致**：s1 全部判「inside scope」，s2 判「outside scope」（理由：P7/P8 发生在 mount 初始化阶段、非 publish 失败）。s2 额外对 Q2（跨行一致性追问）给出「Yes，Arm TABLE 在 P5(consistent) 与 P8(conflict) 之间用同一套推理判出不同结论」，s1 给出「no such case」；按 Arm TABLE 自己的定义（T4 与 T1–T3 的动作本就不同：T4 = 同 STOP 的 T4，不做探针写），P5 与 P8 判词不同更像是源于阶段动作本身的差异，不必然是「同一推理不一致地应用」——这一点记为观测，不替代主 agent 的判断 |
| part3b（表3，丙戊×T1–T4）s1 vs s2 | 逐格核 Arm TABLE 在 T1、T2、T3 的「Caller receive」列是否复述了 Arm TABLE 自己的定义（提示原文「follow the failure table…probe write…switch or read only」） | s1 正确复述（三行 Caller receive 均写「if it succeeds, go to an instance switch…if it fails, go read only…」，与 Arm TABLE 定义吻合）。**s2 三行（T1、T2、T3）的 Caller receive 全部写成「the original device error is handed back to the caller unchanged」**——这是 Arm STOP 在 T1（或泛指 fact 13「今天的代码」）的行为，不是 Arm TABLE 自己定义的「先做探针写、按结果切换或只读」；s2 在这三行把 Arm TABLE 的第一问答成了别的东西，**判 ✗**（与提示里写死的 Arm TABLE 定义字面不符，不是与 s1 的单纯分歧） |

**六份样本小计**：核了 3 组（part3a-1、part3a-2、part3b 各一组 s1/s2 对照），每组内部逐格核对；判定：2 组记为「样本间不稳定/分歧」（part3a-1 的 P3、part3a-2 的 P7/P8），1 组内 s2 判 ✗（part3b 的 T1–T3 Caller receive 与 Arm TABLE 定义不符）。

### 四·二 转述核对表（`c381-r2-local-attack-translation-audit.md`）逐条核对「原文文件:行」

| 核了什么 | 结果 | 命令 |
|---|---|---|
| `.claude/kb/decisions/16-发布语义.md:169`（Fact 1、Fact 2 引） | ✓ 逐字一致 | `sed -n '169p'` |
| `.claude/kb/decisions/23-journal的角色与格式.md:365`（Fact 4、5、6）、`:368`（Fact 7）、`:369`（Fact 8）、`:370`（Fact 9）、`:371`（Fact 10、及英文比原文多出的限定词那一条） | ✓ 6/6 逐字一致 | `sed -n` 逐行 |
| `.claude/kb/decisions/18-块里携带什么信息.md:313`（Fact 11） | ✓ 摘引部分逐字命中（含省略号处） | `sed -n '313p'` |
| `crates/singlefs-core/src/mount.rs:863`（Fact 12） | ✓ 逐字一致 | `sed -n '862,864p'` |
| `crates/singlefs-core/src/transaction.rs:1265-1269`（Fact 13） | ✓（与二·二已核结果一致） | 同上 |
| `research/prompts/_c381-r2-body.md:35`（Fact 13 来源标注）、`:38`（Fact 15）、`:39`（Fact 14）、`:90`（P1–P8 定义来源）、`:45`（T1–T4 表头）、`:47`（Arm STOP 丙行）、`:48`（Arm TABLE 戊行）、`:15`（前提一「整段一起引」） | ✓ 8/8 逐字一致（均为本轮自己的正文材料，未误标成 kb 行号） | `sed -n` 逐行 |
| `research/prompts/c381-r2-local-attack.md:277-280`（悬空前向引用「addressed separately in question 5」原文） | ✓ 确认该悬空引用存在于原提示第 278 行 | `sed -n '277,280p'` |

**转述核对表小计**：核了 16 处，✓ 16 处，✗ 0 处。

### 四·三 运行记录（`c381-r2-local-attack-runlog.md`）与产物核对

| 核了什么 | 结果 | 命令 |
|---|---|---|
| part1 s1/s2、part2 s1/s2/s3 五份样本词数（682、540、67、830、1442） | ✓ 5/5 与 `wc -w` 结果逐一致 | `wc -w` |
| 第 1 次调用（原整份提示）产物文件名与状态 | **✗** | runlog 正文与「没做什么」两处明确写「`c381-r2-local-attack-output-s1.md` 是重定向建出的 0 字节空文件……不是脚本自己判红留下的 `-output-void*.md`……`$dir/$base-output-void*.md` 也没有产生」「没有对……这个 0 字节空文件做任何改名或归档处理」；但（a）当前目录下没有 `c381-r2-local-attack-output-s1.md` 这个文件，只有 `c381-r2-local-attack-output-void-gateway1.md`（0 字节，时间戳 19:00:51，紧跟在提示文件 19:00:29 之后，时间上对应同一次调用）；（b）runlog 自己在 part3 一节明确写「已按 c381-r1 与**本轮 part 拆分前那次**的同一惯例改名为 `…-output-void-gateway1.md`（`mv -n`，未删除）」——「本轮 part 拆分前那次」正是指第 1 次调用那一次。runlog 前后两处对同一份文件的命运给出相反陈述，文件系统证据支持后一处（确实被 `mv -n` 改名），与前一处「没有产生 void 文件、没有改名」的说法矛盾 | `ls -la`、`ls -la --time-style=full-iso research/prompts/c381-r2-local-attack*.md` |
| `save_void()` 在 `ask-local.sh` 里的实际命名规则（`$dir/$base-output-void$n.md`，不含「gateway」字样） | ✓（旁证） | `grep -n` `research/scripts/ask-local.sh`；证实「-void-gatewayN.md」不是脚本自动生成的名字，是调用方自己按惯例手工改的名，runlog 对脚本行为本身的描述（判红即 exit 5 才触发 `save_void`，网关级 exit 3 不触发）是准确的，出问题的是它对**这一次具体文件命运**的陈述前后矛盾 |
| part3a-1 s1/s2、part3a-2 s1/s2、part3b s1/s2 六份词数（918、1004、1279、1356、657、1291） | ✓ 6/6 | `wc -w` |
| 交接摘要 `c381-r2-local-attack-handover.md` 里的子 agent id、中断时刻 | ✓ 与 runlog「追加」段一致 | `grep -n` |

**运行记录小计**：核了 5 处，✓ 4 处，✗ 1 处（第 1 次调用产物文件的命运，runlog 前后自相矛盾且与文件系统现状对不上）。

**本地攻方腿合计**：核了 3(样本组)+16(转述表)+5(运行记录) = 24 处，✓ 21 处，✗ 2 处（part3b s2 的 Caller receive 与 Arm TABLE 定义不符；runlog 关于第 1 次调用产物命运的自相矛盾），另 2 组样本记「分歧/不稳定」不计入 ✓/✗。

## 五、总计

| 腿 | 核了 | ✓ | ✗ | 其他 |
|---|---|---|---|---|
| Opus 攻方 | 22 | 22 | 0 | — |
| Sonnet 辩方 | 13 | 11 | 2 | — |
| 本地攻方 | 24 | 21 | 2 | 2 组样本记「分歧/不稳定」，不计入 ✓/✗ |
| 合计 | 59 | 54 | 4 | 2 组分歧另记 |

## 六、没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身（例如不判 K1/K2/K3/K4/K5 该判打中还是没打中，不判「戊/丙/乙′」该选哪一个）。
- 不重复核主 agent 已在 `c381-r2-main-verification.md` 第一节坐实的六格：K1 打中一/二/三、K5 实现量、K4 第一轮判决、K2 乙′那一行——只核了三条腿在这六格之外新引的行号与数据（如 D23:360、D2:225、`transaction.rs`/`mount.rs` 的其余各行、milestone 409/426 等）。
- 不核 `.claude/kb/checks-owed.md` 里各条目**除标题外**的正文细节是否被三条腿准确复述——只核了条目编号与标题是否存在、是否与括注对应；已知这份文件与快照不同，按当前工作区核。
- 不判本地攻方样本之间的分歧（part3a-1 的 P3 阶段归属、part3a-2 的 P7/P8 scope）哪一份读法更贴合提示原文的语义边界——只核了各自援引的原文片段字面是否存在、是否被准确摘录；这类分歧本身是否要紧、要不要再抽样，留给主 agent。
- 不编译 `crates/`，不跑 `gate.sh`，不跑 `mutations.tsv`。
- 没有对 Opus 报告第十节自陈的限度（切换是近似、切换预留没建、探针写落点是攻方自选、用户动作只扫到长 1、断电点只到 60、每段只跑一次、checker 只跑 `check_pool_image`）做进一步验证——那些是攻方腿自己交代的限度，不是需要核的「引用」。
- 没有把本地攻方腿「首稿」版本的英文措辞（转述核对表「首稿缺的」列）与任何留存的首稿文件做比对——没有首稿文件留存，这部分内容按无法核实处理，未计入 ✓/✗ 也未记「核不动」（它不是一条可核的「文件:行号」引用，是过程说明）。
- 没有跑任何长活（无需编译、无需虚机、无需网络的复跑），本轮全部核查用 Read/grep/sed/awk/diff 完成，未使用 `run_in_background`。
