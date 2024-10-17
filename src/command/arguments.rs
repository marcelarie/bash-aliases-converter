use std::sync::OnceLock;

pub struct CliArgs {
    pub file_path: String,
    pub no_comments: bool,
}

struct GatheredArgs {
    file_path: Option<String>,
    no_comments: bool,
    arguments: Vec<String>,
    #[allow(unused)]
    remaining_args: Vec<String>,
}

pub static DEBUG_MODE_GLOBAL: OnceLock<bool> = OnceLock::new();

pub fn is_debug_mode() -> bool {
    *DEBUG_MODE_GLOBAL.get().unwrap_or(&false)
}

impl CliArgs {
    fn gather() -> GatheredArgs {
        let mut arguments: Vec<String> = Vec::new();
        let mut script_name = None;
        let mut args = std::env::args();
        let mut no_comments = false;

        // Skip the program name
        args.next();

        while let Some(arg) = args.next() {
            if !arg.starts_with('-') && script_name.is_none() {
                if arg.ends_with(".nu") {
                    eprintln!("Error: Invalid script name '{}'.\nThe input should be a bash aliases script, not a Nushell script.", arg);
                    std::process::exit(1);
                }

                script_name = Some(arg);
                // TODO: Check if this should be continue or break
                continue;
            };

            let flag_value = match arg.as_ref() {
                "--no-comments" | "-nc" => {
                    no_comments = true;
                    Some(arg.to_string())
                }
                "--debug" | "-d" => {
                    DEBUG_MODE_GLOBAL.get_or_init(|| true);
                    Some(arg.to_string())
                }
                "--help" | "-h" => Some(arg.to_string()),
                _ => {
                    let chars = arg.chars().collect::<Vec<char>>();
                    for (index, c) in chars.iter().enumerate() {
                        if c == &'-' {
                            println!("Invalid flag: {}", arg);
                            println!("Use -h for help");
                            std::process::exit(1);
                        }
                        let flag = format!("-{}", c);
                        let value =
                            arg.chars().skip(index + 1).collect::<String>();
                        arguments.push(flag);
                        arguments.push(value);
                        break;
                    }
                    None
                }
            };

            if let Some(flag_value) = flag_value {
                arguments.push(flag_value);
            }
        }

        GatheredArgs {
            arguments,
            file_path: script_name,
            no_comments,
            remaining_args: args.collect(),
        }
    }

    fn print_help() {
        let program_name = std::env::args()
            .next()
            .unwrap_or_else(|| "nu-alias-converter".to_string());

        println!("Nu Alias Converter");
        println!("A tool that converts bash aliases to nushell without breaking your nu config.");
        println!();
        println!("Usage: {} [options] <bash-aliases-script>", program_name);
        println!();
        println!("Options:");
        println!("  -nc, --no-comments  Do not include comments with the failed aliases in the output");
        println!(
            "  -d,  --debug        Print debug information during conversion"
        );
        println!("  -h,  --help         Display this help message and exit");
        println!();
        println!("Arguments:");
        println!(
            "  <file_path>         Path to the alias shell file to convert"
        );
        println!();
        println!("Example:");
        println!("  {} --no-comments ~/.bash_aliases", program_name);
    }

    pub fn new() -> Result<Self, &'static str> {
        let gathered = Self::gather();
        let is_help_request = gathered
            .arguments
            .iter()
            .any(|arg| arg == "--help" || arg == "-h");

        if is_help_request {
            Self::print_help();
            std::process::exit(0);
        }

        let file_path = gathered.file_path.ok_or(
            "No file path provided.\nShow the help menu with -h or --help",
        )?;

        Ok(Self {
            file_path,
            no_comments: gathered.no_comments,
        })
    }
}
