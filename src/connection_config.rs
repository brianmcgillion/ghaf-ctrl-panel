use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};

mod imp {
    use glib::subclass::Signal;
    use glib::Binding;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::{glib, Button, CompositeTemplate, Entry};
    use std::cell::RefCell;
    use std::sync::OnceLock;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/ae/tii/ghaf/controlpanelgui/ui/connection_config.ui")]
    pub struct ConnectionConfig {
        #[template_child]
        pub address_entry: TemplateChild<Entry>,
        #[template_child]
        pub port_entry: TemplateChild<Entry>,
        #[template_child]
        pub apply_button: TemplateChild<Button>,
        #[template_child]
        pub cancel_button: TemplateChild<Button>,

        // Vector holding the bindings to properties of `Object`
        pub bindings: RefCell<Vec<Binding>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ConnectionConfig {
        const NAME: &'static str = "ConnectionConfig";
        type Type = super::ConnectionConfig;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl ConnectionConfig {
        #[template_callback]
        fn on_apply_clicked(&self) {
            let (addr, port) = self.obj().get_config();
            self.obj()
                .emit_by_name::<()>("new-config-applied", &[&addr, &port]);
        }
        #[template_callback]
        fn on_cancel_clicked(&self) {
            self.obj().close();
        }
    } //end #[gtk::template_callbacks]

    impl ObjectImpl for ConnectionConfig {
        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![Signal::builder("new-config-applied")
                    .param_types([String::static_type(), u32::static_type()])
                    .build()]
            })
        }
    }
    impl WidgetImpl for ConnectionConfig {}
    impl BoxImpl for ConnectionConfig {}
    impl WindowImpl for ConnectionConfig {}
}

glib::wrapper! {
pub struct ConnectionConfig(ObjectSubclass<imp::ConnectionConfig>)
@extends gtk::Widget, gtk::Window, @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self::new("", 1)
    }
}

impl ConnectionConfig {
    pub fn new(address: &str, port: u16) -> Self {
        let config_widget: Self = glib::Object::builder().build();
        config_widget.imp().address_entry.set_text(address);
        config_widget
            .imp()
            .port_entry
            .set_text(port.to_string().as_str());
        config_widget
    }

    pub fn get_config(&self) -> (String, u32) {
        let text = self.imp().port_entry.text().to_string();
        let port = text.parse::<u16>().unwrap_or(0);
        (
            String::from(self.imp().address_entry.buffer().text()),
            port.into(),
        )
    }
}
