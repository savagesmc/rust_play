
// Include the `items` module, which is generated from items.proto.
// It is important to maintain the same structure as in the proto.
pub mod hello_proto {
    pub mod items {
        include!(concat!(env!("OUT_DIR"), "/hello_proto.items.rs"));
    }
}

use hello_proto::items;

/// Returns a large shirt of the specified color
pub fn create_large_shirt(color: String) -> items::Shirt {
    let mut shirt = items::Shirt::default();
    shirt.color = color;
    shirt.set_size(items::shirt::Size::Large);
    shirt
}

fn main() {
    let shirt = create_large_shirt("Red".to_string());
    println!("Hello, world! {} :: {}", shirt.color, shirt.size);
}
