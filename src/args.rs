use clap::Parser;
#[derive(Parser, Debug)]
#[command(name = "pong")]
pub(crate) struct Args {
    #[arg(long, short, default_value_t = 6000)]
    pub(crate) port: u16,

    #[arg(long, short, default_value_t = 5)]
    pub(crate) rate_limit: u32,

    #[arg(long, short)]
    pub(crate) tunnel:bool
}