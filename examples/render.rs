// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! The `render` example generates a WAV file from a serialized [DiskProject].

use clap::Parser;
use ensnare::prelude::*;
use ensnare_entity::traits::EntityBounds;

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

struct RenderProject {
    title: ProjectTitle,
    orchestrator: Orchestrator<dyn EntityBounds>,
}
impl From<DiskProject> for RenderProject {
    fn from(project: DiskProject) -> Self {
        Self {
            title: project.title,
            orchestrator: Orchestrator::new(), // TODO
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    for input_filename in args.input {
        match std::fs::File::open(input_filename.clone()) {
            Ok(f) => match serde_json::from_reader::<_, DiskProject>(std::io::BufReader::new(f)) {
                Ok(project) => {
                    let mut render_project: RenderProject = project.into();
                    eprintln!(
                        "Successfully read {} from {}",
                        render_project.title, input_filename
                    );
                    if args.wav {
                        let re = regex::Regex::new(r"\.json$").unwrap();
                        let output_filename = re.replace(&input_filename, ".wav");
                        if input_filename == output_filename {
                            panic!("would overwrite input file; couldn't generate output filename");
                        }
                        let output_path = std::path::PathBuf::from(output_filename.to_string());
                        let mut helper = OrchestratorHelper::<dyn EntityBounds>::new_with(
                            &mut render_project.orchestrator,
                        );
                        if let Err(e) = helper.write_to_file(&output_path) {
                            eprintln!(
                                "error while writing {input_filename} render to {}: {e:?}",
                                output_path.display()
                            );
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
