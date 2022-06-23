// js functions we want to call from rust

function async_sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

function set_auth_cookie(value) {
    document.cookie = "PBSAuthCookie=" + value;
}

function clear_auth_cookie() {
    document.cookie = "PBSAuthCookie=; expires=Thu, 01-Jan-1970 00:00:01 GMT;";
}

function get_cookie() {
    return document.cookie;
}

function test_alert() {
    alert("This is a test!");
}

function create_popper(content, tooltip, opts) {

    let setSameWidth = function(data) {
	//console.log(data);
	let state = data.state;
	state.styles.popper["min-width"] = `${state.rects.reference.width}px`;
	return state;
    };

    // Always use sameWidth? Maybe we need to make this optional
    opts.modifiers.push({
            name: "sameWidth",
            enabled: true,
            phase: "beforeWrite",
            fn: setSameWidth,
            requires: ["computeStyles"],
    });

    return Popper.createPopper(content, tooltip, opts);
}

function update_popper(popper) {
    popper.update();
}

function show_modal_dialog(dialog) {
    dialog.showModal();
}

function close_dialog(dialog) {
    dialog.close();
}

export {
    async_sleep,
    get_cookie,
    set_auth_cookie,
    clear_auth_cookie,
    test_alert,
    create_popper,
    update_popper,
    show_modal_dialog,
    close_dialog,
};
