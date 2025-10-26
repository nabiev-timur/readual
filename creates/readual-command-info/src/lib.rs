use clap::Parser;
use readual_md::{DocumentHierarchy, parse_markdown_file};

/// Shows repository information
#[derive(Parser, Debug)]
pub struct InfoCommand {
    /// Show detailed repository information
    #[arg(short, long)]
    pub verbose: bool,
    /// Hierarchy output style
    #[arg(short, long, default_value = "tree")]
    pub format: String,
}

/// Displays document hierarchy as a tree
pub fn print_tree(hierarchy: &DocumentHierarchy) {
    println!("📄 Document structure:");
    println!();

    if hierarchy.headings.is_empty() {
        return;
    }

    // Создаем стек для отслеживания статуса веток на каждом уровне
    let mut branch_status = Vec::new();
    
    for (i, heading) in hierarchy.headings.iter().enumerate() {
        let current_level = heading.level as usize;
        
        // Trim stack to current level
        while branch_status.len() >= current_level {
            branch_status.pop();
        }
        
        // Determine if nesting will increase or decrease further
        let is_last_at_level = is_last_element_at_level(hierarchy, i, current_level as u32);
        
        // Build prefix for current element
        let mut tree_chars = String::new();
        
        // Add symbols for all previous levels
        for &is_continuing in branch_status.iter() {
            if is_continuing {
                tree_chars.push_str("│  ");
            } else {
                tree_chars.push_str("   ");
            }
        }
        
        // Add symbol for current element
        if is_last_at_level {
            tree_chars.push_str("└── ");
        } else {
            tree_chars.push_str("├── ");
        }
        
        println!("{}{}", tree_chars, heading.text);
        
        // Add current level to stack (true = has continuation, false = last)
        branch_status.push(!is_last_at_level);
    }
}

/// Determines if element is last at its level
/// Analyzes not only next element of same level, but also general trend
fn is_last_element_at_level(hierarchy: &DocumentHierarchy, current_index: usize, current_level: u32) -> bool {
    let headings = &hierarchy.headings;
    
    // If this is the last element in the list
    if current_index >= headings.len() - 1 {
        return true;
    }
    
    // Look for next element of same level or higher
    for i in (current_index + 1)..headings.len() {
        let next_level = headings[i].level;
        
        // If found element of same level - current is not last
        if next_level == current_level {
            return false;
        }
        
        // If found element above current level - current is last
        if next_level < current_level {
            return true;
        }
        
        // If element is deeper - continue search
        // (this is child element, doesn't affect current status)
    }
    
    // If reached the end - current is last
    true
}

/// Displays hierarchy as a list
pub fn print_list(hierarchy: &DocumentHierarchy) {
    println!("Document structure:");
    println!();

    for (i, heading) in hierarchy.headings.iter().enumerate() {
        let indent = "  ".repeat((heading.level - 1) as usize);
        println!("{}{}. {}", indent, i + 1, heading.text);
    }
}

/// Shows repository information
pub fn execute_info_command(args: &InfoCommand) -> Result<(), String> {
    if args.verbose {
        println!("🔍 Analyzing document with detailed information...");
    }
    
    println!("🔍 Searching for README.md in current directory...");
    
    // Look for README.md in current directory
    let current_dir = std::env::current_dir()
        .map_err(|e| format!("Error getting current directory: {}", e))?;
    let readme_path = current_dir.join("README.md");
    
    if !readme_path.exists() {
        println!("❌ README.md not found in current directory");
        println!("💡 Make sure README.md file exists in current folder");
        return Err("README.md not found".to_string());
    }
    
    println!("✅ Found README.md");
    println!();
    
    // Parse file using readual-md
    let hierarchy = parse_markdown_file(&readme_path)?;
    
    if hierarchy.headings.is_empty() {
        println!("⚠️  No headings found in file");
        return Ok(());
    }
    
    // Display hierarchy in selected format
    match args.format.as_str() {
        "list" => print_list(&hierarchy),
        "tree" | _ => print_tree(&hierarchy),
    }
    Ok(())
}
