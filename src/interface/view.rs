use glib::{Sender, Receiver};
use gtk::Label;
use gtk::{
    prelude::*,
    traits::{ButtonExt, ContainerExt, WidgetExt},
    Align, Application, ApplicationWindow, Button, Window, Builder,
};

use std::{env, println, thread};
use std::sync::{Arc, Mutex};


pub struct View {
    pub sender: Sender<(ViewObject, ViewObjectType)>,
    //pub receiver: Receiver<String>,
    //pub receiver: Arc<Mutex<Receiver<String>>>,
    //pub builder: Builder,
    pub window: Window
}

// CAMBIAR EXPECTS !!!!!!!!
impl View {

    pub fn new()-> Arc<Mutex<View>>{
        let title = format!("Nodo Bitcoin - Los Rustybandidos");
        let glade_src = include_str!("window.glade");
        let builder = Builder::from_string(glade_src);
        let window: Window = builder.object("window").expect("Error: No encuentra objeto 'window'");
        window.set_title(&title);

        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let view = Arc::new(Mutex::new(View {
            window,
            sender
        }));

        let view_clone = Arc::clone(&view);
        let view_result = view_clone.lock();
        if let Ok(view_guard) = view_result {
            view_guard.window.show_all();
            receiver.attach(None, move |(view_object, object_type)| {
                let text: String = view_object.get_text();
                let id = String::from(view_object.get_id());
                println!("RECEIVER {}, {}", view_object.id, view_object.text);

                match object_type {
                    ViewObjectType::Label(label) => {
                        let label: Label = builder.object(&id).expect("error");
                        label.set_text(&text);
                    }
                    ViewObjectType::Button(button) => {
                        let button: Button = builder.object(&id).expect("error");
                        button.set_label(&text);
                    }
                }
                glib::Continue(true)
            });
            view_guard.window.connect_delete_event(|_, _| {
                gtk::main_quit();
                Inhibit(false)
            });
        } else {
            eprintln!("Failed to acquire lock on interface");
        }
        view
    }
}

pub enum ViewObjectType {
    Label(Label),
    Button(Button)
}

pub struct ViewObject {
    pub id: String,
    pub text: String
}

impl ViewObject {
    pub fn get_id(&self) -> String{
        (self.id).clone()
    }

    pub fn get_text(&self) -> String{
        (self.text).clone()
    }
}