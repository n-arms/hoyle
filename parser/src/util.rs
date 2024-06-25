macro_rules! parser {
    ($lifetime:lifetime, $typ:ty) => {
        impl Parser<Token<$lifetime>, $typ, Error = Simple<Token<$lifetime>>> + Clone
    }
}
