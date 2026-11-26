
#[derive(Debug, Clone, PartialEq)]
pub enum ExprTokens {
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

pub fn tokenize_expr(input: &str) -> Result<Vec<ExprTokens>, String> {
    let mut chars = input.chars().peekable();
    let mut tokens: Vec<ExprTokens> = Vec::new();

    while let Some(&char) = chars.peek() {

        if char.is_ascii_whitespace() {
            chars.next();
            continue
        }

        match char {
            '+' => {
                tokens.push(ExprTokens::Plus);
                chars.next();
            },
            '-' => {
                tokens.push(ExprTokens::Minus);
                chars.next();
            }
            '*' => {
                tokens.push(ExprTokens::Times);
                chars.next();
            }
            '/' => {
                tokens.push(ExprTokens::Divide);
                chars.next();
            }
            '^' => {
                tokens.push(ExprTokens::Power);
                chars.next();
            }
            ',' => {
                tokens.push(ExprTokens::Comma);
                chars.next();
            }
            '(' => {
                tokens.push(ExprTokens::OpenPar);
                chars.next();
                
            }
            ')' => {
                tokens.push(ExprTokens::ClosedPar);
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
                tokens.push(ExprTokens::Identifier(str));
            },
            ch if ch.is_ascii_digit() || ch == '.' => {
                let mut str = String::new();
                str.push(ch);
                chars.next();

                let mut has_dot = ch == '.';
                while let Some(&c) = chars.peek() && (c.is_ascii_digit() || c == '.') {
                    if c == '.' && has_dot {
                        return Err(format!("Unexpected `{}`",char ));
                    }
                    if c == '.' {
                        has_dot = true;
                    }
                    str.push(c);
                    chars.next();
                }
                let num = str.parse::<f32>().map_err(|_| "Failed parsing number")?;
                tokens.push(ExprTokens::Number(num));
            }
            _=> {
                return Err(format!("Unexpected `{}`",char ));
            }
        }
    }

    tokens.push(ExprTokens::Eof);

    Ok(tokens)
}