#[test]
fn probe() {
    let mut pressed_pool = pool_with_new_inodes("probe-pressed", 3);
    let (pressed, pressed_image) =
        create_one_inode_with_the_watermark_from(&mut pressed_pool, pressed_back_to(3));
    println!("PROBE watermark={} birth3={:?} txg={} verdict_I96={:?} violated={:?}",
        pressed.inode_number_watermark(), object_birth_of(&pressed, 3), pressed.root.checkpoint_txg.0,
        verdict_of(&pressed_image, "I-9.6"), violated_invariants(&pressed_image));
}
