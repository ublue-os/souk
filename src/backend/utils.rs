use appstream::enums::Bundle;
use appstream::Component;

pub fn get_flatpak_ref(component: &Component) -> String {
    match &component.bundles[0] {
        Bundle::Flatpak { reference, runtime: _, sdk: _ } => {
            return reference.clone();
        }
        _ => return "".to_string(),
    };
}
