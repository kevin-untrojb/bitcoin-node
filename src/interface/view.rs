use glib::Sender;
use gtk::{
    prelude::*,
    traits::{ButtonExt, WidgetExt},
    Builder, Button, Dialog, Entry, Label, ResponseType, Spinner, Window,
};
use gtk::{CellRendererText, ComboBox, ListStore, MessageType};
use std::sync::{Arc, Mutex};
use std::{thread, vec};

use std::println;

use crate::errores::{InterfaceError, InterfaceMessage};
use crate::wallet::user::Account;
use crate::{
    app_manager::{self, ApplicationManager},
    blockchain::transaction::Transaction,
};

pub enum ViewObject {
    Label(ViewObjectData),
    Spinner(ViewObjectStatus),
    Error(InterfaceError),
    Message(InterfaceMessage),
    UploadTransactions(Vec<Transaction>),
    UploadAmounts((u64, u64)),
}

pub struct ViewObjectData {
    pub id: String,
    pub text: String,
}

pub struct ViewObjectStatus {
    pub id: String,
    pub active: bool,
}

pub fn create_view() -> Sender<ViewObject> {
    let title = "Nodo Bitcoin - Los Rustybandidos".to_string();
    let glade_src = include_str!("window.glade");

    let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    let app_manager = ApplicationManager::new(sender.clone());
    let app_manager_mutex = Arc::new(Mutex::new(app_manager));

    let builder = Builder::from_string(glade_src);
    let window: Window;
    if let Some(res) = builder.object("window") {
        window = res;
        window.set_title(&title);
        window.show_all();

        let manager_close_app = app_manager_mutex.clone();
        window.connect_delete_event(move |_, _| {
            close(manager_close_app.clone());
            gtk::main_quit();
            Inhibit(false)
        });
    };

    let manager_create_wallet: Arc<Mutex<ApplicationManager>> = app_manager_mutex.clone();
    create_combobox_wallet_list(&builder, manager_create_wallet);

    let builder_receiver_clone = builder.clone();
    receiver.attach(None, move |view_object: ViewObject| {
        match view_object {
            ViewObject::Label(data) => {
                if let Some(label) = builder_receiver_clone.object::<Label>(&String::from(data.id))
                {
                    label.set_text(&data.text.to_string());
                }
            }
            ViewObject::Spinner(data) => {
                if let Some(spinner) =
                    builder_receiver_clone.object::<Spinner>(&String::from(data.id))
                {
                    spinner.set_active(data.active);
                }
            }
            ViewObject::Error(error) => {
                open_message_dialog(true, &builder_receiver_clone, error.to_string());
            }
            ViewObject::Message(message) => {
                open_message_dialog(false, &builder_receiver_clone, message.to_string());
            }
            ViewObject::UploadTransactions(_) => {
                println!("Actualizar transactions");
            }
            ViewObject::UploadAmounts((available, pending)) => {
                if let Some(label) = builder_receiver_clone.object::<Label>("available") {
                    label.set_text(&available.to_string());
                }

                if let Some(label) = builder_receiver_clone.object::<Label>("pending") {
                    label.set_text(&pending.to_string());
                }
            }
        }
        glib::Continue(true)
    });

    let builder_wallet_clone = builder.clone();
    let manager_open_modal_wallet: Arc<Mutex<ApplicationManager>> = app_manager_mutex.clone();
    if let Some(dialog) = builder.object::<Dialog>("wallet_dialog") {
        let dialog_clone = dialog.clone();
        if let Some(new_wallet_button) = builder_wallet_clone.object::<Button>("new_wallet_button")
        {
            new_wallet_button.connect_clicked(move |_| {
                open_wallet_dialog(
                    &dialog_clone,
                    &builder_wallet_clone,
                    manager_open_modal_wallet.clone(),
                );
            });
        }
    }

    let manager_change_wallet: Arc<Mutex<ApplicationManager>> = app_manager_mutex.clone();
    let sender_clone = sender.clone();
    if let Some(combobox_wallet) = builder.object::<ComboBox>("combobox_wallet") {
        combobox_wallet.connect_changed(move |combobox| {
            if let Some(active_iter) = combobox.active_iter() {
                match combobox.model() {
                    Some(model) => {
                        let value: String = match model.value(&active_iter, 0).get() {
                            Ok(res) => res,
                            Err(_) => todo!(),
                        };
                        select_current_account(
                            manager_change_wallet.clone(),
                            value,
                            sender_clone.clone(),
                        );
                    }
                    None => todo!(),
                };
            }
        });
    }

    sender
}

fn select_current_account(
    app_manager: Arc<Mutex<ApplicationManager>>,
    value: String,
    sender: Sender<ViewObject>,
) {
    let mut app_manager_thread = match app_manager.lock() {
        Ok(res) => res,
        Err(_) => return,
    };
    if value == "None" {
        let txs_current_account = Vec::<Transaction>::new();
        let _ = sender.send(ViewObject::UploadTransactions(txs_current_account));
        let _ = sender.send(ViewObject::UploadAmounts((0, 0)));
    } else {
        &app_manager_thread.select_current_account(value);
    }
    drop(app_manager_thread);
}

fn close(app_manager: Arc<Mutex<ApplicationManager>>) {
    let mut app_manager_thread = match app_manager.lock() {
        Ok(res) => res,
        Err(_) => return,
    };
    &app_manager_thread.close();
    drop(app_manager_thread);
}

pub fn start_loading(sender: Sender<ViewObject>, text: String) {
    let id: String = "loading_message".to_string();

    let view_object_data = ViewObjectData { id, text };

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
        text: "".to_string(),
    };

    let view_object_status = ViewObjectStatus {
        id: "loading_spinner".to_string(),
        active: false,
    };

    let _ = sender.send(ViewObject::Spinner(view_object_status));
    let _ = sender.send(ViewObject::Label(view_object_data));
}

fn open_wallet_dialog(
    dialog: &Dialog,
    builder: &Builder,
    app_manager: Arc<Mutex<ApplicationManager>>,
) {
    let key_entry: Entry;
    if let Some(res) = builder.object::<Entry>("key") {
        key_entry = res;
    } else {
        return;
    }

    let address_entry: Entry;
    if let Some(res) = builder.object::<Entry>("address") {
        address_entry = res;
    } else {
        return;
    }

    let name_entry: Entry;
    if let Some(res) = builder.object::<Entry>("name") {
        name_entry = res;
    } else {
        return;
    }

    let builder_clone = builder.clone();

    dialog.connect_response(move |dialog, response_id| {
        match response_id {
            ResponseType::Ok => {
                let key = key_entry.text().to_string();
                let address = address_entry.text().to_string();
                let name = name_entry.text().to_string();
                if !key.is_empty() && !address.is_empty() && !name.is_empty() {
                    let mut app_manager_thread = match app_manager.lock() {
                        Ok(res) => res,
                        Err(_) => return,
                    };
                    let account = app_manager_thread.create_account(key, address, name);
                    add_wallet_combobox(&builder_clone, &account);
                    open_message_dialog(false, &builder_clone, "Cuenta creada".to_string());
                    drop(app_manager_thread);
                }
            }
            _ => dialog.hide(),
        }
        key_entry.set_text("");
        address_entry.set_text("");
        name_entry.set_text("");

        dialog.hide();
    });

    dialog.show_all();
    dialog.run();
}

fn create_combobox_wallet_list(builder: &Builder, app_manager: Arc<Mutex<ApplicationManager>>) {
    let mut app_manager_thread = match app_manager.lock() {
        Ok(res) => res,
        Err(_) => return,
    };
    let accounts = &app_manager_thread.accounts;

    let combobox_wallet: ComboBox;
    if let Some(res) = builder.object::<ComboBox>("combobox_wallet") {
        combobox_wallet = res;
    } else {
        return;
    };

    let list_store: ListStore;
    if let Some(res) = builder.object::<ListStore>("accounts") {
        list_store = res;
    } else {
        return;
    };

    for account in accounts {
        let name = &account.wallet_name as &dyn ToValue;
        list_store.insert_with_values(None, &[(0, name)]);
    }
    list_store.insert_with_values(Some(0 as u32), &[(0, &"None".to_string() as &dyn ToValue)]);

    let cell_renderer = CellRendererText::new();
    combobox_wallet.pack_start(&cell_renderer, true);
    combobox_wallet.add_attribute(&cell_renderer, "text", 0);
    combobox_wallet.set_active(Some(0));
    drop(app_manager_thread);
}

fn add_wallet_combobox(builder: &Builder, account: &Account) {
    let list_store: ListStore;
    if let Some(res) = builder.object::<ListStore>("accounts") {
        list_store = res;
    } else {
        return;
    };

    let name = &account.wallet_name as &dyn ToValue;
    list_store.insert_with_values(None, &[(0, name)]);
}

// Info message: app_manager.sender_frontend.send(ViewObject::Error(InterfaceError::enum));
// Error message: app_manager.sender_frontend.send(ViewObject::Message(InterfaceMessage::enum));
pub fn open_message_dialog(error: bool, builder: &Builder, message: String) {
    if let Some(dialog) = builder.object::<Dialog>("message_dialog") {
        dialog.show_all();

        if error {
            dialog.set_title("Error");
            dialog.set_property("message-type", MessageType::Error);
        } else {
            dialog.set_title("Information");
            dialog.set_property("message-type", MessageType::Info);
        }
        dialog.set_property("text", &message);

        dialog.connect_response(move |dialog, response_id| match response_id {
            ResponseType::Close => dialog.hide(),
            _ => dialog.hide(),
        });

        dialog.run();
    }
}
