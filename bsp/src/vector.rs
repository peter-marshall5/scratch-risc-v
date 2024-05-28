pub type Vec3 = [f32; 3];
pub type Vec2 = [f32; 2];

pub fn subtract<const SIZE: usize>(v1: &[f32; SIZE], v2: &[f32; SIZE]) -> [f32; SIZE] {
    assert_eq!(v1.len(), v2.len());
    let mut result = [0.0; SIZE];
    for i in 0..v1.len() {
        result[i] = v1[i] - v2[i];
    };
    result
}
pub fn cross_product3(v1: &Vec3, v2: &Vec3) -> Vec3 {
    [
        v1[1] * v2[2] - v1[2] * v2[1],
        v1[2] * v2[0] - v1[0] * v2[2],
        v1[0] * v2[1] - v1[1] * v2[0]
    ]
}
pub fn cross_product2(v1: &Vec2, v2: &Vec2) -> f32 {
    v1[0] * v2[1] - v1[1] * v2[0]
}
pub fn magnitude<const SIZE: usize>(v: &[f32; SIZE]) -> f32 {
    let mut square_sum = 0.0;
    for n in v.iter() {
        square_sum += n * n;
    };
    square_sum.sqrt()
}
pub fn dot_product<const SIZE: usize>(v1: &[f32; SIZE], v2: &[f32; SIZE]) -> f32 {
    v1.iter()
        .zip(v2.iter())
        .map(|(n1, n2)| n1 * n2)
        .sum()
}
// pub fn sum<const SIZE: usize>(v: &[f32; SIZE]) -> f32 {
//     v.iter()
//         .sum()
// }
pub fn normalize<const SIZE: usize>(v: &[f32; SIZE]) -> [f32; SIZE] {
    let mut result = [0.0; SIZE];
    let magnitude = magnitude(v);
    for i in 0..SIZE {
        result[i] = v[i] / magnitude;
    };
    result
}
