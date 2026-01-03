use std::error::Error;
use std::fs;
use std::io::{self, Read};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();

    let data = match args.get(1).map(|s| s.as_str()) {
        None | Some("-") => {
            let mut buffer = Vec::new();
            io::stdin().read_to_end(&mut buffer)?;
            buffer
        }
        Some(path) => fs::read(path)?,
    };

    let ascii = relish_ascii::relish2ascii(&data)?;
    println!("{}", ascii);
    Ok(())
}
