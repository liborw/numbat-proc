
use std::{env, fs::File, io::{stdin, Read}};
use numbat::Context;


fn main() {
    let args: Vec<String> = env::args().collect();
    let mut input = String::new();
    if args.len() >= 2 {
        let mut file = File::open(args[2].clone()).expect("Failed to open file");
        file.read_to_string(&mut input).expect("Failed to load file");
    } else {
        stdin().read_to_string(&mut input).expect("Failed to read stdin");
    };

    let mut context = Context::new_without_importer();

    let mut batch = vec![];
    for line in input.lines() {
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
