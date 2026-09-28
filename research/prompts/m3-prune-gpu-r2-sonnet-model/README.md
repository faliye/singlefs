# 云端正推（Sonnet）模型目录

只有一份模型：`derive_b1_overlap_signature.py`，核 B1（节点身份）的设计——
「种类串 + 覆盖关系签名」是不是真的把第一轮攻方 kinds 世界里合并的两个条件签名分开。

**只做字符串解析与算术**，不编译、不跑 `crates/` 代码、不改仓里任何文件：
读 `research/prompts/m3-prune-gpu-r1-opus-model/outputs/kinds.out` 这一份已经跑出的原始输出，
从它的 `detail=` 字段里解出两条条件签名，各自映射出「这次根槽写是不是重写了路径上更早的根槽写」
（`root_slot_rewrite=true/false`）这一位，核对加上这一位之后两条签名是不是分成了两个不同的节点身份。

跑法见 `rerun.sh`。产物 `derive_b1_overlap_signature.out` 是这一次跑出来的原样输出（没有带耗时的行，
不需要剥字段）。
