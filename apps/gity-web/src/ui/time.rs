pub fn format_timestamp(value: &str) -> String {
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(value));
    if !date.get_time().is_finite() {
        return value.into();
    }
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        date.get_full_year(),
        date.get_month() + 1,
        date.get_date(),
        date.get_hours(),
        date.get_minutes(),
        date.get_seconds()
    )
}

pub fn relative_time(value: &str) -> String {
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(value));
    if !date.get_time().is_finite() {
        return value.into();
    }
    let seconds = (date.get_time() - js_sys::Date::now()) / 1000.;
    let absolute = seconds.abs();
    let (unit, divisor) = if absolute < 45. {
        ("second", 1.)
    } else if absolute < 45. * 60. {
        ("minute", 60.)
    } else if absolute < 22. * 3600. {
        ("hour", 3600.)
    } else if absolute < 26. * 86400. {
        ("day", 86400.)
    } else if absolute < 11. * 2_592_000. {
        ("month", 2_592_000.)
    } else {
        ("year", 31_536_000.)
    };
    let locales = js_sys::Array::new();
    locales.push(&wasm_bindgen::JsValue::from_str("en"));
    js_sys::Intl::RelativeTimeFormat::new(&locales, &js_sys::Object::new())
        .format((seconds / divisor).round(), unit)
        .into()
}
