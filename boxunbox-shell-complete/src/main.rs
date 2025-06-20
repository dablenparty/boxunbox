use boxunbox::cli::BoxUnboxCli;
use clap::{CommandFactory, ValueEnum};
use clap_complete::Shell;

fn main() {
    let command = BoxUnboxCli::command();
    for shell in Shell::value_variants() {
        let name = command
            .get_bin_name()
            .unwrap_or_else(|| command.get_name())
            .to_string();
        todo!("generating {name} completions for {shell}");
        // example:
        // clap_complete::generate(*shell, &mut command, name, &mut std::io::stdout());
    }
}
