use clap::Parser;
use clap::builder::styling::{AnsiColor, Effects, Styles};

fn styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .usage(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Green.on_default())
}

#[derive(Parser, Debug, Clone)]
#[command(version, author, about, long_about = None, styles = styles())]
pub struct Args {
    /// The path of input file
    #[arg(short, long, num_args = 1..,value_delimiter = ' ', default_value = "example/barcode21.fastq.gz")]
    pub inputs: Vec<String>,
    /// The name of outdir
    #[arg(short, long, default_value = "outdir")]
    pub outdir: String,
    /// Number of threads
    #[arg(short, long, default_value = "4")]
    pub threads: usize,
    /// whether to plot the data
    // #[arg(short, long, default_value = "false")]
    // plot: bool,
	#[arg(long = "db",default_value="example/pattern.db")]
    pub pattern_db_file: String,
	#[arg(short,long, num_args = 1..,value_delimiter = ' ', default_value="example/primer.list example/index.list")]
	pub pattern_files: Vec<String>,

    #[arg(long, num_args = 1..,value_delimiter = ' ', default_value="dual dual single",value_parser = ["single","dual"])]
    pub pattern_match: Vec<String>,

    // #[arg(long, num_args = 1..,value_delimiter = ' ', default_value="1 2 3",value_parser = clap::value_parser!(u16).range(1..4))]
    // pub pattern_order: Vec<u16>,

    #[arg(long,value_delimiter = ',', default_value="400,400")]
    pub window_size: Vec<usize>,
    #[arg(long = "pos", help="detect pattern on previous patern pos")]
    pub pattern_pos: bool,

    #[arg(long = "log_num", default_value = "100000", help="handle reads num log per num")]
    pub log_num: u32,

    // #[arg(long, num_args = 1..,value_delimiter = ' ', default_value="0 0 0")]
    // pub pattern_shift: Vec<usize>,
    #[arg(long, num_args = 1..,value_delimiter = ' ', default_value="0.2,0.3 0.2,0.3 0.2,0.3",value_parser=errrate_validator,help="set a errate for multiple pattern use whiteblack,set left and right errate use comma, err = pattern_len x errate, errate range in <0-0.5>")]
    pub pattern_errate: Vec<(f32,f32)>,
    #[arg(long, num_args = 1..,value_delimiter = ',', default_value="4,4,4")]
    pub pattern_maxdist: Vec<usize>,
}

fn errrate_validator(input: &str) -> Result<(f32,f32), String> {
    let pattern_errate: Vec<&str> =  input.split(',').collect();
    if pattern_errate.len() != 2 {
        return Err("pattern_errate should be two comma-separated values".to_string());
    }
    let errate1 = pattern_errate[0].parse();
    let errate2 = pattern_errate[1].parse();
    match (errate1, errate2) {
        (Ok(errate1), Ok(errate2)) if errate1 >= 0.0 && errate1 <= 0.5 && errate2 >= 0.0 && errate2 <= 0.5 => {
            Ok((errate1, errate2))
        },
        _ => Err("Error pattern_errate. They should be floats in the range 0 to 0.5.".to_string()),
    }
}

// pub fn get_input_args(){
// 	let args = Args::parse();
// 	println!("{:?}", args);
// 	info!("{:?}", args);
// }
