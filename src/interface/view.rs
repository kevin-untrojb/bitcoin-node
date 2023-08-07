use glib::Sender;
use gtk::{
    prelude::*,
    traits::{ButtonExt, WidgetExt},
    Box, Builder, Button, Dialog, Entry, Label, MenuItem, ProgressBar, ResponseType, Spinner,
    TreeView, Window,
};
use gtk::{CellRendererText, ComboBox, ListStore};
use std::sync::{Arc, Mutex};

use crate::{app_manager::ApplicationManager, config};
use crate::{common::utils_timestamp::timestamp_to_datetime, wallet::user::Account};
use crate::{
    errores::{InterfaceError, InterfaceMessage},
    wallet::uxto_set::TxReport,
};

use super::public::{open_message_dialog, start_loading};
use crate::common::uint256::Uint256;
use crate::merkle_tree::merkle_root::ProofOrder;

pub enum ViewObject {
    Label(ViewObjectData),
    Spinner(ViewObjectStatus),
    Error(InterfaceError),
    Message(InterfaceMessage),
    UploadTransactions(Vec<TxReport>),
    UploadAmounts((u64, i128)),
    _NewBlock(String),
    _NewTx(String),
    CloseApplication,
    UpdateButtonPoiStatus(String),
    UploadProgressBar((usize, usize, usize)),
    PoiResponse(Vec<(Uint256, ProofOrder)>),
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
    let nombre_grupo = match config::get_valor("NOMBRE_GRUPO".to_string()) {
        Ok(valor) => valor,
        Err(_) => "Los Rustybandidos".to_string(),
    };

    let title = "Nodo Bitcoin - ".to_string() + nombre_grupo.as_str();
    let glade_src = include_str!("window.glade");

    let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    let app_manager = ApplicationManager::new(sender.clone());
    let app_manager_mutex = Arc::new(Mutex::new(app_manager));

    let selected_tx = ViewObjectData {
        id: "poi".to_string(),
        text: "".to_string(),
    };
    let shared_tx = Arc::new(Mutex::new(selected_tx));

    let builder = Builder::from_string(glade_src);
    let window: Window;
    if let Some(res) = builder.object("window") {
        window = res;
        window.set_title(&title);
        _ = sender.send(ViewObject::UploadProgressBar((0, 0, 0)));
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
    let shared_tx_receiver = shared_tx.clone();
    let sender_clone = sender.clone();
    receiver.attach(None, move |view_object: ViewObject| {
        match view_object {
            ViewObject::Label(data) => {
                if let Some(label) = builder_receiver_clone.object::<Label>(&data.id) {
                    label.set_text(&data.text);
                }
            }
            ViewObject::Spinner(data) => {
                if let Some(spinner) = builder_receiver_clone.object::<Spinner>(&data.id) {
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
                upload_transactions_table(&builder_receiver_clone, transactions);
                let _ = sender_clone.send(ViewObject::UpdateButtonPoiStatus("".to_string()));
            }
            ViewObject::CloseApplication => {
                gtk::main_quit();
            }
            ViewObject::UploadAmounts((available, pending)) => {
                let mut total: i128 = 0;
                if let Some(label) = builder_receiver_clone.object::<Label>("available") {
                    total += available as i128;
                    let btc_available = satoshis_u64_to_btc_string(available);
                    label.set_text(&btc_available);
                }

                if let Some(label) = builder_receiver_clone.object::<Label>("pending") {
                    total += pending;
                    let btc_pending = satoshis_i128_to_btc_string(pending);
                    label.set_text(&btc_pending);
                }

                if let Some(label) = builder_receiver_clone.object::<Label>("total") {
                    let btc_total = satoshis_i128_to_btc_string(total);
                    label.set_text(&btc_total);
                }
            }
            ViewObject::_NewBlock(_message) => {
                //open_message_dialog(false, &builder_receiver_clone, message);
            }
            ViewObject::_NewTx(_message) => {
                //open_message_dialog(false, &builder_receiver_clone, message);
            }
            ViewObject::UpdateButtonPoiStatus(tx_id) => {
                if let Some(button) = builder_receiver_clone.object::<Button>("poi") {
                    if !tx_id.is_empty() {
                        button.set_sensitive(true);
                        if shared_tx_receiver.lock().is_ok() {
                            let mut shared_tx_guard = shared_tx_receiver.lock().unwrap();
                            shared_tx_guard.text = tx_id;
                            drop(shared_tx_guard)
                        }
                    } else {
                        button.set_sensitive(false)
                    }
                }
            }
            ViewObject::UploadProgressBar((n_headers, n_blocks, n_saved_blocks)) => {
                if let Some(progress_section) =
                    builder_receiver_clone.object::<Box>("progress_section")
                {
                    if let Some(label) = builder_receiver_clone.object::<Label>("total_headers") {
                        label.set_text(&n_headers.to_string());
                    }
                    if n_blocks == 0 && n_saved_blocks == 0 {
                        progress_section.hide();
                    } else {
                        if let Some(label) = builder_receiver_clone.object::<Label>("total_blocks")
                        {
                            label.set_text(&n_blocks.to_string());
                        }
                        if let Some(label) = builder_receiver_clone.object::<Label>("saved_blocks")
                        {
                            label.set_text(&n_saved_blocks.to_string());
                        }
                        if let Some(progress_bar) =
                            builder_receiver_clone.object::<ProgressBar>("progress_bar")
                        {
                            let porcentaje = (n_saved_blocks as f64 / n_blocks as f64) * 100.0;
                            progress_bar.set_fraction(porcentaje / 100.0);
                        }
                        progress_section.show();
                    }
                }
            }
            ViewObject::PoiResponse(res) => {
                let concatenated_text: String = res
                    .iter()
                    .map(|(hash, direction)| {
                        let s_hash = hash.to_hexa_le_string();

                        format!("{:?} ->> {:?}", direction, s_hash)
                    })
                    .collect::<Vec<String>>()
                    .join("\n");
                open_path_dialog(&builder_receiver_clone, concatenated_text);
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

    let sender_row_transaction_clone = sender.clone();
    handle_row_transaction_selected(sender_row_transaction_clone, builder.clone());

    let manager_poi: Arc<Mutex<ApplicationManager>> = app_manager_mutex;
    let shared_tx_handler = shared_tx;
    handle_poi(manager_poi, builder, shared_tx_handler);

    sender
}

fn handle_row_transaction_selected(sender: Sender<ViewObject>, builder: Builder) {
    let tree_view: TreeView;
    if let Some(res) = builder.object::<TreeView>("transactions_tree_view") {
        tree_view = res;
        tree_view.connect_cursor_changed(move |tree_view| {
            if let Some((model, iter)) = tree_view.selection().selected() {
                let tx_id = match model.value(&iter, 2).get::<String>() {
                    Ok(value) => value,
                    Err(_) => "Error al obtener tx id de selected row".to_string(),
                };
                let _ = sender.send(ViewObject::UpdateButtonPoiStatus(tx_id));
            }
        });
    };
}

fn handle_poi(
    manager_poi: Arc<Mutex<ApplicationManager>>,
    builder: Builder,
    shared_tx: Arc<Mutex<ViewObjectData>>,
) {
    if let Some(res) = builder.object::<Dialog>("poi_dialog") {
        let dialog = res;
        if let Some(button) = builder.object::<Button>("poi") {
            button.connect_clicked(move |_| {
                if shared_tx.lock().is_ok() {
                    let shared_tx_guard = shared_tx.lock().unwrap();
                    let tx_id = shared_tx_guard.text.clone();
                    drop(shared_tx_guard);
                    open_poi_dialog(&dialog, &builder, manager_poi.clone(), tx_id);
                }
            });
        }
    }
}

fn open_poi_dialog(
    dialog: &Dialog,
    builder: &Builder,
    app_manager: Arc<Mutex<ApplicationManager>>,
    tx_id: String,
) {
    let hash_entry: Entry;
    if let Some(res) = builder.object::<Entry>("block_hash") {
        hash_entry = res;
    } else {
        return;
    }

    let tx_entry: Entry;
    if let Some(res) = builder.object::<Entry>("tx_poi") {
        tx_entry = res;
    } else {
        return;
    }

    tx_entry.set_text(&tx_id);

    dialog.connect_response(move |dialog, response_id| {
        match response_id {
            ResponseType::Ok => {
                let hash = hash_entry.text().to_string();
                let tx = tx_entry.text().to_string();
                let mut app_manager_thread = match app_manager.lock() {
                    Ok(res) => res,
                    Err(_) => return,
                };
                if !hash.is_empty() && !tx.is_empty() && hash.len() == 64 && tx.len() == 64 {
                    let hash_bytes: Vec<u8> = hash
                        .as_bytes()
                        .chunks(2)
                        .map(|chunk| {
                            let chunk_str = String::from_utf8_lossy(chunk);
                            u8::from_str_radix(&chunk_str, 16).unwrap_or(0)
                        })
                        .collect();

                    let tx_id_vec: Vec<u8> = tx
                        .as_bytes()
                        .chunks(2)
                        .map(|chunk| {
                            let chunk_str = String::from_utf8_lossy(chunk);
                            u8::from_str_radix(&chunk_str, 16).unwrap_or(0)
                        })
                        .collect();

                    let mut tx_id_bytes = [0u8; 32];
                    tx_id_bytes.copy_from_slice(tx_id_vec.as_slice());

                    let _ =
                        &app_manager_thread.proof_of_inclusion_from_front(hash_bytes, tx_id_bytes);
                }
                drop(app_manager_thread);
            }
            _ => dialog.hide(),
        }
        hash_entry.set_text("");
        dialog.hide();
    });

    dialog.show_all();
    dialog.run();
}

fn open_path_dialog(builder: &Builder, path: String) {
    let label;
    if let Some(res) = builder.object::<Label>("path") {
        label = res;
    } else {
        return;
    }

    label.set_text(&path);

    let dialog;
    if let Some(res) = builder.object::<Dialog>("merkle_path_dialog") {
        dialog = res;

        dialog.connect_response(move |dialog, response_id| {
            match response_id {
                _ => dialog.hide(),
            }
            label.set_text(&"");

            dialog.hide();
        });

        dialog.show_all();
        dialog.run();
    }
}

fn satoshis_u64_to_btc_string(satoshis: u64) -> String {
    let btc = satoshis as f64 / 100_000_000.0;
    format!("{:.8} BTC", btc)
}

fn satoshis_i128_to_btc_string(satoshis: i128) -> String {
    let btc = satoshis as f64 / 100_000_000.0;
    format!("{:.8} BTC", btc)
}

fn handle_modal_wallet(
    manager_open_modal_wallet: Arc<Mutex<ApplicationManager>>,
    builder: Builder,
) {
    if let Some(dialog) = builder.object::<Dialog>("wallet_dialog") {
        let dialog_clone = dialog;
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

    let txs_current_account = Vec::<TxReport>::new();
    let _ = sender.send(ViewObject::UploadTransactions(txs_current_account));
    let _ = sender.send(ViewObject::UploadAmounts((0, 0)));

    if value != "None" {
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
    list_store.insert_with_values(Some(0_u32), &[(0, &"None".to_string() as &dyn ToValue)]);

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

fn upload_transactions_table(builder: &Builder, transactions: Vec<TxReport>) {
    let list_store: ListStore;
    if let Some(res) = builder.object::<ListStore>("transactions") {
        list_store = res;
    } else {
        return;
    };
    list_store.clear();

    for transaction in transactions {
        let status = if transaction.is_pending {
            "Pending".to_string()
        } else {
            "Confirmed".to_string()
        };
        let is_pending = &status as &dyn ToValue;
        let date = &timestamp_to_datetime(transaction.timestamp as i64).to_string() as &dyn ToValue;
        let tx_id = &transaction.tx_id.to_hexa_le_string() as &dyn ToValue;
        let amount = &(transaction.amount as i64) as &dyn ToValue;

        list_store.insert_with_values(None, &[(0, is_pending), (1, date), (2, tx_id), (3, amount)]);
    }
}
