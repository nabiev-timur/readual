use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Heading {
    pub level: u32,
    pub text: String,
    pub line_number: usize,
}

#[derive(Debug)]
pub struct DocumentHierarchy {
    pub headings: Vec<Heading>,
}

impl DocumentHierarchy {
    pub fn new() -> Self {
        Self {
            headings: Vec::new(),
        }
    }

    pub fn add_heading(&mut self, level: u32, text: String, line_number: usize) {
        self.headings.push(Heading {
            level,
            text,
            line_number,
        });
    }
}

/// Parses Markdown file at specified path and returns heading hierarchy
/// 
/// # Arguments
/// 
/// * `file_path` - path to Markdown file
/// 
/// # Returns
/// 
/// * `Result<DocumentHierarchy, String>` - structure with headings or error
/// 
/// # Examples
/// 
/// ```rust
/// use readual_md::parse_markdown_file;
/// 
/// let hierarchy = parse_markdown_file("README.md")?;
/// for heading in &hierarchy.headings {
///     println!("{} {}", "#".repeat(heading.level as usize), heading.text);
/// }
/// ```
pub fn parse_markdown_file<P: AsRef<Path>>(file_path: P) -> Result<DocumentHierarchy, String> {
    let path = file_path.as_ref();
    
    // Check if file exists
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()));
    }
    
    // Check if it's a file
    if !path.is_file() {
        return Err(format!("Path is not a file: {}", path.display()));
    }
    
    // Read file content
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Error reading file {}: {}", path.display(), e))?;
    
    // Parse content
    Ok(parse_markdown_content(&content))
}

/// Parses Markdown content and returns heading hierarchy
/// 
/// # Arguments
/// 
/// * `content` - Markdown file content
/// 
/// # Returns
/// 
/// * `DocumentHierarchy` - structure with headings
pub fn parse_markdown_content(content: &str) -> DocumentHierarchy {
    let mut hierarchy = DocumentHierarchy::new();
    let lines: Vec<&str> = content.lines().collect();

    for (line_number, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        // Check if line is a heading
        if trimmed.starts_with('#') {
            let level = trimmed.chars()
                .take_while(|&c| c == '#')
                .count() as u32;
            
            // Limit heading level (Markdown supports up to 6 levels)
            if level <= 6 {
                let text = trimmed[level as usize..].trim().to_string();
                if !text.is_empty() {
                    hierarchy.add_heading(level, text, line_number + 1);
                }
            }
        }
    }

    hierarchy
}

/// Extracts commands from code blocks between ``` and ```
pub fn extract_commands(content: &str) -> Vec<String> {
    let mut commands = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut in_code_block = false;
    
    for line in lines {
        let trimmed = line.trim();
        
        if trimmed == "```" {
            in_code_block = !in_code_block;
            continue;
        }
        
        if in_code_block && !trimmed.is_empty() {
            commands.push(trimmed.to_string());
        }
    }
    
    commands
}
