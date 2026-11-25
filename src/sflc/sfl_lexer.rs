#[derive(PartialEq, Debug)]
pub enum SdflTokens {
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
    OpenBracket,
    ClosedBracket,
    Identifier(String),
    Eof
}
pub fn tokenize_sfl(input: &str) -> Result<Vec<SdflTokens>, String> {
    let mut chars = input.chars().peekable();
    let mut tokens: Vec<SdflTokens> = Vec::new();

    while let Some(&char) = chars.peek() {
        if char.is_ascii_whitespace() {
            chars.next();    
            continue;
        }
        
        match char {
            '=' => {
                tokens.push(SdflTokens::Equals);
                chars.next();
            },
            '+' => {
                tokens.push(SdflTokens::MathPlus);
                chars.next();
            },
            '-' => {
                tokens.push(SdflTokens::MathMinus);
                chars.next();
            },
            '*' => {
                tokens.push(SdflTokens::MathTimes);
                chars.next();
            },
            '/' => {
                tokens.push(SdflTokens::MathDivide);
                chars.next();
            },
            '^' => {
                tokens.push(SdflTokens::MathPow);
                chars.next();
            },
            ',' => {
                tokens.push(SdflTokens::Comma);
                chars.next();
            },
            ';' => {
                tokens.push(SdflTokens::SemiColon);
                chars.next();
            },
            '(' => {
                tokens.push(SdflTokens::OpenPar);
                chars.next();
            },
            ')' => {
                tokens.push(SdflTokens::ClosedPar);
                chars.next();
            },
            '{' => {
                tokens.push(SdflTokens::OpenCurly);
                chars.next();
            },
            '}' => {
                tokens.push(SdflTokens::ClosedCurly);
                chars.next();
            },
            '[' => {
                tokens.push(SdflTokens::OpenBracket);
                chars.next();
            },
            ']' => {
                tokens.push(SdflTokens::ClosedBracket);
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
                tokens.push(SdflTokens::Identifier(str));
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
                tokens.push(SdflTokens::Num(num));
            },
            _=> return Err(format!("Unexpected `{}`", char))
        }

    }

    tokens.push(SdflTokens::Eof);

    Ok(tokens)
}