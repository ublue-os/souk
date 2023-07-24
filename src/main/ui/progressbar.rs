// Souk - progressbar.rs
// Copyright (C) 2023  Felix Häcker <haeckerfelix@gnome.org>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::cell::Cell;
use std::time::Duration;

use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{PropertyAnimationTarget, TimedAnimation};
use glib::{clone, ParamSpec, Properties};
use gtk::glib;
use once_cell::unsync::OnceCell;

mod imp {
    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::SkProgressBar)]
    pub struct SkProgressBar {
        #[property(get, set = Self::set_fraction)]
        fraction: Cell<f32>,
        #[property(get, set = Self::set_pulsing)]
        pulsing: Cell<bool>,

        progressbar: gtk::ProgressBar,
        animation: OnceCell<TimedAnimation>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SkProgressBar {
        const NAME: &'static str = "SkProgressBar";
        type ParentType = adw::Bin;
        type Type = super::SkProgressBar;
    }

    impl ObjectImpl for SkProgressBar {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> glib::Value {
            Self::derived_property(self, id, pspec)
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &ParamSpec) {
            Self::derived_set_property(self, id, value, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();

            self.progressbar.set_pulse_step(1.0);
            self.progressbar.set_valign(gtk::Align::Center);
            self.obj().set_child(Some(&self.progressbar));

            let target = PropertyAnimationTarget::new(&self.progressbar, "fraction");
            let animation = TimedAnimation::new(&self.progressbar, 0.0, 0.0, 1000, target);
            self.animation.set(animation).unwrap();
        }

        fn dispose(&self) {
            // Workaround copied from
            // https://github.com/YaLTeR/plitki/blob/b0c43452e407d906c57b55fdb08980aed29831e4/plitki-gnome/src/hit_light.rs#L49
            let animation = self.animation.get().unwrap();
            animation.set_target(&adw::CallbackAnimationTarget::new(|_| ()));
        }
    }

    impl WidgetImpl for SkProgressBar {}

    impl BinImpl for SkProgressBar {}

    impl SkProgressBar {
        fn set_pulsing(&self, pulsing: bool) {
            if pulsing {
                self.progressbar.pulse();
                // For whatever reason we need to do it twice to make the pulse animation to
                // start instantly
                self.progressbar.pulse();

                glib::timeout_add_local(
                    Duration::from_millis(500),
                    clone!(@weak self as this => @default-return Continue(false), move || {
                        let pulsing = this.pulsing.get();

                        if pulsing {
                            this.progressbar.pulse();
                        }else{
                            this.progressbar.set_fraction(this.fraction.get() as f64);
                        }

                        Continue(pulsing)
                    }),
                );
            }

            self.pulsing.set(pulsing);
            self.obj().notify_pulsing();
        }

        fn set_fraction(&self, fraction: f32) {
            let animation = self.animation.get().unwrap();

            animation.skip();
            let current_value = animation.value();

            animation.reset();
            animation.set_value_from(current_value);
            animation.set_value_to(fraction as f64);
            animation.play();

            self.fraction.set(fraction);
            self.obj().notify_fraction();
        }
    }
}

glib::wrapper! {
    pub struct SkProgressBar(
        ObjectSubclass<imp::SkProgressBar>)
        @extends gtk::Widget, adw::Bin;
}

impl SkProgressBar {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

impl Default for SkProgressBar {
    fn default() -> Self {
        Self::new()
    }
}
