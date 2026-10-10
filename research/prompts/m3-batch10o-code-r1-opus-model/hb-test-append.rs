/// 攻方原型（m3-batch10o-code-r1 opus，只在冻结副本的拷贝上）：H-b 那张镜像上 I-5.1 红的原文，与对照镜像上 I-5.1 的判定。
#[test]
fn opus_o1_print_the_i51_text_on_the_h_b_image() {
    for shape in [
        SecondInstanceTablePageShape::CopyOnDeviceZeroBroken,
        SecondInstanceTablePageShape::FirstLocationOfItsPointerNamesTheFirstPage,
    ] {
        let image = image_with_the_newest_instance_table_in_two_pages_and_the_abandoned_instance_row_on_the_second(
            "opus-o1-h-b-i51",
            shape,
        );
        let verdicts = check_pool_image(&image);
        for (name, found) in &verdicts {
            if *name == "I-5.1" || *name == "I-2.1" || *name == "I-9.29" {
                println!("OPUSO1-HB shape={shape:?} {name} {found:?}");
            }
        }
        println!("OPUSO1-HB shape={shape:?} violated={:?}", violated_invariants(&verdicts));
    }
}
