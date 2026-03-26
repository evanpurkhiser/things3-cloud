use crate::common::assert_fixture;

#[test]
fn test_reorder_before_payload() {
    assert_fixture("test_reorder_before_payload");
}

#[test]
fn test_reorder_rebalance_payload_and_ancestors() {
    assert_fixture("test_reorder_rebalance_payload_and_ancestors");
}
