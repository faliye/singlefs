
// ---------- K1-3 反过来：有树而文件集合为空的一版上被宽护卡住，要出路做不做得成 ----------

/// K1 那一形挂上之后被拒一次，unlink 第一个文件（走空走完，现行那一版有树而没有文件），再覆盖写 `ow` 次之外只能建 inode：
/// 把没挡的槽压到 `leave` 个，建 `count` 个 inode；报卡住的，要出路、照出路二真走（推环、再建同样多个 inode）。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k1_stuck_on_a_version_with_trees_and_no_file() {
    let leaves: Vec<u64> = atk_env_list("ATK_LEAVES", "0,2,4,6,8,10,12,14,16,18,20,22,24,26,28,30,35,40");
    let counts: Vec<u64> = atk_env_list("ATK_COUNTS", "1,4,16,64,256");
    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
    let stuck_once = crashed.mounted_and_stuck_once_by_a_write_grown_by(DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY);
    let mut emptied = AtkSnap::of(&stuck_once).pool();
    emptied.write_time_seconds += 1;
    let unlink = AtkChange::Unlink { inode: InodeNumber(FIRST_INODE_NUMBER), write_time_seconds: emptied.write_time_seconds };
    let unlinked = atk_apply(&mut emptied, &unlink);
    let without_file = matches!(emptied.session().current, PoolVersion::WithTreesAndNoFile(_));
    println!("ATK k1nofile unlink={} with_trees_and_no_file={without_file}", atk_kind(&unlinked));
    let emptied = AtkSnap::of(&emptied);
    let mut tally: BTreeMap<String, usize> = BTreeMap::new();
    for leave in &leaves {
        for count in &counts {
            let mut pool = emptied.pool();
            atk_leave_unblocked(&mut pool, *leave);
            pool.write_time_seconds += 1;
            let change = AtkChange::Inodes { count: *count, write_time_seconds: pool.write_time_seconds };
            let outcome = atk_apply(&mut pool, &change);
            let Some(stuck) = atk_stuck_of(&outcome) else {
                *tally.entry(format!("first={}", atk_head(atk_kind(&outcome)))).or_insert(0) += 1;
                continue;
            };
            let on_no_file = matches!(pool.session().current, PoolVersion::WithTreesAndNoFile(_));
            let after = AtkSnap::of(&pool);
            let (line, ok) = atk_follow_way_two(&after, stuck, &change);
            *tally.entry(format!("stuck on_no_file={on_no_file} ok={ok}")).or_insert(0) += 1;
            println!("ATK k1nofile leave={leave} count={count} on_no_file={on_no_file} {line}");
        }
    }
    println!("ATK k1nofile totals {tally:?}");
}
