use fltk::{
    enums::{CallbackTrigger, Event},
    prelude::{ImageExt, *},
    *,
};
// use fork::{daemon};
// use run_script::ScriptOptions;
use std::fs;
use std::process::{exit, Command, Stdio};


const W: i32 = 900;
const H: i32 = 600;

#[derive(Clone, Copy)]
enum Message {
    Filter,
    Select,
    OpenFile,
    KeyInput(enums::Key),
}

fn main() {
    let mut model = Vec::new();
    let appimages_path = "/home/jayanta/Softwares/appimage/".to_string();

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
            model.push(path.file_name().to_str().unwrap().to_string());
            // println!("{:?}", path.path().to_str().unwrap());
        }
    }

    let a = app::App::default().with_scheme(app::Scheme::Gtk);
    let mut wind = window::Window::default().with_label("Appimages");
    wind.set_size(W, H);
    let (sender, receiver) = app::channel::<Message>();
    sender.send(Message::Filter);

    let mut filter_input = input::Input::default()
        .with_size(W / 2, 30)
        .with_pos(5, 5)
        .with_label("search");

    let mut list_browser = browser::HoldBrowser::default()
        .with_pos(5, 40)
        .with_size(W / 2, H - 50);

    list_browser.handle(move |_wi, ev| match ev {
        Event::KeyDown => {
            if app::event_key() == enums::Key::Enter {
                sender.send(Message::OpenFile);
            };
            true
        }
        _ => false,
    });
    list_browser.emit(sender, Message::Select);

    filter_input.set_trigger(CallbackTrigger::Changed);
    filter_input.emit(sender, Message::Filter);

    filter_input.handle(move |_, ev| match ev {
        // Event::KeyDown => {
        //     if app::event_key() == enums::Key::Enter {
        //         sender.send(Message::OpenFile);
        //     }
        //     true
        // }
        Event::KeyDown => {
            sender.send(Message::KeyInput(app::event_key()));
            true
        }
        _ => false,
    });

    let mut fr = frame::Frame::default()
        .with_pos(W / 2 + 20, H / 2 - W / 6)
        .with_size(W / 3, W / 3);

    let mut image_logo = image::SharedImage::load("src/logo.png").unwrap();
    image_logo.scale(200, 200, true, true);
    fr.set_image(Some(image_logo));

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

            Some(Message::OpenFile) => {
                Command::new(appimages_path.clone()+&list_browser.selected_text().unwrap())
                .stdin(Stdio::null())
                .stderr(Stdio::null())
                .stdout(Stdio::null())
                .spawn()
                .expect("some err");

                // println!("{}",appimages_path.clone()+&list_browser.selected_text().unwrap()+" &");
                // wind.hide();
                // exit(0);

                // let options = ScriptOptions::new();
                // let args = vec![];
                // let script = "#!/usr/bin/env sh\n".to_string()+&(appimages_path.clone()+&list_browser.selected_text().unwrap());
                // let child = run_script::spawn(&(script+" &") ,&args, &options).unwrap();

                // match daemon(false, true) {
                //     Ok(_) => {
                //         println!("hello from fork");
                //                     Command::new(appimages_path.clone()+&list_browser.selected_text().unwrap())
                //                     .spawn ()
                //                     .expect("some err");
                //     },
                //     Err(_) => {},
                // }
                exit(0);
            }
            Some(Message::KeyInput(key)) => {
                if key == enums::Key::Enter {
                    sender.send(Message::OpenFile);
                } else if key == enums::Key::Down {
                    list_browser.select(list_browser.value() + 1);
                } else if key == enums::Key::Up {
                    list_browser.select(list_browser.value() - 1);
                }
            }

            None => {}
        }
    }
}
