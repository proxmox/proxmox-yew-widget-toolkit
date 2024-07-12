// js functions we want to call from rust

function test_alert() {
    alert("This is a test!");
}

function show_modal_dialog(dialog) {
    dialog.showModal();
    dialog.setAttribute('aria-modal', 'true');
}

function show_dialog(dialog) {
    dialog.show();
    dialog.setAttribute('aria-modal', 'false');
}

function close_dialog(dialog) {
    dialog.close();
    dialog.removeAttribute('aria-modal');
}

export {
    test_alert,
    show_dialog,
    show_modal_dialog,
    close_dialog,
};
