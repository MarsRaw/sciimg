use sciimg::debayer::DebayerMethod;
use std::str::FromStr;

#[test]
fn test_debayer_method_from_string() {
    assert_eq!(
        DebayerMethod::from_str("malvar").unwrap(),
        DebayerMethod::Malvar
    );

    assert_eq!(
        DebayerMethod::from_str("MALVAR").unwrap(),
        DebayerMethod::Malvar
    );

    assert_eq!(
        DebayerMethod::from_str("amaze").unwrap(),
        DebayerMethod::AMaZE
    );

    assert_eq!(
        DebayerMethod::from_str("AMAZE").unwrap(),
        DebayerMethod::AMaZE
    );

    assert_eq!(
        DebayerMethod::from_str("AMaze").unwrap(),
        DebayerMethod::AMaZE
    );

    assert_eq!(
        DebayerMethod::from_str("bilinear").unwrap(),
        DebayerMethod::Bilinear
    );

    assert_eq!(
        DebayerMethod::from_str("BILINEAR").unwrap(),
        DebayerMethod::Bilinear
    );

    assert!(DebayerMethod::from_str("cvsdfdvs").is_err());
}
