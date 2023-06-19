use glib::Sender;
use gtk::prelude::Continue;
use gtk::{
    prelude::*,
    traits::{ButtonExt, WidgetExt},
    Builder, Button, Dialog, Entry, Label, ResponseType, Spinner, Window,
};
use gtk::{CellRendererText, ComboBox, ListStore};
use std::sync::{Arc, Mutex};
use std::{thread, vec};

use std::println;

use crate::app_manager::{self, ApplicationManager};
use crate::errores::NodoBitcoinError;
use crate::wallet::user::Account;

pub enum ViewObject {
    Label(ViewObjectData),
    Spinner(ViewObjectStatus),
    NewAccount(Account),
    ErrorPopup(NodoBitcoinError),
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

    let builder = Builder::from_string(glade_src);
    let window: Window;
    if let Some(res) = builder.object("window") {
        window = res;
        window.set_title(&title);
        window.show_all();

        let app_manager_clone = app_manager.clone();
        window.connect_delete_event(move |_, _| {
            app_manager_clone.close();
            gtk::main_quit();
            Inhibit(false)
        });
    };

    create_combobox_wallet_list(&builder, &app_manager.accounts);

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
            ViewObject::NewAccount(account) => {
                add_wallet_combobox(&builder_receiver_clone, &account)
            }
            ViewObject::ErrorPopup(error) => {
                // TODO: popup con el error
            }
        }
        glib::Continue(true)
    });

    /*let builder_send_clone = builder.clone();
    let send_btc_button: Button = builder
        .object("send_btc_button")
        .expect("Couldn't get open_modal_button");
    send_btc_button.connect_clicked(move |_| {
        println!("Create transaction");
    });*/

    let builder_wallet_clone = builder.clone();
    if let Some(dialog) = builder.object::<Dialog>("wallet_dialog") {
        let dialog_clone = dialog.clone();
        if let Some(new_wallet_button) = builder_wallet_clone.object::<Button>("new_wallet_button")
        {
            new_wallet_button.connect_clicked(move |_| {
                open_wallet_dialog(&dialog_clone, &builder_wallet_clone, app_manager.clone());
            });
        }
    }

    if let Some(combobox_wallet) = builder.object::<ComboBox>("combobox_wallet") {
        combobox_wallet.connect_changed(|combobox| {
            if let Some(active_iter) = combobox.active_iter() {
                match combobox.model() {
                    Some(model) => {
                        let value: String = match model.value(&active_iter, 0).get() {
                            Ok(res) => res,
                            Err(_) => todo!(),
                        };
                        //app_manager.clone().select_current_account(value);
                        println!("Opción seleccionada: {}", value);
                    }
                    None => todo!(),
                };
            }
        });
    }

    sender
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

fn open_wallet_dialog(dialog: &Dialog, builder: &Builder, mut app_manager: ApplicationManager) {
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

    let app_manager_mutex = Arc::new(Mutex::new(app_manager));
    dialog.connect_response(move |dialog, response_id| match response_id {
        ResponseType::Ok => {
            let app_manager_clone = app_manager_mutex.clone();

            let key = key_entry.text().to_string();
            let address = address_entry.text().to_string();
            let name = name_entry.text().to_string();
            if !key.is_empty() && !address.is_empty() && !name.is_empty() {
                let mut app_manager_thread = match app_manager_clone.lock() {
                    Ok(res) => res,
                    Err(_) => return,
                };
                let account_added = app_manager_thread.create_account(key, address, name);
                drop(app_manager_thread);
                if account_added.is_ok() {
                    // mostrar popup de cuenta creada ok
                } else {
                    // mostrar popup de cuenta no creada
                }
            }
            key_entry.set_text("");
            address_entry.set_text("");
            name_entry.set_text("");

            dialog.hide();
        }
        ResponseType::Close => dialog.hide(),
        _ => dialog.hide(),
    });

    dialog.show_all();
    dialog.run();
}

fn create_combobox_wallet_list(builder: &Builder, accounts: &Vec<Account>) {
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
