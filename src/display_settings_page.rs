use crate::settings_gobject::SettingsGObject;
use gtk::subclass::prelude::*;
use gtk::{glib, StringList};
use regex::Regex;
use std::process::{Command, Stdio};

use crate::prelude::*;

mod imp {
    use glib::subclass::Signal;
    use glib::Binding;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::{glib, CheckButton, CompositeTemplate, DropDown, StringList};
    use std::cell::RefCell;
    use std::sync::OnceLock;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/org/gnome/controlpanelgui/ui/display_settings_page.ui")]
    pub struct DisplaySettingsPage {
        #[template_child]
        pub resolution_switch: TemplateChild<DropDown>,
        #[template_child]
        pub scale_switch: TemplateChild<DropDown>,
        #[template_child]
        pub light_theme_button: TemplateChild<CheckButton>,
        #[template_child]
        pub dark_theme_button: TemplateChild<CheckButton>,

        //must be read from somewhere
        pub supported_resolutions: StringList,
        pub supported_scales: StringList,

        // Vector holding the bindings to properties of `Object`
        pub bindings: RefCell<Vec<Binding>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DisplaySettingsPage {
        const NAME: &'static str = "DisplaySettingsPage";
        type Type = super::DisplaySettingsPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl DisplaySettingsPage {
        #[template_callback]
        fn on_reset_clicked(&self) {
            self.obj().restore_default();
            self.obj()
                .emit_by_name::<()>("default-display-settings", &[]);
        }

        //TODO: revise logic
        #[template_callback]
        fn on_apply_clicked(&self) {
            let resolution_idx = self.resolution_switch.selected();
            let scale_idx = self.scale_switch.selected();
            let is_resolution_set = self.obj().set_resolution(resolution_idx);
            let is_scale_set = self.obj().set_scale(scale_idx);

            //if error occures then show popup and return
            if !is_resolution_set || !is_scale_set {
                self.obj().emit_by_name::<()>("display-settings-error", &[]);
                return;
            }

            //if all non-default settings are applied
            //then show confirmation popup
            if (resolution_idx > 0 || scale_idx > 0) && is_resolution_set && is_scale_set {
                self.obj()
                    .emit_by_name::<()>("display-settings-changed", &[]);
            }
        }
    } //end #[gtk::template_callbacks]

    impl ObjectImpl for DisplaySettingsPage {
        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();

            // Setup
            let obj = self.obj();
            obj.init();
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                [
                    "display-settings-changed",
                    "default-display-settings",
                    "display-settings-error",
                ]
                .into_iter()
                .map(|sig| Signal::builder(sig).build())
                .collect()
            })
        }
    }
    impl WidgetImpl for DisplaySettingsPage {}
    impl BoxImpl for DisplaySettingsPage {}
}

glib::wrapper! {
pub struct DisplaySettingsPage(ObjectSubclass<imp::DisplaySettingsPage>)
    @extends gtk::Widget, gtk::Box;
}

impl Default for DisplaySettingsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl DisplaySettingsPage {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
    //TODO: read all supported modes
    pub fn init(&self) {
        self.read_supported_resolutions();
        self.set_supported_scales();

        //set the current settings selected in the Dropdowns
        self.display_current_settings();
    }

    fn read_supported_resolutions(&self) {
        //temporary, must be read from somewhere
        let supported_resolutions = self.imp().supported_resolutions.clone();
        //the list is taken from /sys/kernel/debug/dri/0/i915_display_info
        supported_resolutions.splice(0, 0, &["1920x1200", "1936x1203", "1952x1217", "2104x1236"]);
        //supported_resolutions.append(&String::from("2560x1600"));//for testing

        let switch = self.imp().resolution_switch.get();
        switch.set_model(Some(&supported_resolutions));
    }

    fn set_supported_scales(&self) {
        let supported_scales = self.imp().supported_scales.clone();
        supported_scales.splice(0, 0, &["100%", "125%", "150%"]);

        let switch = self.imp().scale_switch.get();
        switch.set_model(Some(&supported_scales));
    }

    fn display_current_settings(&self) {
        //let output = Command::new("xrandr")//for testing
        let output = Command::new("wlr-randr")
            .env("PATH", "/run/current-system/sw/bin")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    self.set_current_resolution(&stdout);
                    self.set_current_scale(&stdout);
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    error!("wlr-randr error: {stderr}");
                }
            }
            Err(e) => {
                error!("Failed to execute wlr-randr: {e}");
            }
        }
    }

    #[inline]
    fn set_current_resolution(&self, stdout: &str) {
        //for standart eDP-1
        let current_resolution_regex =
            Regex::new(r"eDP-1[\s\S]*?(\d+x\d+)\s*px[^\n]*current").unwrap();
        //let current_resolution_regex = Regex::new(r"(\d+x\d+)\s+\d+\.\d+\*").unwrap();//for testing
        if let Some(cap) = current_resolution_regex.captures(stdout) {
            let resolution = &cap[1];
            debug!("Current resolution: {resolution}");
            let supported_resolutions = self.imp().supported_resolutions.clone();

            match index_of(&supported_resolutions, resolution) {
                Some(index) => {
                    debug!("Found {resolution} at index: {index}");
                    let switch = self.imp().resolution_switch.get();
                    switch.set_selected(index);
                }
                None => warn!("Resolution not found"),
            }
        } else {
            warn!("No current resolution found.");
        }
    }

    #[inline]
    fn set_current_scale(&self, stdout: &str) {
        //for standart eDP-1
        let current_scale_regex = Regex::new(r"eDP-1[\s\S]*?Scale:\s*([\d.]+)").unwrap();
        if let Some(cap) = current_scale_regex.captures(stdout) {
            let scale = &cap[1];
            debug!("Current scale: {scale}");

            //transform to percents
            if let Ok(scale) = scale.parse::<f32>() {
                let scale_percent = format!("{scale:.0}%");

                let supported_scales = &self.imp().supported_scales;

                match index_of(supported_scales, &scale_percent) {
                    Some(index) => {
                        debug!("Found {scale_percent} at index: {index}");
                        let switch = self.imp().scale_switch.get();
                        switch.set_selected(index);
                    }
                    None => warn!("Scale not found"),
                }
            } else {
                warn!("Failed to parse scale.");
            }
        } else {
            warn!("No current scale found.");
        }
    }

    pub fn bind(&self, _settings_object: &SettingsGObject) {
        //unbind previous ones
        self.unbind();
        //make new
    }

    pub fn unbind(&self) {
        // Unbind all stored bindings
        for binding in self.imp().bindings.borrow_mut().drain(..) {
            binding.unbind();
        }
    }

    pub fn restore_default(&self) {
        self.imp().resolution_switch.set_selected(0);
        self.imp().scale_switch.set_selected(0);
        self.set_resolution(0);
        self.set_scale(0);
    }

    pub fn set_resolution(&self, index: u32) -> bool {
        //default: wlr-randr --output eDP-1 --mode 1920x1200
        //custom: wlr-randr --output eDP-1 --custom-mode 'resolution@fps'
        let mut result: bool = false;
        let resolution = self.imp().supported_resolutions.string(index).unwrap();
        let output = Command::new("wlr-randr")
            .arg("--output")
            .arg("eDP-1")
            .arg(if index > 0 { "--custom-mode" } else { "--mode" })
            .arg(if index > 0 {
                String::from(resolution) + &String::from("@60")
            } else {
                String::from(resolution)
            })
            .env("PATH", "/run/current-system/sw/bin")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    debug!("wlr-randr output: {stdout}");

                    result = true;
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    error!("wlr-randr error: {stderr}");
                }
            }
            Err(e) => {
                error!("Failed to execute wlr-randr: {e}");
            }
        }
        result
    }

    #[allow(clippy::unused_self)]
    pub fn set_scale(&self, index: u32) -> bool {
        let mut result: bool = false;
        let factor = match index {
            1 => 1.25,
            2 => 1.5,
            _ => 1.0,
        };

        let output = Command::new("wlr-randr")
            .arg("--output")
            .arg("eDP-1")
            .arg("--scale")
            .arg(factor.to_string())
            .env("PATH", "/run/current-system/sw/bin")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    debug!("wlr-randr scale output: {stdout}");

                    //now it is not needed, the task SSRCSP-6073 is closed
                    //self.reload_taskbar();

                    result = true;
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    error!("wlr-randr error: {stderr}");
                }
            }
            Err(e) => {
                error!("Failed to execute wlr-randr: {e}");
            }
        }
        result
    }

    //can be used if SSRCSP-6073 is opened again
    #[allow(clippy::unused_self)]
    pub fn reload_taskbar(&self) {
        let output = Command::new("systemctl")
            .arg("--user")
            .arg("reload")
            .arg("ewwbar")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    debug!("Taskbar has been reloaded: {stdout}");
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    error!("Taskbar reloading error: {stderr}");
                }
            }
            Err(e) => {
                error!("Failed to execute taskbar reloading: {e}");
            }
        }
    }
}

#[inline]
fn index_of(list: &StringList, target: &str) -> Option<u32> {
    (0..)
        .map_while(|i| list.string(i).map(|s| (i, s)))
        .find_map(|(i, s)| (s == target).then_some(i))
}
