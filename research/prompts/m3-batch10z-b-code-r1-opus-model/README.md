# m3-batch10z-b-code-r1 攻方腿的原型

报告：`research/prompts/m3-batch10z-b-code-r1-opus-output.md`。数都是在冻结副本的拷贝上量的，是副本上的数。

- `src/opus-arms-and-mutations.patch`：打在冻结副本 `crates/singlefs-core/src/allocator.rs`、`mount.rs` 上，`patch -p0`。环境变量不设时行为与冻结副本相同。
  - `OPUS10ZB_ARM=today`（缺省）：冻结副本原样。
  - `OPUS10ZB_ARM=slot`：抬 F 那一串（抬 F 重算影子账那一遍与准入算上限那一遍）读根环时，根环表里记成被抛弃根的槽不算「知道住着根、重读仍坏就拒」的槽；这一遍没读出它的，照根环表里记着的落点补进影子账（并进 C 那一份，只豁免现行账）。
  - `OPUS10ZB_ARM=cring`：抬 F 那一串里按现行实例表被抛弃、根环表里没记着落点的根，取 C 那一圈在它那一槽记着的落点当记着的落点。
  - `OPUS_MUT=m6`：挂载时记进根环表的被抛弃根落点只记这一次真隔离了的（候选根、现行账豁免的不记）。
  - `OPUS_MUT=m7`：抬 F 那一串里根环表记着的落点是空串时不用它、照读盘。
  - `OPUS_MUT=m11`：抬 F 那一串里表里没记着、账读不出的被抛弃根不从中央映射现算，直接重读、看凭据、拒。
- `src/opus10zb_probe_abandoned_slot.rs`（X1）：入库用例同一造法的被抛弃根 C；挂上之后 C 的根槽或 C 的账读报错，用户动作放开扫。
- `src/opus10zb_probe_form1.rs`（F1）：入库 `common_unwitnessed_publish` 造法的第 1 形 R；R 的账挂上之后读报错（A）或只在挂载那一刻读回全零（B）。
- `scripts/`：分批跑的脚本与规格、汇总脚本；`out/`：原样日志、`counts.txt`、`k3.txt`、两份汇总。

复跑：`SINGLEFS_FROZEN=$HOME/.local/share/singlefs/work/m3-batch10z-b-code-r1/tree SINGLEFS_TOOLS=<仓根> bash rerun.sh <新工作目录>`，末尾两行应为「counts.txt 全同」「k3.txt 全同」。
