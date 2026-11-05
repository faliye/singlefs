// 让共用镜像那份文件进工作区（目标 images 经 mod 引到它）。
mod common_images_for_the_given_states_engine;

#[test]
fn hands_out_an_image() {
    common_images_for_the_given_states_engine::hand_out_allowing_the_gpu_to_refuse_its_tables();
}
