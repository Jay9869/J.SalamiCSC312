use std::ptr::null;

use crate::{
    parser::parse,
    token::{
        keyword, Token,
        TokenType::{
            self, And, Bang, BangEqual, Comma, Else, Eof, Equal, EqualEqual, False, Fun, Greater,
            GreaterEqual, Identifier, If, LBrace, LParen, Less, LessEqual, Minus, Nil, Number, Or,
            Plus, Print, RBrace, RParen, Return, Semicolon, Slash, Star, Str, True, Var, While,
        },
    },
};

/// Cut `source` into tokens. Returns everything it managed to scan alongside every
/// error it found; the caller decides whether to go on.
pub fn scan(source: &str) -> (Vec<Token>, Vec<String>) {
    let mut s = Scanner {
        src: source.chars().collect(),
        start: 0,
        current: 0,
        line: 1,
        tokens: Vec::new(),
        errors: Vec::new(),
    };
    s.run();
    (s.tokens, s.errors)
}

struct Scanner {
    src: Vec<char>,
    start: usize,
    current: usize,
    line: usize,
    tokens: Vec<Token>,
    errors: Vec<String>,
}

impl Scanner {
    fn run(&mut self) {
        // TODO(you): drive the scan: read one token at a time until the source runs out, then
        //            add the EOF token. Spec 6.1 says which line EOF carries.
        while self.at_end() == false {
            // println!(
            //     "still in bounds, {} - {}. src.len is {}",
            //     self.start,
            //     self.current,
            //     self.src.len()
            // );
            self.start = self.current;
            self.scan_token();
        }

        // most difficult thing to implement
        self.start = self.current; // the fix (from Mr Francis assistance) without it, noticed that the start was value len-1, current was len but with it the start was len and the current was len
                                   // println!(
                                   //     "still in bounds, {} - {}. src.len is {}",
                                   //     self.start,
                                   //     self.current,
                                   //     self.src.len()
                                   // );
        let last_token = self.tokens.last();
        let mut last_token_line = 1;

        if last_token.is_some() {
            last_token_line = last_token.unwrap().line;
            self.line = last_token_line;
        } else {
            self.line = last_token_line;
        }
        self.add(Eof);
    }

    fn scan_token(&mut self) {
        // TODO(you): recognise one token. Spec 1.2 lists every token type, 1.1 covers
        //            whitespace and comments, and an unrecognised character is 'Character is
        //            not part of any token.' (5.1).

        // punctuation and arithmetic group tokens
        let pattern_beginning: char = self.advance();

        // println!("pattern beginning is {}", pattern_beginning);
        if pattern_beginning == '(' {
            self.add(LParen);
        } else if pattern_beginning == ')' {
            self.add(RParen);
        } else if pattern_beginning == '{' {
            self.add(LBrace);
        } else if pattern_beginning == '}' {
            self.add(RBrace);
        } else if pattern_beginning == ',' {
            self.add(Comma);
        } else if pattern_beginning == ';' {
            self.add(Semicolon);
        } else if pattern_beginning == '+' {
            self.add(Plus);
        } else if pattern_beginning == '-' {
            self.add(Minus);
        } else if pattern_beginning == '*' {
            self.add(Star);
        }
        // two character tokens (equality group)

        // ! possibilities
        else if pattern_beginning == '!' {
            // I realized I could have used matches for this section later on...
            if self.peek() == '=' {
                self.add(BangEqual);
            } else {
                self.add(Bang);
            }
        }
        // = possibilities
        else if pattern_beginning == '=' {
            if self.peek() == '=' {
                self.add(EqualEqual);
            } else {
                self.add(Equal);
            }
        }
        // > possibilities
        else if pattern_beginning == '>' {
            if self.peek() == '=' {
                self.add(GreaterEqual);
            } else {
                self.add(Greater);
            }
        }
        // < possibilities
        else if pattern_beginning == '<' {
            if self.peek() == '=' {
                self.add(LessEqual);
            } else {
                self.add(Less);
            }
        }
        // \ possibilities alongside whitespace
        else if pattern_beginning == '\t' || pattern_beginning == '\r' || pattern_beginning == ' '
        {
        }
        // new line
        else if pattern_beginning == '\n' {
            self.line += 1;
        }
        // / possibilities
        else if pattern_beginning == '/' {
            if self.peek() == '/' && !self.at_end() {
                while self.peek() != '\n' {
                    self.advance();
                }
            } else {
                self.add(Slash);
            }
        }
        // strings
        else if pattern_beginning == '"' {
            self.string();
        }
        // numbers
        else if pattern_beginning == '0'
            || pattern_beginning == '1'
            || pattern_beginning == '2'
            || pattern_beginning == '3'
            || pattern_beginning == '4'
            || pattern_beginning == '5'
            || pattern_beginning == '6'
            || pattern_beginning == '7'
            || pattern_beginning == '8'
            || pattern_beginning == '9'
        {
            self.number();
        }
        // identifiers
        else if pattern_beginning.is_ascii_alphabetic() || pattern_beginning == '_' {
            self.identifier();
        } else {
            self.error(self.line, "Character is not part of any token.");
        }
    }

    fn string(&mut self) {
        // TODO(you): scan a string literal. A string may span lines (1.5); an unterminated one
        //            is reported at the line it opened on (5.1).

        let mut string_holder: Vec<char> = Vec::new();

        let current_string_line = self.line;
        while self.peek() != '"' {
            if self.peek() == '\n' {
                self.line += 1;
            }
            if self.at_end() {
                self.error(current_string_line, "String is never closed.");
                return;
            }
            string_holder.push(self.advance());
        }
        if self.peek() == '"' {
            // self.advance();
            self.add(Str);
        }
    }

    fn number(&mut self) {
        // TODO(you): scan a number literal: digits, then a fractional part only when a digit
        //            follows the dot (1.4).
        let nunber_holder: Vec<char> = Vec::new();

        // println!(
        //     "still in bounds, val {} at {} - {}. src.len is {}",
        //     self.src[self.current],
        //     self.start,
        //     self.current,
        //     self.src.len()
        // );

        while self.peek() == '0'
            || self.peek() == '1'
            || self.peek() == '2'
            || self.peek() == '3'
            || self.peek() == '4'
            || self.peek() == '5'
            || self.peek() == '6'
            || self.peek() == '7'
            || self.peek() == '8'
            || self.peek() == '9'
        {
            // self.advance();
        }

        if self.peek() == '.' {
            if self.peek_next() == '0'
                || self.peek_next() == '1'
                || self.peek_next() == '2'
                || self.peek_next() == '3'
                || self.peek_next() == '4'
                || self.peek_next() == '5'
                || self.peek_next() == '6'
                || self.peek_next() == '7'
                || self.peek_next() == '8'
                || self.peek_next() == '9'
            {
                // self.advance();
                while self.peek() == '0'
                    || self.peek() == '1'
                    || self.peek() == '2'
                    || self.peek() == '3'
                    || self.peek() == '4'
                    || self.peek() == '5'
                    || self.peek() == '6'
                    || self.peek() == '7'
                    || self.peek() == '8'
                    || self.peek() == '9'
                {
                    // nunber_holder.push(self.advance());
                }
            }
        } else {
            // nunber_holder.push(self.advance());
        }

        self.add(Number);
    }

    fn identifier(&mut self) {
        // TODO(you): scan an identifier, then decide whether it is a keyword; keyword() in
        //            token.rs does the lookup (1.2, 1.3).
        let mut identifier_holder: Vec<char> = Vec::new();

        identifier_holder.push(self.src[self.current - 1]); // I used a debug println and discovered without this, the first character of the identifier wouldnt be present
        while self.peek() != ' ' && !self.at_end() {
            identifier_holder.push(self.advance());
        }

        let holder_concat: String = identifier_holder.iter().collect();
        // let holder_slice: &str = &holder_concat;
        // let holder_slice: &'static str = &holder_concat;
        let holder_slice: String = holder_concat;

        if holder_slice == "and" {
            self.add(And);
        } else if holder_slice == "else" {
            self.add(Else);
        } else if holder_slice == "false" {
            self.add(False);
        } else if holder_slice == "fun" {
            self.add(Fun);
        } else if holder_slice == "if" {
            self.add(If);
        } else if holder_slice == "nil" {
            self.add(Nil);
        } else if holder_slice == "or" {
            self.add(Or);
        } else if holder_slice == "print" {
            self.add(Print);
        } else if holder_slice == "return" {
            self.add(Return);
        } else if holder_slice == "true" {
            self.add(True);
        } else if holder_slice == "var" {
            self.add(Var);
        } else if holder_slice == "while" {
            self.add(While);
        } else {
            self.add(Identifier);
        }

        // println!("holder_slice is {}", holder_slice)
    }

    // --- primitives ---------------------------------------------------------------

    fn at_end(&self) -> bool {
        self.current >= self.src.len()
    }

    fn advance(&mut self) -> char {
        let c = self.src[self.current];
        self.current += 1;
        c
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.at_end() || self.src[self.current] != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn peek(&self) -> char {
        if self.at_end() {
            '\0'
        } else {
            self.src[self.current]
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.src.len() {
            '\0'
        } else {
            self.src[self.current + 1]
        }
    }

    fn add(&mut self, kind: TokenType) {
        self.tokens.push(Token {
            kind,
            lexeme: self.src[self.start..self.current].iter().collect(),
            line: self.line,
        });
    }

    fn error(&mut self, line: usize, message: &str) {
        self.errors
            .push(format!("[line {}] Error: {}", line, message));
    }
}
