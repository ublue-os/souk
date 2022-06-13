// Souk - utils.rs
// Copyright (C) 2022  Felix Häcker <haeckerfelix@gnome.org>
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
use gtk::gio::Cancellable;
use gtk::prelude::*;
use libflatpak::prelude::*;
use libflatpak::{Ref, Remote};
use xb::prelude::*;

pub fn component_from_remote(ref_: &Ref, remote: &Remote) -> Option<Component> {
    debug!("Load appstream file...");
    let appstream_dir = remote
        .appstream_dir(Some(&ref_.arch().unwrap().to_string()))
        .unwrap();
    let appstream_file = appstream_dir.child("appstream.xml");

    // Load
    let source = xb::BuilderSource::new();
    let res = source.load_file(
        &appstream_file,
        xb::BuilderSourceFlags::NONE,
        Cancellable::NONE,
    );
    if let Err(err) = res {
        error!("Unable to load appstream file: {}", err.to_string());
        return None;
    }

    add_source_fixups(&source);

    // Compile appstream xml into xmlb
    let builder = xb::Builder::new();
    builder.import_source(&source);
    let silo = builder
        .compile(xb::BuilderCompileFlags::NONE, Cancellable::NONE)
        .unwrap();

    // Query for appstream component
    let xpath = format!(
        "components/component/id[text()='{}']/..",
        ref_.name().unwrap()
    );
    if let Ok(node) = silo.query_first(&xpath) {
        let xml = node.export(xb::NodeExportFlags::NONE).unwrap().to_string();
        let element = xmltree::Element::parse(xml.as_bytes()).unwrap();
        return Component::try_from(&element).ok();
    }

    None
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
