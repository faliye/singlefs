
/// m3-w3-r3 正推腿的探针（只在副本里）：真的 `after_a_container_is_opened_at_the_right_end` 逐个新开容器的形状，给 `cost_model.py` 对账。
#[cfg(test)]
mod w3_probe_shape {
    use super::*;

    #[test]
    fn w3_shape_probe() {
        if std::env::var("W3_PROBE").unwrap_or_default().contains("shape") {
            for fanout in [2usize, 3, 5, 135] {
                let mut shape = ContainerTreeShape::zero_root();
                let limit = if fanout == 135 { 20000 } else { 300 };
                for containers in 1..=limit {
                    shape = shape.after_a_container_is_opened_at_the_right_end(fanout);
                    let levels = usize::from(shape.root_level());
                    if fanout != 135 || containers <= 5 || containers % 1000 == 0 || containers == 135 || containers == 136 || containers == 18225 || containers == 18226 {
                        let nodes: Vec<usize> = (1..=levels).map(|level| shape.nodes_at_level(u8::try_from(level).expect("层"))).collect();
                        println!("W3SHAPE fanout={fanout} containers={containers} root_level={levels} nodes_per_level={nodes:?} rightmost_path_len={}", shape.rightmost_path().len());
                    }
                }
            }
        }
    }
}
