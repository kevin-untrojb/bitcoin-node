use glib::Sender;
use gtk::{
    prelude::*,
    traits::{ButtonExt, WidgetExt},
    Builder, Button, Label, Window,
};

use std::{println, thread};

pub enum ViewObject {
    Label(ViewObjectData),
    Button(ViewObjectData),
    Spinner(ViewObjectStatus),
    TextView(ViewObjectData),
}

pub struct ViewObjectData {
    pub id: String,
    pub text: String,
}

pub struct ViewObjectStatus {
    pub id: String,
    pub active: bool,
}

// CAMBIAR EXPECTS !!!!!!!!
/*
pub fn create_view()-> Sender<ViewObject>{
    let title = format!("Nodo Bitcoin - Los Rustybandidos");
    let glade_src = include_str!("window.glade");

    let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    thread::spawn(move || {
        let builder = Builder::from_string(glade_src);
        let window: Window = builder.object("window").expect("Error: No encuentra objeto 'window'");
        window.set_title(&title);
        window.show_all();

        receiver.attach(None, move |view_object: ViewObject| {
            match view_object {
                ViewObject::Label(data) => {
                    println!("RECEIVER LABEL {} {}", &String::from(&data.id), &data.text.to_string());
                    let label: Label = builder.object(&String::from(data.id)).expect("error");
                    label.set_text(&data.text.to_string());
                }
                ViewObject::Button(data) => {
                    println!("RECEIVER BUTTON {} {}", String::from(&data.id), data.text.to_string());
                    let button: Button = builder.object(&String::from(data.id)).expect("error");
                    button.set_label(&data.text.to_string());
                }
            }
            glib::Continue(true)
        });
    });

    sender
}
*/
