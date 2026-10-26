
// 读数用例（同上，只在 instrument.py 改过的副本里存在）：整理的一批（publish_relocation_batch，搬 4 个进映射的数据单元）里，
// 同样几处比较了几次，随池里文件数（即 units 的长度）怎么涨。
#[test]
fn probe_linear_finds_on_a_relocation_batch_by_pool_size() {
    for files in [8u64, 16, 32, 64] {
        let mut pool = pool_with_files_written_once(files, 8);
        let planned =
            planned_relocations_of_the_data_units(version_of(&pool.session), pool.inodes[3], 4);
        reset_probe_counters();
        relocate_a_batch_and_check_the_relocation_sources(&mut pool, &planned, 0);
        print_probe_counters("relocation_batch_of_4", usize::try_from(files).expect("文件数"));
    }
}
