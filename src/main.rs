use fltk::{
    enums::{CallbackTrigger, Event},
    group::Flex,
    prelude::{ImageExt, *},
    window::DoubleWindow,
    *,
};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::process::{exit, Command, Stdio};
use std::{error::Error, fs, path::Path, env};
use std::{
    fs::{File, OpenOptions},
    io::Read,
};
mod refresh;
use lazy_static::lazy_static;
use refresh::create_thumbnails;
// use once_cell::sync::Lazy;
// use std::cell::Cell;
use std::sync::Mutex;

lazy_static! {
    static ref appimages_path: Mutex<String> = Mutex::new("".to_string());
}
lazy_static! {
    static ref icons_dir: Mutex<String> = Mutex::new("".to_string());
}
const W: i32 = 900;
const H: i32 = 600;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    appimages_path: String,
    icons_dir: String,
}

fn config_dir() -> String{
    let mut home_dir = match env::home_dir() {
        Some(path) => path.to_str().unwrap().to_string(),
        None => "/home/".to_string(),
    };
    if home_dir.ends_with('/') {
        home_dir.to_string().pop();
        format!("{}{}", home_dir, "/.config/appimage-launcher")
    } else {
        format!("{}{}", home_dir, "/.config/appimage-launcher")
    }
}

fn load_config() -> Result<Config, Box<dyn Error>> {
    let mut config_file = File::open(config_dir())?;
    let mut contents = String::new();
    config_file.read_to_string(&mut contents)?;
    let config: Config = serde_json::from_str(&contents)?;
    Ok(config)
}

fn refresh_popup() {
    {
        let mut config_window = window::Window::default()
            .with_label("Config")
            .with_size(W, H)
            .center_screen();
        let mut pack1 = group::Pack::default()
            .with_size(3 * W / 4 - 50 - 10, 250)
            .with_pos(50, 50)
            .with_type(group::PackType::Vertical);
        pack1.set_spacing(5);
        let mut appimages_path_input = input::FileInput::default().with_size(0, 30);
        appimages_path_input.deactivate();
        let mut icons_dir_input = input::FileInput::default().with_size(0, 30);
        icons_dir_input.deactivate();
        pack1.end();

        let mut pack2 = group::Pack::default()
            .with_size(W / 4 - pack1.x(), 250)
            .with_pos(3 * W / 4, 50)
            .with_type(group::PackType::Vertical);
        pack2.set_spacing(5);
        let mut btn1 = button::Button::default()
            .with_size(0, 30)
            .with_label("appimage dir");
        let mut btn2 = button::Button::default()
            .with_size(0, 30)
            .with_label("icons dir");
        pack2.end();

        let mut pack3 = group::Pack::default()
            .with_size(200, 30)
            .with_pos(W / 2 - 100, 250)
            .with_type(group::PackType::Horizontal);
        pack3.set_spacing(50);
        let mut btn_cancel = button::Button::default()
            .with_size(100, 0)
            .with_label("Cancel");
        let mut btn_ok = button::Button::default().with_size(100, 0).with_label("OK");
        pack3.end();

        btn1.set_callback({
            move |_| {
                let mut dialog =
                    dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseDir);
                dialog.show();
                let dialog_value = dialog.filename().to_str().unwrap().to_string();
                appimages_path_input.set_value(&dialog_value);
                *appimages_path.lock().unwrap() = dialog_value;
            }
        });

        btn2.set_callback(move |_| {
            let mut dialog =
                dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseDir);
            dialog.show();
            *icons_dir.lock().unwrap() = dialog.filename().to_str().unwrap().to_string();
            icons_dir_input.set_value(icons_dir.lock().unwrap().clone().as_str());
        });
        btn_cancel.set_callback(move |btn| {
            btn.window().unwrap().hide();
        });

        btn_ok.set_callback(move |btn| {
            let config: Config = Config {
                appimages_path: appimages_path.lock().unwrap().to_string(),
                icons_dir: icons_dir.lock().unwrap().to_string(),
            };
            let data = serde_json::to_string_pretty(&config).unwrap();
            let mut config_file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(config_dir())
                .unwrap();
            // write data to config.json
            write!(&mut config_file, "{}", data).unwrap();
            // println!("{}", data);
            btn.window().unwrap().hide();
        });

        config_window.end();
        config_window.make_modal(true);
        config_window.show();
        while config_window.shown() {
            app::wait();
        }
    }
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
}

fn main() {
    let a = app::App::default().with_scheme(app::Scheme::Gtk);
    let (sender, receiver) = app::channel::<Message>();
    //vector for storing the appimage names
    let mut model = Vec::new();

    if let Ok(mut config) = load_config() {
        //remove trailing slash in appimages_path if exists
        *appimages_path.lock().unwrap() = if config.appimages_path.ends_with("/") {
            config.appimages_path.pop();
            config.appimages_path
        } else {
            config.appimages_path
        };
        *icons_dir.lock().unwrap() = if config.icons_dir.ends_with("/") {
            config.icons_dir.pop();
            config.icons_dir
        } else {
            config.icons_dir
        };
    } else {
        refresh_popup();
    }

    *appimages_path.lock().unwrap() = match load_config() {
        Ok(config) => config.appimages_path,
        Err(_) => "./".to_string(),
    };
    *icons_dir.lock().unwrap() = match load_config() {
        Ok(config) => config.icons_dir,
        Err(_) => "./.icons".to_string(),
    };

    let paths = match fs::read_dir(&*appimages_path.lock().unwrap().to_string()) {
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
    let mut wind = window::Window::default()
        .with_label("Appimages")
        .with_size(W, H)
        .center_screen();
    sender.send(Message::Filter);
    wind.make_resizable(true);
    //make window borderless
    wind.set_border(false);
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

    let refresh_img_handler = |list_browser: &mut browser::HoldBrowser,
                                   wind: &mut DoubleWindow,
                                   fr: &mut frame::Frame|
     -> Result<(), Box<dyn Error>> {
        let binding = format!(
            "{}{}{}",
            &*appimages_path.lock().unwrap().clone(),
            "/",
            list_browser.selected_text().unwrap_or_default()
        );
        let appimage_name = Path::new(&binding).file_stem().unwrap();
        // let mut binding = appimages_path.lock().unwrap().clone().to_string()
        // + &(("/.icons/".to_string() + appimage_name.to_str().unwrap()).to_string() + ".png");

        let binding = format!(
            "{}{}{}{}",
            &*icons_dir.lock().unwrap().clone().to_string(),
            "/",
            appimage_name.to_str().unwrap(),
            ".png"
        );

        let appimage_logo_path = Path::new(&binding);
        let mut image_logo = image::SharedImage::load(appimage_logo_path.to_str().unwrap())?;
        image_logo.scale(200, 200, true, true);
        fr.set_image(Some(image_logo));
        fr.set_label("");
        wind.redraw();
        Ok(())
    };

    let mut refresh_img = |list_browser: &mut browser::HoldBrowser, wind: &mut DoubleWindow| {
        // let fr1 = &fr;
        if let Err(_err) = refresh_img_handler(list_browser, wind, &mut fr) {
            let err_logo = image::SharedImage::load("././err.png").unwrap();
            fr.set_image(Some(err_logo));
            fr.set_label("refresh to generate thumbnail");
            wind.redraw();
        }
    };

    let mut button_refresh = button::Button::default().with_label("Refresh thumbnails");
    flex_inner_right.fixed(&mut button_refresh, 35i32);
    button_refresh.set_callback(move |_| {
        sender.send(Message::Refresh);
    });

    let mut button_change_appimage_dir = button::Button::default().with_label("Change directory");
    flex_inner_right.fixed(&mut button_change_appimage_dir, 35i32);
    button_change_appimage_dir.set_callback(move |_| {
        sender.send(Message::ChangeDir);
    });

    let mut button_quit = button::Button::default().with_label("Quit");
    flex_inner_right.fixed(&mut button_quit, 35i32);
    button_quit.set_callback(move |_| {
        sender.send(Message::Close);
    });
    flex_outer.fixed(&mut flex_inner_right, W / 3);
    flex_inner_right.end();
    flex_outer.end();

    // handle events
    // filter events
    filter_input.set_trigger(CallbackTrigger::Changed);
    filter_input.emit(sender, Message::Filter);
    //list browser events
    list_browser.emit(sender, Message::Select);
    list_browser.handle(move |_, ev| match ev {
        //check if its a doubleclick event
        Event::Push => {
            if app::event_clicks(){
                sender.send(Message::OpenFile);
            };
            true
        }
        _ => false,
    });

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
    wind.handle(move |_w, evt| match evt {
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
                refresh_img(&mut list_browser, &mut wind);
            }

            Some(Message::Select) => {
                if list_browser.value() != 0 {
                    refresh_img(&mut list_browser, &mut wind);
                    // println!("{:?}", list_browser.selected_text().unwrap());
                }
            }
            Some(Message::Refresh) => {
                let appimages_path_copy = &*appimages_path.lock().unwrap().to_string();
                let icons_dir_copy = &*icons_dir.lock().unwrap().to_string();
                create_thumbnails(&appimages_path_copy, icons_dir_copy, &mut model);
                filter_input.take_focus().unwrap();
            }

            // handle opening the selected appimage
            Some(Message::OpenFile) => {
                Command::new(format!(
                    "{}{}{}",
                    &*appimages_path.lock().unwrap().clone(),
                    "/",
                    &list_browser.selected_text().unwrap()
                ))
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
                    refresh_img(&mut list_browser, &mut wind);
                } else if key == enums::Key::Up {
                    list_browser.select(list_browser.value() - 1);
                    refresh_img(&mut list_browser, &mut wind);
                } else if key == enums::Key::Escape {
                    wind.hide();
                }
            }

            Some(Message::ChangeDir) => {
                // let mut dialog =
                //     dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseDir);
                // dialog.show();
                // *appimages_path.lock().unwrap() = dialog.filename().to_str().unwrap().to_string();
                refresh_popup();
            }

            Some(Message::Close) => {
                wind.hide();
                // exit(1);
            }

            None => {}
        }
    }
}
