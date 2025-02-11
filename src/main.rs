mod collections;

use collections::cache_lru::CacheLru;

fn main() {
    let mut cache = CacheLru::new(3);
    cache.insert("cuba", 7);
    cache.insert("ramona", 4);
    cache.get("cuba");
    cache.insert("ramona", 5);
    println!("cuba 1: {}", cache.get("cuba").unwrap());
    cache.insert("cuba", 283);
    println!("cuba 2 after insert: {:?}", cache.get("cuba"));

    cache.insert("nube", 1);
    cache.insert("barcelona", 3982);
    cache.insert("cuba", 3234);

    println!();
    println!("{:?}", cache);

    println!(
        "\ncuba: {:?}\nramona: {:?}\nnube: {:?}\nbarcelona: {:?}",
        cache.get("cuba"),
        cache.get("ramona"),
        cache.get("nube"),
        cache.get("barcelona")
    );
}
