use anyhow::{Context, Result};
use clap::Parser;
use std::fs::{read_to_string, write};
use std::path::Path;

/// 压缩代码格式化工具：符合行业规范的 HTML/CSS/JS/TS 格式化（高可读性）
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// 输入文件路径（必填）
    #[arg(short = 'i', long = "input", required = true, help = "输入压缩代码的文件路径")]
    input: String,

    /// 输出文件路径（必填）
    #[arg(short = 'o', long = "output", required = true, help = "格式化后代码的输出文件路径")]
    output: String,

    /// 缩进空格数（可选，默认 4）
    #[arg(short = 'n', long = "indent", default_value_t = 4, help = "缩进空格数量，默认 4")]
    indent: u8,

    /// 单行最大长度（可选，默认 80）
    #[arg(short = 'l', long = "line-length", default_value_t = 80, help = "单行最大字符长度，默认 80")]
    line_length: usize,
}

/// 根据文件扩展名判断代码类型
fn get_file_type(file_path: &str) -> Result<&str> {
    let ext = Path::new(file_path)
        .extension()
        .context("文件无扩展名，无法识别代码类型")?
        .to_str()
        .context("扩展名编码无效")?;

    match ext.to_lowercase().as_str() {
        "html" => Ok("html"),
        "css" => Ok("css"),
        "js" => Ok("js"),
        "ts" => Ok("ts"),
        _ => Err(anyhow::anyhow!("不支持的文件类型：{}，仅支持 html/css/js/ts", ext)),
    }
}

// ------------------------------
// 通用工具函数
// ------------------------------
/// 为运算符/符号添加规范空格
fn add_operator_spaces(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    
    while let Some(c) = chars.next() {
        match c {
            '=' | '+' | '-' | '*' | '/' | '%' | '>' | '<' | '!' | '&' | '|' | '^' | '~' => {
                let mut op = String::from(c);
                while let Some(&next_c) = chars.peek() {
                    if next_c == '=' || (op == "&" && next_c == '&') || (op == "|" && next_c == '|') {
                        op.push(next_c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if !result.is_empty() && !result.ends_with(' ') && !result.ends_with('(') && !result.ends_with('[') && !result.ends_with('{') {
                    result.push(' ');
                }
                result.push_str(&op);
                if let Some(&next_c) = chars.peek() {
                    if !next_c.is_whitespace() && next_c != ')' && next_c != ']' && next_c != '}' && next_c != ',' && next_c != ';' {
                        result.push(' ');
                    }
                }
            }
            ',' | ';' => {
                result.push(c);
                if let Some(&next_c) = chars.peek() {
                    if !next_c.is_whitespace() && next_c != ')' && next_c != ']' && next_c != '}' && next_c != '\n' {
                        result.push(' ');
                    }
                }
            }
            '(' | '[' | '{' => {
                if !result.is_empty() && !result.ends_with(' ') && !result.ends_with('(') && !result.ends_with('[') && !result.ends_with('{') && 
                   !result.ends_with('=') && !result.ends_with('+') && !result.ends_with('-') && !result.ends_with('*') && !result.ends_with('/') {
                    result.push(' ');
                }
                result.push(c);
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_whitespace() {
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
            ')' | ']' | '}' => {
                if !result.is_empty() && result.ends_with(' ') {
                    result.pop();
                }
                result.push(c);
                let mut current_line_length = 0;
                if let Some(&next_c) = chars.peek() {
                    if !next_c.is_whitespace() && next_c != ')' && next_c != ']' && next_c != '}' && next_c != ',' && next_c != ';' && next_c != '+' && next_c != '-' && next_c != '*' && next_c != '/' {
                        result.push(' ');
                        current_line_length += 1;
                    }
                }
            }
            _ => {
                result.push(c);
            }
        }
    }
    result
}

/// 拆分扎堆的括号并保留缩进
fn split_clustered_brackets(s: &str, indent_unit: &str, current_indent: usize) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    let mut bracket_stack = Vec::new();
    let mut consecutive_brackets = 0;
    let mut current_line_length = 0;

    while let Some(c) = chars.next() {
        match c {
            '(' | '[' | '{' => {
                bracket_stack.push(c);
                consecutive_brackets += 1;
                result.push(c);
                current_line_length += 1;

                if consecutive_brackets >= 3 || current_line_length > 80 {
                    result.push('\n');
                    result.push_str(&indent_unit.repeat(current_indent + bracket_stack.len()));
                    current_line_length = indent_unit.len() * (current_indent + bracket_stack.len());
                    consecutive_brackets = 0;
                }
            }
            ')' | ']' | '}' => {
                if !bracket_stack.is_empty() {
                    bracket_stack.pop();
                }
                consecutive_brackets += 1;
                if consecutive_brackets >= 3 || current_line_length > 80 {
                    result.push('\n');
                    result.push_str(&indent_unit.repeat(current_indent + bracket_stack.len()));
                    current_line_length = indent_unit.len() * (current_indent + bracket_stack.len());
                    consecutive_brackets = 0;
                }
                if !result.is_empty() && result.ends_with(' ') {
                    result.pop();
                }
                result.push(c);
                current_line_length += 1;
                if let Some(&next_c) = chars.peek() {
                    if !next_c.is_whitespace() && next_c != ')' && next_c != ']' && next_c != '}' && next_c != ',' && next_c != ';' {
                        result.push(' ');
                        current_line_length += 1;
                    }
                }
            }
            '\n' => {
                result.push(c);
                current_line_length = 0;
                consecutive_brackets = 0;
                // 核心修复：换行后自动补充当前缩进
                result.push_str(&indent_unit.repeat(current_indent + bracket_stack.len()));
                current_line_length = indent_unit.len() * (current_indent + bracket_stack.len());
            }
            ' ' => {
                if !result.ends_with(' ') {
                    result.push(c);
                    current_line_length += 1;
                }
            }
            _ => {
                result.push(c);
                current_line_length += 1;
                consecutive_brackets = 0;
            }
        }
    }
    result
}

// ------------------------------
// HTML 格式化（完整缩进 + 最后一行处理）
// ------------------------------
fn format_html(content: &str, indent: u8, max_line_length: usize) -> Result<String> {
    let indent_unit = " ".repeat(indent as usize);
    let mut result = String::new();
    let mut current_indent_level = 0;
    let mut chars = content.chars().peekable();
    let mut in_tag = false;
    let mut in_comment = false;
    let mut current_line_length = 0;

    // 初始缩进
    result.push_str(&indent_unit.repeat(current_indent_level));

    while let Some(c) = chars.next() {
        match c {
            '<' if !in_tag && !in_comment => {
                in_tag = true;
                if !result.is_empty() && !result.ends_with('\n') && !result.ends_with(' ') && current_line_length > max_line_length {
                    result.push('\n');
                    result.push_str(&indent_unit.repeat(current_indent_level));
                    current_line_length = indent_unit.len() * current_indent_level;
                }
                result.push(c);
                current_line_length += 1;

                let mut tag_buf = String::new();
                let mut tag_length = 1;
                while let Some(&next_char) = chars.peek() {
                    tag_buf.push(next_char);
                    tag_length += 1;
                    chars.next();

                    if tag_buf.starts_with("!--") {
                        in_comment = true;
                    }
                    if in_comment && tag_buf.ends_with("-->") {
                        in_comment = false;
                    }

                    if next_char == '>' && !in_comment {
                        break;
                    }
                }

                // 标签属性格式化
                let mut formatted_tag = String::new();
                let mut tag_chars = tag_buf.chars().peekable();
                while let Some(tc) = tag_chars.next() {
                    match tc {
                        '=' => {
                            formatted_tag.push(' ');
                            formatted_tag.push('=');
                            formatted_tag.push(' ');
                            while let Some(&next_tc) = tag_chars.peek() {
                                if next_tc.is_whitespace() {
                                    tag_chars.next();
                                } else {
                                    break;
                                }
                            }
                        }
                        '"' | '\'' => {
                            formatted_tag.push(tc);
                            while let Some(&next_tc) = tag_chars.peek() {
                                if next_tc == tc {
                                    formatted_tag.push(next_tc);
                                    tag_chars.next();
                                    break;
                                } else {
                                    formatted_tag.push(next_tc);
                                    tag_chars.next();
                                }
                            }
                        }
                        ' ' => {
                            if !formatted_tag.ends_with(' ') && !formatted_tag.ends_with('=') {
                                formatted_tag.push(tc);
                            }
                        }
                        _ => {
                            formatted_tag.push(tc);
                        }
                    }
                }

                let tag_str = formatted_tag.trim_end_matches('>');
                if tag_str.starts_with('/') {
                    current_indent_level = current_indent_level.saturating_sub(1);
                    // 闭合标签前回退缩进
                    if result.ends_with(&indent_unit.repeat(current_indent_level + 1)) {
                        result.truncate(result.len() - indent_unit.len());
                    }
                    result.push_str(tag_str);
                    result.push('>');
                    current_line_length += tag_str.len() + 1;
                    in_tag = false;
                    
                    // 闭合标签后换行并补充缩进
                    result.push('\n');
                    result.push_str(&indent_unit.repeat(current_indent_level));
                    current_line_length = indent_unit.len() * current_indent_level;
                } else if tag_str.ends_with('/') || ["meta", "link", "img", "br", "hr"].contains(&tag_str) {
                    result.push_str(&formatted_tag);
                    current_line_length += formatted_tag.len();
                    in_tag = false;
                    
                    // 自闭合标签后换行并补充缩进
                    result.push('\n');
                    result.push_str(&indent_unit.repeat(current_indent_level));
                    current_line_length = indent_unit.len() * current_indent_level;
                } else if !tag_str.starts_with('!') && !tag_str.starts_with('?') && !tag_str.starts_with("!--") {
                    result.push_str(&formatted_tag);
                    current_line_length += formatted_tag.len();
                    current_indent_level += 1;
                    in_tag = false;
                    
                    // 开始标签后换行并增加缩进
                    result.push('\n');
                    result.push_str(&indent_unit.repeat(current_indent_level));
                    current_line_length = indent_unit.len() * current_indent_level;
                } else {
                    result.push_str(&formatted_tag);
                    current_line_length += formatted_tag.len();
                    in_tag = false;
                    
                    // 注释/DOCTYPE 后换行并保留缩进
                    result.push('\n');
                    result.push_str(&indent_unit.repeat(current_indent_level));
                    current_line_length = indent_unit.len() * current_indent_level;
                }
            }
            _ if !in_tag && !in_comment => {
                if c.is_whitespace() {
                    if result.ends_with(&[' ', '\t'][..]) {
                        continue;
                    }
                    result.push(' ');
                    current_line_length += 1;
                } else {
                    result.push(c);
                    current_line_length += 1;
                    if current_line_length > max_line_length {
                        result.push('\n');
                        result.push_str(&indent_unit.repeat(current_indent_level));
                        current_line_length = indent_unit.len() * current_indent_level;
                    }
                }
            }
            _ => {
                result.push(c);
                current_line_length += 1;
            }
        }
    }

    // 最终格式化：保留缩进 + 规范空格
    let mut formatted = add_operator_spaces(&result);
    formatted = formatted.replace("  ", " ").trim_end_matches(&[' ', '\t'][..]).to_string();
    if !formatted.ends_with('\n') {
        formatted.push('\n');
    }
    Ok(formatted)
}

// ------------------------------
// CSS 格式化（完整缩进 + 最后一行处理）
// ------------------------------
fn format_css(content: &str, indent: u8, max_line_length: usize) -> Result<String> {
    let indent_unit = " ".repeat(indent as usize);
    let mut result = String::new();
    let mut chars = content.chars().peekable();
    let mut current_indent_level = 0;
    let mut in_brace = false;
    let mut in_comment = false;
    let mut current_selector = String::new();
    let mut current_declarations = Vec::new();
    let mut temp_char = String::new();

    while let Some(c) = chars.next() {
        match c {
            '/' if !in_comment && !in_brace => {
                if let Some(&next_c) = chars.peek() {
                    if next_c == '*' {
                        in_comment = true;
                        result.push(c);
                        result.push(next_c);
                        chars.next();
                        continue;
                    }
                }
                temp_char.push(c);
            }
            '*' if in_comment => {
                result.push(c);
                if let Some(&next_c) = chars.peek() {
                    if next_c == '/' {
                        in_comment = false;
                        result.push(next_c);
                        chars.next();
                        result.push('\n');
                        // 注释后补充缩进
                        result.push_str(&indent_unit.repeat(current_indent_level));
                    }
                }
            }
            '{' if !in_comment => {
                current_selector = temp_char.trim().to_string();
                temp_char.clear();
                
                // 选择器格式化（保留缩进）
                if !current_selector.is_empty() {
                    result.push_str(&indent_unit.repeat(current_indent_level));
                    result.push_str(&current_selector);
                    result.push(' ');
                }
                result.push('{');
                current_indent_level += 1;
                in_brace = true;
                result.push('\n');
                // 大括号内增加缩进
                result.push_str(&indent_unit.repeat(current_indent_level));
            }
            '}' if !in_comment => {
                if !temp_char.is_empty() {
                    let decl = temp_char.trim().to_string();
                    if !decl.is_empty() {
                        current_declarations.push(decl);
                    }
                    temp_char.clear();
                }

                // 格式化声明（带缩进）
                let mut formatted_decls = Vec::new();
                for decl in &current_declarations {
                    formatted_decls.push(decl.replace(":", ": "));
                }
                if !formatted_decls.is_empty() {
                    let declarations_str = formatted_decls.join("; ");
                    if declarations_str.len() + indent_unit.len() * current_indent_level < max_line_length && formatted_decls.len() <= 3 {
                        result.push_str(&declarations_str);
                        result.push(';');
                    } else {
                        for (i, decl) in formatted_decls.iter().enumerate() {
                            if i > 0 {
                                result.push('\n');
                                result.push_str(&indent_unit.repeat(current_indent_level));
                            }
                            result.push_str(decl);
                            result.push(';');
                        }
                    }
                }
                current_declarations.clear();
                
                // 闭合大括号前回退缩进
                current_indent_level = current_indent_level.saturating_sub(1);
                result.push('\n');
                result.push_str(&indent_unit.repeat(current_indent_level));
                result.push('}');
                result.push('\n');
                // 样式块后空行 + 保留缩进
                result.push_str(&indent_unit.repeat(current_indent_level));
                in_brace = false;
            }
            ';' if !in_comment && in_brace => {
                // 修复：先复制再移动，避免所有权问题
                let decl_str = temp_char.trim().to_string();
                if !decl_str.is_empty() {
                    current_declarations.push(decl_str.clone()); // 克隆后移动
                    // 单个声明格式化（带缩进）
                    result.push_str(&decl_str.replace(":", ": ")); // 使用克隆的字符串
                    result.push(';');
                    result.push('\n');
                    result.push_str(&indent_unit.repeat(current_indent_level));
                }
                temp_char.clear();
            }
            _ if !in_comment => {
                if c.is_whitespace() && temp_char.ends_with(&[' ', '\t'][..]) {
                    continue;
                }
                temp_char.push(c);
            }
            _ => {
                result.push(c);
            }
        }
    }

    // 处理最后一个样式块（带完整缩进）
    if !temp_char.is_empty() || !current_declarations.is_empty() {
        if !temp_char.is_empty() {
            let decl = temp_char.trim().to_string();
            if !decl.is_empty() {
                current_declarations.push(decl);
            }
        }
        let mut formatted_decls = Vec::new();
        for decl in &current_declarations {
            formatted_decls.push(decl.replace(":", ": "));
        }
        if !formatted_decls.is_empty() {
            result.push_str(&indent_unit.repeat(current_indent_level));
            let declarations_str = formatted_decls.join("; ");
            if declarations_str.len() + indent_unit.len() * current_indent_level < max_line_length && formatted_decls.len() <= 3 {
                result.push_str(&declarations_str);
                result.push(';');
            } else {
                for decl in formatted_decls {
                    result.push('\n');
                    result.push_str(&indent_unit.repeat(current_indent_level));
                    result.push_str(&decl);
                    result.push(';');
                }
            }
            if in_brace {
                current_indent_level = current_indent_level.saturating_sub(1);
                result.push('\n');
                result.push_str(&indent_unit.repeat(current_indent_level));
                result.push('}');
                result.push('\n');
            }
        }
    }

    // 最终处理：保留缩进 + 规范空格
    let mut formatted = add_operator_spaces(&result);
    formatted = formatted.replace("\n\n\n", "\n\n").trim_end().to_string() + "\n";
    Ok(formatted)
}

// ------------------------------
// JS/TS 格式化（完整缩进 + 最后一行处理）
// ------------------------------
fn format_js_ts(content: &str, indent: u8, max_line_length: usize) -> Result<String> {
    let indent_unit = " ".repeat(indent as usize);
    let mut result = String::new();
    let mut chars = content.chars().peekable();
    let mut current_indent_level = 0;
    let mut in_string = None;
    let mut in_comment_single = false;
    let mut in_comment_multi = false;
    let mut current_statement = String::new();
    let mut current_line_length = 0;

    // 初始缩进
    result.push_str(&indent_unit.repeat(current_indent_level));

    while let Some(c) = chars.next() {
        if in_comment_single {
            result.push(c);
            if c == '\n' {
                in_comment_single = false;
                // 单行注释后补充缩进
                result.push_str(&indent_unit.repeat(current_indent_level));
                current_line_length = indent_unit.len() * current_indent_level;
            }
            continue;
        }

        if in_comment_multi {
            result.push(c);
            if c == '*' && chars.peek() == Some(&'/') {
                in_comment_multi = false;
                result.push(chars.next().unwrap());
                result.push('\n');
                // 多行注释后补充缩进
                result.push_str(&indent_unit.repeat(current_indent_level));
                current_line_length = indent_unit.len() * current_indent_level;
            }
            continue;
        }

        if let Some(quote) = in_string {
            current_statement.push(c);
            current_line_length += 1;
            if c == quote {
                in_string = None;
            }
            continue;
        }

        match c {
            '"' | '\'' => {
                in_string = Some(c);
                current_statement.push(c);
                current_line_length += 1;
            }
            '/' if chars.peek() == Some(&'/') => {
                if !current_statement.is_empty() {
                    result.push_str(&current_statement);
                    current_statement.clear();
                }
                in_comment_single = true;
                result.push(c);
                result.push(chars.next().unwrap());
            }
            '/' if chars.peek() == Some(&'*') => {
                if !current_statement.is_empty() {
                    result.push_str(&current_statement);
                    current_statement.clear();
                }
                in_comment_multi = true;
                result.push(c);
                result.push(chars.next().unwrap());
            }
            '{' | '(' | '[' => {
                current_statement.push(c);
                current_line_length += 1;
                let is_brace = c == '{';
                
                // 左大括号前格式化
                result.push_str(&current_statement);
                if is_brace {
                    result.push('\n');
                    current_indent_level += 1;
                    // 大括号内增加缩进
                    result.push_str(&indent_unit.repeat(current_indent_level));
                }
                current_statement.clear();
                current_line_length = indent_unit.len() * current_indent_level;
            }
            '}' | ')' | ']' => {
                let is_brace = c == '}';
                
                // 右大括号前回退缩进
                if is_brace {
                    result.push('\n');
                    current_indent_level = current_indent_level.saturating_sub(1);
                    result.push_str(&indent_unit.repeat(current_indent_level));
                }
                
                // 处理当前语句
                if !current_statement.is_empty() {
                    result.push_str(&current_statement);
                    current_statement.clear();
                }
                result.push(c);
                current_line_length += 1;
                
                // 右括号后加空格/换行
                if let Some(&next_c) = chars.peek() {
                    if next_c != ',' && next_c != ';' && next_c != '}' && next_c != ')' && current_line_length > max_line_length {
                        result.push('\n');
                        result.push_str(&indent_unit.repeat(current_indent_level));
                        current_line_length = indent_unit.len() * current_indent_level;
                    }
                }
            }
            ';' => {
                current_statement.push(c);
                // 语句格式化（带缩进）
                let stmt_with_spaces = add_operator_spaces(&current_statement);
                result.push_str(&stmt_with_spaces);
                // 分号后换行并补充缩进
                result.push('\n');
                result.push_str(&indent_unit.repeat(current_indent_level));
                current_statement.clear();
                current_line_length = indent_unit.len() * current_indent_level;
            }
            ',' => {
                current_statement.push(c);
                current_line_length += 1;
                if current_line_length > max_line_length {
                    result.push_str(&current_statement);
                    result.push('\n');
                    // 逗号后换行并补充缩进
                    result.push_str(&indent_unit.repeat(current_indent_level));
                    current_statement.clear();
                    current_line_length = indent_unit.len() * current_indent_level;
                }
            }
            _ => {
                if c.is_whitespace() && current_statement.ends_with(&[' ', '\t'][..]) {
                    continue;
                }
                current_statement.push(c);
                current_line_length += 1;
            }
        }
    }

    // 处理最后一个语句（带完整缩进）
    if !current_statement.is_empty() {
        let mut final_stmt = add_operator_spaces(&current_statement);
        final_stmt = split_clustered_brackets(&final_stmt, &indent_unit, current_indent_level);
        // 最后语句补充当前缩进
        result.push_str(&indent_unit.repeat(current_indent_level));
        result.push_str(&final_stmt);
        if !final_stmt.ends_with(';') && !final_stmt.ends_with('}') && !final_stmt.ends_with(')') && !final_stmt.ends_with(']') {
            result.push(';');
        }
        result.push('\n');
    }

    // 最终处理：保留所有缩进 + 拆分括号
    let mut formatted = split_clustered_brackets(&result, &indent_unit, current_indent_level);
    formatted = formatted.replace("  ", " ").trim_end().to_string() + "\n";
    Ok(formatted)
}

/// 统一格式化入口
fn format_code(content: &str, file_type: &str, indent: u8, line_length: usize) -> Result<String> {
    match file_type {
        "html" => format_html(content, indent, line_length),
        "css" => format_css(content, indent, line_length),
        "js" | "ts" => format_js_ts(content, indent, line_length),
        // 修复：定义 ext 变量并使用
        _ => {
            let ext = file_type;
            Err(anyhow::anyhow!("不支持的文件类型：{}，仅支持 html/css/js/ts", ext))
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let raw_content = read_to_string(&cli.input)
        .with_context(|| format!("无法读取输入文件：{}", cli.input))?;
    
    // 预处理：保留换行符（避免缩进丢失）
    let binding = raw_content.replace("\r", "");
    let content = binding.trim();

    let file_type = get_file_type(&cli.input)?;
    println!("[INFO] 格式化 {} 文件（缩进：{} 空格，单行长度：{}）", 
             file_type, cli.indent, cli.line_length);

    let formatted_content = format_code(content, file_type, cli.indent, cli.line_length)
        .context("代码格式化失败")?;

    write(&cli.output, formatted_content)
        .with_context(|| format!("无法写入输出文件：{}", cli.output))?;

    println!("[SUCCESS] 格式化完成！输出文件：{}", cli.output);
    Ok(())
}

