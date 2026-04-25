//! Seeded fixed module for the synod test-fix loop demo.

pub fn answer() -> i32 {
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn answer_should_be_zero() {
        assert_eq!(answer(), 0);
    }
}
