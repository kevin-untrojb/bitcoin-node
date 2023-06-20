use super::view::{ViewObject, ViewObjectStatus, ViewObjectData};
use glib::Sender;
use gtk::{
    prelude::*,
    traits::{ButtonExt, WidgetExt},
    Builder, Button, Dialog, Entry, Label, ResponseType, Spinner, Window,
};
use gtk::{CellRendererText, ComboBox, ListStore, MessageType};

// Activa spinner y muestra mensaje. Realizar llamada antes de hacer operacion y usar end_loading una vez que termine.
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


// Info message: app_manager.sender_frontend.send(ViewObject::Error(InterfaceError::enum));
// Error message: app_manager.sender_frontend.send(ViewObject::Message(InterfaceMessage::enum));
pub fn open_message_dialog(error: bool, builder: &Builder, message: String) {
    if let Some(dialog) = builder.object::<Dialog>("message_dialog") {
        dialog.show_all();

        if error {
            dialog.set_property("text", "Error");
            dialog.set_property("message-type", MessageType::Error);
        } else {
            dialog.set_property("text", "Information");
            dialog.set_property("message-type", MessageType::Info);
        }

        dialog.set_title("");
        dialog.set_property("secondary-text", &message);

        dialog.connect_response(move |dialog, response_id| match response_id {
            ResponseType::Close => dialog.hide(),
            _ => dialog.hide(),
        });

        dialog.run();
    }
}
