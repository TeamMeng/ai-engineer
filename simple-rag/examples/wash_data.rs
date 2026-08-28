use icu_normalizer::{ComposingNormalizer, ComposingNormalizerBorrowed};

fn is_zero_width_char(c: char) -> bool {
    matches!(c, '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{FEFF}')
}

pub struct TextCleaner {
    normalizer: ComposingNormalizerBorrowed<'static>,
}

impl TextCleaner {
    pub fn new() -> Self {
        Self {
            normalizer: ComposingNormalizer::new_nfkc(),
        }
    }

    pub fn normalize_text(&self, input: &str) -> String {
        let mut out = String::with_capacity(input.len());
        let mut pending_space = false;
        let mut has_non_space = false;

        for c in self.normalizer.normalize_iter(input.chars()) {
            if is_zero_width_char(c) {
                continue;
            }

            if c.is_whitespace() {
                if has_non_space {
                    pending_space = true;
                }
            } else {
                if pending_space {
                    out.push(' ');
                    pending_space = false;
                }
                out.push(c);
                has_non_space = true;
            }
        }

        out
    }
}

impl Default for TextCleaner {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    let cleaner = TextCleaner::new();

    let data = [
        "Hello,\u{200B} world!",  // 1. 含零宽不可见字符
        "  多余   空格   文本  ", // 2. 含首尾及多余连续空格
        "ｈｅｌｌｏ　ｗｏｒｌｄ", // 3. 全角英文字母与全角空格
        "A\u{FEFF}B\u{200C}C",    // 4. 多种特殊零宽字符
        "序号：① 特殊连字：ﬁ",    // 5. 带圈数字与专业印刷连字
    ];

    for original in &data {
        let cleaned = cleaner.normalize_text(original);
        println!("原始: {:?}", original);
        println!("清洗: {:?}", cleaned);
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleaner() {
        let cleaner = TextCleaner::new();
        assert_eq!(cleaner.normalize_text(""), "");
        assert_eq!(
            cleaner.normalize_text("Hello,\u{200B} world!"),
            "Hello, world!"
        );
        assert_eq!(
            cleaner.normalize_text("ｈｅｌｌｏ　ｗｏｒｌｄ"),
            "hello world"
        );
        assert_eq!(
            cleaner.normalize_text("  多余   \n\t 空格   文本  "),
            "多余 空格 文本"
        );
        assert_eq!(cleaner.normalize_text("① ② ﬁ"), "1 2 fi");
    }
}
