mod read;
mod repl;

use bumpalo::Bump;

fn main() {
    // let bump = Bump::new();
    // let mut repl = repl::Repl::new(&bump, &bump, &bump);
    // read::event_loop("Welcome to the Hoyle repl", |tokens, errors| {
    //     if errors.success() {
    //         repl.run(tokens)
    //     } else {
    //         println!("error while lexing: {:?}", errors);
    //         read::ExitStatus::Error
    //     }
    // })
    // .unwrap()
}
