use fltk::{
    enums::{CallbackTrigger, Event},
    group::{Flex},
    prelude::{ImageExt, *},
    *, window::DoubleWindow,
};
use std::{fs::{File, OpenOptions}, io::Read, path, ffi::FromBytesWithNulError};
use serde::{Serialize, Deserialize};
use std::io::BufWriter;
use std::{fs, path::Path,error::Error};
use std::process::{exit, Command, Stdio};
mod refresh;
use refresh::create_thumbnails;

const W: i32 = 900;
const H: i32 = 600;


#[derive(Serialize, Deserialize, Debug)]
struct Config {
    appimages_path: String,
    icons_dir: String,
}

fn load_config() -> Result<Config, Box<dyn Error>> {
    let mut config_file = File::open("config.json")?;
    let mut contents = String::new();
    config_file.read_to_string(&mut contents)?;
    let config: Config = serde_json::from_str(&contents)?;
    Ok(config)
}

#[derive(Clone, Copy)]
enum Message {
    Filter,
    Select,
    OpenFile,
    Refresh,
    KeyInput(enums::Key),
    Close,
    ChangeDir,
    // flex_inner_left_resize,
}

fn main() {
    let mut a = app::App::default().with_scheme(app::Scheme::Gtk);
    let mut appimages_path: String = "".to_string();
    let mut icons_dir: String = "".to_string();
    let (sender, receiver) = app::channel::<Message>();
        //vector for storing the appimage names
        let mut model = Vec::new();
        
        
        if let Ok(config) = load_config() {
            appimages_path = config.appimages_path;
            icons_dir = config.icons_dir;
        } else {
            let mut config_window = window::Window::default().with_label("Config").with_size(W, H).center_screen();
            let mut pack1 = group::Pack::default()
            .with_size(3*W/4 - 50 - 10, 250)
            .with_pos(50, 50)
            .with_type(group::PackType::Vertical);
            pack1.set_spacing(5);
            let mut appimages_path_input = input::FileInput::default().with_size(0, 30);
            appimages_path_input.deactivate();
            let mut icons_dir_input = input::FileInput::default().with_size(0, 30);
            icons_dir_input.deactivate();
            pack1.end();

            let mut pack2 = group::Pack::default()
            .with_size(W/4 - pack1.x(), 250)
            .with_pos(3*W/4, 50)
            .with_type(group::PackType::Vertical);
            pack2.set_spacing(5);
            let mut btn1 = button::Button::default().with_size(0, 30).with_label("appimage dir");
            let mut btn2 = button::Button::default().with_size(0, 30).with_label("icons dir");
            pack2.end();

            let mut pack3 = group::Pack::default()
            .with_size(200, 30)
            .with_pos(W/2 - 100, 250)
            .with_type(group::PackType::Horizontal);
            pack3.set_spacing(50);
            let mut btn_cancel = button::Button::default().with_size(100, 0).with_label("Cancel");
            let mut btn_ok = button::Button::default().with_size(100, 0).with_label("OK");
            pack3.end();

            

                btn1.set_callback(move |_| {
                let mut dialog = dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseDir);
                dialog.show();
                appimages_path = dialog.filename().to_str().unwrap().to_string();
                appimages_path_input.set_value(&appimages_path);

            });

            btn2.set_callback(move |_| {
                let mut dialog = dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseDir);
                dialog.show();
                icons_dir = dialog.filename().to_str().unwrap().to_string();
                icons_dir_input.set_value(&icons_dir);
            });
            btn_cancel.set_callback(
                move |btn| {
                    btn.window().unwrap().hide();
                }
            );

            btn_ok.set_callback(move |_| {
                let mut config_file = OpenOptions::new().write(true).create(true).open("config.json");
                // let mut config: Config = Config {
                //     appimages_path: appimages_path,
                //     icons_dir: icons_dir,
                // };
            });

            config_window.end();
            config_window.make_modal(true);
            config_window.show();
            while config_window.shown() {
                app::wait();
            } 

        }
        
        let mut appimages_path = match load_config() {
            Ok(config) => {config.appimages_path},
            Err(_) => {"/home/jayanta/Desktop/learning/rst/appimage-launcher/src/appimages".to_string()},
        };
        // let mut appimages_path = "/home/jayanta/Desktop/learning/rst/appimage-launcher/src/appimages".to_string();
        let icons_dir = match load_config() {
            Ok(config) => {config.icons_dir},
            Err(_) => {"./src/appimages/.icons/".to_string()}
        };
         
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
            //check the extension of the file and add it to the vector
            if extension == "appimage" || extension == "AppImage" {
                model.push(path.file_name().to_str().unwrap().to_string());
            }
        }
        
        
        // define the UI
        // outermost flex with a rows
        let mut wind = window::Window::default().with_label("Appimages").with_size(W, H).center_screen();
        sender.send(Message::Filter);
        wind.make_resizable(true);
    let mut flex_outer = Flex::default().with_pos(0, 0).size_of_parent().row();

    // left inner flex
    let mut flex_inner_left = Flex::default().with_pos(0, 0).column();
    
    // filter input and list browser in the left inner flex
    let mut filter_input = input::Input::default().with_label("Search");
    let mut list_browser = browser::HoldBrowser::default();
    flex_inner_left.fixed(&filter_input, 30i32);
    flex_inner_left.end();
    
    /*
    // divider at the middle
    let mut divider = frame::Frame::default();
    
    flex_outer.fixed(&divider, 30);
    */
    
    // right inner flex
    let mut flex_inner_right = Flex::default().column();
    let mut fr = frame::Frame::default();
    

    let mut refresh_img_handler = |appimages_path: &String, list_browser: &mut browser::HoldBrowser, wind: &mut DoubleWindow, fr: &mut frame::Frame| -> Result<(), Box<dyn Error>>{
        let binding = format!("{}{}{}",appimages_path.clone(),"/",list_browser.selected_text().unwrap_or_default());
        // appimages_path.clone().push_str("/") + &list_browser.selected_text().unwrap();
        let appimage_name = Path::new(&binding).file_stem().unwrap();
        let binding = appimages_path.clone()+&(("/.icons/".to_string()+appimage_name.to_str().unwrap()).to_string()+".png");
        let appimage_logo_path =Path::new(&binding);
        
        let mut image_logo = image::SharedImage::load(appimage_logo_path.to_str().unwrap())?;
        image_logo.scale(200, 200, true, true);
        fr.set_image(Some(image_logo));
        fr.set_label("");
        wind.redraw();
        Ok(())
    };

    let mut refresh_img = |appimages_path: &String, list_browser: &mut browser::HoldBrowser, wind: &mut DoubleWindow|{
        // let fr1 = &fr;
        if let Err(_err) =  refresh_img_handler(&appimages_path, list_browser, wind, &mut fr) {
            let mut err_logo = image::SharedImage::load("./src/err.png").unwrap();
            fr.set_image(Some(err_logo));
            fr.set_label("refresh to generate thumbnail");
            wind.redraw();
        }
    };

    let mut button_refresh = button::Button::default().with_label("Refresh");
    flex_inner_right.fixed(&mut button_refresh, 35i32);
    button_refresh.set_callback(move |_| {
        sender.send(Message::Refresh);
    });

    let mut button_change_appimage_dir = button::Button::default().with_label("Change directory");
    flex_inner_right.fixed(&mut button_change_appimage_dir, 35i32);
    button_change_appimage_dir.set_callback(move |_| {
        sender.send(Message::ChangeDir);
    });
    flex_outer.fixed(&mut flex_inner_right, W/3);
    flex_inner_right.end();

    flex_outer.end();










    // handle events
    // filter events
    filter_input.set_trigger(CallbackTrigger::Changed);
    filter_input.emit(sender, Message::Filter);
    //list browser events
    list_browser.emit(sender, Message::Select);





    // handle events by closure
    // filter input
    filter_input.handle(move |_, ev| match ev {
        Event::KeyDown => {
            sender.send(Message::KeyInput(app::event_key()));
            true
        }
        _ => false,
    });


    /*
    //divider
    divider.handle({
        let mut x = 0;
        let mut y = 0;
        move |w, ev| match ev {
            enums::Event::Push => {
                let coords = app::event_coords();
                x = coords.0;
                y = coords.1;
                true
            }
            enums::Event::Drag => {
                let coords = app::event_coords();
                let x1 = x - coords.0;
                // let y = coords.1;
                flex_outer.fixed(&flex_inner_left, flex_inner_left.width() - x1);
                println!("dragged");
                
                flex_inner_right.make_resizable(true);
                // flex_inner_left.resize(0, 0, flex_inner_left.width() - x, flex_inner_left.height());
                true
            }
            _ => false,
        }
    });
    */

    wind.end();
    wind.show();
    wind.handle(move |w, evt| match evt {
        // close the window when it no longer has focus
        enums::Event::Unfocus => {
                // if app::screen_coords().x() < w.x() {
                //     println!("{:?}{}", app::screen_coords(), w.x());
                //     sender.send(Message::Close);
                // }

            true
        }
        _ => false,
    });


    /*
    // Popup window to restrict user interaction during background refresh
    let mut popup = window::MenuWindow::default().with_size(150, 60);
    let mut content = frame::Frame::default()
    .size_of_parent()
    .center_of_parent()
    .with_label("This is a popup");
content.set_frame(enums::FrameType::BorderBox);
popup.end();

    // this function sets the NOBORDER and OVERRIDE flags
    // NOTE: this is not currently exposed in the rust binding
    popup.set_override();
    
    popup.handle(|p, evt| match evt {
        enums::Event::Push => {
            p.hide();
            // stop the popup window from intercepting all events
            app::set_grab::<window::MenuWindow>(None);
            true
        }
        _ => false,
    });
    */


    while a.wait() {
        match receiver.recv() {
            Some(Message::Filter) => {
                let prefix = filter_input.value().to_lowercase();
                list_browser.clear();
                for item in &model {
                    if item.to_lowercase().starts_with(&prefix) {
                        list_browser.add(item);
                    }
                }
                list_browser.select(1);
                refresh_img(&appimages_path,&mut list_browser, &mut wind);
            }

            Some(Message::Select) => {
                if list_browser.value() != 0 {
                    refresh_img(&appimages_path, &mut list_browser, &mut wind);
                    println!(
                        "{:?}", list_browser.selected_text().unwrap()
                    );
                }
            }
            Some(Message::Refresh) => {
                let appimages_path_copy = appimages_path.clone();
                create_thumbnails(&appimages_path_copy, &icons_dir, &mut model);
                filter_input.take_focus().unwrap();
                // popup.set_pos(50, 50);
                // popup.show();
                // set popup window to intercept other events
                // app::set_grab(Some(popup.clone()));

            }

            // handle opening the selected appimage
            Some(Message::OpenFile) => {
                Command::new(appimages_path.clone() + &list_browser.selected_text().unwrap())
                    .stdin(Stdio::null())
                    .stderr(Stdio::null())
                    .stdout(Stdio::null())
                    .spawn()
                    .expect("some err");
                exit(0);
            }
            Some(Message::KeyInput(key)) => {
                if key == enums::Key::Enter {
                    sender.send(Message::OpenFile);
                } else if key == enums::Key::Down {
                    list_browser.select(list_browser.value() + 1);
                    refresh_img(&appimages_path, &mut list_browser, &mut wind);
                    
                } else if key == enums::Key::Up {
                    list_browser.select(list_browser.value() - 1);
                    refresh_img(&appimages_path, &mut list_browser, &mut wind);

                } else if key == enums::Key::Escape {
                    wind.hide();

                } 
            }

            Some(Message::ChangeDir) => {
                let mut dialog = dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseDir);
                dialog.show();
                appimages_path = dialog.filename().to_str().unwrap().to_string();
                println!("{}", appimages_path);
            }

            Some(Message::Close) => {
                wind.hide();
                // exit(1);
            }

            None => {}
        }
    }
}
