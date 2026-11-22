#[derive(PartialEq, Debug)]
pub enum SflTokens {
    Num(f32),
    MathPlus,
    MathMinus,
    MathDivide,
    MathTimes,
    MathPow,
    Comma,
    SemiColon,
    Equals,
    OpenPar,
    ClosedPar,
    OpenCurly,
    ClosedCurly,
    Identifier(String),
    Eof
}
pub fn tokenize_sfl(input: &str) -> Result<Vec<SflTokens>, String> {
    let mut chars = input.chars().peekable();
    let mut tokens: Vec<SflTokens> = Vec::new();

    while let Some(&char) = chars.peek() {
        if char.is_ascii_whitespace() {
            chars.next();    
            continue;
        }
        
        match char {
            '=' => {
                tokens.push(SflTokens::Equals);
                chars.next();
            },
            '+' => {
                tokens.push(SflTokens::MathPlus);
                chars.next();
            },
            '-' => {
                tokens.push(SflTokens::MathMinus);
                chars.next();
            },
            '*' => {
                tokens.push(SflTokens::MathTimes);
                chars.next();
            },
            '/' => {
                tokens.push(SflTokens::MathDivide);
                chars.next();
            },
            '^' => {
                tokens.push(SflTokens::MathPow);
                chars.next();
            },
            ',' => {
                tokens.push(SflTokens::Comma);
                chars.next();
            },
            ';' => {
                tokens.push(SflTokens::SemiColon);
                chars.next();
            },
            '(' => {
                tokens.push(SflTokens::OpenPar);
                chars.next();
            },
            ')' => {
                tokens.push(SflTokens::ClosedPar);
                chars.next();
            },
            '{' => {
                tokens.push(SflTokens::OpenCurly);
                chars.next();
            },
            '}' => {
                tokens.push(SflTokens::ClosedCurly);
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
                tokens.push(SflTokens::Identifier(str));
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
                tokens.push(SflTokens::Num(num));
            },
            _=> return Err(format!("Unexpected `{}`", char))
        }

    }

    tokens.push(SflTokens::Eof);

    Ok(tokens)
}