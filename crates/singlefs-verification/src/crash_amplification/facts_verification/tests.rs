use super::*;

/// L8：核对键只由事实表摘要定（核对器的代码不进键，CPU 参照与 GPU 核对内核同一把）：事实表改一个字节键就换（那一块现核），不改照旧（复用）；
/// 键就是带标签的那一段摘要。
/// 判别力（`crates/mutations.tsv`）：核对键去掉事实表摘要，这条红。
#[test]
fn the_facts_verifier_key_changes_with_one_byte_of_the_facts_and_stays_without_it() {
    let facts_bytes = b"facts of a flow".to_vec();
    let facts_digest = sha256_digest(&facts_bytes);
    let mut changed_bytes = facts_bytes;
    changed_bytes[0] ^= 1;
    let changed_digest = sha256_digest(&changed_bytes);
    let key = facts_verifier_row_key(&facts_digest);
    assert_eq!(
        facts_verifier_row_key(&facts_digest),
        key,
        "事实表不改，核对键不变：复用"
    );
    assert_ne!(
        facts_verifier_row_key(&changed_digest),
        key,
        "事实表改一个字节，核对键换：现核"
    );
    let mut message = b"singlefs facts verifier row\n".to_vec();
    message.extend_from_slice(&facts_digest);
    assert_eq!(
        key,
        sha256_digest(&message),
        "核对键就是带标签的事实表摘要的摘要，没有别的输入"
    );
    let (directory, mut store) = an_empty_library_for("facts-verifier-key");
    let block_key = VerdictBlockKey {
        node_input_digest: NodeInputDigest([3; DIGEST_BYTES]),
        enumeration_plan_hash: EnumerationPlanHash([4; DIGEST_BYTES]),
        block_start: BlockStartStateIndex(0),
    };
    store
        .store_verifier_block(&block_key, &key, &facts_digest, &[0, 1])
        .expect("存得进核对行");
    assert!(
        store
            .read_verifier_block(&block_key, &key)
            .expect("读得了")
            .is_some(),
        "同一份事实表读得到核对行"
    );
    assert!(
        store
            .read_verifier_block(&block_key, &facts_verifier_row_key(&changed_digest))
            .expect("读得了")
            .is_none(),
        "事实表改了的那一把读不到：那一块现核"
    );
    drop(store);
    drop(directory);
}
