// research-build-and-replay.sh 的 research-unit-tests 格样本：一条单测按字面 concat!(env!("CARGO_MANIFEST_DIR"), "/../../research/results/<名>") 读一份留存产物，
// 产物树里没有、git 历史里也没有：这一格开 test-plan 之前就判红「取不回」，一个 cargo test 都不起
pub fn add(left: u32, right: u32) -> u32 {
    left + right
}

#[cfg(test)]
mod tests {
    #[test]
    fn adds_two_and_two_to_four() {
        assert_eq!(super::add(2, 2), 4);
    }

    #[test]
    fn reads_an_archived_product() {
        let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../research/results/sample-product.out")).expect("读得到留存产物");
        assert!(!text.is_empty());
    }
}
