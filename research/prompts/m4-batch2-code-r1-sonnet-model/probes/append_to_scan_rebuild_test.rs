
fn red_list(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(_) => Some(format!("{invariant}=Violated")),
            InvariantVerdict::NotApplicable(reason) if invariant == "I-1.13" => {
                Some(format!("{invariant}=NotApplicable({reason})"))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn probe_red_lists_of_the_worlds() {
    let pool = build_pool("probe-world-1");
    let mut image = pool.memory_pool();
    move_the_first_location_of_the_data_unit_mapping_entry(&mut image);
    println!("PROBE_P3 world1: {:?}", red_list(&image));

    let pool = build_pool("probe-world-2");
    let mut image = pool.memory_pool();
    zero_the_object_key_of_both_copies_of_the_data_unit(&mut image, &pool);
    println!("PROBE_P3 world2_both: {:?}", red_list(&image));

    let pool = build_pool("probe-world-2-one");
    let mut image = pool.memory_pool();
    zero_the_object_key_of_the_copy_at(&mut image, pool.output.data_pointers[0].locations[0]);
    println!("PROBE_P3 world2_one: {:?}", red_list(&image));

    let mut pool = pool_with_new_inodes("probe-unlink-only", 0);
    unlink_to_the_end(&mut pool, FIRST_INODE_NUMBER);
    println!("PROBE_P3 unlink_no_other_inodes: {:?}", red_list(&pool.memory_pool()));
}
