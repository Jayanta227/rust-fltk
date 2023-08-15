use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;
use std::process::exit;

pub fn create_thumbnails(appimages_path: &str, icons_dir: &String, model: &mut Vec<String>) {
    /*
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
    */    

    println!("{:?}", model);
    for appimage in model {
        let mut appimage_clone = appimage.clone();
        let appimage_file_name = Path::new(&appimage).file_stem().unwrap().to_str().unwrap().to_string();
        // appimage_clone.remove(0);
        appimage_clone = format!("{}{}", icons_dir, &appimage_file_name);
        appimage_clone.push_str(".png");



        let mut cmd = Command::new("./src/thumbnailer.sh")
        .arg(&appimage) //first argument of the thumbnailer.sh
        .arg(appimage_clone) //second argument of the thumbnailer.sh
        .arg("200x200") //third argument of the thumbnailer.sh
        .spawn().expect("some error");

        cmd.wait().unwrap();
    }

}