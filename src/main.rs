extern crate clap;
use clap::{App, AppSettings, Arg, SubCommand};
use std::fs::{self, DirEntry};
use std::io;
use std::path::Path;

fn main() {
    let verbose_arg = Arg::with_name("verbose")
        .short("v")
        .long("verbose")
        .multiple(true)
        .help("Sets the level of verbosity");

    let files_arg = Arg::with_name("FILE")
        .help("Sets the file to use")
        .required(true)
        .multiple(true)
        .index(1);

    let matches = App::new("fast")
        .version("1.0")
        .author("magoo <mael.gouacide@gmail.com>")
        .about("Do it fast")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("cp")
                .about("copy files")
                .arg(&files_arg)
                .arg(&verbose_arg),
        )
        .subcommand(
            SubCommand::with_name("rm")
                .about("delete files")
                .arg(&files_arg)
                .arg(&verbose_arg),
        )
        .get_matches();

    let verbose = matches.is_present("verbose");

    if let Some(matches) = matches.subcommand_matches("rm") {
        let vals: Vec<&Path> = matches.values_of("FILE").unwrap().map(Path::new).collect();

        run_rm_cmd(&vals, verbose)
    }

    fn run_rm_cmd(targets: &Vec<&Path>, verbose: bool) {
        for path in targets {
            println!(
                "{:?} - exists: {}, isdir: {}, isfile: {}",
                path.to_str(),
                path.exists(),
                path.is_dir(),
                path.is_file()
            )

            if (path.is_dir()) {
                visit_dirs(dir: &Path, cb: &dyn Fn(&DirEntry))
            }
        }
    }

    // more program logic goes here...
}

fn visit_dirs(dir: &Path, cb: &dyn Fn(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}
