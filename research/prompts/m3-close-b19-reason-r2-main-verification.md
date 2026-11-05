# m3-close-b19-reason-r2 判决（第二轮，2026-11-02）

正文 `research/prompts/_m3-close-b19-reason-r2-body.md`。腿：辩方 `research/prompts/m3-close-b19-reason-r2-sonnet-output.md`（2d2840c4…）与承接 `research/prompts/m3-close-b19-reason-r2-sonnet-c2-output.md`，模型 `research/prompts/m3-close-b19-reason-r2-sonnet-model/`；攻方 `research/prompts/m3-close-b19-reason-r2-opus-output.md`（82fedc06…）与承接 `research/prompts/m3-close-b19-reason-r2-opus-c2-output.md`（07c4846b…），模型 `research/prompts/m3-close-b19-reason-r2-opus-model/`。两条腿第一段都没跑模型、读数都是推的；主 agent 跑完模型之后各派同立场的承接腿读数。

## 一、核查

- 开工快照倒推：任务 5076 `python3 scripts/agents/snapshot-tree.py m3-close-b19-reason-r2 --out ~/.local/share/singlefs/work/m3-close-b19-reason-r2-main/m3-close-b19-reason-r2-tree --repository <仓根>`：「快照 m3-close-b19-reason-r2：清单点名 11 份，倒推出 11 份，sha256 对上 11 份」。代码快照 `research/prompts/m3-close-b19-reason-r2-snapshot/crates-sha256.txt` 是派腿时补记的（正文第二节 `walk.rs` 行号是 10-31 的，派发提示给了当天行号）。
- 引文：`python3 scripts/agents/cite-check.py <四份腿报告> --root <倒推的树> --background research/prompts/_m3-close-b19-reason-r2-background.md`：「核了 268 处引文、12 处裸行号，对上 79 处，没判 201 处」，没判的 197 处是「路径认不出」（腿的模型输出 `*-model/out/` 不在快照清单里，另有 `crates/singlefs-core/src` 10 处、kb 5 处）。同一条命令 `--root <仓根>`：「核了 268 处引文、12 处裸行号，对上 268 处，没判 12 处（其中裸行号没有引文可比 12 处）」。0 处对不上。
- 复跑：两份模型拷进 `~/.local/share/singlefs/work/m3-close-b19-reason-r2-main/`，在副本里交 `tools/rerun.sh --model <副本>`：攻方 5077「19 件 `_output_identical`」，辩方 5078「9 件 `_output_identical`」。攻方承接腿新加的 `i31_q1_sweep_quarantine_counted_in_i311.py`（8a15c966…，读过：只写标准输出）在副本里跑 `sweep 4`（5079），输出 sha256 与腿交回那一份相同（4aaccbaa…）。
- 现读坐实：写者算每块盘上最新的有效根时按 `target_for_publish(root.checkpoint_txg, …).region` 认盘（`crates/singlefs-core/src/mount.rs:3955`–`3968`，`crates/singlefs-core/src/root_ring.rs:172` `region: checkpoint_txg.0 % ROOT_RING_REGIONS`）；checker 按根所在那一槽的区域认盘（`crates/singlefs-checker/src/walk.rs:8886`、`9004`、`9248`，`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1821` `seen.device = handoff_region_device(region);`）。D16（发布语义） 已定项 1 的式子是「min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)」（`.claude/kb/decisions/16-发布语义.md:38`）。

## 二、逐格判决

| 格 | 判决 | 依据 |
|---|---|---|
| Q1 第 1 问　漏 | **打中（两腿收敛，模型上量的）**：改记之后加宽那条记录的尾槽压在另一条记录上，I-3.1 报部分没判、除 I-3.18 外都不红。攻方修正 I-3.11 判据（加隔离豁免，照 `walk.rs:21568`）之后在 N = 4 的小世界里：改记后有重叠且除 I-3.18 外都不红 1533 个；「走到的单元」那一形 0 个（与攻方第一段的证明相同）；「隔离单元尾槽压在释放代落在 (t, θ_high] 的记录上」那一形 4 个（手造镜像 `quarantined_unit_tail_on_a_record_released_between_t_and_theta_high` 判 `I-3.1=partial`、`others_red_quarantine_counted=[]`）。辩方量到窄记录加尾槽上另一条记录 1048 个（844 + 204）。改宽罩进隔离豁免的槽（18 个）两条记录都带记号、不复用，不算病 | 攻方承接 `quarantine-sweep-4.txt` 第 1 行、`quarantine-mirrors.txt` 第 9 行；辩方承接读数表 |
| Q1　改法 | **定（零轮）**：取宽那一支在四判之外再加一判：改记之后各条未释放记录罩住的槽区间两两不相交，相交判红、违例点名两条记录与相交的槽。两腿各自提出、同一形 | 两腿 |
| Q1　`!=` 与 `>` | `!=` 站得住：两者在新读法上互不支配（`!=` 部分没判而 `>` 红 40812 个，反过来 1064 个），判决否掉 `>` 的理由对新读法够不着，换成第一轮判决 Q9 那一句的条件 | 辩方承接 |
| Q1 第 2 问　误红 | **造不出（两腿一致）**：这一支只在按记录自述三判不成立时才进，进来的盘面 I-3.18 都红；改记集合非空即 I-3.18 红 | `walk.rs:21422` |
| Q1 第 3 问　说明里的数 | **不够（两腿一致）**：再加 U、隔离豁免、走到三样各自的槽数，两种读法差在哪几槽，被改记的是哪几条记录 | 两腿 |
| Q2　C944 的合法来源 | 最新根带种类 3 分配记录树、按最新根的 E(θ_high) 看时，两腿都造不出合法来源；写得出的只有没有种类 3 树的那几版（mkfs 第 0 版、树表 0 条写过行的版、暖机零单元发布版） | 两腿 |
| Q2　归属 | **新立一条（两腿收敛，零轮）**：遍历走到的每个引用起点，都是最新根 E(θ_high) 里某条未释放记录的起点；前置是最新根的树表带种类 3 记录树且读全，不满足报这一条判不了。攻方修正模型上「I-3.1 成立、除 I-3.18 外都不红」的 C944 形 1195 个，新条要抓的正是它们。坏镜像照 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 第一轮那张宽镜像改：映射树根那条记录跨度改 2，树表单元那条记录拿掉，记账不动；只红 I-3.18 与新条 | 攻方承接 `quarantine-sweep-4.txt` 第 1 行 `holds_c944_nothing_else_red=1195`；两腿造法 |
| Q3 第 1 问　枚举全 | **全（两腿一致）**：三处停下的条件逐一核过；前提是 C934 那一条、txg 只增、mkfs 种子齐全 | 两腿；辩方 `q3enum … NOT_COVERED=0`（`p_rootfail=0` 六格） |
| Q3 第 2 问　窗口外那一停 | **等价（两腿一致）**：最旧有效根、每块盘最新有效根、非空去重三项都是先滤有效再算 | 两腿 |
| Q3 第 3 问　第 0 代根记判不了 | **单是认得第 0 代根不改红的数（两腿各自量的）**：辩方 `today_known_genesis` 与 `today` 在坏写者上逐格相同（S = 4：red 13225、partial 838；S = 8：red 22765、partial 333），攻方 `genesis_known` 坏写者一段没变。**另一处改法会把部分没判改成红**：槽 0 而抬之前 F 为 0 时不记「空着」（mkfs 在每个区域槽 0 都写了第 0 代根，那一格不可能空着），连同第 0 代根按 ABSENT 记，坏写者红 +7（S = 4）、+3（S = 8），合法部分没判降 55、370。**定（零轮）**：两处同批改 | 辩方 `out/q3-bad-4-128.txt`、`out/q3-bad-8-128.txt` 各行；`.claude/kb/layout/01-first-txn.md:59`（m2 树表第 0 版 0 条） |
| Q3 第 3 问　4096 与部分没判比例 | 4096 几乎不咬（S = 8、J = 128 只 5 个组合超界）。今天的实现在两腿模型上 J = 128 时合法抬 F 根部分没判 8.63%（S = 4）/ 3.80%（S = 8），两腿同数；J = 32 时 10.5% / 36.2%（辩方），比例是 J 的强函数，J 在真池上取多少没核 | 两腿 |
| Q3 第 4 问　CPU 与 GPU | 逐支对上（两腿一致）；GPU 对拍缺「往下认两层以上」与「第 0 代根被抬 F 盖掉」两张镜像，造法在两腿报告第四节 | 两腿 |
| Q3 外带　第 0 代根记在哪块盘 | **打中（攻方，模型上量的；真写者上可不可达是推的）**：三个区域槽 0 的第 0 代根，写者都按 txg 0 记在区域 0 的盘上，checker 按所在那一槽记。攻方手造一段合法历史（R = 3、S = 4、区域 1 独占盘 1、区域 1 每一版都崩在根槽之前）：写者按自己的式子把 F 抬到 5，checker CPU 与 GPU 都判 I-7.9 红。照 D16（发布语义） 已定项 1 字面「每块盘上最新的持久有效根」，第 0 代根物理上在盘 1，盘 1 上最新的有效根是它，上限 0：**checker 对、写者错**。改法（零轮）：写者读环时按每一份根所在那一槽的区域认盘 | 第一节现读；攻方第四节表 |

## 三、交用户的

无岔路。告知一件：写者算抬 F 上限时按 txg 推根在哪块盘，与 D16（发布语义） 已定项 1 字面「每块盘上最新的持久有效根」在第 0 代根上不一致，一段合法崩溃历史能让写者把 F 抬过盘 1 上唯一那一份根（攻方手造，模型上量的；真写者上能不能走到还没跑）。照字面判写者错，改写者，第三轮攻。

## 四、被攻过零轮的

取宽那一支的「改记之后两两不相交」一判；C944 新立的那一条（射程、前置、坏镜像）；第 0 代根两处同批改（ABSENT、槽 0 且 F 为 0 不记空着）；写者按槽认盘；I-3.1 部分没判说明补的数。三轮之后停，第三轮攻这几样，连同：两张 GPU 对拍镜像、J 在真池上的取值、写者按槽认盘那一形在真写者上可不可达。

## 五、去向

| 编号 | 去向 |
|---|---|
| C937（I-3.1 照 I-3.18 改记槽数取宽的条件被攻过零轮） | 留着：第二轮在改记读法上打中「改记后重叠」，改法定了（零轮），随下一批实现，第三轮攻 |
| C944（走到的单元起点没有自己的记录） | 留着：归属定为新立一条（零轮），随下一批实现与不变量清单，第三轮攻 |
| 新登记（写者与 checker 对第 0 代根记在哪块盘不一致） | 写者按 txg 推区域认盘，checker 按所在槽认盘；照 D16（发布语义） 已定项 1 字面改写者，随下一批实现，第三轮攻；编号 C953（写者与 checker 对第 0 代根记在哪块盘不一致） |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 备料 | 2026-11-02 不受影响：式子本身不改，这一轮判的是写者在第 0 代根上没照字面实现 |
| D23（journal 的角色与格式） 已定项 14 | 不影响 | 2026-11-02 不受影响：这一轮只判 I-7.9 丁全放开那一形的枚举、窗口外那一停与第 0 代根的处置，按 journal 重建根环所靠的前提没动 |
