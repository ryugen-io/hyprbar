use super::BarRenderer;
use crate::config::BarConfig;
use crate::modules::logging::*;
use crate::widget::Widget;
use hyprink::config::Config;

impl BarRenderer {
    pub(crate) fn init_widgets(
        names: &[String],
        _config: &BarConfig,
        config_ink: &Config,
        provider: &dyn crate::widget::WidgetProvider,
    ) -> Vec<Box<dyn Widget>> {
        log_debug("WIDGET", &format!("Initializing {} widgets", names.len()));

        let mut widgets: Vec<Box<dyn Widget>> = Vec::new();
        let log_fmt = config_ink
            .layout
            .labels
            .get("widget_loaded")
            .cloned()
            .unwrap_or_else(|| "Loaded Dish: {0} (Type: {1})".to_string());

        for raw_name in names {
            log_debug("WIDGET", &format!("Parsing widget spec: {}", raw_name));

            let (name, alias) = if let Some((n, a)) = raw_name.split_once('.') {
                (n, a)
            } else {
                raw_name
                    .split_once('#')
                    .unwrap_or((raw_name.as_str(), raw_name.as_str()))
            };

            log_debug(
                "WIDGET",
                &format!("Resolved: name={}, alias={}", name, alias),
            );

            match provider.create_widget(name) {
                Some(mut plugin_widget) => {
                    plugin_widget.set_instance_config(alias.to_string());

                    let display_name = if name != alias {
                        format!("{} as {}", name, alias)
                    } else {
                        name.to_string()
                    };

                    let msg = log_fmt
                        .replace("{0}", &display_name)
                        .replace("{1}", "Plugin");
                    log_info("WIDGET", &msg);
                    widgets.push(plugin_widget);
                }
                None => {
                    log_error("WIDGET", &format!("Failed to create widget: {}", name));
                    log_warn("WIDGET", &format!("Unknown widget: {}", name));
                }
            }
        }

        log_info(
            "WIDGET",
            &format!("Loaded {} widgets successfully", widgets.len()),
        );
        widgets
    }
}
