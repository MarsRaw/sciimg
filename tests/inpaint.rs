use sciimg::{imagebuffer::ImageBuffer, inpaint};

const INPAINT_TEST_IMAGE: &str = "tests/testdata/MSL_MAHLI_INPAINT_Sol2904_V1.png";

#[test]
fn test_find_starting_point() {
    let inpaint_mask = ImageBuffer::from_file(&String::from(INPAINT_TEST_IMAGE)).unwrap();

    let start_point = inpaint::find_starting_point(&inpaint_mask);

    assert!(start_point.is_some());

    let start = start_point.unwrap();
    assert_eq!(1581, start.x);
    assert_eq!(15, start.y);
    assert_eq!(0, start.score);
}
