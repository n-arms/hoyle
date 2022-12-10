use lexer::scan_tokens;

fn main() {
    let text = "123 abc([+Do remi";
    //let text = "ib";
    let (tokens, errors) = scan_tokens(text);

    if !errors.success() {
        println!("{:?}", errors);
    }

    for token in &tokens {
        println!("{:?}", token);
    }
}
