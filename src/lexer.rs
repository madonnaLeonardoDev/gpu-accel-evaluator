#[derive(Debug, Clone, PartialEq)]
pub enum Tokens {
    Number(f32),
    OpenPar,
    ClosedPar,
    Comma,
    Plus,
    Minus,
    Times,
    Divide,
    Power,
    Identifier(String),
    Eof
}

pub fn tokenize(input: &str) -> Result<Vec<Tokens>, String> {
    let mut chars = input.chars().peekable();
    let mut tokens: Vec<Tokens> = Vec::new();

    while let Some(&char) = chars.peek() {

        if char.is_ascii_whitespace() {
            chars.next();
            continue
        }

        match char {
            '+' => {
                tokens.push(Tokens::Plus);
                chars.next();
            },
            '-' => {
                tokens.push(Tokens::Minus);
                chars.next();
            }
            '*' => {
                tokens.push(Tokens::Times);
                chars.next();
            }
            '/' => {
                tokens.push(Tokens::Divide);
                chars.next();
            }
            '^' => {
                tokens.push(Tokens::Power);
                chars.next();
            }
            ',' => {
                tokens.push(Tokens::Comma);
                chars.next();
            }
            '(' => {
                tokens.push(Tokens::OpenPar);
                chars.next();
                
            }
            ')' => {
                tokens.push(Tokens::ClosedPar);
                chars.next();
            },
            ch if ch.is_ascii_alphabetic() => {
                let mut str = String::new();
                str.push(ch);
                chars.next();
                while let Some(&c) = chars.peek() && c.is_ascii_alphanumeric() {
                    str.push(c);
                    chars.next();
                }
                tokens.push(Tokens::Identifier(str));
            },
            ch if ch.is_ascii_digit() || ch == '.' => {
                let mut str = String::new();
                str.push(ch);
                chars.next();

                let mut has_dot = ch == '.';
                while let Some(&c) = chars.peek() && (c.is_ascii_digit() || c == '.') {
                    if c == '.' && has_dot {
                        return Err(format!("Unexpected `{}`",c ));
                    }
                    if c == '.' {
                        has_dot = true;
                    }
                    str.push(c);
                    chars.next();
                }
                let num = str.parse::<f32>().map_err(|_| "Failed parsing number")?;
                tokens.push(Tokens::Number(num));
            }
            _=> {
                return Err(format!("Unexpected `{}`",char ));
            }
        }
    }

    tokens.push(Tokens::Eof);

    Ok(tokens)
}