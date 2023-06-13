use glib::Sender;
use gtk::{
    prelude::*,
    traits::{ButtonExt, WidgetExt},
    Builder, Button, Label, Window, TextView, Spinner, Dialog,
};

use std::{println};

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

pub fn create_view()-> Sender<ViewObject>{
    let title = "Nodo Bitcoin - Los Rustybandidos".to_string();
    let glade_src = include_str!("window.glade");

    let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    let builder = Builder::from_string(glade_src);
    let window: Window = builder
        .object("window")
        .expect("Error: No encuentra objeto 'window'");
    window.set_title(&title);
    window.show_all();

    receiver.attach(None, move |view_object: ViewObject| {
        match view_object {
            ViewObject::Label(data) => {
                /*println!(
                    "RECEIVER LABEL {} {}",
                    &String::from(&data.id),
                    &data.text.to_string()
                );*/
                let label: Label = builder.object(&String::from(data.id)).expect("error");
                label.set_text(&data.text.to_string());
            }
            ViewObject::Button(data) => {
                println!(
                    "RECEIVER BUTTON {} {}",
                    String::from(&data.id),
                    data.text.to_string()
                );
                let button: Button = builder.object(&String::from(data.id)).expect("error");
                button.set_label(&data.text.to_string());
            }
            ViewObject::Spinner(data) => {
                /*println!(
                    "RECEIVER SPINNER {} {}",
                    String::from(&data.id),
                    data.active.to_string()
                );*/
                let button: Spinner = builder.object(&String::from(data.id)).expect("error");
                button.set_active(data.active);
            }
            ViewObject::TextView(data) => {
                let text_view: TextView = builder.object(&String::from(data.id)).expect("error");
                let buffer = text_view.buffer().unwrap();
                buffer.insert_at_cursor(&data.text.to_string());
            }
        }
        glib::Continue(true)
    });

    /*let new_wallet_button: Button = builder.object("new_wallet_button").expect("Couldn't get open_modal_button");
    new_wallet_button.connect_clicked(move |_| {
        open_modal_dialog(&builder);
    });*/

    sender
}

pub fn start_loading(sender: Sender<ViewObject>, text: String) {
    let id: String = "loading_message".to_string();

    let view_object_data = ViewObjectData {
        id,
        text,
    };

    let view_object_status = ViewObjectStatus {
        id: "loading_spinner".to_string(),
        active: true,
    };

    let _ = sender.send(ViewObject::Spinner(view_object_status));
    let _ = sender.send(ViewObject::Label(view_object_data));
}

pub fn end_loading(sender: Sender<ViewObject>) {
    let id: String = "loading_message".to_string();

    let view_object_data = ViewObjectData {
        id,
        text: "".to_string()
    };

    let view_object_status = ViewObjectStatus {
        id: "loading_spinner".to_string(),
        active: false,
    };

    let _ = sender.send(ViewObject::Spinner(view_object_status));
    let _ = sender.send(ViewObject::Label(view_object_data));
}

/*fn open_modal_dialog(builder: &Builder) {
    let dialog: Dialog = builder.object("modal_dialog").expect("Couldn't get modal_dialog");
    
    let save_button: Button = builder.object("save_button_modal").expect("Couldn't get save_button_modal");
    save_button.connect_clicked(move |_| {
        save_button_clicked(dialog.clone());
        dialog.close();
    });
    
    dialog.run();
    dialog.close();
}*/

/*
    let view_object_data = ViewObjectData {
        id,
        text
    };
    let _ = sender.send(ViewObject::TextView(view_object_data));
*/