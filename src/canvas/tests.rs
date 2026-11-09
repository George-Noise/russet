use super::*;

#[test]
fn region_bytes_has_expected_size_and_corners() {
    let mut canvas = Canvas::new(4, 4, Color32::WHITE);
    let stride = canvas.width as usize * 4;
    // offset(row, col) = row * stride + col * 4 — byte offset of a pixel.
    let offset = |row: usize, col: usize| row * stride + col * 4;
    {
        let data = canvas.pixmap_mut().data_mut();
        let top_left = offset(1, 1);
        data[top_left..top_left + 4].copy_from_slice(&[10, 20, 30, 255]);
        let bottom_right = offset(2, 2);
        data[bottom_right..bottom_right + 4].copy_from_slice(&[40, 50, 60, 255]);
    }

    let region = canvas.region_bytes(1, 1, 2, 2);
    assert_eq!(region.len(), 2 * 2 * 4);
    assert_eq!(&region[0..4], &[10, 20, 30, 255]);
    assert_eq!(&region[12..16], &[40, 50, 60, 255]);
}

#[test]
fn region_bytes_clips_out_of_bounds() {
    let canvas = Canvas::new(4, 4, Color32::WHITE);
    assert!(canvas.region_bytes(3, 3, 10, 10).len() == 4);
    assert!(canvas.region_bytes(10, 10, 2, 2).is_empty());
}
