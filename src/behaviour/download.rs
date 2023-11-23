use crate::fatal;
use crate::Params;

use rand::Rng;
use regex::Regex;
use reqwest::{self, header};
use std::collections::HashMap;
use std::error::Error;
use std::fs::*;
use std::io::{Read, Write};
use std::path::Path;

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
    let resolution = params.resolution.clone().expect("Resolutions are wrong");
    let empty_dir = params.download_empty;
    let sfw = params.download_sfw;

    let dir = Path::new(&params.dir);
    if empty_dir {
        //del previous files or dirs
        for entry in dir
            .read_dir()
            .expect(&format!("Directory {} can't read", params.dir))
        {
            if let Ok(entry) = entry {
                match entry.file_type() {
                    Ok(t) => {
                        if t.is_dir() {
                            if let Err(e) = remove_dir_all(entry.path()) {
                                if e.kind() != std::io::ErrorKind::Other
                                    && e.kind() != std::io::ErrorKind::NotFound
                                {
                                    fatal!("Directory {} is failed remove:{:?}", &params.dir, e);
                                }
                            }
                        } else {
                            if let Err(e) = remove_file(entry.path()) {
                                if e.kind() != std::io::ErrorKind::Other
                                    && e.kind() != std::io::ErrorKind::NotFound
                                {
                                    fatal!(
                                        "File {} is failed remove:{:?}",
                                        entry.path().to_string_lossy(),
                                        e
                                    );
                                }
                            }
                        }
                    }
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
            Ok(()) => {}
            Err(e) => match e.kind() {
                std::io::ErrorKind::AlreadyExists => {}
                _ => {
                    fatal!("Couldn't create dir {}:{:?}", pic_dir, e);
                }
            },
        }
    }

    let mut pics: HashMap<String, Vec<Pic>> = HashMap::new();
    let mut pic_count: usize = 0;
    //get pictures
    println!("Starting...");
    for i in resolution.iter() {
        let mut available_wallpapers: Vec<Box<dyn Wallpaper>> = vec![Box::new(Wallhaven)];
        
        while available_wallpapers.len() > 0 {
            let index = rand::thread_rng().gen_range(0, available_wallpapers.len());
            match available_wallpapers
                .get(index)
                .unwrap()
                .get_pics(i, sfw, &params.proxy)
            {
                Ok(ret) => {
                    pic_count += ret.len();
                    pics.insert(i.to_owned(), ret);

                    break;
                },
                Err(e) => {
                    match e {
                        DownloadError::NotFound => {
                            available_wallpapers.remove(index);

                            if available_wallpapers.len() == 0 {
                                fatal!("Main error Not Found");
                            }
                        },
                        _ => panic!("Main error {}", e),
                    }
                    
                },
            }
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
            }
            Ok(file) => file,
        };

        match file.write_all(&pic.body) {
            Err(why) => {
                fatal!("Couldn't write to {}:{}", file_name, why.to_string());
            }
            Ok(_) => {}
        }
    }
}

static mut SINGLETON_HTTP_CLIENT: Option<Singleton> = None;

struct Singleton {
    v: reqwest::blocking::Client,
}

impl Singleton {
    fn new(proxy: &Option<String>) -> &Singleton {
        unsafe {
            match &SINGLETON_HTTP_CLIENT {
                Some(r) => r,
                None => {
                    let mut headers = header::HeaderMap::new();
                    headers.insert("User-agent", header::HeaderValue::from_static(
                            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.0.4638.69 Safari/537.36"));
                    let mut client = reqwest::blocking::Client::builder().default_headers(headers);
                    if let Some(p) = proxy {
                        client = client.proxy(reqwest::Proxy::https(p).unwrap());
                    }
                    let client = client.build().unwrap();
                    SINGLETON_HTTP_CLIENT = Some(Singleton { v: client });
                    Singleton::new(proxy)
                }
            }
        }
    }
}

#[derive(Debug)]
enum DownloadError {
    Reqwest(reqwest::Error),
    Io(std::io::Error),
    Regex(regex::Error),
    NotFound,
}

impl std::fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadError::Reqwest(error) => {
                write!(f, "{}", error)
            },
            DownloadError::Io(error) => {
                write!(f, "{}", error)
            },
            DownloadError::Regex(error) => {
                write!(f, "{}", error)
            },
            DownloadError::NotFound => {
                write!(f, "Not found")
            },
        }
    }
}

impl From<reqwest::Error> for DownloadError {
    fn from(err: reqwest::Error) -> Self {
        DownloadError::Reqwest(err)
    }
}

impl From<std::io::Error> for DownloadError {
    fn from(err: std::io::Error) -> Self {
        DownloadError::Io(err)
    }
}

impl From<regex::Error> for DownloadError {
    fn from(err: regex::Error) -> Self {
        DownloadError::Regex(err)
    }
}

impl std::error::Error for DownloadError {}

struct Wallhaven;
struct Wstock;

trait Wallpaper {
    fn get_pics(
        &self,
        resolution: &str,
        sfw: bool,
        proxy: &Option<String>,
    ) -> Result<Vec<Pic>, DownloadError>;
}

impl Wallhaven {
    fn get_pic_from_detail_page_url(
        &self,
        url: &str,
        proxy: &Option<String>,
    ) -> Result<Pic, Box<dyn Error>> {
        let client = &Singleton::new(proxy).v;
        let mut res = client.get(url).send()?;
        let mut body = "".to_string();

        res.read_to_string(&mut body)?;

        let re = Regex::new("id=\"wallpaper\" src=\"(.*?)\"")?;
        let mut full_pic_url = "".to_string();
        for caps in re.captures_iter(&body) {
            full_pic_url = Self::parse_pic_url(caps[1].to_string());
        }

        let filename = get_basename(&full_pic_url);

        let mut res = client.get(&full_pic_url).send()?;
        let mut body = Vec::new();
        res.read_to_end(&mut body)?;

        Ok(Pic::new(filename, body))
    }

    fn parse_pic_url(mut url: String) -> String {
        if url.starts_with("/cdn-cgi") {
            url = format!("https://w.wallhaven.cc{}", url);
        }
        url
    }
}

impl Wstock {
    fn get_pic_from_detail_page_url(
        &self,
        url: &str,
        proxy: &Option<String>,
    ) -> Result<Pic, Box<dyn Error>> {
        let client = &Singleton::new(proxy).v;

        let filename = get_basename(url);
        let mut res = client.get(url).send()?;
        let mut body = Vec::new();
        res.read_to_end(&mut body)?;

        Ok(Pic::new(filename, body))
    }
}

impl Wallpaper for Wallhaven {
    fn get_pics(
        &self,
        resolution: &str,
        sfw: bool,
        proxy: &Option<String>,
    ) -> Result<Vec<Pic>, DownloadError> {
        let mut category = 111;
        let mut purity = 110;
        if sfw {
            category = 110;
            purity = 100;
        }

        let client = &Singleton::new(proxy).v;
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
            match self.get_pic_from_detail_page_url(&caps[1], proxy) {
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
}

impl Wallpaper for Wstock {
    fn get_pics(
        &self,
        resolution: &str,
        _sfw: bool,
        proxy: &Option<String>,
    ) -> Result<Vec<Pic>, DownloadError> {
        let client = &Singleton::new(proxy).v;
        let prefix_url = "https://wallpaperstock.net";
        let url = format!("{}/wallpapers_{}r.html", prefix_url, resolution,);
        let mut res = client.get(&url).send().expect("an error");
        let mut body = "".to_string();
        res.read_to_string(&mut body)?;
        let re = Regex::new("class=\"pagination\">.*?a>\\.\\.\\.<a.*?>(\\d+)</")?;
        let matches = &re.captures_iter(&body).next();
        if matches.is_none() {
            return Err(DownloadError::NotFound);
        }

        let max_page_match = (matches.as_ref().unwrap()[1]).parse::<u32>().unwrap();

        let random_page = rand::thread_rng().gen_range(1, max_page_match);
        let url = format!(
            "{}/wallpapers_p{}_{}r.html",
            prefix_url, random_page, resolution,
        );
        let mut res = client.get(&url).send().expect("an error");
        let mut body = "".to_string();
        res.read_to_string(&mut body)?;

        let re = Regex::new("class=\"links\">[\\s\\S]*?href='(.*?)'")?;

        let mut pics = Vec::new();
        for caps in re.captures_iter(&body) {
            let new_path = &caps[1]
                .replace("-wallpapers_w", "_wallpapers_")
                .replace(".html", &format!("_{}{}", resolution, ".jpg"));
            match self.get_pic_from_detail_page_url(&format!("{}{}", prefix_url, new_path), proxy) {
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
}

fn get_basename(url: &str) -> String {
    let pieces = url.split("/");
    pieces.last().unwrap().to_string()
}
