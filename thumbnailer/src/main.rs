use std::env;
use std::fs;
use std::io;
use std::process::Command;
use std::process::Stdio;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().collect();
    let appimages_path:String;

    if args.len() > 1 {
        appimages_path = args[1].clone();
    }
    else {
        appimages_path = String::from("./");
    }
    let mut model: Vec<String> = Vec::new();
    // let mut tmp_path: String;
    let paths = match fs::read_dir(&appimages_path) {
        Ok(path) => path,
        Err(_) => {
            exit(1);
        }
    };

    for path in paths {
        let mut extension = String::new();
        let path = path.unwrap();
        if path.file_type().unwrap().is_file() {
            extension = match path.path().extension() {
                Some(ext) => ext.to_str().unwrap().to_string(),
                None => "others".to_string(),
            };
        };

        if extension == "appimage" || extension == "AppImage" {
            // tmp_path = String::from(path.path().to_str().unwrap());
            model.push(path.path().to_str().unwrap().to_string());
            // println!("{}",path.path().to_str().unwrap());

        }
    }

    println!("{:?}", model);
    for appimage in model {
        let mut appimage_clone = appimage.clone();
        appimage_clone.remove(0);
        appimage_clone.push_str(".png");

        appimage_clone = "./.icons".to_string()+&appimage_clone;


        let mut cmd = Command::new("./thumbnailer.sh").arg(&appimage)
        .arg(appimage_clone)
        .arg("200x200")
        .spawn().expect("some error");

        cmd.wait().unwrap();
    }

}
