use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{Parser, Subcommand};


fn styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .usage(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Green.on_default())
}

#[derive(Parser, Debug, Clone)]
#[command(
    help_template = "{usage-heading} {usage} \nVersion: {version} {about-section}Author:{author} Email:jiancghen2@genomics.cn/cherryamme@qq.com\n {all-args} {tab}"
)]
#[command(version, author, about, long_about = None, styles = styles(), subcommand_negates_reqs = true, args_conflicts_with_subcommands = true)]
pub struct Args {

    #[command(subcommand)]
    pub command: Option<Commands>,

    /// The path of input file
    #[arg(short, long, num_args = 1..,value_delimiter = ' ')]
    pub inputs: Vec<String>,
    /// The name of outdir
    #[arg(short, long, default_value = "outdir")]
    pub outdir: String,
    /// Number of threads
    #[arg(short, long, default_value = "20")]
    pub threads: usize,
    /// filter read by min_length
    #[arg(short, long, default_value = "100")]
    pub min_length: usize,
    /// pattern_files for split
	#[arg(short,long, required = true, num_args = 1..,value_delimiter = ' ', required = true)]
	pub pattern_files: Option<Vec<String>>,
    /// pattern_db_file for split
	#[arg(short = 'd', long = "db", required = true)]
    pub pattern_db_file: Option<String>,
    /// fusion file to detect fusion
    #[arg(short = 'f', long = "fusion", default_value= "")]
    pub fusion_file: String,
    /// fusion errrate
    #[arg(long = "fe", default_value = "0.2")]
    pub fusion_errate: f32,
    /// split log nums per record
    #[arg(short = 'n', long = "num", default_value = "500000")]
    pub log_num: u32,
    /// windows size to finder pattern <left,right>
    #[arg(short,long,value_delimiter = ',', default_value="400,400")]
    pub window_size: Vec<usize>,
    /// set a errate for multiple pattern use whiteblack,set left and right errate use comma, errate range in <0-0.5>, err = pattern_len x errate.
    #[arg(short = 'e', long, num_args = 1..,value_delimiter = ' ', default_value="0.2,0.2",value_parser=errrate_validator)]
    pub pattern_errate: Vec<(f32,f32)>,
    /// whether to trim seq, 0 means trim all, 1 means keep one pattern, 2 means keep two pattern...
    #[arg(long, default_value = "0")]
    pub trim_n: usize,
    /// write_type for split
    #[arg(long, default_value = "type", value_parser = ["names","type"])]
    pub write_type: String,
    /// pattern_match for split, can set multiple splittype <single or dual>
    #[arg(long = "match", num_args = 1..,value_delimiter = ' ', default_value="single",value_parser = ["single","dual"])]
    pub pattern_match: Vec<String>,
    /// detect pattern on previous patern pos, more accurate.
    #[arg(long = "pos")]
    pub pattern_pos: bool,
    /// when detect pattern on previous patern pos, set a shift for multiple pattern split, small for short pattern is more accurate.
    #[arg(long = "shift", num_args = 1..,value_delimiter = ' ', default_value="3")]
    pub pattern_shift: Vec<usize>,
    /// set a maxdist for patterns, set left and right maxdist use comma.
    #[arg(long = "maxdist", num_args = 1..,value_delimiter = ',', default_value="4")]
    pub pattern_maxdist: Vec<usize>,
    /// set record id sep delimiter.
    #[arg(long = "id_sep", default_value="%")]
    pub id_sep: String,




}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Encrypt the database file
    Encrypt {
        /// The database file to encrypt
        file: String,
    },
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
