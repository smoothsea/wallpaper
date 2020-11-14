use crate::Params;
use crate::fatal;
use regex::Regex;
use reqwest;
use std::error::Error;
use std::fs::*;
use std::io::{Read, Write};
use std::path::Path;
use std::collections::HashMap;

#[derive(Debug)]
struct Pic {
    filename: String,
    body: Vec<u8>,
}

impl Pic {
    fn new(filename: String, body: Vec<u8>) -> Pic {
        Pic {
            filename: filename,
            body: body,
        }
    }
}

pub fn download(params: &Params) {
    let resolution = params
        .resolution
        .clone()
        .expect("Resolutions are wrong");
    let empty_dir = params.download_empty;
    let sfw = params.download_sfw;

    let dir = Path::new(&params.dir);
    let metadata = dir.metadata().unwrap();
    if !metadata.is_dir() {
        fatal!("{} is not a dir", &params.dir);
    }; 

    if empty_dir {
        //del previous files or dirs
        for entry in dir.read_dir().expect(&format!("Directory {} can't read", params.dir)) {
            if let Ok(entry) = entry {
                match entry.file_type() {
                    Ok(t) => {
                        if t.is_dir() {
                            if let Err(e) = remove_dir_all(entry.path()){
                                if e.kind() != std::io::ErrorKind::Other && 
                                e.kind() != std::io::ErrorKind::NotFound
                                {
                                    fatal!("Directory {} is failed remove:{:?}",&params.dir, e);
                                }
                            }
                        } else {
                            if let Err(e) = remove_file(entry.path()){
                                if e.kind() != std::io::ErrorKind::Other &&
                                e.kind() != std::io::ErrorKind::NotFound
                                {
                                    fatal!("File {} is failed remove:{:?}",entry.path().to_string_lossy(), e);
                                }
                            }
                        }
                    },
                    Err(e) => {
                        fatal!("Get file types error:{:?}", e);
                    }
                }

            }
        }
    }

    //create picture dirs
    for i in resolution.iter() {
        let pic_dir = format!("{}{}", params.dir, i);
        match create_dir(&pic_dir) {
            Ok(()) => {},
            Err(e) => {
                match e.kind() {
                    std::io::ErrorKind::AlreadyExists => {},
                    _ => {fatal!("Couldn't create dir {}:{:?}", pic_dir, e);},
                }
            }
        }
    }

    let mut pics:HashMap<String, Vec<Pic>> = HashMap::new();
    let mut pic_count:usize = 0;
    //get pictures
    println!("Starting...");
    for i in resolution.iter() {
        match get_pics(i, sfw) {
            Ok(ret) => {
                pic_count += ret.len();
                pics.insert(i.to_owned(), ret);
            },
            Err(e) => panic!("Main error {}", e),
        }
    }
    println!("Gets {} wallpapers", pic_count);



    for (r, p) in pics.iter() {
        let pic_dir = format!("{}{}", params.dir, r);
        save_pics(p, &pic_dir);
    }
}

fn save_pics(pics: &Vec<Pic>, pic_dir: &str) {
    for pic in pics.iter() {
        let file_name = format!("{}/{}", pic_dir, pic.filename);
        let path = Path::new(&file_name);

        let mut file = match File::create(path) {
            Err(why) => {
                fatal!("Couldn't create {}: {}", file_name, why.to_string());
            },
            Ok(file) => file,
        };

        match file.write_all(&pic.body) {
            Err(why) => {
                fatal!("Couldn't write to {}:{}", file_name, why.to_string());
            },
            Ok(_) => {}
        }
    }
}

fn get_pics(resolution: &str, sfw: bool) -> Result<Vec<Pic>, Box<dyn Error>> {
    let mut category = 111;
    let mut purity = 110;
    if sfw {
        category = 110;
        purity = 100;
    }

    let client = reqwest::Client::new();
    let url = format!(
        "{}{}{}{}{}{}{}",
        "https://wallhaven.cc/search?categories=",
        category,
        "&purity=",
        purity,
        "&atleast=",
        resolution,
        "&sorting=random&order=desc"
    );
    let mut res = client.get(&url).send().expect("an error");
    let mut body = "".to_string();

    res.read_to_string(&mut body)?;

    let re = Regex::new("class=\"preview\"\\s+href=\"(.*?)\"")?;
    let mut pics = Vec::new();
    for caps in re.captures_iter(&body) {
        match get_pic_from_detail_page_url(&caps[1]) {
            Ok(pic) => {
                pics.push(pic);
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }

    Ok(pics)
}

fn get_pic_from_detail_page_url(url: &str) -> Result<Pic, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let mut res = client.get(url).send()?;
    let mut body = "".to_string();

    res.read_to_string(&mut body)?;

    let re = Regex::new("id=\"wallpaper\" src=\"(.*?)\"")?;
    let mut full_pic_url = "".to_string();
    for caps in re.captures_iter(&body) {
        full_pic_url = caps[1].to_string();
    }

    let filename = get_basename(&full_pic_url);

    let mut res = client.get(&full_pic_url).send()?;
    let mut body = Vec::new();
    res.read_to_end(&mut body)?;

    Ok(Pic::new(filename, body))
}

fn get_basename(url: &str) -> String {
    let pieces = url.split("/");
    pieces.last().unwrap().to_string()
}
