
use std::{env, fs};
use numbat::Context;


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("No enough arguments");
    }

    let file = args[1].clone();
    let content = fs::read_to_string(&file).expect("Failed to load file");

    let mut context = Context::new_without_importer();


    let mut batch = vec![];
    for line in content.lines() {
        if let Some(at) = line.find("#=") {
            let (expression, _) = line.split_at(at);
            batch.push(expression);
            print!("{expression}#=");

            let code = batch.join("\n");
            batch.clear();

            let result = context.interpret(&code, numbat::resolver::CodeSource::Text);
            match result {
                Err(e) => panic!("Failed {e}"),
                Ok((_, res)) => {
                    let value = res.value_as_string().unwrap();
                    println!(" {value}")
                },
            }
        } else {
            batch.push(line);
            println!("{line}");
        }
    }
}
