# E151 第二次跑：跑前登记（2026-09-13，D3 未定项 8 第二轮反推腿打中之后）

反推腿（`d3-r2-opus-output.md`）在它自己的副本上量出三件事，第一次跑的装置报不出来。第二次跑把它们做成常设臂与常设指标，在入库的装置上复核。判据在跑之前写死：

| # | 反推腿的说法 | 常设臂 / 指标 | 坐实的观测 | 推翻的观测 |
|---|---|---|---|---|
| R1 | 按 D16 已定项 5 的切分纪律读到达序（请求之间洗牌、请求内按 key 升序），runs8 上丙 = 7334 ≈ 乙 7312，不是 8183 | `arrival_by_request` | runs8 三格上它与 `bump_seg` 差 ≤ 5%（`by_request_tied_within_5pct=true`），uniform 上与 `arrival` 逐格相同 | runs8 上它落在 `arrival` 那一档（≥ 8100）⇒ 副本的 7334 是它自己装置的产物，5-1 的修正不成立 |
| R2 | 提交内生块的回落次数：用户数据走最低空槽时 0 次，走 bump 时 137 488 次（种子 0，M = 9） | `metadata_fallback_seed0` | `first_fit` 全部六格（M = 4 / 9）为 0，`bump_seg` / `arrival*` 全部 > 0 | `first_fit` 任一格 > 0 |
| R3 | 打包规则不改、打包前的落点换成最低空槽，全空段数 0 → 73（K = 16，runs8，M = 9） | `container_firstfit_k16` / `k64` | `container_firstfit_*` 的 `empty_segments_median` > 0 而 `container_*` 为 0；`any_container_firstfit_cell_has_empty_segments=true` | 两族都为 0 |
| R4 | 全空段数是轨迹不是端点：bump 系的 32 个初始全空段被吃光后再没出现过正读数 | `drained_at_seed0`、`empty_peak_after_drain_median`、`rounds_with_empty_after_drain_seed0` | bump 系 `drained_at=32`、吃光后峰值 0；`first_fit` 系 `drained_at=never` | bump 系吃光后峰值 > 0 |

**作废条款**：任一常设单测红、变异表有一条没被抓、或 `verification_ran` 少于登记数 ⇒ 整轮作废重跑。
**什么观测会让它触发**：`cargo test` 有红；`mutate.sh` 报「一个测试都没红」的条目。

**不改的**：第一次跑的臂、几何、种子、`PACK_SWEEP_PER_CHECKPOINT = 64`——它与 `DIRTY_PER_CHECKPOINT` 相等这件事反推腿已经指出（C317 加一句），这一次不扫巡回预算。
