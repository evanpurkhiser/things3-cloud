use crate::common::assert_fixture;

#[test]
fn test_new_bare_create_payload() {
    assert_fixture("test_new_bare_create_payload");
}

#[test]
fn test_new_when_today_payload() {
    assert_fixture("test_new_when_today_payload");
}

#[test]
fn test_new_after_gap_payload() {
    assert_fixture("test_new_after_gap_payload");
}
