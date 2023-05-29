use sciimg::output;

#[test]
fn test_determine_output_format_from_filename() {
    assert_eq!(
        output::determine_output_format_from_filename("foo.png").unwrap(),
        output::OutputFormat::PNG
    );
    assert_eq!(
        output::determine_output_format_from_filename("foo.PNG").unwrap(),
        output::OutputFormat::PNG
    );
    assert_eq!(
        output::determine_output_format_from_filename("foo.PnG").unwrap(),
        output::OutputFormat::PNG
    );
    assert_eq!(
        output::determine_output_format_from_filename("foo.tif").unwrap(),
        output::OutputFormat::TIFF
    );
    assert_eq!(
        output::determine_output_format_from_filename("foo.TIF").unwrap(),
        output::OutputFormat::TIFF
    );
    assert_eq!(
        output::determine_output_format_from_filename("foo.tiff").unwrap(),
        output::OutputFormat::TIFF
    );
    assert_eq!(
        output::determine_output_format_from_filename("foo.TIFF").unwrap(),
        output::OutputFormat::TIFF
    );
    assert_eq!(
        output::determine_output_format_from_filename("foo.TiF").unwrap(),
        output::OutputFormat::TIFF
    );
    assert_eq!(
        output::determine_output_format_from_filename("foo.jpg").unwrap(),
        output::OutputFormat::JPEG
    );
    assert_eq!(
        output::determine_output_format_from_filename("foo.JPG").unwrap(),
        output::OutputFormat::JPEG
    );
    assert_eq!(
        output::determine_output_format_from_filename("foo.jpeg").unwrap(),
        output::OutputFormat::JPEG
    );
    assert_eq!(
        output::determine_output_format_from_filename("foo.JPEG").unwrap(),
        output::OutputFormat::JPEG
    );
    assert_eq!(
        output::determine_output_format_from_filename("foo.JpEg").unwrap(),
        output::OutputFormat::JPEG
    );
    assert_eq!(
        output::determine_output_format_from_filename("foo.dng").unwrap(),
        output::OutputFormat::DNG
    );
    assert_eq!(
        output::determine_output_format_from_filename("foo.DNG").unwrap(),
        output::OutputFormat::DNG
    );
    assert_eq!(
        output::determine_output_format_from_filename("foo.DnG").unwrap(),
        output::OutputFormat::DNG
    );
    assert_eq!(
        output::determine_output_format_from_filename("/blah/blah/yadda/yaddo/foo.PNG").unwrap(),
        output::OutputFormat::PNG
    );
    assert_eq!(
        output::determine_output_format_from_filename("../foo.PNG").unwrap(),
        output::OutputFormat::PNG
    );
    assert_eq!(
        output::determine_output_format_from_filename("../foo.PNG/foo.JPG").unwrap(),
        output::OutputFormat::JPEG
    );

    assert!(output::determine_output_format_from_filename("../foo.PNG/foo.bar").is_err());
}

#[test]
fn test_replace_extension_with() {
    assert_eq!(
        output::replace_extension_with("/foo/bar.png", "dng").unwrap(),
        "/foo/bar.dng"
    );
}

#[test]
fn test_replace_extension_for() {
    assert_eq!(
        output::replace_extension_for("foo.bar", output::OutputFormat::DNG).unwrap(),
        "foo.dng"
    );
    assert_eq!(
        output::replace_extension_for("foo.bar", output::OutputFormat::TIFF).unwrap(),
        "foo.tif"
    );
    assert_eq!(
        output::replace_extension_for("foo.bar", output::OutputFormat::PNG).unwrap(),
        "foo.png"
    );
    assert_eq!(
        output::replace_extension_for("foo.bar", output::OutputFormat::JPEG).unwrap(),
        "foo.jpg"
    );

    assert_eq!(
        output::replace_extension_for("foo/bar", output::OutputFormat::JPEG).unwrap(),
        "foo/bar.jpg"
    );
}
