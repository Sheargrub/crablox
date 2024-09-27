#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct Token {
    pub data : TokenData,
    pub line : usize,
}

impl Token {
    pub fn new(data: TokenData, line: usize) -> Token {
        Token{data, line}
    }

    pub fn to_string(self) -> String {
        format!("[{}] {}", self.line, self.data.to_string())
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum TokenData {
    // Delimiters
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Semicolon,

    // Arithmetic operators
    Minus, Plus, Slash, Star, Percent,

    // Negation operator
    Bang,

    // Assignment
    Equal, 
    
    // Comparators
    EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,
    BangEqual,

    // Literals
    Identifier(String),
    StringData(String),
    Number(f64),

    // Reserved words
    Nil, True, False,
    And, Class, Else, Fun, For, If, Or,
    Print, Return, Super, This, Var, While,

    // End of file
    EndOfFile,
}

#[cfg(test)]
mod token_tests {
    use super::*;

    #[test]
    fn construct_identifier () {
        let my_token = Token {
            data : TokenData::Identifier(String::from("println")),
            line : 5,
        };

        assert_eq!(
            "[5] Identifier | println", 
            my_token.to_string(),
        );
    }
}

impl TokenData {
    pub fn to_string(self) -> String {
        match self {
            TokenData::Identifier(val) => format!("Identifier | {val}"),
            TokenData::StringData(val) => format!("String | \"{val}\""),
            TokenData::Number(val) => format!("Number | {val}"),
            
            TokenData::LeftParen => String::from("("),
            TokenData::RightParen => String::from(")"),
            TokenData::LeftBrace => String::from("{"),
            TokenData::RightBrace => String::from("}"),

            _ => String::from("Unidentified static token"), // todo
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
        let my_token = TokenData::StringData(String::from("Hello world!"));
        assert_eq!(
            "String | \"Hello world!\"", 
            my_token.to_string(),
        );
    }

    #[test]
    fn construct_number () {
        let my_token = TokenData::Number(42.0);
        assert_eq!(
            "Number | 42", 
            my_token.to_string(),
        );
    }

}