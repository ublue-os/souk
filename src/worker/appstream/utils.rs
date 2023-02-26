// Souk - utils.rs
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

use appstream::Component;
use flatpak::prelude::*;
use flatpak::{Ref, Remote};
use gtk::gio::Cancellable;
use gtk::prelude::*;
use xb::prelude::*;

use crate::shared::flatpak::dry_run::DryRunPackage;

// TODO: This currently compiles the appstream into xmlb every single time for
// every single runtime / package.... TODO: Check if appstream for remote
// exists, otherwise update
pub fn set_dry_run_package_appstream(package: &mut DryRunPackage, ref_str: &str, remote: &Remote) {
    let ref_ = Ref::parse(&ref_str).unwrap();
    let name = ref_.name().unwrap().to_string();
    let arch = ref_.arch().unwrap().to_string();

    // Those Flatpak subrefs usually don't include appstream data.
    // So we strip the suffixes, and retrieve the appstream data of the actual ref.
    //
    // We use here the same subrefs as Flatpak, see:
    // https://github.com/flatpak/flatpak/blob/600e18567c538ecd306d021534dbb418dc490676/common/flatpak-ref-utils.c#L451
    let name = name.trim_end_matches(".Locale");
    let name = name.trim_end_matches(".Debug");
    let name = name.trim_end_matches(".Sources");

    let appstream_dir = remote.appstream_dir(Some(&arch)).unwrap();
    let appstream_file = appstream_dir.child("appstream.xml");

    debug!("Load appstream file {:?}", appstream_file.path().unwrap());

    // Load
    let source = xb::BuilderSource::new();
    let res = source.load_file(
        &appstream_file,
        xb::BuilderSourceFlags::NONE,
        Cancellable::NONE,
    );
    if let Err(err) = res {
        error!("Unable to load appstream file: {}", err.to_string());
        return;
    }

    add_source_fixups(&source);

    // Compile appstream xml into xmlb
    let builder = xb::Builder::new();
    builder.import_source(&source);
    let silo = builder
        .compile(xb::BuilderCompileFlags::NONE, Cancellable::NONE)
        .unwrap();

    // Query for appstream component
    let xpath = format!("components/component/id[text()='{name}']/..");
    if let Ok(node) = silo.query_first(&xpath) {
        let xml = node.export(xb::NodeExportFlags::NONE).unwrap().to_string();
        let element = xmltree::Element::parse(xml.as_bytes()).unwrap();

        if let Ok(component) = Component::try_from(&element) {
            package.appstream_component = Some(serde_json::to_string(&component).unwrap()).into();
        } else {
            warn!("Couldn't find appstream component for {name}");
        }
    }

    // Icon
    let icon_file = appstream_dir.child(format!("icons/128x128/{}.png", name));
    if let Ok((bytes, _)) = icon_file.load_bytes(Cancellable::NONE) {
        package.icon = Some(bytes.to_vec()).into();
    }
}

// Based on the gnome-software fixups
// https://gitlab.gnome.org/GNOME/gnome-software/-/blob/35e2d0e4191d0c81bf48e5f05bbb1c110572f917/plugins/flatpak/gs-flatpak.c#L677
fn add_source_fixups(source: &xb::BuilderSource) {
    // Ensure the <id> matches the flatpak ref ID
    let fixup = xb::BuilderFixup::new("FixIdDesktopSuffix", |_, node, _| {
        if node.element().is_some() {
            let id = node.child("id", None);
            let bundle = node.child("bundle", None);

            if id.is_none() || bundle.is_none() {
                return true;
            }
            let id = id.unwrap();
            let bundle = bundle.unwrap();

            let bundle_txt = bundle.text();
            let split: Vec<&str> = bundle_txt.split('/').collect();
            if split.len() != 4 {
                return true;
            }

            let component_id = id.text();
            let correct_id = split[1];

            if component_id != correct_id {
                debug!("Fix component id: {} -> {}", component_id, correct_id);
                id.set_text(Some(correct_id));

                // Add the "wrong" id to "provides", so we can find the component by the wrong
                // id as well
                let provides = match node.child("provides", None) {
                    Some(provides) => provides,
                    None => {
                        let n = xb::BuilderNode::new("provides");
                        node.add_child(&n);
                        n
                    }
                };

                if provides.child("id", Some(&component_id)).is_none() {
                    let id_node = xb::BuilderNode::new("id");
                    id_node.set_text(Some(&component_id));
                    provides.add_child(&id_node);
                };
            }
        }

        true
    });
    fixup.set_max_depth(2);
    source.add_fixup(&fixup);

    // Add tokens to allow much faster searching
    let fixup = xb::BuilderFixup::new("TextTokenize", |_, node, _| {
        if let Some(element) = node.element() {
            let tokens = vec!["id", "keyword", "launchable", "mimetype", "name", "summary"];

            if tokens.contains(&element.as_str()) {
                node.tokenize_text();
            }
        }

        true
    });
    fixup.set_max_depth(2);
    source.add_fixup(&fixup);
}
