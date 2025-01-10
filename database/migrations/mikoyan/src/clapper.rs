use clap::Parser;

/// Script to generate a schema for a MongoDB collection
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Log file to write to
    #[arg(short, long, default_value = "logs.log")]
    pub logs: String,

    /// MongoDB connection string
    #[arg(
        short,
        long,
        default_value = "mongodb://127.0.0.1:27017/freecodecamp?directConnection=true"
    )]
    pub uri: String,

    /// Use bulk write
    #[arg(long, default_value = "false")]
    pub bulk_write: bool,

    /// Size of the batch to read and write
    #[arg(long, default_value = "100")]
    pub batch_size: usize,
}
