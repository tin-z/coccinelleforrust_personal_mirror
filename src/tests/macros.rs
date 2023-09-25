#![allow(dead_code)]
use super::transformationtest::TransformTest;

static MACROSTEST: TransformTest = TransformTest {
    prefix: "./src/tests/macros/"
};


#[test]
pub fn test1() {
    assert!(MACROSTEST.testtransformation("test1.cocci", "test1.rs", "expected1.rs"))
}
