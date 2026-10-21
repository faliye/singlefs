# 里程碑四预开发整段：代码轮第三轮（最终轮）正文（2026-10-16）

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 Q5 Q6 R1 R2 R3 R6 R7 R8 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 F1 F2 F3 F4 F5 F6 H1 H2 H3 H4 H5 H6 G3 G4 U2 -->

## 一、题面

这是这一段代码轮的第三轮，也是最终轮（`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」）。前两轮判决：`research/prompts/m4-segment-code-r1-main-verification.md`、`research/prompts/m4-segment-code-r2-main-verification.md`。第二轮的改法 H1–H5 已作为修复批三打进工作树 `~/.local/share/singlefs/work/m4-predev`（交回与主 agent 的判定在 `records/2026-10-13-里程碑四预开发.md` 第三节「甲三」「乙三」「H5 重做」「丁三」那几行）；修复批二、三的合入后验证、崩溃与变异轻型还没跑（等卡与内存），它们的红归验证段，不算这一轮打中。

这一轮只攻前两轮站住的形态，分三面：前两轮判「站得住」的修补与判定撑不撑得住；前两轮开着的岔路与形状清单全不全；两轮报出来、判决引用的数能不能核。这一轮新冒出来的零轮形态不再为它开轮，写进判决交用户表、标「零轮」。

被判的产品代码与前两轮同一组 20 份（sha256 在 `research/prompts/m4-segment-code-r3-snapshot/crates-sha256.txt`）；修复批三改到的是 `crates/singlefs-core/src/compaction_plan.rs`、`mounted_session.rs`、`transaction.rs`、`crates/singlefs-reference-filesystem/src/lib.rs`，改动全文 `research/prompts/_m4-segment-code-r3-diff.md`（相对第二轮冻结副本）。决策：D26（后台整理与放置回收）、D23（journal 的角色与格式）、D5（快照 / 空间记账机制）、D16（发布语义）。

## 二、实现今天的样子（主 agent 的观测，2026-10-16）

- 冻结副本 `~/.local/share/singlefs/work/m4-segment-code-r3/repo/`（打完三批修复之后的 `crates/`、`Cargo.toml`、`Cargo.lock`）。
- 修复批三的做法（实现员报告，都被攻过零轮）：
  - H4（`~/.local/share/singlefs/work/m4-fix-r3-core-writer/patch/report.md`）：计划先去掉这次发布本来就重写的节点、再照原 `plan_batch` 补满批大小；R 里只剩本来就重写的节点时全部带上、同发布删意图；会话先试算，被拒就跳过点名的 key 再计划。
  - H3：意图里 `segs0` 记签发那次发布之前的全空聚簇段数，修复批一 F3 的换值重算整段删了；模型照旧不记不比 `segs0`（主 agent 定）。
  - H2：两条「核不过就照抄意图」的退路注释照实写（瞬时读错走得到），行为不变。
  - H1（`~/.local/share/singlefs/work/m4-fix-r3-model-reffs-writer/patch/report.md`）：模型回退那一版的搬根记录只取目标版的。
  - H5（`~/.local/share/singlefs/work/m4-fix-r3-reffs-writer/patch/report.md`）：照 `research/prompts/m3-formal-r6-main-verification.md` 第 28 行 U2，抛弃最新根的那次恢复以读出内容重新起头、不判；重新起头的快照集合取允许集合里每个状态之前最后一次已确认发布那一刻的、并起来；上一次恢复之后还没有已确认发布那一支照旧取上一次恢复那一刻的；一版不止一条根时「之前那一版」照字面取。
- 实现员交下来、主 agent 列进这一轮的：两条退路照抄出来的那一版模型判不合（R 按记录全空而意图还在）；只剩本来就重写的节点那一批搬的个数可能超过批大小 K（推的）；H5 那两处读法（没有已确认发布那一支、一版多条根）。
- 已知、归验证段或阶段同步、报出来不算打中的：修复批二三的验证还没跑；第一轮判决 F1 一条照 D23（journal 的角色与格式） 改写（H6）、D26 字段表 `segs0` 那一句、I-3.5 / I-5.1 条文、变异第 4491 行行名归阶段同步；主仓那份形式化门禁点不中 `compaction_plan_proofs` 子模块（合主仓前处理）；T8 少发 discard（欠账）。后果超出这里写的范围时算打中。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### Q1　前两轮判「站得住」的撑不撑得住
F1（回退映射只重写位置变了的路径）、F4、F5、F6、第一轮 T7、T8、T10 三处不改代码的判定、第二轮 R8 两次抽样没打中的六格（T1、T2、T3、T7、T9、T10）：换前两轮都没用过的攻击面再攻，撑不住的给反例。

### Q2　修复批三的修补本身
H4 计划补满（先去掉本来就重写的节点、试算被拒跳过 key 再计划、只剩本来就重写的节点时全带上并删意图）、H3 `segs0` 取签发之前与停机谓词、H2 退路、H1 模型回退、H5 照 U2 的快照集合取法：每一处造反例；实现员交下来的四处（第二节）各攻一次。

### Q3　开着的清单全不全
前两轮判决的交用户表、欠账与「第三轮清单」：有没有前两轮两腿都碰到过、却没进任何一张表的形状；判决里写「站得住」所依的条款原文有没有读窄。

### Q4　判决引用的数能不能核
前两轮判决第二节引的每一个数（场景数、段数、格数、违例数、种子号、余量）：照腿的复跑脚本在新的草稿目录复跑抽查，数对不上的、复跑脚本缺步的给原样输出。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | Q1、Q3、Q4 | 复核前两轮判决的结论与判定过程：每条「站得住」「打中」的依据够不够得着、是不是同样打中所有替代方案；清单全不全；数能不能核 |
| 云端攻方（Opus） | Q1、Q2、Q3 | 在冻结副本的拷贝上造反例：攻前两轮站住的形态与修复批三的修补；不重复前两轮攻方报告（`research/prompts/m4-segment-code-r1-opus-output.md`、`m4-segment-code-r2-opus-output.md`）里已用过的攻击面与种子段 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷冻结副本，不在冻结副本里改。拷编译目录当种子之前先 `find crates -type f -exec touch {} +`。
- 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 8G 的上限，同一时刻只起一条 cargo；用自己的编译目录。前台命令预计超过 4 分钟的拆成小段。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU，不碰 `crates/singlefs-checker-tier/`。
