#[cfg(test)]
mod tests {
    use crate::next_piece;

    #[test]
    fn test_next_piece() {
        assert!(next_piece() >= 0);
        assert!(next_piece() <= 7);
    }
}
