use super::*;

use crate::props::FieldBuilder;
use crate::widget::form::number::{NumberField, NumberTypeInfo};
use crate::widget::form::{ManagedField, Number};

fn options(disabled: bool) -> FieldOptions {
    FieldOptions {
        submit: true,
        submit_empty: true,
        required: true,
        disabled,
    }
}

fn invalid() -> Option<SubmitValidateFn<Value>> {
    Some(SubmitValidateFn::new(|_| Err(anyhow::anyhow!("invalid"))))
}

fn register(form: &FormContext, name: &str, value: Value) -> FieldHandle {
    form.register_field(
        name.to_string(),
        value.clone(),
        value,
        false,
        None,
        options(false),
        false,
    )
}

fn number_validator<T: NumberTypeInfo>(props: &Number<T>) -> Option<SubmitValidateFn<Value>> {
    let args = NumberField::<T>::validation_args(props);
    Some(SubmitValidateFn::new(move |value| {
        NumberField::<T>::validator(&args, value)
    }))
}

fn number<T: NumberTypeInfo>(form: &FormContext, props: &Number<T>) -> FieldHandle {
    // These are the representations used by NumberField::create, not a string-only validator.
    let value = props
        .default
        .map(|n| json!(n.format()))
        .unwrap_or(Value::Null);
    let default = props
        .default
        .map(|n| n.number_to_value())
        .unwrap_or(Value::Null);
    let mut field = form.register_field(
        "number",
        value,
        default,
        false,
        number_validator(props),
        options(false),
        false,
    );
    field.set_input_normalizer(NumberField::<T>::input_normalizer().unwrap());
    field
}

#[test]
fn invalid_numeric_default_is_not_an_edit() {
    let form = FormContext::new();
    let _field = form.register_field(
        "number",
        json!("1"),
        json!(1),
        false,
        invalid(),
        options(false),
        false,
    );
    assert!(form.read().is_dirty());
    assert!(!form.read().is_modified());
}

#[test]
fn a_validation_change_is_not_an_edit() {
    let form = FormContext::new();
    let mut field = form.register_field(
        "number",
        json!("1"),
        json!(1),
        false,
        Some(SubmitValidateFn::new(|value: &Value| {
            Ok(json!(value.as_str().unwrap().parse::<u64>()?))
        })),
        options(false),
        false,
    );
    assert!(!form.read().is_modified());
    field.update_validate(invalid());
    assert!(form.read().is_dirty());
    assert!(!form.read().is_modified());
}

#[test]
fn clearing_required_number_restores_empty_default() {
    let form = FormContext::new();
    let mut field = form.register_field(
        "number",
        Value::Null,
        Value::Null,
        false,
        Some(SubmitValidateFn::new(|value: &Value| {
            Ok(json!(value.as_str().unwrap_or("").parse::<u64>()?))
        })),
        options(false),
        false,
    );
    assert!(!form.read().is_modified());
    field.set_value(json!("1"));
    assert!(form.read().is_modified());
    field.set_value(json!(""));
    assert!(!form.read().is_modified());
}

#[test]
fn disabled_radio_group_is_not_a_modified_enabled_field() {
    let form = FormContext::new();
    let mut radio = form.register_field(
        "choice",
        json!("one"),
        json!("one"),
        true,
        None,
        options(false),
        false,
    );
    radio.set_value(json!("two"));
    assert!(form.read().is_modified());
    radio.update_field_options(options(true));
    assert!(!form.read().is_modified());
    assert!(form.read().is_dirty()); // The existing dirty predicate includes disabled radios.
    assert_eq!(form.get_submit_data(), json!({"choice": "two"}));
}

#[test]
fn number_normalization_is_independent_of_constraints() {
    let form = FormContext::new();
    let props = Number::<i64>::new().default(1).required(true).min(2);
    let mut field = number(&form, &props);
    assert_eq!(field.get_value(), json!("1"));
    assert!(field.get_valid().is_err());
    assert!(!form.read().is_modified());
    for value in [json!(1), json!("01"), json!("+1")] {
        field.set_value(value);
        assert!(!form.read().is_modified());
        assert!(form.read().is_dirty());
    }
    field.set_value(json!("-1"));
    assert!(form.read().is_modified());
    field.reset();
    assert_eq!(field.get_value(), json!(1));
    assert!(!form.read().is_modified());
    assert!(!form.read().is_valid());
    assert_eq!(form.get_submit_data(), json!({}));
}

#[test]
fn number_validation_changes_preserve_both_pristine_and_edited_state() {
    let form = FormContext::new();
    let props = Number::<i64>::new().default(1);
    let mut field = number(&form, &props);
    field.update_validate(number_validator(&props.clone().min(2)));
    assert!(!form.read().is_modified());
    field.set_value(json!("2"));
    assert!(form.read().is_modified());
    field.update_validate(number_validator(&props.clone().min(3)));
    assert!(form.read().is_modified());
    field.update_validate(number_validator(&props));
    assert!(form.read().is_modified());
    field.set_value(json!("01"));
    assert!(!form.read().is_modified());
    assert_eq!(form.get_submit_data(), json!({"number": 1}));
    assert!(!form.read().is_dirty());
}

#[test]
fn required_and_optional_number_empty_input() {
    for required in [false, true] {
        let form = FormContext::new();
        let mut field = number(&form, &Number::<u64>::new().required(required));
        assert!(!form.read().is_modified());
        field.set_value(json!("1"));
        assert!(form.read().is_modified());
        field.set_value(json!(""));
        assert!(!form.read().is_modified());
        assert_eq!(form.read().is_dirty(), required);
        assert_eq!(form.read().is_valid(), !required);
        field.reset();
        assert_eq!(field.get_value(), Value::Null);
        assert!(!form.read().is_modified());
    }
}

#[test]
fn loaded_number_defaults_keep_both_representations() {
    let form = FormContext::new();
    let mut field = number(&form, &Number::<i64>::new().min(2));
    for default in [json!(1), json!("01")] {
        form.load_form(json!({"number": default}));
        assert!(!form.read().is_modified());
        assert!(form.read().is_dirty());
        field.set_value(json!("1"));
        assert!(!form.read().is_modified());
        field.set_value(json!("3"));
        assert!(form.read().is_modified());
        form.write().reset_form();
        assert_eq!(field.get_value(), default);
        assert!(!form.read().is_modified());
    }
    form.load_form(json!({"number": "003"}));
    assert!(!form.read().is_modified());
    assert!(form.read().is_dirty()); // load_form keeps the raw reset default for is_dirty.
    field.set_value(json!(3));
    assert!(!form.read().is_modified());
    assert_eq!(form.get_submit_data(), json!({"number": 3}));
}

#[test]
fn number_defaults_can_be_replaced_without_accepting_edits() {
    let form = FormContext::new();
    let mut field = number(&form, &Number::<i64>::new().default(1).min(3));
    field.set_value(json!("2"));
    assert!(form.read().is_modified());
    field.set_default(json!(1));
    assert!(form.read().is_modified());
    field.set_default(json!(2));
    assert!(!form.read().is_modified());
    field.set_default(json!(4));
    assert!(form.read().is_modified());
    assert_eq!(field.get_value(), json!("2"));
    field.reset();
    assert_eq!(field.get_value(), json!(4));
    assert!(!form.read().is_modified());
    assert!(!form.read().is_dirty());
    assert_eq!(form.get_submit_data(), json!({"number": 4}));
}

#[test]
fn number_parse_errors_remain_distinct() {
    let form = FormContext::new();
    let mut field = number(&form, &Number::<u8>::new());
    form.load_form(json!({"number": "256"}));
    assert!(!form.read().is_modified());
    field.set_value(json!("0256"));
    assert!(form.read().is_modified());
    field.set_value(json!("256"));
    assert!(!form.read().is_modified());
    field.set_value(json!("-1"));
    assert!(form.read().is_modified());
    field.set_value(json!("unfinished"));
    assert!(form.read().is_modified());
}

#[test]
fn unsigned_number_normalization_does_not_lose_precision() {
    let form = FormContext::new();
    let mut field = number(&form, &Number::<u64>::new().default(u64::MAX));
    field.set_value(json!(u64::MAX));
    assert!(!form.read().is_modified());
    field.set_value(json!((u64::MAX - 1).to_string()));
    assert!(form.read().is_modified());
    field.set_value(json!(format!("0{}", u64::MAX)));
    assert!(!form.read().is_modified());
}

#[test]
fn normalized_submit_values_and_validation_updates_are_separate() {
    let form = FormContext::new();
    let mut field = form.register_field(
        "text",
        json!(" default "),
        json!("default"),
        false,
        Some(SubmitValidateFn::new(|value: &Value| {
            Ok(json!(value.as_str().unwrap().trim()))
        })),
        options(false),
        false,
    );
    field.set_value(json!("default  "));
    assert!(!form.read().is_modified());
    field.update_validate(invalid());
    assert!(!form.read().is_modified());
    field.validate();
    assert!(!form.read().is_modified());
    form.write().set_field_valid("text", Ok(json!("different")));
    assert!(!form.read().is_modified());
    field.set_value(json!("edit"));
    assert!(form.read().is_modified());
    form.write().set_field_valid("text", Ok(json!("default")));
    assert!(form.read().is_modified());
    field.set_value(json!("edit")); // A repeated value is not a new input change.
    assert!(form.read().is_modified());
    field.set_value(json!(" default "));
    assert!(!form.read().is_modified());
}

#[test]
fn reinstalling_input_equivalence_does_not_reconsider_validation() {
    let form = FormContext::new();
    let mut field = form.register_field(
        "text",
        json!("default"),
        json!("default"),
        false,
        Some(SubmitValidateFn::new(|value: &Value| {
            Ok(json!(value.as_str().unwrap().trim()))
        })),
        options(false),
        true,
    );
    field.set_value(json!(" default "));
    assert!(!form.read().is_modified());
    field.update_validate(invalid());
    field.set_input_normalizer(Callback::from(|value| value));
    assert!(!form.read().is_modified());
    assert!(form.read().is_dirty());
}

#[test]
fn loading_current_input_accepts_edits_without_revalidation() {
    let form = FormContext::new();
    let mut field = number(&form, &Number::<i64>::new().default(1));
    field.set_value(json!("2"));
    form.write()
        .set_field_valid("number", Err("external error".into()));
    assert!(form.read().is_modified());
    form.load_form(json!({"number": "2"}));
    assert!(!form.read().is_modified());
    assert_eq!(field.get_valid(), Err("external error".into()));
    assert!(form.read().is_dirty());
    assert_eq!(form.read().dirty_count(), 1);
    assert_eq!(form.get_submit_data(), json!({}));
}

#[test]
fn registration_accepts_input_even_when_constructor_default_differs() {
    for validator in [None, invalid()] {
        let form = FormContext::new();
        let mut field = form.register_field(
            "text",
            json!("initial"),
            json!("default"),
            false,
            validator,
            options(false),
            false,
        );
        assert!(!form.read().is_modified());
        field.set_value(json!("edit"));
        assert!(form.read().is_modified());
        field.reset();
        assert!(!form.read().is_modified());
    }
}

#[test]
fn scalar_empty_equivalence_is_explicit_and_overridable() {
    let form = FormContext::new();
    let mut field = register(&form, "value", Value::Null);
    field.set_value(json!(""));
    assert!(!form.read().is_modified());
    for value in [json!([]), json!({}), json!(false), json!(0), json!("0")] {
        field.set_value(value);
        assert!(form.read().is_modified());
    }
    field.set_input_normalizer(Callback::from(|value| value));
    field.set_value(json!(""));
    assert!(form.read().is_modified());
    assert_eq!(form.get_submit_data(), json!({"value": ""}));
    field.set_value(Value::Null);
    assert!(!form.read().is_modified());
}

#[test]
fn text_and_numeric_json_are_not_generically_equivalent() {
    let form = FormContext::new();
    let mut field = register(&form, "value", json!(1));
    field.set_value(json!("1"));
    assert!(form.read().is_modified());
    form.load_form(json!({"value": "01"}));
    field.set_value(json!("1"));
    assert!(form.read().is_modified());
}

#[test]
fn disabled_and_non_submitting_fields_retain_edits() {
    let form = FormContext::new();
    let mut field = register(&form, "text", json!("default"));
    field.set_value(json!("edit"));
    field.update_field_options(FieldOptions {
        submit: false,
        ..options(false)
    });
    assert!(form.read().is_modified());
    assert_eq!(form.get_submit_data(), json!({}));
    field.update_field_options(options(true));
    assert!(!form.read().is_modified());
    assert!(!form.read().is_dirty());
    field.update_field_options(options(false));
    assert!(form.read().is_modified());
    form.write().reset_form();
    assert!(!form.read().is_modified());
}

#[test]
fn partial_group_load_and_field_removal() {
    let form = FormContext::new();
    let mut first = register(&form, "group", json!("one"));
    let mut second = register(&form, "group", json!("two"));
    first.set_value(json!("edit one"));
    second.set_value(json!("edit two"));
    form.load_form(json!({"unrelated": 0}));
    assert!(form.read().is_modified());
    form.load_form(json!({"group": ["loaded"]}));
    assert_eq!(first.get_value(), json!("loaded"));
    assert_eq!(second.get_value(), json!("edit two"));
    assert!(form.read().is_modified());
    second.reset();
    assert!(!form.read().is_modified());
    assert_eq!(form.get_submit_data(), json!({"group": ["loaded", "two"]}));
    first.set_value(json!("edit"));
    drop(first);
    assert!(!form.read().is_modified());
    second.set_value(json!("edit"));
    drop(second);
    assert!(!form.read().is_modified());
    let _replacement = register(&form, "group", json!("replacement"));
    assert!(!form.read().is_modified());
}

#[test]
fn removing_an_invalid_edit_notifies_even_when_dirty_does_not_change() {
    let form = FormContext::new();
    let _first = form.register_field(
        "first",
        json!(""),
        json!(""),
        false,
        invalid(),
        options(false),
        false,
    );
    let mut second = form.register_field(
        "second",
        json!(""),
        json!(""),
        false,
        invalid(),
        options(false),
        false,
    );
    second.set_value(json!("edit"));
    let notifications = Rc::new(RefCell::new(Vec::new()));
    let captured = notifications.clone();
    let _observer = form.add_listener(move |form: FormContext| {
        captured.borrow_mut().push(form.read().is_modified());
    });
    drop(second);
    assert_eq!(*notifications.borrow(), vec![false]);
    assert!(form.read().is_dirty());
}

#[test]
fn replacing_defaults_notifies_modification_observers() {
    let form = FormContext::new();
    let mut field = register(&form, "text", json!("one"));
    let notifications = Rc::new(RefCell::new(Vec::new()));
    let captured = notifications.clone();
    let _observer = form.add_listener(move |form: FormContext| {
        captured.borrow_mut().push(form.read().is_modified());
    });
    field.set_default(json!("two"));
    field.set_default(json!("two"));
    field.set_default(json!("one"));
    assert_eq!(*notifications.borrow(), vec![true, false]);
}

#[test]
fn unique_registration_retains_its_baseline_and_edits() {
    let form = FormContext::new();
    let mut first = form.register_field(
        "shared",
        json!("one"),
        json!("one"),
        false,
        None,
        options(false),
        true,
    );
    first.set_value(json!("edit"));
    drop(first);
    assert!(form.read().is_modified());
    let mut second = form.register_field(
        "shared",
        json!("other"),
        json!("other"),
        false,
        None,
        options(false),
        true,
    );
    assert_eq!(second.get_value(), json!("edit"));
    assert!(form.read().is_modified());
    second.reset();
    assert_eq!(second.get_value(), json!("one"));
    assert!(!form.read().is_modified());
}

#[test]
fn radio_group_enable_load_reset_and_remount() {
    let form = FormContext::new();
    let mut first = form.register_field(
        "choice",
        json!("one"),
        json!("one"),
        true,
        None,
        options(false),
        false,
    );
    let mut second = form.register_field(
        "choice",
        json!(""),
        json!(""),
        true,
        None,
        options(false),
        false,
    );
    second.set_value(json!("two"));
    first.update_field_options(options(true));
    assert!(form.read().is_modified());
    second.update_field_options(options(true));
    assert!(!form.read().is_modified());
    first.update_field_options(options(false));
    assert!(form.read().is_modified());
    form.load_form(json!({"choice": "two"}));
    assert!(!form.read().is_modified());
    first.set_value(json!("one"));
    form.write().reset_form();
    assert_eq!(second.get_value(), json!("two"));
    assert!(!form.read().is_modified());
    first.set_value(json!("edit"));
    drop(first);
    drop(second);
    assert!(!form.read().is_modified());
    let _replacement = form.register_field(
        "choice",
        json!("three"),
        json!("three"),
        true,
        None,
        options(false),
        false,
    );
    // The existing radio group retains its selection and default after its last member is removed.
    assert_eq!(form.read().get_field_value("choice"), Some(json!("edit")));
    assert!(form.read().is_modified());
    form.write().reset_form();
    assert!(!form.read().is_modified());
}

#[test]
fn mixed_radio_and_ordinary_members_check_enabled_fields_independently() {
    let form = FormContext::new();
    let mut radio = form.register_field(
        "mixed",
        json!("one"),
        json!("one"),
        true,
        None,
        options(true),
        false,
    );
    let mut ordinary = register(&form, "mixed", json!("text"));
    radio.set_value(json!("two"));
    assert!(!form.read().is_modified());
    ordinary.set_value(json!("edit"));
    assert!(form.read().is_modified());
    ordinary.reset();
    assert!(!form.read().is_modified());
}

#[test]
fn legacy_reset_and_named_reset_accept_restored_input() {
    let form = FormContext::new();
    let mut field = number(&form, &Number::<i64>::new().default(1).min(2));
    field.set_value(json!("edit"));
    form.write().reset_field("number");
    assert_eq!(field.get_value(), json!(1));
    assert!(!form.read().is_modified());
    field.set_value(json!("edit"));
    form.write().reset_form_old();
    assert!(!form.read().is_modified());
    assert!(form.read().is_dirty());
}
