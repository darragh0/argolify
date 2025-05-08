pub trait CharExtensions {
    fn is_num_sign(&self) -> bool;
    fn is_num_char(&self) -> bool;
    fn is_other_char(&self) -> bool;
}

impl CharExtensions for char {
    fn is_num_sign(&self) -> bool {
        "+-".contains(*self)
    }

    fn is_other_char(&self) -> bool {
        self.is_whitespace() || "#;{},".contains(*self)
    }

    fn is_num_char(&self) -> bool {
        *self == '_' || self.is_ascii_digit()
    }
}
