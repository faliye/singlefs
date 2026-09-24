// 样本：射程外的实验二进制，一条绝对值断言都没有；本阶段不判它，只在成功行里把它的目录列成没罩到的。
fn main() {
    let measured_ratio = 3.0_f64 / 2.0_f64;
    assert!(measured_ratio > measured_ratio / 2.0);
}
