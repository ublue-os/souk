// Souk - mod.rs
// Copyright (C) 2022-2023  Felix Häcker <haeckerfelix@gnome.org>
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

mod appstream;
mod context;
mod flatpak;
mod task;
mod ui;

mod app;
mod dbus_proxy;
mod error;
mod i18n;
mod worker;

pub use app::SkApplication;

// https://gtk-rs.org/gtk-rs-core/git/docs/glib/fn.spawn_future_local.html
pub fn spawn_future_local<R: 'static, F: std::future::Future<Output = R> + 'static>(
    f: F,
) -> gtk::glib::JoinHandle<R> {
    let ctx = gtk::glib::MainContext::ref_thread_default();
    ctx.spawn_local(f)
}
