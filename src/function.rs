use crate::fatal;
use std::error::Error;
use std::process::{Command, Stdio};
use std::fs::{read_dir};
use rand::Rng;

pub fn get_resolution() -> Result<Vec<String>, Box<dyn Error>> {
    check_application("xrandr");
    // Feh says 'Cm xrandr --listmonitor to determine how Xinerama monitor IDs map to screens / monitors in your setup'.But that is not working in my device.
    let ret = Command::new("sh")
        .arg("-c")
        .arg("xrandr --listactivemonitors|sed 1d|awk '{print $3}'|sed -E 's/^(.+)\\/.+x(.+?)\\/.+$/\\1x\\2/'")
        .output()?
        .stdout;
    let output = String::from_utf8(ret)?;
    
    let ret = output.split("\n").filter(|x| x.trim() != "").map(|x| x.to_string()).collect::<Vec<String>>();

    if ret.len() == 0 {
        Err("Get resolutions error")?
    }

    Ok(ret)
}

pub fn check_application(app: &str) {
    let slice = app.split(" ").collect::<Vec<&str>>();
    let mut iter = slice.iter();
    let app = iter.next().unwrap();

    let mut command = Command::new(app);
    for p in iter {
        command.arg(p);   
    }
    match command
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(_out) => {}
        Err(e) => {
            if let std::io::ErrorKind::NotFound = e.kind() {
                fatal!("Program {} does not exist, please install first", app);
            } else {
                fatal!("{} takes error:(", app);
            }
        }
    }
}

pub fn get_random_file(dir: &str) -> String {
    let mut pictures:Vec<String> = vec!();
    let mut rand = dir.to_string();
    match read_dir(dir) {
        Ok(r) => {
            for i in r {
                if let Ok(file) = i {
                    pictures.push(file.file_name().into_string().unwrap());
                }
            }
        },
        Err(_e) => {
            return rand;
        }
    }
    
    if pictures.len() > 0 {
        let mut rng = rand::thread_rng();
        let rand_index = rng.gen_range(0, pictures.len());
        rand = format!("{}{}{}", rand, "/", pictures.get(rand_index).unwrap().to_owned());
    }
    
    rand
}
