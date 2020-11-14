mod behaviour;
mod function;

use clap::{App, Arg, SubCommand};
use rand::{Rng};
use std::env;
use std::error::Error;
use std::process::{Command};
use std::thread::{spawn};
use std::{fs, thread, time};
use function::{get_resolution, check_application};

use crate::behaviour::download::{download};

#[macro_export]
macro_rules! fatal {
    ($($tt: tt)*) => {
        use std::io::Write;
        writeln!(&mut ::std::io::stderr(), $($tt)*).unwrap();        
        ::std::process::exit(1);
    };
}

#[derive(Debug, Clone)]
pub struct Params {
    dir: String,
    is_video: bool,
    is_download: bool,
    video_file: Option<String>,
    video_compress_dir: Option<String>,
    download_empty: bool,
    resolution: Option<Vec<String>>,
    download_sfw: bool,
    only_download: bool
}

impl Params {
    fn new(
        dir: String,
        is_video: bool,
        video_file: Option<String>,
        is_download: bool,
        video_compress_dir: Option<String>,
        download_empty: bool,
        resolution: Option<Vec<String>>,
        download_sfw: bool,
        only_download: bool
    ) -> Params {
        Params {
            dir,
            is_video,
            is_download,
            video_file,
            video_compress_dir,
            download_empty,
            resolution,
            download_sfw,
            only_download
        }
    }
}

fn main() {
    let params:Params = match get_params() {
        Ok(p) => p,
        Err(e) => {
            fatal!("{}", e);
        }
    };
    check_dependency(&params);

    handle_exit(params.clone());

    if params.is_download {
        let params_c = params.clone();
        let handle = spawn( move || {
            download(&params_c) }
        );

        if params.only_download {
            handle.join().unwrap();
            std::process::exit(1);
        }
    }

    loop {
        if params.is_video {
            video(&params);
        } else {
            image(&params);
        }
    }
}

fn video(params: &Params) {
    let interval = time::Duration::from_millis(1);
    let dir = params.video_compress_dir.clone().unwrap();
    fs::create_dir(&dir).unwrap();
    println!("Start processing video files...");
    Command::new("ffmpeg")
        .arg("-y")
        .arg("-ss")
        .arg("00:00")
        .arg("-i")
        .arg(params.video_file.clone().unwrap())
        .arg(format!("{}/filename%09d.jpg", &dir))
        .output()
        .expect("sh command failed to start");

    let mut v: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    v.sort_by_key(|dir| dir.path());
    let count = v.len();
    println!("Ok");

    let mut i = 0;
    while i < count {
        let pic_path = format!("{}", &v[i].path().display());
        let mut command = "/usr/bin/feh --bg-scale ".to_string();
        command.push_str(&pic_path);
        Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .expect("faild");

        i += 1;
        if i == count - 1 {
            i = 0;
        }

        thread::sleep(interval);
    }
}

fn image(params: &Params) {
    let ten_millis = time::Duration::from_millis(60000);
    let mut dir:Vec<String> = vec!();
    let resolutions = params.resolution.clone();
    for r in resolutions.unwrap().iter() {
        let resolution_dir = format!("{}{}", &params.dir, r);
        if let Ok(_) = fs::read_dir(&resolution_dir) {
            dir.push(resolution_dir);
        } else {
            // default dir
            let default_dir = format!("{}", &params.dir);
            if let Err(e) = fs::read_dir(&default_dir) {
                fatal!("Directory {} takes error:{}", &default_dir, e);
            } else {
                dir.push(default_dir);
            }
        }
    }

    let mut command = "feh --bg-scale ".to_string();
    for d in dir.iter() {
        command = format!("{} --randomize {} ", command, d);
    }
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("faild");

    thread::sleep(ten_millis);
}

fn get_params() -> Result<Params, Box<dyn Error>> {
    let matches = App::new("Wallpaper")
        .version("1.0")
        .help_message("help").version_message("version")
        .author("smoothsea@yeah.net")
        .about("Manage wallpapers")
        .arg(
            Arg::with_name("directory")
                .short("d")
                .long("directory")
                .help("Wallpaper folder")
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("video").help_message("help").version_message("version")
            .about("Setting video as dynamic wallpaper").arg(
                Arg::with_name("file")
                    .short("f")
                    .long("file")
                    .help("Video path,video needs to be .mp4 suffix")
                    .validator(is_mp4)
                    .required(true)
                    .takes_value(true),
            ),
        )
        .subcommand(
            SubCommand::with_name("download").help_message("help").version_message("version")
            .about("Downloading wallpapers").arg(
                Arg::with_name("empty")
                .short("e")
                .long("empty")
                .help("Empting floder")
                .empty_values(true),
            ).arg(
                Arg::with_name("resolution")
                .short("r")
                .long("resolution")
                .help("Setting resolution of the download wallpaper")
                .takes_value(true)
                .empty_values(false)
            ).arg(
                Arg::with_name("sfw")
                .long("sfw")
                .help("Safe for work")
                .empty_values(true)
            ).arg(
                Arg::with_name("directory")
                .short("d")
                .long("directory")
                .help("Dowanload wallpaper folder")
                .takes_value(true)
            ).arg(
                Arg::with_name("only_download")
                .long("only_download")
                .help("Only download")
                .empty_values(true)
            ),
        )
        .get_matches();
    let is_video = matches.is_present("video");
    let is_download = matches.is_present("download");
    let mut video_file = None;
    let mut video_compress_dir = None;
    if is_video {
        video_file = Some(
            matches
                .subcommand_matches("video")
                .unwrap()
                .value_of("file")
                .unwrap_or("no file")
                .to_owned(),
        );

        let rand_string: String = gen_rand_string();
        let mut path = env::temp_dir();
        path.push(rand_string);
        video_compress_dir = Some(path.as_path().to_str().unwrap().to_owned());
    }

    let mut default_dir = "".to_owned();
    for (key, val) in env::vars() {
        if key == "HOME".to_owned() {
            default_dir = format!("{}{}", default_dir, val);
        }
    }
    default_dir.push_str("/.wallpaper/");
    let mut download_empty = false;
    let mut download_sfw = false;
    let mut only_download = true;
    let mut download_resolution = Some(get_resolution().unwrap());
    let mut dir;
    if is_download {
        download_empty = matches.subcommand_matches("download")
                        .unwrap()
                        .is_present("empty");
        download_sfw = matches.subcommand_matches("download")
                        .unwrap()
                        .is_present("sfw");
        only_download = matches.subcommand_matches("download")
                        .unwrap()
                        .is_present("only_download");

        match matches
            .subcommand_matches("download")
            .unwrap()
            .value_of("resolution") {
            Some(r) => {
                download_resolution = Some(vec![r.to_owned()])
            },
            None => {},
        };

        dir = matches.subcommand_matches("download")
        .unwrap()
        .value_of("directory")
        .unwrap_or(&default_dir)
        .to_owned();
    } else {
       dir = matches
        .value_of("directory")
        .unwrap_or(&default_dir)
        .to_owned();
    }
    if !dir.ends_with("/") {
        dir.push_str("/");
    }


    Ok(Params::new(
        dir,
        is_video,
        video_file,
        is_download,
        video_compress_dir,
        download_empty,
        download_resolution,
        download_sfw,
        only_download,
    ))
}

fn is_mp4(file: String) -> Result<(), String> {
    if !file.ends_with(".mp4") {
        return Err(String::from("Video needs to be .mp4 suffix"));
    }

    match fs::OpenOptions::new().read(true).open(file) {
        Ok(_f) => {}
        Err(e) => {
            return Err(format!("{}", e));
        }
    }

    Ok(())
}

fn check_dependency(params: &Params) {
    let mut dependencies: Vec<&str> = vec![];
    if !params.is_download {
        dependencies.append(&mut vec!["feh -h"]);
    }

    if params.is_video {
        dependencies.append(&mut vec![
            "ffmpeg", "convert", "xrandr", "xdg-open", "bash", "sed",
        ]);
    }

    for i in dependencies.iter() {
        check_application(i);
    }
}

fn gen_rand_string() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    const PASSWORD_LEN: usize = 20;
    let mut rng = rand::thread_rng();

    (0..PASSWORD_LEN)
        .map(|_| {
            let idx = rng.gen_range(0, CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

fn handle_exit(params: Params) {
     ctrlc::set_handler(move || {
         if params.is_video {
             fs::remove_dir_all(params.video_compress_dir.clone().unwrap()).unwrap();
         }
         std::process::abort();
     })
     .expect("Error setting Ctrl-C handler");
}
