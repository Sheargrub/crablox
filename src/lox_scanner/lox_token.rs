pub struct Token {
    pub data : TokenData,
    pub line : u32,
}

impl Token {
    pub fn to_string(self) -> String {
        format!("{} | {}", self.line, self.data.to_string())
    }
}

enum TokenData {
    // Delimiters
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Semicolon,

    // Arithmetic Operators
    Minus, Plus, Slash, Star, Percent,

    // ??
    Bang, BangEqual,

    // Assignment
    Equal, 
    
    // Comparators
    EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    // Literals
    Identifier(String),
    String(String),
    Number(f64),

    // Constant Literals
    Nil,
    True,
    False,

    // Keywords
    And, Class, Else, Fun, For, If, Or,
    Print, Return, Super, This, Var, While,

    // End of File
    EndOfFile,
}

impl TokenData {
    pub fn to_string(self) -> String {
        match self {
            TokenData::Identifier(val) => format!("Identifier | {val}"),
            TokenData::String(val) => format!("String | \"{val}\""),
            TokenData::Number(val) => format!("Number | {val}"),
            
            TokenData::LeftParen => String::from("("),
            TokenData::RightParen => String::from(")"),
            TokenData::LeftBrace => String::from("{"),
            TokenData::RightBrace => String::from("}"),

            other => String::from("Unidentified static token"), // todo
        }
    }
}

#[cfg(test)]
mod token_data_tests {
    use super::*;

    #[test]
    fn construct_left_paren () {
        let my_token = TokenData::LeftParen;
        assert_eq!(
            "(", 
            my_token.to_string(),
        );
    }

    #[test]
    fn construct_identifier () {
        let my_token = TokenData::Identifier(String::from("println"));
        assert_eq!(
            "Identifier | println", 
            my_token.to_string(),
        );
    }

    #[test]
    fn construct_string () {
        let my_token = TokenData::String(String::from("Hello world!"));
        assert_eq!(
            "String | \"Hello world!\"", 
            my_token.to_string(),
        );
    }

}