use std::{error::Error, io::{stdin,stdout, Write}};

pub fn input(msg: impl Into<String>) -> Result<String, Box<dyn Error>> {
    print!("{}", msg.into());
    let _ = stdout().flush();
    
    let mut buf = String::new();
    let _ = stdin().read_line(&mut buf)?;

    let trimmed = buf
        .trim()
        .to_string();

    if trimmed == "" {
        return Err("Empty buffer".into());
    }

    return Ok(trimmed);
}