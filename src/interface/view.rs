use glib::{Sender};
use gtk::{
    prelude::*,
    traits::{ButtonExt, WidgetExt},
    Builder, Button, Label, Window, TextView, Spinner, Dialog, Entry, ResponseType,
};
use gtk::prelude::Continue;
use std::{cmp, thread, vec};

use std::{println};

use crate::wallet::user::Account;

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

    let builder_receiver_clone = builder.clone();
    receiver.attach(None, move |view_object: ViewObject| {
        match view_object {
            ViewObject::Label(data) => {
                let label: Label = builder_receiver_clone.object(&String::from(data.id)).expect("error");
                label.set_text(&data.text.to_string());
            }
            ViewObject::Button(data) => {
                println!(
                    "RECEIVER BUTTON {} {}",
                    String::from(&data.id),
                    data.text.to_string()
                );
                let button: Button = builder_receiver_clone.object(&String::from(data.id)).expect("error");
                button.set_label(&data.text.to_string());
            }
            ViewObject::Spinner(data) => {
                let button: Spinner = builder_receiver_clone.object(&String::from(data.id)).expect("error");
                button.set_active(data.active);
            }
            ViewObject::TextView(data) => {
                let text_view: TextView = builder_receiver_clone.object(&String::from(data.id)).expect("error");
                let buffer = text_view.buffer().unwrap();
                buffer.insert_at_cursor(&data.text.to_string());
            }
        }
        glib::Continue(true)
    });

    let builder_wallet_clone = builder.clone();
    let new_wallet_button: Button = builder.object("new_wallet_button").expect("Couldn't get open_modal_button");
    new_wallet_button.connect_clicked(move |_| {
        open_wallet_dialog(&builder_wallet_clone);
    });

    let builder_send_clone = builder.clone();
    let send_btc_button: Button = builder.object("send_btc_button").expect("Couldn't get open_modal_button");
    send_btc_button.connect_clicked(move |_| {
        println!("Create transaction");
    });

    window.connect_delete_event(|_, _| {
        // Corta ejecucion al cerrar la ventana, aca cerrariamos hilos
        gtk::main_quit();
        Inhibit(false)
    });

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

fn open_wallet_dialog(builder: &Builder) {
    let dialog: Dialog = builder.object("wallet_dialog").expect("Couldn't get wallet_dialog");

    let key_entry: Entry = builder.object("key").expect("Couldn't get key");
    let address_entry: Entry = builder.object("address").expect("Couldn't get address");
    let name_entry: Entry = builder.object("name").expect("Couldn't get address");

    dialog.connect_response(move |dialog, response_id| {
        println!("RESPONSE {}", response_id);

        match response_id {
            ResponseType::Ok => {
                let key = key_entry.text();
                let address = address_entry.text();
                let name = name_entry.text();

                gtk::glib::idle_add(move || {
                    // Test: creo hilo para asegurar que sigue el proceso en segundo plano.
                    //       Al cerrar ventana general, finaliza.
                    //       En este hilo habria que guardar los datos
                    let key_clone = key.clone();
                    let address_clone = address.clone();
                    let name_clone = name.clone();

                    let test_thread = thread::spawn(move || {
                        let new_account = Account::new(
                            key_clone.to_string(),
                            address_clone.to_string(),
                            name_clone.to_string(),
                        );
                        let accounts = vec![new_account];
                        Account::save_all_accounts(accounts);
                        let read = Account::get_all_accounts();
                        
                    });
                    /* if test_thread.is_finished(){println!("LIIIISTO user")};
                    test_thread.join(); */

                    Continue(false)
                });

                dialog.hide();
            }
            ResponseType::Close => dialog.hide(),
            _ => dialog.hide(),
        }
    });

    dialog.show_all();
    dialog.run();
}
