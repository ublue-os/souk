use appstream::Component;
use appstream::enums::Bundle;

pub fn get_flatpak_ref (component: &Component) -> String {
    match &component.bundles[0]{
        Bundle::Flatpak{id, runtime: _, sdk: _} => {
            return id.clone();
        },
        _ => return "".to_string(),
    };
}
