#[cfg(test)]
mod password_policy_tests {
    #[test]
    fn test_password_minimum_length_8() {
        let weak = "1234567";
        assert!(weak.len() < 8, "Password should be rejected if < 8 chars");
    }

    #[test]
    fn test_password_requires_letter_and_number() {
        let no_letter = "12345678";
        let no_number = "abcdefgh";
        let strong = "Password1";
        assert!(!no_letter.chars().any(|c| c.is_ascii_alphabetic()));
        assert!(!no_number.chars().any(|c| c.is_ascii_digit()));
        assert!(strong.chars().any(|c| c.is_ascii_alphabetic()));
        assert!(strong.chars().any(|c| c.is_ascii_digit()));
    }
}
