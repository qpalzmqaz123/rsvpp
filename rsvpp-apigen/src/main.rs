use getopts::Options;
use rsvpp_apigen::{Generator, Result};

struct Config {
    input: String,
    output: String,
}

fn main() -> Result<()> {
    let cfg = parse_cmd()?;
    let mut gen = Generator::new(cfg.output, cfg.input)?;
    gen.gen()?;

    Ok(())
}

fn parse_cmd() -> Result<Config> {
    let args: Vec<String> = std::env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("i", "", "Binapi input dir, like $VPP/share/vpp/api", "DIR");
    opts.optopt("o", "", "Rust output dir", "DIR");
    opts.optflag("h", "help", "Print help message");
    let matches = opts.parse(&args[1..])?;

    let input = matches.opt_str("i");
    let output = matches.opt_str("o");

    if (input.is_none() && output.is_none()) || matches.opt_present("h") {
        print!("{}", opts.usage(&format!("Usage: {} [options]", program)));
        std::process::exit(1);
    }

    let input = input.ok_or("Empty input")?;
    let output = output.ok_or("Empty output")?;

    Ok(Config { input, output })
}
