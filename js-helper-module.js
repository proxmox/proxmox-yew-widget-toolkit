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

function uplot(opts, data, node) {
    return new uPlot(opts, data, node);
}

function uplot_set_data(uplot, data) {
    uplot.setData(data);
}

function uplot_set_size(uplot, width, height) {
    uplot.setSize({
	width: width,
	height: height,
    });
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

function render_server_epoch(epoch) {
    let tzoffset = new Date().getTimezoneOffset()*60000;
    let servertime = new Date((epoch * 1000) + tzoffset);
    return servertime.toString();
}

// epoch to "Y-m-d H:i:s" (localtime)
function render_timestamp(epoch) {
    let date = new Date((epoch * 1000));

    let Y = date.getFullYear();
    let m = (date.getMonth() + 1);
    if (m < 10) m = '0' + m;

    let d = date.getDate();
    if (d < 10) d = '0' + d;


    let h = date.getHours();
    if (h < 10) h = '0' + h;

    let i = date.getMinutes();
    if (i < 10) i = '0' + i;

    let s = date.getSeconds();
    if (s < 10) s = '0' + s;

    return Y + '-' + m + '-' + d + ' ' + h + ':' + i + ':' + s;
}

export {
    async_sleep,
    get_cookie,
    set_auth_cookie,
    clear_auth_cookie,
    test_alert,
    uplot,
    uplot_set_data,
    uplot_set_size,
    create_popper,
    update_popper,
    show_modal_dialog,
    close_dialog,
    render_server_epoch,
    render_timestamp,
};
