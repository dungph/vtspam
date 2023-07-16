use std::io::{BufRead, BufReader, Read, Write};

use rayon::{
    self,
    prelude::{IntoParallelRefIterator, ParallelIterator},
};

use clap::Parser;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(short)]
    target_folder: std::path::PathBuf,
    #[arg(short)]
    apikey_file_path: std::path::PathBuf,
    #[arg(short)]
    list_sha256_file_path: std::path::PathBuf,
}

fn main() {
    let cli = Cli::parse();
    let target = cli.target_folder;
    std::fs::create_dir_all(&target).ok();

    let key = {
        let mut file = std::fs::File::open(cli.apikey_file_path).unwrap();
        let mut key = String::new();
        file.read_to_string(&mut key).unwrap();
        key.trim().to_string()
    };
    println!("{key}");

    let sha256_list = std::fs::File::open(&cli.list_sha256_file_path).unwrap();
    BufReader::new(sha256_list)
        .lines()
        .map_while(Result::ok)
        .collect::<Vec<String>>()
        .par_iter()
        .for_each(|s| {
            let mut file_path = target.clone();
            file_path.push(s);

            if file_path.exists() {
                let sum = sha256::try_digest(file_path.as_path()).unwrap();
                if s.to_lowercase() == sum {
                    println!("checked {s}");
                    return;
                } else {
                    println!("redownload {s}");
                }
            }

            let mut content = Vec::new();
            println!("downloading {s}");
            ureq::get("https://androzoo.uni.lu/api/download")
                .query("apikey", &key)
                .query("sha256", s)
                .call()
                .unwrap()
                .into_reader()
                .read_to_end(&mut content)
                .unwrap();
            std::fs::OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(file_path)
                .unwrap()
                .write_all(&content)
                .unwrap();
            println!("done {s}");
        });
}
