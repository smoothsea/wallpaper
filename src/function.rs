use crate::fatal;
use std::error::Error;
use std::process::{Command, Stdio};

pub fn get_resolution()->Result<Vec<String>, Box<dyn Error>> {
    check_application("xrandr");
    let ret = Command::new("sh")
        .arg("-c")
        .arg("xrandr|grep \\*|awk '{print $1}'")
        .output()?.stdout;
    let output = String::from_utf8(ret)?;
    let resolutions = output.split("\n").collect::<Vec<&str>>();
    let ret = resolutions.iter().filter(|x| x!=&&"").map(|x| x.to_string()).collect::<Vec<String>>();

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
                fatal!("{} 程序不存在，请先安装它", app);
            } else {
                fatal!("{} 发生错误 :(", app);
            }
        }
    }
}
