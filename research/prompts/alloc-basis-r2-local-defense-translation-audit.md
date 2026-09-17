# alloc-basis-r2 本地辩方英文提示：逐句核对表（2026-09-17）

依据 `.claude/rules/three-way-inference.md`「给本地腿的提示一律用英文」：英文里的每一句转述写完都要与原文并排核，缺限定词就补。
下表逐项对到 kb 文件 / `research/prompts/_alloc-basis-r2-body.md` 自己的行号（现查，不从附录 `_alloc-basis-r2-appendix.md` 里数）。定稿即 `research/prompts/alloc-basis-r2-local-defense.md` 现在的内容，发给本地模型的就是它。

| 英文项 | 原文（文件:行） | 首稿缺的 / 放宽的 | 定稿 |
|---|---|---|---|
| SERIAL 定义句（「第一道不过就拒」） | `research/prompts/_alloc-basis-r2-body.md:69`（Z3 段「串：照 D3 已定项12 两道闸串联，第一道不过就拒」） | 首稿只引了 D3 已定项12 本身（F1），没有把「第一道不过就拒」这个由主 agent 在 body 里给 串 下的定义单独摘出来，导致 SERIAL 与 F1 之间的界限不清——F1 是被串读法所依据的条款，但「失败即拒、不重试」这句判断是 body 对它的读法，不在 03-空间分配.md 逐字里 | 加一段独立的 Definition of SERIAL，整句译出并标明出处是「the material that names it」（即 body），与 F1（决策条款本身）分开陈述 |
| F1 D3 已定项12（两道闸串联，七条合取）+ ⚠️ 没走三方 | `.claude/kb/decisions/03-空间分配.md:430`（定案原文）、`:432`（⚠️ 未走三方那句） | 无 | 与原文同范围，七条合取逐条译出 |
| F2 D28 已定项1 九项式子 | `.claude/kb/decisions/28-挂载期承诺量.md:21` | 无 | 逐项译出，与原文顺序一致 |
| F3 D5 已定项4 式子逐项对照（已分配→第1项、defer待释放→第5项）+ 第5项消费者与维度 | `.claude/kb/decisions/05-快照-空间记账机制.md:345`、`:347`、`:362` | 无 | 同 |
| F4 D16 已定项1 七行（可再分配、环里最旧有效根、F、抬F上限、生效、准入、df） | `.claude/kb/decisions/16-发布语义.md:371`（可再分配）、`:372`（环里最旧有效根）、`:373`（F）、`:374`（抬F上限）、`:375`（生效）、`:377`（准入）、`:378`（df） | 首稿曾考虑带上「回退候选集」（`:376`）与「环深下限」（`:379`）两行，这两行与本轮问题无关，删去不译，避免无关事实分散模型注意力 | 只译与准入、抬F、df 直接相关的七行；不译回退候选集与环深下限 |
| F5 D28 已定项3 的 df 句 + 「这一项」范围的说明 | `.claude/kb/decisions/28-挂载期承诺量.md:90` | 首稿把「df 报可用」直接译成「df equals the full F2 formula」当作既定事实；核对原文发现「这一项」字面只指切换预留这一个子项，「df 报可用」没有显式限定范围，是解读空间不是既定事实，改稿加了一句显式提醒模型这是开放的解读问题、要求它说明自己依赖哪种读法 | 加入独立的一句 note，明确「whether this licenses X versus Y is an open interpretive question; you must state explicitly which reading you rely on」 |
| F6 D3 已定项9 两条 + ⚠️ 警告 | `.claude/kb/decisions/03-空间分配.md:363`（第1条）、`:365`（第2条，含界=3、空发布不算）、`:369`（⚠️ 界3按字面判、E139上有越过它的格，含 D16已定项1 与 E139 自己的两句逐字引文） | 无 | 三段均整段译出，含 E139 自己那句「按它自己的 df_S…」；df_S 简化译为「its own df definition」，未保留下标 S（该下标在给定事实里没有单独定义，保留会引入未定义符号），在此标注 |
| F7 「合」「串-重判」两个其它读法（仅作背景，不辩护） | `research/prompts/_alloc-basis-r2-body.md:69`（Z3 段三种读法定义） | 无 | 只译为背景陈述，明确写「you are not defending either of them」，并加防串-重判混入的守卫句 |
| F8 今天实现的事实（allocator.rs / mount.rs） | `research/prompts/_alloc-basis-r2-body.md:38`（allocator.rs：isolate、mark_reclaimed、reclaim_released_up_to）、`:39`（mount.rs：reclaim_floor、回收只在两处调用、同一次挂载内转环不触发回收） | 无 | 与 body 原文同范围；hash-object 未译入提示（英文提示不需要），此处补记：allocator.rs=6bd429edf5ec6dc8701659169e988479918f2d4f、mount.rs=34246932bbd7a85f4a0be11463f4d461e87cf10c，与 body 一致 |
| 攻方问题（主 agent 派发提示原文，整行抄） | 派发提示正文：「主 agent 的倾向是『合』，理由是串读法下『已释放而不可再分配』的空间在第一道闸上不算可用，只有抬 F 才装得下的请求在第一道闸就被拒、D16 的抬 F 路径走不到。」 | 无 | 逐句译出，1a 要求模型把这句攻击拆开、逐部分标出对应事实编号 |
| 1b（可以考虑的方向第一条：df 比 D16 的 df 小，第一道闸本来就过得去） | 派发提示正文第2点第一条 | 无 | 译成要求模型用 F2/F3/F5/F6 的引文链条自己论证，不直接把结论喂给它 |
| 1c（可以考虑的方向第二条：有界步数是否要靠抬F兑现，谁触发） | 派发提示正文第2点第二条 | 无 | 译成开放问题，附加「不许把答案悄悄变成串-重判」的守卫句（对齐防串-重判混入） |
| 1d（可以考虑的方向第三条：⚠️ 界3按字面判、E139越界说明了什么） | 派发提示正文第2点第三条 | 无 | 译成开放问题，要求引用 F4、F6 的原文 |
| 1e（站不住就说明站不住在哪一步） | 派发提示正文第3点 | 无 | 译成「do not answer not enough information; either construct the mechanism… or name the specific fact that rules it out」 |
