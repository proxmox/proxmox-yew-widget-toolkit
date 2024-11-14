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

function hide_popover(popover) {
    popover.hidePopover();
}

function show_popover(popover) {
    popover.showPopover();
}

function toggle_popover(popover) {
    popover.togglePopover();
}

export {
    test_alert,
    show_dialog,
    show_modal_dialog,
    close_dialog,
    hide_popover,
    show_popover,
    toggle_popover,
};
