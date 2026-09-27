# 材料员报告：m2-closeout-code-r2

门禁：58 号绿（退出码 0，检查 49 份、0 份未查）。47 号红（退出码 1），点名脚本 check-segment-registry.py——不是我用的 quote-kb.py / kb-sections.py / checklist-specs.py / quote-rust-items.py，照写继续。

产出四份：
- research/prompts/_m2-closeout-code-r2-checklist.md（482 行，14 个 kb 文件小节清单：invariants.md、checks-owed.md、decisions/02、03、08、09、13、15、16、18、19、22、23、28；36 行标抄、375 行标不抄）
- research/prompts/_m2-closeout-code-r2-appendix.md（1233 行，40 段）
- research/prompts/_m2-closeout-code-r2-diff.md（40610 行：一、git diff --stat 全表 71 文件；二、src+测试 69 文件完整 diff；三、mutations.tsv 只放 numstat 与新增 311 行变异名列；四、bin/e161_* 只列路径注明是 singlefs-e1 会话的文件）
- research/prompts/_m2-closeout-code-r2-background.md（1821 行 = 正文106+清单482+附录1233，按定义顺序拼、排他新建）

快照目录 research/prompts/m2-closeout-code-r2-snapshot/：kb-sha256.txt（16 行，主 agent 已给，未动）、crates-src-sha256.txt（主 agent 已给，未动）、dispatch-time.txt（已在，未动）；新写 crates-sha256.txt（71 行，diff 里 71 个 crates/ 文件在 r2 快照上的 sha256sum，路径从仓根起，排他新建）。开工前核对 16 份 sha256 与 kb-sha256.txt 逐行一致。

checklist-specs.py 原样末行：「  ✓ 40 段整抄进 research/prompts/_m2-closeout-code-r2-appendix.md，回读逐字节一致，清单里标「抄」的每一节都在，正文提到的每个 kb 文件都有清单」，退出码 0。

做法：正文 D/E/C/I 编号（含裸引用，「已定/压着」等动词全仓核过）映射到 14 个 kb 文件；invariants.md 按 I-1/I-2/I-7 三类整段抄（I-1.10、I-2.4、I-7.13 分住三类，@标题 各自不带出别的类）；checks-owed.md 大表用 --extra 取 C307、C476、C571-C579（连续九行一次取）、C545；decisions 文件除正文直接压着的已定项外，追加了两跳内被已抄材料转引的项（D8 已定项11、D18 已定项7/18、D19 已定项3/5 由 I-1.10/I-2.4/C572 转引；D9 已定项4/5/6/8/10 由已抄的 D18 已定项7/11 转引），这些行的父节标题本身标不抄（不是被父节 @标题 带出），理由未用「随」开头，各自单独 @标题 取。

不判正文问得对不对；机械抽取保证「抄的没错」，不保证「该抄的都抄了」。

没做什么：D9 转引止于两跳，未继续深挖第三跳；mutations.tsv 与 bin/e161_* 按指示不放全文 diff；清单「不抄」行与理由留在清单文件本身，未列入回复。
