// 样本：判定段整份（登记表 judging-stage 那一行），从判定库读回的判表上判
use singlefs_checker::walk::check_pool_image;
pub fn judge_one_block_read_back(reader: &Reader) -> usize { check_pool_image(reader).len() }
