// js functions we want to call from rust

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
    test_alert,
    create_popper,
    update_popper,
    show_modal_dialog,
    close_dialog,
};
