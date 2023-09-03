use std::fs;
use std::path::Path;
use std::process::Command;

pub fn create_thumbnails(appimages_path: &str, icons_dir: &str, model: &mut Vec<String>) {
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

    // println!("{:?}", model);
    for appimage in model {
        // let mut appimage_clone = appimage.clone();
        let appimage_file_stem = Path::new(&appimage).file_stem().unwrap().to_str().unwrap();
        // appimage_clone.remove(0);
        let mut thumbnail_out = format!("{}{}{}", icons_dir,"/",&appimage_file_stem);
        thumbnail_out.push_str(".png");

        let appimage_file = format!("{}{}{}",appimages_path,"/",Path::new(&appimage).file_name().unwrap().to_str().unwrap());
        // println!("{}",appimage_file);

        // create icons_dir if not exists
        if !Path::new(icons_dir).exists() {
            fs::create_dir(icons_dir).unwrap();
        }
        // println!("debug :{}", &appimage);
        let mut cmd = Command::new("././thumbnailer.sh")
        .arg(appimage_file) //first argument of the thumbnailer.sh for the appimage path
        .arg(thumbnail_out) //second argument for the thumbnail output dir
        .arg("200x200") //third argument for the thumbnail size
        .arg(appimages_path)
        .spawn().expect("some error");

        cmd.wait().unwrap();
    }

}