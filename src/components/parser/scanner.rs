use crate::components as lox;
use lox::parser::token::*;

pub struct LoxScanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    error_strings: Vec<String>,
    start: usize,
    current: usize,
    line: usize,
    inited: bool,
    valid: bool,
}

impl LoxScanner {

    pub fn new(source: &str) -> LoxScanner {
        let source = source.chars().collect();
        let tokens = Vec::new();
        let error_strings = Vec::new();
        
        LoxScanner {
            source,
            tokens,
            error_strings,
            start : 0,
            current : 0,
            line : 1,
            inited : false,
            valid : true,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, Vec<String>> {
        if self.inited {
            if self.valid { Ok(self.tokens.clone()) }
            else { Err(self.error_strings.clone()) }
        }
        else {
            self.inited = true;

            while !self.is_at_end() {
                let c = self.source[self.current];
                self.current += 1;

                match c {
                    '(' => self.add_token(TokenData::LeftParen),
                    ')' => self.add_token(TokenData::RightParen),
                    '{' => self.add_token(TokenData::LeftBrace),
                    '}' => self.add_token(TokenData::RightBrace),
                    ',' => self.add_token(TokenData::Comma),
                    '.' => self.add_token(TokenData::Dot),
                    ';' => self.add_token(TokenData::Semicolon),
                    '+' => self.add_token(TokenData::Plus),
                    '-' => self.add_token(TokenData::Minus),
                    '*' => self.add_token(TokenData::Star),
                    // Slash needs extra handling for comments - see below
                    '%' => self.add_token(TokenData::Percent),

                    '!' => {
                        if self.match_char('=') { self.add_token(TokenData::BangEqual) }
                        else                    { self.add_token(TokenData::Bang) }
                    },
                    '=' => {
                        if self.match_char('=') { self.add_token(TokenData::EqualEqual) }
                        else                    { self.add_token(TokenData::Equal) }
                    },
                    '<' => {
                        if self.match_char('=') { self.add_token(TokenData::LessEqual) }
                        else                    { self.add_token(TokenData::Less) }
                    },
                    '>' => {
                        if self.match_char('=') { self.add_token(TokenData::GreaterEqual) }
                        else                    { self.add_token(TokenData::Greater) }
                    },

                    '/' => {
                        if self.match_char('/') { self.process_comment() }
                        else { self.add_token(TokenData::Slash) }
                    },

                    '"' => self.process_string(),
                    '0'..='9' => self.process_number(),
                    'A'..='z' => self.process_identifier(),

                    ' ' => (),
                    '\r' => (),
                    '\t' => (),

                    '\n' => self.line += 1,

                    _ => self.add_error(&format!("Unexpected character '{c}'.")),
                };

                self.start = self.current;
            }

            self.add_token(TokenData::EndOfFile);
            if self.valid { Ok(self.tokens.clone()) }
            else { Err(self.error_strings.clone()) }
        }
    }

    // Returns false if the scanner cannot provide output.
    // Returns none if the scanner has yet to attempt parsing its string.
    pub fn is_valid(&self) -> Option<bool> {
        if self.inited { Some(self.valid) }
        else { None }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    } 

    // If the next character matches c, consumes it and returns true.
    // Otherwise, returns false.
    fn match_char(&mut self, c: char) -> bool {
        if self.is_at_end() { false }
        else if self.source[self.current] != c { false }
        else {
            self.current += 1;
            true
        }
    }

    fn process_comment(&mut self) {
        while !self.is_at_end() && self.source[self.current] != '\n' {
            self.current += 1;
        }
    }

    fn process_string(&mut self) {
        let begin = self.current;
        let start_line = self.line;
        while !self.is_at_end() && self.source[self.current] != '"' {
            if self.source[self.current] == '\n' { self.line += 1 };
            self.current += 1;
        };
        
        if self.is_at_end() {
            self.add_error(&format!("Unterminated string starting at line [{start_line}]."))
        }
        else {
            let source_slice = &self.source[begin..self.current];
            let input_string: String = source_slice.iter().collect();
            self.add_token(TokenData::StringData(input_string));
            self.current += 1;
        }
    }

    fn process_number(&mut self) {
        let begin = self.current - 1; // Since the number itself triggers this function
        while !self.is_at_end() && is_number(self.source[self.current]) {
            self.current += 1;
        };
        if !self.is_at_end() && self.source[self.current] == '.' {
            self.current += 1;
            while !self.is_at_end() && is_number(self.source[self.current]) {
                self.current += 1;
            };
        };
        let source_slice = &self.source[begin..self.current];
        let number_str: String = source_slice.iter().collect();
        let input_number: f64 = number_str.parse().expect("FATAL: Recieved non-numeric character while parsing number. Note that this should be impossible.");
        self.add_token(TokenData::Number(input_number));
    }

    fn process_identifier(&mut self) {
        let begin = self.current - 1; // Since the letter itself triggers this function
        while !self.is_at_end() && is_alphanumeric(self.source[self.current]) {
            self.current += 1;
        };
        let source_slice = &self.source[begin..self.current];
        let input_string: String = source_slice.iter().collect();
        match input_string.as_str() { // TODO: hash map would likely be more efficient
            "nil" => self.add_token(TokenData::Nil),
            "true" => self.add_token(TokenData::True),
            "false" => self.add_token(TokenData::False),
            "and" => self.add_token(TokenData::And),
            "class" => self.add_token(TokenData::Class),
            "else" => self.add_token(TokenData::Else),
            "fun" => self.add_token(TokenData::Fun),
            "for" => self.add_token(TokenData::For),
            "if" => self.add_token(TokenData::If),
            "or" => self.add_token(TokenData::Or),
            "print" => self.add_token(TokenData::Print),
            "return" => self.add_token(TokenData::Return),
            "super" => self.add_token(TokenData::Super),
            "this" => self.add_token(TokenData::This),
            "var" => self.add_token(TokenData::Var),
            "while" => self.add_token(TokenData::While),
            _ => self.add_token(TokenData::Identifier(input_string)),
        };
    }

    fn add_token(&mut self, data: TokenData) {
        self.tokens.push(Token{data, line: self.line});
    }

    fn add_error(&mut self, message: &str) {
        self.error_strings.push(lox::error::new_error_string(self.line, message));
        self.valid = false;
    }

}

fn is_number(c: char) -> bool {
    match c {
        '0'..='9' => true,
        _ => false,
    }
}

fn is_alpha(c: char) -> bool {
    match c {
        'A'..='z' => true,
        _ => false,
    }
}

fn is_alphanumeric(c: char) -> bool {
    is_alpha(c) || is_number(c)
}



#[cfg(test)]
mod tests {
    use super::*;
    use TokenData::*;

    fn test_scan_generic(in_string: &str, expected_tokens: Vec<Token>) {
        let mut scanner = LoxScanner::new(in_string);
        let tokens = scanner.scan_tokens().expect("Unknown scanning failure.");

        let mut i = 0;
        for expected in expected_tokens {
            assert_eq!(
                expected,
                tokens[i],
                "Expected to recieve token on left, got token on right."
            );
            i += 1;
        };
    }

    #[test]
    fn test_scan_delimiters () {
        let expected_tokens = vec![
            Token::new(LeftParen, 1),
            Token::new(RightParen, 1),
            Token::new(LeftBrace, 1),
            Token::new(RightBrace, 1),
            Token::new(Dot, 1),
            Token::new(Comma, 1),
            Token::new(Semicolon, 1),
            Token::new(EndOfFile, 1),
        ];
        test_scan_generic("(){}.,;", expected_tokens);
    }

    #[test]
    fn test_scan_math_ops () {
        let expected_tokens = vec![
            Token::new(Plus, 1),
            Token::new(Minus, 1),
            Token::new(Star, 1),
            Token::new(Slash, 1),
            Token::new(Percent, 1),
            Token::new(EndOfFile, 1),
        ];
        test_scan_generic("+-*/%", expected_tokens);
    }

    #[test]
    fn test_scan_comparators () {
        let expected_tokens = vec![
            Token::new(Less, 1),
            Token::new(Greater, 1),
            Token::new(LessEqual, 1),
            Token::new(GreaterEqual, 1),
            Token::new(BangEqual, 1),
            Token::new(EqualEqual, 1),
            Token::new(EndOfFile, 1),
        ];
        test_scan_generic("<><=>=!===", expected_tokens);
    }

    #[test]
    fn test_scan_comments () {
        let comment_str = "\
//This comment should be ignored.
//Same with this one.";
        let expected_tokens = vec![
            Token::new(EndOfFile, 2),
        ];
        test_scan_generic(comment_str, expected_tokens);
    }

    #[test]
    fn test_scan_strings () {
        let string_str = "\
\"This string should be processed.\"
\"Same with this one.\"";
        let expected_tokens = vec![
            Token::new(StringData(String::from("This string should be processed.")), 1),
            Token::new(StringData(String::from("Same with this one.")), 2),
            Token::new(EndOfFile, 2),
        ];
        test_scan_generic(string_str, expected_tokens);
    }

    #[test]
    fn test_scan_numbers () {
        let number_str = "\
3.45
83
245.30";
        let expected_tokens = vec![
            Token::new(Number(3.45), 1),
            Token::new(Number(83.0), 2),
            Token::new(Number(245.3), 3),
            Token::new(EndOfFile, 3),
        ];
        test_scan_generic(number_str, expected_tokens);
    }

    #[test]
    fn test_scan_identifiers() {
        let identifier_str = "\
nil true false and class else
fun for if or print return super
this var while foo bar foobar2 cr4b";
        let expected_tokens = vec![
            Token::new(Nil, 1),
            Token::new(True, 1),
            Token::new(False, 1),
            Token::new(And, 1),
            Token::new(Class, 1),
            Token::new(Else, 1),
            Token::new(Fun, 2),
            Token::new(For, 2),
            Token::new(If, 2),
            Token::new(Or, 2),
            Token::new(Print, 2),
            Token::new(Return, 2),
            Token::new(Super, 2),
            Token::new(This, 3),
            Token::new(Var, 3),
            Token::new(While, 3),
            Token::new(Identifier(String::from("foo")), 3),
            Token::new(Identifier(String::from("bar")), 3),
            Token::new(Identifier(String::from("foobar2")), 3),
            Token::new(Identifier(String::from("cr4b")), 3),
            Token::new(EndOfFile, 3),
        ];
        test_scan_generic(identifier_str, expected_tokens);
    }

    #[test]
    fn test_scan_misc() {
        let misc_str = "= !";
        let expected_tokens = vec![
            Token::new(Equal, 1),
            Token::new(Bang, 1),
            Token::new(EndOfFile, 1),
        ];
        test_scan_generic(misc_str, expected_tokens);
    }

    #[test]
    fn test_scan_range_edges() {
        let misc_str = "000.0 999.9 AAA zzz \"AAA\" \"zzz\"";
        let expected_tokens = vec![
            Token::new(Number(0.0), 1),
            Token::new(Number(999.9), 1),
            Token::new(Identifier(String::from("AAA")), 1),
            Token::new(Identifier(String::from("zzz")), 1),
            Token::new(StringData(String::from("AAA")), 1),
            Token::new(StringData(String::from("zzz")), 1),
            Token::new(EndOfFile, 1),
        ];
        test_scan_generic(misc_str, expected_tokens);
    }

    #[test]
    fn test_scan_program () {
        let program_str = "\
for (var i = 0; i <= 10; i = i + 1) {
    print \"Looping once again~\";
    print i;
    myFunction();
    // Helpful documentation
}";
        let expected_tokens = vec![
            Token::new(For, 1),
            Token::new(LeftParen, 1),
            Token::new(Var, 1),
            Token::new(Identifier(String::from("i")), 1),
            Token::new(Equal, 1),
            Token::new(Number(0.0), 1),
            Token::new(Semicolon, 1),
            Token::new(Identifier(String::from("i")), 1),
            Token::new(LessEqual, 1),
            Token::new(Number(10.0), 1),
            Token::new(Semicolon, 1),
            Token::new(Identifier(String::from("i")), 1),
            Token::new(Equal, 1),
            Token::new(Identifier(String::from("i")), 1),
            Token::new(Plus, 1),
            Token::new(Number(1.0), 1),
            Token::new(RightParen, 1),
            Token::new(LeftBrace, 1),

            Token::new(Print, 2),
            Token::new(StringData(String::from("Looping once again~")), 2),
            Token::new(Semicolon, 2),

            Token::new(Print, 3),
            Token::new(Identifier(String::from("i")), 3),
            Token::new(Semicolon, 3),

            Token::new(Identifier(String::from("myFunction")), 4),
            Token::new(LeftParen, 4),
            Token::new(RightParen, 4),
            Token::new(Semicolon, 4),

            // Comment on line 5 yields no tokens

            Token::new(RightBrace, 6),
            Token::new(EndOfFile, 6),
        ];
        test_scan_generic(program_str, expected_tokens);
    }

    #[test]
    fn test_error_unexpected_char () {
        let string_str = "\
#@
$";
        let mut scanner = LoxScanner::new(string_str);
        let outcome = scanner.scan_tokens();
        if let Err(error_strings) = outcome {
            assert_eq!(
                3,
                error_strings.len(),
                "Expected to recieve 3 errors, got {}.", error_strings.len()
            );
            assert_eq!(
                vec![
                    String::from("[Line 1] Error: Unexpected character '#'."),
                    String::from("[Line 1] Error: Unexpected character '@'."),
                    String::from("[Line 2] Error: Unexpected character '$'."),
                ],
                error_strings,
                "Expected to recieve error message on left, got error message on right."
            );
        } else {
            panic!("Unterminated string failed to return an error.");
        }
    }

    #[test]
    fn test_error_unterminated_string () {
        let string_str = "\
        \"This string is unterminated.
        It even trails onto a second line. Yikes.";
        let mut scanner = LoxScanner::new(string_str);
        let outcome = scanner.scan_tokens();
        if let Err(error_strings) = outcome {
            assert_eq!(
                1,
                error_strings.len(),
                "Expected to recieve 1 error, got {}.", error_strings.len()
            );
            assert_eq!(
                String::from("[Line 2] Error: Unterminated string starting at line [1]."),
                error_strings[0],
                "Expected to recieve error message on left, got error message on right."
            );
        } else {
            panic!("Unterminated string failed to return an error.");
        }
    }
    
    #[test]
    fn test_repeated_calls() {
        let mut valid_scanner = LoxScanner::new("...");
        let _ = valid_scanner.scan_tokens(); // discard
        let tokens = valid_scanner.scan_tokens().expect("Unknown scanning failure.");
        let expected_tokens = vec![Token::new(Dot, 1), Token::new(Dot, 1), Token::new(Dot, 1), Token::new(EndOfFile, 1)];
        assert_eq!(
            tokens,
            expected_tokens,
            "Was unable to accurately fetch scan_tokens() on a repeated call to a valid LoxScanner object."
        );

        let mut invalid_scanner = LoxScanner::new("@");
        let _ = invalid_scanner.scan_tokens(); // discard
        let error = invalid_scanner.scan_tokens();
        let expected_error_str = String::from("[Line 1] Error: Unexpected character '@'.");
        let expected_error: Result<Vec<Token>, Vec<String>> = Err(vec![expected_error_str]);
        assert_eq!(
            error,
            expected_error,
            "Was unable to accurately fetch scan_tokens() on a repeated call to an invalid LoxScanner object."
        );
    }
}