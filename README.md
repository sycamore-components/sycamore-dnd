# Sycamore DnD

A drag and drop library for sycamore

Adds the `create_draggable` and `create_droppable` functions that abstract the difficult
parts of drag and drop.
This library is still fairly low level and makes no assumptions about drop behaviour. This makes
it possible to do things like reading a file, but requires custom code for things like sortable
lists or other common drag and drop scenarios.

# Compatibility

This library currently requires the GitHub version of Sycamore because of the features it offers.
Once the `Attributes` change is released to `crates.io` this will no longer be necessary.

# Example Usage

```rust
#[component]
fn App<'cx, G: Html>(cx: Scope<'cx>) -> View<G> {
    let inside = create_signal(cx, false);

    let drop_inside = create_droppable(cx)
        .on_drop(move |_: ()| inside.set(true))
        .hovering_class("drag-over")
        .build();
    let drop_outside = create_droppable(cx)
        .on_drop(move |_: ()| inside.set(false))
        .hovering_class("drag-over")
        .build();
    let drag = create_draggable(cx)
        .dragging_class("dragging")
        .build();

    view! { cx,
        div(class = "container") {
            div(style = "min-height:100px;width:100%;", ref = drop_outside) {
                (if !*inside.get() {
                    view! { cx,
                        div(class = "item", ref = drag) {
                            "Drag me"
                        }
                    }
                } else {
                    View::empty()
                })
            }
            div(class="box", ref = drop_inside) {
                (if *inside.get() {
                    view! { cx,
                        div(class = "item", ref = drag) {
                            "Drag me"
                        }
                    }
                } else {
                    View::empty()
                })
            }
        }
    }
}
```
