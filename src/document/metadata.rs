#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DocumentStats {
    pub char_count: usize,
    pub word_count: usize,
    pub line_count: usize,
    pub reading_time_secs: u32,
}

impl DocumentStats {
    #[allow(dead_code)]
    pub fn compute(content: &str) -> Self {
        let char_count = content.chars().count();
        let word_count = content.split_whitespace().count();
        let line_count = if content.is_empty() {
            0
        } else {
            content.lines().count()
        };

        // Average reading speed: 200 words per minute
        let reading_time_secs = if word_count == 0 {
            0
        } else {
            ((word_count as f64 / 200.0) * 60.0).ceil() as u32
        };

        Self {
            char_count,
            word_count,
            line_count,
            reading_time_secs,
        }
    }
}
