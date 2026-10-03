# A3 算术：根槽起始偏移在 (pbs, io_min) 各组合下按判定宽度 W = max(pbs, io_min) 是不是整数倍。
# 常量取自 crates/singlefs-format/src/lib.rs（SLOT_BYTES 14 行、ROOT_RING_PRIME_STEP 258 行、ROOT_RING_CHUNK_BYTES 260 行、ROOT_RING_BASE_SLOT 263 行、
# FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES 231 行）；槽距式子取自 crates/singlefs-core/src/system_configuration.rs:161；
# 根槽偏移式子取自 crates/singlefs-checker/src/image.rs:602 起。R = 3、S = 8 是 mkfs 默认（layout/01-first-txn.md 一「根环区域数 R 3」「每区槽数 S 8」）。
SLOT = 16384
CHUNK = 1048576
PRIME_STEP = 3
BASE_SLOT = 64
FLOOR = 4096


def spacing(io_min):
    return FLOOR if io_min == 0 else -(-FLOOR // io_min) * io_min


def check(pbs, io_min, slots_per_region=8, regions=3):
    slot_spacing = spacing(io_min)
    width = max(pbs, io_min)
    misaligned_root_slots = []
    for region in range(regions):
        for slot in range(slots_per_region):
            offset = BASE_SLOT * SLOT + region * PRIME_STEP * CHUNK + slot * slot_spacing
            if offset % width:
                misaligned_root_slots.append((region, slot, offset, offset % width))
    system_configuration_slots_misaligned_to_pbs = [
        (index, index * slot_spacing) for index in (0, 1) if (index * slot_spacing) % pbs
    ]
    return slot_spacing, width, len(misaligned_root_slots), misaligned_root_slots[:2], system_configuration_slots_misaligned_to_pbs


for pbs, io_min in [(4096, 3072), (512, 3072), (512, 4096), (4096, 0), (512, 0), (4096, 65536), (4096, 8192), (512, 2097152)]:
    print(f"pbs={pbs} io_min={io_min} -> (槽距, 判定宽度, 不对齐的根槽数, 前两个(区域,槽,偏移,余数), 按 pbs 不对齐的系统配置槽) = {check(pbs, io_min)}")
