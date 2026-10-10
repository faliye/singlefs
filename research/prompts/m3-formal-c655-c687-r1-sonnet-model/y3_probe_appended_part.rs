// ===================== Y3 probe（正推腿副本，不入库）=====================
fn y3_report(name: &str, image: &MemoryPool) {
    let verdicts = check_pool_image(image);
    let red = violated_invariants(&verdicts);
    let i55 = verdicts
        .iter()
        .find(|(id, _)| *id == "I-5.5")
        .map(|(_, v)| format!("{v:?}"))
        .unwrap();
    let partial: Vec<&'static str> = verdicts
        .iter()
        .filter(|(_, v)| matches!(v, InvariantVerdict::PartiallyJudged(_)))
        .map(|(id, _)| *id)
        .collect();
    println!("[y3] {name}: 红={red:?} 部分没判={partial:?}\n[y3]     I-5.5={i55}");
}

fn y3_zero_unit(image: &mut MemoryPool, slot: u64, length: usize) {
    for device in DEVICES {
        write(image, device, slot * SLOT, &vec![0u8; length]);
    }
}

#[test]
fn y3_probe_bad_images() {
    let clean = build_pool("y3-probe").memory_pool();
    y3_report("clean（新池新建文件，txg 3）", &clean);

    // a：树表读不出
    let mut image = clean.clone();
    y3_zero_unit(&mut image, TREE_TABLE, 16384);
    y3_report("a 树表单元两份全零（不补引用链）", &image);

    // b：种类 1 两条（把 livelist 条目的种类改成 1，重封并补引用链）
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        TREE_TABLE,
        |bytes| set_u16(bytes, tree_table_entry_offset(LIVELIST_TREE_TABLE_ENTRY_INDEX) + TREE_TABLE_ENTRY_KIND_OFFSET, 1),
        true,
    );
    y3_report("b 种类 1（extent）有两条条目", &image);

    // c：extent 树读不全
    let mut image = clean.clone();
    y3_zero_unit(&mut image, EXTENT_ROOT, 16384);
    y3_report("c extent 树根两份全零（不补引用链）", &image);

    // d：数据指针两个位置条目都在盘 1 上（盘 0 上一个数据单元都没有）
    let mut image = clean.clone();
    let data = read(&clean, 0, DATA_UNIT * SLOT, 32768);
    let checksum = crc32_castagnoli(&data);
    mutate_unit(
        &mut image,
        EXTENT_ROOT,
        |bytes| {
            let changed = replace_all(
                bytes,
                &location_pattern(0, DATA_UNIT, checksum),
                &location_pattern(1, DATA_UNIT, checksum),
            );
            assert!(changed, "extent 根里有第一个文件的数据指针");
        },
        true,
    );
    y3_report("d 数据指针两条位置条目都在盘 1（盘 0 上没有数据单元）", &image);

    // e1：种类 3 条目一条都没有（把分配记录那一条的种类改成没登记的 13）
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        TREE_TABLE,
        |bytes| set_u16(bytes, tree_table_entry_offset(2) + TREE_TABLE_ENTRY_KIND_OFFSET, 13),
        true,
    );
    y3_report("e1 种类 3 条目 0 条（改成没登记的 13）", &image);

    // e2：种类 3 两条（把记账那一条的种类改成 3）
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        TREE_TABLE,
        |bytes| set_u16(bytes, tree_table_entry_offset(3) + TREE_TABLE_ENTRY_KIND_OFFSET, 3),
        true,
    );
    y3_report("e2 种类 3 条目 2 条（记账那条改成 3）", &image);

    // e3：分配记录树读不全（根节点两份全零）
    let mut image = clean.clone();
    y3_zero_unit(&mut image, 50249, 16384);
    y3_report("e3 分配记录树根两份全零（不补引用链）", &image);
}

#[test]
fn y3_probe_bad_images_second_batch() {
    let clean = build_pool("y3-probe2").memory_pool();
    // b2：种类 1 两条，第二条的头 ID 也照 extent 条目写 65537，I-9.27 不再先红
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        TREE_TABLE,
        |bytes| {
            let entry = tree_table_entry_offset(LIVELIST_TREE_TABLE_ENTRY_INDEX);
            set_u16(bytes, entry + TREE_TABLE_ENTRY_KIND_OFFSET, 1);
            set_u64(bytes, entry + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET, 65537);
        },
        true,
    );
    y3_report("b2 种类 1 两条，第二条头 ID 也写 65537", &image);

    // d2：数据指针的两条位置条目是 (盘 0, 盘 2)，池里没有盘 2
    let mut image = clean.clone();
    let data = read(&clean, 0, DATA_UNIT * SLOT, 32768);
    let checksum = crc32_castagnoli(&data);
    mutate_unit(
        &mut image,
        EXTENT_ROOT,
        |bytes| {
            let changed = replace_all(
                bytes,
                &location_pattern(1, DATA_UNIT, checksum),
                &location_pattern(2, DATA_UNIT, checksum),
            );
            assert!(changed, "extent 根里有第一个文件的数据指针");
        },
        true,
    );
    y3_report("d2 数据指针的位置条目是 (盘 0, 盘 2)，池里没有盘 2", &image);
}
