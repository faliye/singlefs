// research-build-and-replay.sh 的 research-unit-tests 格样本：一条单测按字面 concat!(env!("CARGO_MANIFEST_DIR"), "/../../research/results/<名>") 读一份留存产物
// （写法照 scripts/experiment-apparatus/lib.rs 那一段）；产物只在 git 历史里，这一格开 test-plan 之前先取回、放回原处，单测读得到，判绿
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
