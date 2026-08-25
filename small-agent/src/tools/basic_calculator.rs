use serde::Deserialize;

use crate::tools::Tool;

#[derive(Deserialize)]
struct CalculatorArgs {
    operation: String,
    a: f64,
    b: f64,
}

pub struct BasicCalculatorTool;

impl Tool for BasicCalculatorTool {
    fn name(&self) -> &'static str {
        "basic_calculator"
    }

    fn description(&self) -> &'static str {
        "数学计算器。支持直接传入算术表达式（如 '1+7+8+7+6+5+6+5+9+2' 或 '21+21'），也支持 JSON 格式"
    }

    fn call(&self, arguments: &str) -> String {
        let input = arguments.trim().trim_matches('"').trim_matches('\'');

        if let Ok(args) = serde_json::from_str::<CalculatorArgs>(input) {
            let op = args.operation.trim().to_lowercase();
            return match op.as_str() {
                "add" => format!("{} + {} = {}", args.a, args.b, args.a + args.b),
                "subtract" => format!("{} - {} = {}", args.a, args.b, args.a - args.b),
                "multiply" => format!("{} x {} = {}", args.a, args.b, args.a * args.b),
                "divide" => {
                    if args.b == 0.0 {
                        "ERROR: division by zero".to_owned()
                    } else {
                        format!("{} / {} = {}", args.a, args.b, args.a / args.b)
                    }
                }
                _ => format!("ERROR: unsupported operation: '{op}'"),
            };
        }
        match eval_expression(input) {
            Ok(result) => {
                if (result.fract()).abs() < 1e-9 {
                    format!("{input} = {}", result as i64)
                } else {
                    format!("{input} = {result}")
                }
            }
            Err(err) => {
                format!("ERROR: 计算表达式 '{input}' 失败: {err}")
            }
        }
    }
}

fn eval_expression(expr: &str) -> Result<f64, String> {
    let tokens = tokenize(expr)?;
    let mut pos = 0;
    let result = parse_expr(&tokens, &mut pos)?;
    if pos < tokens.len() {
        return Err(format!("未预期的多余字符: {:?}", &tokens[pos..]));
    }
    Ok(result)
}

#[derive(Debug, PartialEq, Clone)]
enum Token {
    Num(f64),
    Plus,
    Minus,
    Mul,
    Div,
    LParen,
    RParen,
}

fn tokenize(s: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        match c {
            '+' => {
                tokens.push(Token::Plus);
                i += 1;
            }
            '-' => {
                tokens.push(Token::Minus);
                i += 1;
            }
            '*' | 'x' | 'X' => {
                tokens.push(Token::Mul);
                i += 1;
            }
            '/' => {
                tokens.push(Token::Div);
                i += 1;
            }
            '(' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            '0'..='9' | '.' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let num_str: String = chars[start..i].iter().collect();
                let num = num_str
                    .parse::<f64>()
                    .map_err(|_| format!("无法解析数字: {num_str}"))?;
                tokens.push(Token::Num(num));
            }
            _ => return Err(format!("非法字符: '{c}'")),
        }
    }

    Ok(tokens)
}

fn parse_expr(tokens: &[Token], pos: &mut usize) -> Result<f64, String> {
    let mut result = parse_term(tokens, pos)?;

    while *pos < tokens.len() {
        match tokens[*pos] {
            Token::Plus => {
                *pos += 1;
                result += parse_term(tokens, pos)?;
            }
            Token::Minus => {
                *pos += 1;
                result -= parse_term(tokens, pos)?;
            }
            _ => break,
        }
    }

    Ok(result)
}

fn parse_term(tokens: &[Token], pos: &mut usize) -> Result<f64, String> {
    let mut result = parse_factor(tokens, pos)?;

    while *pos < tokens.len() {
        match tokens[*pos] {
            Token::Mul => {
                *pos += 1;
                result *= parse_factor(tokens, pos)?;
            }
            Token::Div => {
                *pos += 1;
                let divisor = parse_factor(tokens, pos)?;
                if divisor == 0.0 {
                    return Err("除数不能为零".to_string());
                }
                result /= divisor;
            }
            _ => break,
        }
    }

    Ok(result)
}

fn parse_factor(tokens: &[Token], pos: &mut usize) -> Result<f64, String> {
    if *pos >= tokens.len() {
        return Err("意外到达表达式末尾".to_string());
    }

    match &tokens[*pos] {
        Token::Num(n) => {
            let val = *n;
            *pos += 1;
            Ok(val)
        }
        Token::Plus => {
            *pos += 1;
            parse_factor(tokens, pos)
        }
        Token::Minus => {
            *pos += 1;
            Ok(-parse_factor(tokens, pos)?)
        }
        Token::LParen => {
            *pos += 1;
            let val = parse_expr(tokens, pos)?;
            if *pos < tokens.len() && tokens[*pos] == Token::RParen {
                *pos += 1;
                Ok(val)
            } else {
                Err("缺少右括号 ')'".to_string())
            }
        }
        tok => Err(format!("未预期的标记: {tok:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_digit_sum() {
        let tool = BasicCalculatorTool;
        let res = tool.call("1+7+8+7+6+5+6+5+9+2");
        assert!(res.contains("56"));
    }

    #[test]
    fn test_expression_arithmetic() {
        let tool = BasicCalculatorTool;
        let res = tool.call("(2025 + 4 + 11) * 2");
        assert!(res.contains("4080"));
    }
}
