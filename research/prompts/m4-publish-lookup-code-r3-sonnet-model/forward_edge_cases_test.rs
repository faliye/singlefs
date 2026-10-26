
// ---- 正推腿的边界用例（只追加在改后树）：空表、单项、全同身份、重复很多的大表，身份取 u64 与 &[u8]，逐个对线性找 ----
fn edge_case_agrees_with_a_linear_find<Identity: Ord + Copy + std::fmt::Debug>(
    table: &[(Identity, usize)],
    absent: &[Identity],
) -> usize {
    let index = FirstEntryOfEachIdentity::of_the_table(table, |(identity, _)| *identity);
    let mut compared = 0;
    for identity in table.iter().map(|(identity, _)| *identity).chain(absent.iter().copied()) {
        let found_linearly = table.iter().find(|(candidate, _)| *candidate == identity);
        assert!(
            is_the_same_entry(index.first_entry_of(&identity), found_linearly),
            "身份 {identity:?}：索引与线性找不是同一项（表长 {}）",
            table.len()
        );
        compared += 1;
    }
    compared
}

#[test]
fn forward_leg_edge_cases_of_the_sorted_index() {
    let mut compared = 0;
    compared += edge_case_agrees_with_a_linear_find::<u64>(&[], &[0, 5, u64::MAX]);
    compared += edge_case_agrees_with_a_linear_find(&[(4_u64, 0)], &[3, 4, 5]);
    compared += edge_case_agrees_with_a_linear_find(&[(4_u64, 0), (4, 1), (4, 2), (4, 3)], &[3, 5]);
    // 重复很多的大表：2000 项，只有 7 个不同身份，次序被一个与表长互素的步长打乱
    let big: Vec<(u64, usize)> = (0..2000_usize).map(|position| (u64::try_from((position * 37) % 7).unwrap(), position)).collect();
    compared += edge_case_agrees_with_a_linear_find(&big, &[7, 100]);
    // 身份取字节串切片（映射 key 的形态）：长短不一、互为前缀、重复
    let keys: Vec<Vec<u8>> = vec![vec![1, 2], vec![1], vec![], vec![1, 2], vec![1, 2, 0], vec![0xff; 27], vec![1], vec![]];
    let slices: Vec<(&[u8], usize)> = keys.iter().enumerate().map(|(position, key)| (key.as_slice(), position)).collect();
    compared += edge_case_agrees_with_a_linear_find(&slices, &[&[2_u8][..], &[1, 2, 0, 0][..], &[0][..]]);
    // 角色类型 TransactionUnit：每一档都有重复
    let roles = [TransactionUnit::InodeRoot, TransactionUnit::Data(DataUnitIndexInFile(3)), TransactionUnit::ExtentRoot, TransactionUnit::Data(DataUnitIndexInFile(3)), TransactionUnit::InodeRoot, TransactionUnit::AccountingTree];
    let role_table: Vec<(TransactionUnit, usize)> = roles.iter().copied().enumerate().map(|(position, role)| (role, position)).collect();
    compared += edge_case_agrees_with_a_linear_find(&role_table, &[TransactionUnit::DeadlistTree]);
    println!("FWD_EDGE compared={compared}");
}
