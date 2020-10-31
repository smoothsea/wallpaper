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
    let resolutions = output.split("\n").collect();
    println!("{:?}", resolutions);

    Ok(vec!["xxxx".to_string()])
}

pub fn check_application(app: &str) {
    match Command::new(app)
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
