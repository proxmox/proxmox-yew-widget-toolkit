// js functions we want to call from rust

function test_alert() {
    alert("This is a test!");
}

function show_modal_dialog(dialog) {
    dialog.showModal();
}

function close_dialog(dialog) {
    dialog.close();
}

export {
    test_alert,
    show_modal_dialog,
    close_dialog,
};
