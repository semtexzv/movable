pub mod movable;
pub fn register_types(registry: &mut protokit::textformat::reflect::Registry) {
    movable::register_types(registry);
}
