use fltk::{
    enums::{CallbackTrigger, Event},
    group::{Flex, Pack, PackType},
    prelude::{ImageExt, *},
    *, window::DoubleWindow,
};

use std::{fs, path::Path, fmt::format};
use std::process::{exit, Command, Stdio};
mod refresh;
use refresh::create_thumbnails;

const W: i32 = 900;
const H: i32 = 600;

#[derive(Clone, Copy)]
enum Message {
    Filter,
    Select,
    OpenFile,
    Refresh,
    KeyInput(enums::Key),
    Close,
    // flex_inner_left_resize,
}

fn main() {
        //vector for storing the appimage names
        let mut model = Vec::new();
        let appimages_path = "/home/jayanta/Desktop/learning/rst/appimage-launcher/src/appimages".to_string();
    
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
    let mut a = app::App::default().with_scheme(app::Scheme::Gtk);
    let mut wind = window::Window::default().with_label("Appimages").with_size(W, H).center_screen();
    let (sender, receiver) = app::channel::<Message>();
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
    

    // let binding = appimages_path.clone() + &list_browser.selected_text().unwrap();
    // let appimage_name = Path::new(&binding).file_stem().unwrap();
    // let binding = appimages_path.clone()+&((".icons".to_string()+appimage_name.to_str().unwrap()).to_string()+".png");
    // let appimage_logo_path =Path::new(&binding);
    
    
    // let mut image_logo = image::SharedImage::load(appimage_logo_path.to_str().unwrap()).unwrap();
    // image_logo.scale(200, 200, true, true);
    // fr.set_image(Some(image_logo));

    let mut refresh_img = |list_browser: &mut browser::HoldBrowser, wind: &mut DoubleWindow|{
        let binding = format!("{}{}{}",appimages_path.clone(),"/",list_browser.selected_text().unwrap());
        // appimages_path.clone().push_str("/") + &list_browser.selected_text().unwrap();
        let appimage_name = Path::new(&binding).file_stem().unwrap();
        let binding = appimages_path.clone()+&(("/.icons/".to_string()+appimage_name.to_str().unwrap()).to_string()+".png");
        let appimage_logo_path =Path::new(&binding);
        
        let mut image_logo = image::SharedImage::load(appimage_logo_path.to_str().unwrap()).unwrap();
        image_logo.scale(200, 200, true, true);
        fr.set_image(Some(image_logo));
        wind.redraw()
    };


    let mut button_refresh = button::Button::default().with_label("Refresh");
    flex_inner_right.fixed(&mut button_refresh, 35i32);
    button_refresh.set_callback(move |_| {
        sender.send(Message::Refresh);
    });
    flex_outer.fixed(&mut flex_inner_right, W/3);
    flex_inner_right.end();

    flex_outer.end();










    // handle events
    // filter events
    filter_input.set_trigger(CallbackTrigger::Changed);
    filter_input.emit(sender, Message::Filter);
    //list browser events
    list_browser.emit(sender, Message::Filter);
    // refresh_img(&mut list_browser, &mut wind);





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
                    println!(
                        "{:?}",
                        match list_browser.selected_text() {
                            Some(s) => {
                                s
                            }
                            None => {
                                "not selected".to_string()
                            }
                        }
                    );
                }
            }
            Some(Message::Refresh) => {
                create_thumbnails(&appimages_path);
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
                    refresh_img(&mut list_browser, &mut wind);
                    
                } else if key == enums::Key::Up {
                    list_browser.select(list_browser.value() - 1);
                    refresh_img(&mut list_browser, &mut wind);

                }
            }

            Some(Message::Close) => {
                // wind.hide();
                // exit(1);
            }

            None => {}
        }
    }
}
