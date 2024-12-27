

#[test]
fn byte_conv() {
    let mat = Matrix::new_rand(2);
    let byted_mat = bytemuck::cast_slice(&mat.val);
    let conved_mat = Matrix::from_bytes(byted_mat).unwrap();

    assert_eq!(mat.val, conved_mat.val);
}
