use glib::Sender;
use gtk::{
    prelude::*,
    traits::{ButtonExt, WidgetExt},
    Builder, Button, Dialog, Entry, Label, MenuItem, ResponseType, Spinner, Window,
};
use gtk::{CellRendererText, ComboBox, ListStore};
use std::{sync::{Arc, Mutex}, time::Duration, thread};

use std::println;

use crate::wallet::user::Account;
use crate::{app_manager::ApplicationManager, blockchain::transaction::Transaction};
use crate::{
    errores::{InterfaceError, InterfaceMessage},
    wallet::uxto_set::TxReport,
};

use super::public::{open_message_dialog, start_loading, end_loading};

pub enum ViewObject {
    Label(ViewObjectData),
    Spinner(ViewObjectStatus),
    Error(InterfaceError),
    Message(InterfaceMessage),
    UploadTransactions(Vec<TxReport>),
    UploadAmounts((u64, u64)),
    NewBlock(String),
    NewTx(String),
    CloseApplication,
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

        let sender_clone = sender.clone();
        let manager_close_app = app_manager_mutex.clone();
        window.connect_delete_event(move |_, _| {
            start_loading(sender_clone.clone(), "Cerrando aplicación...".to_string());
            close(manager_close_app.clone());
            //gtk::main_quit();
            Inhibit(true)
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
            ViewObject::UploadTransactions(transactions) => {
                println!("Actualizar transactions {}", transactions.len());
            }
            ViewObject::CloseApplication => {
                gtk::main_quit();
            }
            ViewObject::UploadAmounts((available, pending)) => {
                let mut total: u64 = 0;
                if let Some(label) = builder_receiver_clone.object::<Label>("available") {
                    total += available;
                    let btc_available = satoshis_to_btc_string(available);
                    label.set_text(&btc_available);
                }

                if let Some(label) = builder_receiver_clone.object::<Label>("pending") {
                    total += pending;
                    let btc_pending = satoshis_to_btc_string(pending);
                    label.set_text(&btc_pending);
                }

                if let Some(label) = builder_receiver_clone.object::<Label>("total") {
                    let btc_total = satoshis_to_btc_string(total);
                    label.set_text(&btc_total);
                }
            }
            ViewObject::NewBlock(message) => {
                //open_message_dialog(false, &builder_receiver_clone, message);
            }
            ViewObject::NewTx(message) => {
                //open_message_dialog(false, &builder_receiver_clone, message);
            }
        }
        glib::Continue(true)
    });

    let manager_open_modal_wallet: Arc<Mutex<ApplicationManager>> = app_manager_mutex.clone();
    handle_modal_wallet(manager_open_modal_wallet, builder.clone());

    let manager_change_wallet: Arc<Mutex<ApplicationManager>> = app_manager_mutex.clone();
    let sender_combobox_clone = sender.clone();
    handle_combobox(
        manager_change_wallet,
        sender_combobox_clone,
        builder.clone(),
    );

    let manager_transaction: Arc<Mutex<ApplicationManager>> = app_manager_mutex.clone();
    let sender_transaction_clone = sender.clone();
    handle_transaction(
        manager_transaction,
        sender_transaction_clone,
        builder.clone(),
    );

    handle_modal_about(builder.clone());

    sender
}

fn satoshis_to_btc_string(satoshis: u64) -> String {
    let btc = satoshis as f64 / 100_000_000.0;
    format!("{:.8} BTC", btc)
}

fn handle_modal_wallet(
    manager_open_modal_wallet: Arc<Mutex<ApplicationManager>>,
    builder: Builder,
) {
    if let Some(dialog) = builder.object::<Dialog>("wallet_dialog") {
        let dialog_clone = dialog.clone();
        if let Some(new_wallet_button) = builder.object::<Button>("new_wallet_button") {
            new_wallet_button.connect_clicked(move |_| {
                open_wallet_dialog(&dialog_clone, &builder, manager_open_modal_wallet.clone());
            });
        }
    }
}

fn handle_combobox(
    manager_change_wallet: Arc<Mutex<ApplicationManager>>,
    sender: Sender<ViewObject>,
    builder: Builder,
) {
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
                            sender.clone(),
                        );
                    }
                    None => todo!(),
                };
            }
        });
    }
}

fn handle_transaction(
    manager_transaction: Arc<Mutex<ApplicationManager>>,
    sender: Sender<ViewObject>,
    builder: Builder,
) {
    if let Some(send_transaction_button) = builder.object::<Button>("send_transaction_button") {
        send_transaction_button.connect_clicked(move |_| {
            send_transaction(manager_transaction.clone(), builder.clone(), sender.clone());
        });
    }
}

fn handle_modal_about(builder: Builder) {
    if let Some(about_item_menu) = builder.object::<MenuItem>("about_item_menu") {
        about_item_menu.connect_activate(move |_| {
            if let Some(dialog) = builder.object::<Dialog>("about_dialog") {
                dialog.show_all();

                dialog.connect_response(move |dialog, response_id| match response_id {
                    ResponseType::Close => dialog.hide(),
                    _ => dialog.hide(),
                });

                dialog.run();
            }
        });
    }
}

fn send_transaction(
    app_manager: Arc<Mutex<ApplicationManager>>,
    builder: Builder,
    sender: Sender<ViewObject>,
) {
    let to_address_entry: Entry;
    if let Some(res) = builder.object::<Entry>("to_address") {
        to_address_entry = res;
    } else {
        return;
    }

    let transaction_amount_entry: Entry;
    if let Some(res) = builder.object::<Entry>("transaction_amount") {
        transaction_amount_entry = res;
    } else {
        return;
    }

    let transaction_fee_entry: Entry;
    if let Some(res) = builder.object::<Entry>("transaction_fee") {
        transaction_fee_entry = res;
    } else {
        return;
    }

    let to_address = to_address_entry.text().to_string();
    let transaction_amount = transaction_amount_entry.text().to_string();
    let transaction_fee = transaction_fee_entry.text().to_string();
    if to_address.is_empty() || transaction_amount.is_empty() || transaction_fee.is_empty() {
        let _ = sender.send(ViewObject::Error(InterfaceError::EmptyFields));
    } else {
        println!(
            "DATA {} {} {}",
            to_address, transaction_amount, transaction_fee
        );
        let app_manager_thread = match app_manager.lock() {
            Ok(res) => res,
            Err(_) => return,
        };
        let _ =
            &app_manager_thread.send_transaction(to_address, transaction_amount, transaction_fee);
        to_address_entry.set_text("");
        transaction_amount_entry.set_text("");
        transaction_fee_entry.set_text("");
        drop(app_manager_thread);
    }
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
        let txs_current_account = Vec::<TxReport>::new();
        let _ = sender.send(ViewObject::UploadTransactions(txs_current_account));
        let _ = sender.send(ViewObject::UploadAmounts((0, 0)));
    } else {
        let _ = &app_manager_thread.select_current_account(value);
    }
    drop(app_manager_thread);
}

fn close(app_manager: Arc<Mutex<ApplicationManager>>) {
    let app_manager_thread = match app_manager.lock() {
        Ok(res) => res,
        Err(_) => return,
    };
    let _ = &app_manager_thread.close();
    drop(app_manager_thread);
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
    let app_manager_thread = match app_manager.lock() {
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
