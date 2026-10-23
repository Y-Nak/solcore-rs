trait Marker<a> {}
#[derive(Marker, Marker)]
enum Target {}
