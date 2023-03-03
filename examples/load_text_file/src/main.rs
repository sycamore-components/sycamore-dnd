use serde::{Deserialize, Serialize};
use sycadrop::{create_droppable, RawTransfer};
use sycamore::prelude::*;
use wasm_bindgen_futures::JsFuture;

fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    sycamore::render(|cx| {
        view! { cx,
            p { "Hello, World!" }
            App()
        }
    });
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
struct ContentItem {
    id: i32,
    name: String,
}

#[component]
fn App<'cx, G: Html>(cx: Scope<'cx>) -> View<G> {
    let text = create_ref(cx, create_rc_signal("Drop here".to_string()));

    let drop = create_droppable(cx)
        .accept(|transfer: &RawTransfer| {
            let items = transfer.items();
            if items.length() == 1 && items.get(0).unwrap().kind() == "file" {
                true
            } else if let Some(files) = transfer.files() {
                files.length() == 1
            } else {
                false
            }
        })
        .on_drop({
            move |transfer: RawTransfer| {
                let file = transfer
                    .items()
                    .get(0)
                    .map(|item| item.get_as_file().unwrap().unwrap())
                    .or_else(|| transfer.files().unwrap().get(0))
                    .unwrap();
                let text = text.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let file_text = JsFuture::from(file.text())
                        .await
                        .unwrap()
                        .as_string()
                        .unwrap();
                    text.set(file_text);
                });
            }
        })
        .build();

    view! { cx,
        div(class = "container") {
            div(class="box", ref = drop) {
                (text.get())
            }
        }
    }
}
