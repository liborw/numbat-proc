
use std::{fs::File, io::{self, BufRead, Cursor}, path::PathBuf};
use anyhow::{bail, Context as AnyhowContext, Result};
use numbat::{compact_str::CompactString, module_importer::{BuiltinModuleImporter, ChainedImporter, FileSystemImporter}, resolver::CodeSource, Context};
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
    prompt: String,
    input: Input,
    backend: NumbatWrapper
}

impl Cli {


    fn new(args: Args) -> Result<Self> {
        let mut config = NumbatConfig::default();
        config.load_prelude &= !args.no_prelude;

        let input = match args.file {
            Some(path) => Input::File(path),
            None => Input::Stdin,
        };

        let backend = NumbatWrapper::new(config)?;

        Ok(Cli { input,backend, prompt: "#=".to_string() })
    }


    fn run(&mut self) -> Result<()>  {

        let mut batch = vec![];
        for line in self.input.clone().lines() {
            if let Some(at) = line.find(&self.prompt) {
                let (expression, _) = line.split_at(at);
                batch.push(expression.to_string());
                print!("{expression}{}", self.prompt);

                let code = batch.join("\n");
                batch.clear();
                let value = self.backend.eval_block(code)?;
                println!(" {value}");
            } else {
                println!("{line}");
                batch.push(line);
            }
        }

        Ok(())
    }
}

struct NumbatConfig {
    load_prelude: bool,
    load_user_init: bool,
}

impl Default for NumbatConfig{
    fn default() -> Self {
        Self {
            load_prelude: true,
            load_user_init: true,
        }
    }
}

struct NumbatWrapper {
    context: Context,
    config: NumbatConfig,
}

impl NumbatWrapper {

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

    pub fn new(config: NumbatConfig) -> Result<Self> {
        let mut context = Self::make_fresh_context();

        if config.load_prelude {
            let result = context.interpret(
                "use prelude",
                CodeSource::Internal,
            );
            if result.is_err() {
                bail!("Interpreter error in Prelude code")
            }
        }

        Ok(Self{ context,config })
    }

    pub fn eval_block(&mut self, block: String) -> Result<CompactString> {
        let mut block = block;

        if block.starts_with("let") {
            let var = block.split_whitespace().nth(1).context("Failed to extract variable")?;
            block.push_str(format!("\n{var}").as_str());
        }

        let (_, res) = self.context.interpret(&block, numbat::resolver::CodeSource::Text)?;
        res.value_as_string().context("Failed to get value")
    }

}

fn main() {

    let args = Args::parse();
    if let Err(e) = Cli::new(args).and_then(|mut cli| cli.run()) {
        eprintln!("{e:#}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use numbat::compact_str::ToCompactString;
    use crate::{NumbatConfig, NumbatWrapper};

    #[test]
    fn test_minimal_expression() {
        let mut numbat = NumbatWrapper::new(NumbatConfig::default()).unwrap();
        let result = numbat.eval_block("1 + 1".to_string());
        assert_eq!(result.ok(), Some("2".to_compact_string()));
    }

    #[test]
    fn test_minimal_expression_with_units() {
        let mut numbat = NumbatWrapper::new(NumbatConfig::default()).unwrap();
        let result = numbat.eval_block("1 V + 1 V".to_string());
        assert_eq!(result.ok(), Some("2 V".to_compact_string()));
    }

    #[test]
    fn test_minimal_expression_with_units2() {
        let mut numbat = NumbatWrapper::new(NumbatConfig::default()).unwrap();
        let result = numbat.eval_block("1 V + 1 V -> mV".to_string());
        assert_eq!(result.ok(), Some("2000 mV".to_compact_string()));
    }

    #[test]
    fn test_double_line_expression() {
        let mut numbat = NumbatWrapper::new(NumbatConfig::default()).unwrap();
        let result = numbat.eval_block("let voltage = 1 V + 1 V -> mV\nvoltage #=".to_string());
        assert_eq!(result.ok(), Some("2000 mV".to_compact_string()));
    }

    #[test]
    fn test_let_expression() {
        let mut numbat = NumbatWrapper::new(NumbatConfig::default()).unwrap();
        let result = numbat.eval_block("let voltage = 1 V + 1 V -> mV".to_string());
        assert_eq!(result.ok(), Some("2000 mV".to_compact_string()));
    }

}

