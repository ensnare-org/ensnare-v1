// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! The `render` example generates a WAV file from a serialized [DiskProject].

use clap::Parser;
use ensnare::all_entities::EnsnareEntities2;
use ensnare::prelude::*;

#[derive(Parser, Debug, Default)]
#[clap(author, about, long_about = None)]
struct Args {
    /// Names of files to process. Currently accepts JSON-format projects.
    input: Vec<String>,

    /// Render as WAVE file(s) (file will appear next to source file)
    #[clap(short = 'w', long, value_parser)]
    wav: bool,

    /// Enable debug mode
    #[clap(short = 'd', long, value_parser)]
    debug: bool,

    /// Print version and exit
    #[clap(short = 'v', long, value_parser)]
    version: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    for input_filename in args.input {
        match std::fs::File::open(input_filename.clone()) {
            Ok(f) => match serde_json::from_reader::<_, Project>(std::io::BufReader::new(f)) {
                Ok(mut project) => {
                    eprintln!(
                        "Successfully read {} from {}",
                        project.title, input_filename
                    );
                    project.after_deser();
                    if args.wav {
                        let re = regex::Regex::new(r"\.json$").unwrap();
                        let output_filename = re.replace(&input_filename, ".wav");
                        if input_filename == output_filename {
                            panic!("would overwrite input file; couldn't generate output filename");
                        }
                        let output_path = std::path::PathBuf::from(output_filename.to_string());
                        if let Err(e) = project.export_to_wav(output_path) {
                            eprintln!("error while writing {input_filename} render to {output_filename}: {e:?}");
                            return Err(e);
                        }
                    }
                }
                Err(e) => eprintln!("error while parsing {input_filename}: {e:?}"),
            },
            Err(e) => eprintln!("error while opening {input_filename}: {e:?}"),
        }
    }
    Ok(())
}
