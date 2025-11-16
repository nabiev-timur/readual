use readual_md::{read_document, parse_titles};
use std::path::Path;

fn main() {
    let readme_path = Path::new("../../README.md");
    
    match read_document(readme_path) {
        Ok(doc) => {
            match parse_titles(&doc) {
                Ok(titles) => {
                    println!("Найдено заголовков: {}\n", titles.len());
                    
                    for (idx, title) in titles.iter().enumerate() {
                        println!("=== Title[{}] ===", idx);
                        println!("Span: {}..{}", title.span.start, title.span.end);
                        println!("Содержимое секции (первые 100 символов):");
                        let content = doc.slice(&title.span);
                        let preview = if content.len() > 100 {
                            &content[..100]
                        } else {
                            content
                        };
                        println!("{}\n", preview.replace('\n', "\\n"));
                        
                        println!("Директивы ({}):", title.directives.len());
                        for (dir_idx, directive) in title.directives.iter().enumerate() {
                        println!("  [{}] Span: {}..{}", dir_idx, directive.span.start, directive.span.end);
                        if let Some(ref gen) = directive.generated {
                            println!("       Generated: {}", gen);
                        } else {
                            let dir_content = doc.slice(&directive.span);
                            let preview = if dir_content.len() > 80 {
                                &dir_content[..80]
                            } else {
                                dir_content
                            };
                            println!("       Content: {}", preview.replace('\n', "\\n"));
                        }
                        }
                        println!();
                    }
                }
                Err(e) => {
                    eprintln!("Ошибка парсинга: {:?}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Ошибка чтения файла: {:?}", e);
        }
    }
}

