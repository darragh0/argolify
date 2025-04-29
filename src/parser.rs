use crate::tokenizer::tokenize;

pub fn parse(path: &str) -> Result<(), String> {
    let toks = tokenize(path)?;
    for tok in toks {
        println!("{:?}", tok);
    }
    Ok(())
}
