use gtk::glib;

mod imp {
    use glib::Binding;
    use gtk::subclass::prelude::*;
    use gtk::{glib, Button, CompositeTemplate};
    use std::cell::RefCell;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/org/gnome/controlpanelgui/ui/security_settings_page.ui")]
    pub struct SecuritySettingsPage {
        #[template_child]
        pub verify_button: TemplateChild<Button>,
        #[template_child]
        pub password_reset_button: TemplateChild<Button>,

        // Vector holding the bindings to properties of `Object`
        pub bindings: RefCell<Vec<Binding>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SecuritySettingsPage {
        const NAME: &'static str = "SecuritySettingsPage";
        type Type = super::SecuritySettingsPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            //klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    /*#[gtk::template_callbacks]
    impl SecuritySettingsPage {
        #[template_callback]
        fn on_row_selected(&self, row: &gtk::ListBoxRow) {

        }
    }*///end #[gtk::template_callbacks]

    impl ObjectImpl for SecuritySettingsPage {
        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();
        }
    }
    impl WidgetImpl for SecuritySettingsPage {}
    impl BoxImpl for SecuritySettingsPage {}
}

glib::wrapper! {
pub struct SecuritySettingsPage(ObjectSubclass<imp::SecuritySettingsPage>)
    @extends gtk::Widget, gtk::Box;
}

impl Default for SecuritySettingsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl SecuritySettingsPage {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
}
