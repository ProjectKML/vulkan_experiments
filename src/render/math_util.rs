use glam::Vec3;

pub fn direction_from_rotation(rotation: &Vec3) -> Vec3 {
    let cos_y = rotation.y.cos();

    Vec3::new(
        rotation.x.sin() * cos_y,
        rotation.y.sin(),
        rotation.x.cos() * cos_y,
    )
}
