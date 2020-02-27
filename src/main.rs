extern crate clap;
use clap::{App, AppSettings, Arg, SubCommand};
use std::error;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use threadpool::ThreadPool;

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

        let run_cmd_result = run_rm_cmd(&vals, verbose);

        if let Err(error) = run_cmd_result {
            println!("{:?}: {}", std::env::current_exe(), error);
            std::process::exit(1);
        }
        // let f = match result {
        //     Ok(result) => file,
        //     Err(error) => panic!("Problem opening the file: {:?}", error),
        // };
    }
    if let Some(matches) = matches.subcommand_matches("cp") {
        let vals: Vec<&Path> = matches.values_of("FILE").unwrap().map(Path::new).collect();

        let _result = run_cp_cmd(&vals, verbose);
    }
}

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

fn run_cp_cmd(_targets: &Vec<&Path>, _verbose: bool) {
    panic!("")
}

fn run_rm_cmd(targets: &Vec<&Path>, _verbose: bool) -> Result<()> {
    let (files_to_delete_tx, files_to_delete_rx) = channel();
    let (dir_to_delete_tx, dir_to_delete_rx) = channel();

    let n_workers = 8;
    let thread_pool = ThreadPool::new(n_workers);

    for path in targets {
        if !path.exists() {
            //return Err(());
        }
        let path_can = fs::canonicalize(path)?;
        if path_can.is_dir() {
            dir_to_delete_tx.send(fs::canonicalize(path)?)?;
            visit_dirs(&path_can, thread_pool.clone(), files_to_delete_tx.clone())?;
        } else {
            files_to_delete_tx.send(path.to_path_buf())?;
        }
    }
    drop(files_to_delete_tx);
    drop(dir_to_delete_tx);

    files_to_delete_rx.iter().for_each(|file| {
        thread_pool.execute(move || {
            let path_str = file.to_str();

            match path_str {
                Some(path_str) => {
                    println!("file: {}", path_str);
                    let remove_result = fs::remove_file(&file);
                    if let Err(error) = remove_result {
                        println!("{} - {}", path_str, error);
                        std::process::exit(2);
                    };
                }
                None => {}
            };
        });
    });

    dir_to_delete_rx.iter().for_each(|dir| {
        thread_pool.execute(move || {
            let path_str = dir.to_str();

            match path_str {
                Some(path_str) => {
                    println!("dir: {}", path_str);
                }
                None => {}
            };
        })
    });

    thread_pool.join();

    println!();
    println!();
    println!();
    println!();
    println!();
    return Ok(());
}

fn visit_dirs(
    dir_str: &PathBuf,
    pool: ThreadPool,
    cb: std::sync::mpsc::Sender<PathBuf>,
) -> Result<()> {
    let dir = Path::new(dir_str);

    if !dir.is_dir() {
        panic!("")
    }

    for entry in fs::read_dir(&dir)? {
        let entry = entry?;

        let path_buf = entry.path();
        let path = path_buf.as_path();

        let full_path_buf = fs::canonicalize(&path_buf)?;
        if path.is_dir() {
            let files_tx = cb.clone();
            let moveed_pool = pool.clone();
            pool.execute(move || {
                visit_dirs(&full_path_buf, moveed_pool, files_tx).unwrap();
            });
        } else if path.is_file() {
            let file_full_path_buf = fs::canonicalize(&path_buf)?;
            cb.send(file_full_path_buf)?;
        }
    }

    Ok(())
}
