// 样本：判器本体（登记表 judging-body 那一行）与调它的判定段函数（judging-stage 那一行，按函数登记）
/// 一张崩溃镜像的 CPU 判器
pub fn judge_crash_image(image: &Image) -> bool { check_pool_image(image).is_empty() && check_records(image) }
pub fn judge_read_back_range(images: &[Image]) -> usize { images.iter().filter(|image| judge_crash_image(image)).count() }
// 注释里写 check_pool_image( 不算调用点；串字面量里的也不算
pub fn describe() -> &'static str { "check_pool_image(" }
