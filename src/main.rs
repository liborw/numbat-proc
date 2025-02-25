
use std::{fs::File, io::{self, BufRead, Cursor}, path::PathBuf};
use anyhow::{bail, Context as AnyhowContext, Result};
use numbat::{module_importer::{BuiltinModuleImporter, ChainedImporter, FileSystemImporter}, resolver::CodeSource, Context};
use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, name("numbat-proc"), max_term_width = 90)]
struct Args {

    // Path to file to process, if none given stdin is used.
    file: Option<PathBuf>,

    /// Do not load the prelude with predefined physical dimensions and units. This implies --no-init.
    #[arg(short = 'N', long, hide_short_help = true)]
    no_prelude: bool,

    /// Do not load the user init file.
    #[arg(long, hide_short_help = true)]
    no_init: bool
}


struct Config {
    equals_prompt: String,
    load_prelude: bool,
    load_user_init: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            equals_prompt: "#=".to_string(),
            load_prelude: true,
            load_user_init: true,
        }
    }
}

#[derive(Debug, Clone)]
enum Input {
    File(PathBuf),
    Stdin,
    String(String),
}

impl Input {
    fn lines(self) -> Box<dyn Iterator<Item = String>> {
        match self {
            Self::File(path) => {
                let file = File::open(path).expect("Failed to open file");
                Box::new(io::BufReader::new(file).lines().map_while(Result::ok))
            }
            Self::Stdin => {
                Box::new(io::BufReader::new(io::stdin()).lines().map_while(Result::ok))
            }
            Input::String(content) => {
                Box::new(Cursor::new(content).lines().map(|l| l.unwrap_or_default()))
            }
        }
    }
}

struct Cli {
    input: Input,
    config: Config,
    context: Context,
}

impl Cli {

    fn make_fresh_context() -> Context {
        let fs_importer = FileSystemImporter::default();
        // for path in Self::get_modules_paths() {
        //     fs_importer.add_path(path);
        // }

        let importer = ChainedImporter::new(
            Box::new(fs_importer),
            Box::<BuiltinModuleImporter>::default(),
        );

        Context::new(importer)
    }

    fn new(args: Args) -> Result<Self> {
        let mut config = Config::default();
        config.load_prelude &= !args.no_prelude;

        let input = match args.file {
            Some(path) => Input::File(path),
            None => Input::Stdin,
        };
        Self::with_config_and_input(config, input)
    }

    fn with_config_and_input(config: Config, input: Input) -> Result<Self> {
        let context = Cli::make_fresh_context();

        Ok(Self {
            input,
            config,
            context,
        })
    }

    fn run(&mut self) -> Result<()>  {

        if self.config.load_prelude {
            let result = self.context.interpret(
                "use prelude",
                CodeSource::Internal,
            );
            if result.is_err() {
                bail!("Interpreter error in Prelude code")
            }
        }

        let mut batch = vec![];
        for line in self.input.clone().lines() {
            if let Some(at) = line.find(&self.config.equals_prompt) {
                let (expression, _) = line.split_at(at);
                batch.push(expression.to_string());
                print!("{expression}{}", self.config.equals_prompt);

                let code = batch.join("\n");
                batch.clear();

                let result = self.context.interpret(&code, numbat::resolver::CodeSource::Text);
                match result {
                    Err(e) => bail!("Failed {e}"),
                    Ok((_, res)) => {
                        let value = res.value_as_string().context("Failed to get value")?;
                        println!(" {value}")
                    },
                }
            } else {
                println!("{line}");
                batch.push(line);
            }
        }

        Ok(())
    }
}

fn main() {

    let args = Args::parse();
    if let Err(e) = Cli::new(args).and_then(|mut cli| cli.run()) {
        eprintln!("{e:#}");
        std::process::exit(1);
    }
}
