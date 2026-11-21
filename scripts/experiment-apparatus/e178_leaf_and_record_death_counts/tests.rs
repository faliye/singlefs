use super::*;

/// K1、K2、K3、A2 与净荷容量：条款给的值（加法写，不写减法）。
#[test]
fn format_constants_match_the_clause_values() {
    assert_eq!(INODE_LEAF_RECORDS, 232 + 1);
    assert_eq!(TOMBSTONE_CONTAINER_RECORDS, 814 + 1);
    assert_eq!(ROOT_NODE_LEAF_CONTAINERS, 134 + 1);
    assert_eq!(RETIREMENT_RECORD_BYTES, 33 + 1);
    assert_eq!(RETIREMENT_CONTAINER_RECORDS, 958 + 1);
    assert_eq!(DATA_UNIT_PAYLOAD_BYTES, 32_633 + 1);
    assert_eq!(every_reader().len(), 12 + 8);
}

/// 「不给代码键就拒」：不给 --code-key-file → 参数解析报错（不碰转交的管道），出路点名取键脚本。
#[test]
fn without_a_code_key_the_arguments_are_refused() {
    let refused = parse_with(&[], &test_environment_without_relay()).expect_err("不给代码键要报错");
    assert!(refused.contains("device-code-key.py run"), "{refused}");
}

/// 认不出的选项报错（用一个从没用过的名字）。
#[test]
fn an_unrecognized_argument_is_refused() {
    let environment = test_environment_without_relay();
    let refused = parse_with(&["--never-used-option"], &environment).expect_err("认不出的选项要报错");
    assert!(refused.contains("认不得的参数"), "{refused}");
}
