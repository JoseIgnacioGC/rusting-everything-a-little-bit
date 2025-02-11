use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

type SharedNode<T> = Rc<RefCell<Node<T>>>;

trait SharedNodeExt<T> {
    fn set_prev(&mut self, node: SharedNode<T>);
    fn set_next(&mut self, node: SharedNode<T>);
    fn get_node(&self) -> std::cell::Ref<'_, Node<T>>;
    // fn get_mut_node(&self) -> std::cell::RefMut<'_, Node<T>>;
    // fn get_prev(&self) -> Option<SharedNode<T>>;
    // fn get_next(&self) -> Option<SharedNode<T>>;
}

impl<T> SharedNodeExt<T> for SharedNode<T> {
    fn set_prev(&mut self, node: SharedNode<T>) {
        self.as_ref().borrow_mut().prev = Some(node);
    }

    fn set_next(&mut self, node: SharedNode<T>) {
        self.as_ref().borrow_mut().next = Some(node);
    }

    fn get_node(&self) -> std::cell::Ref<'_, Node<T>> {
        self.as_ref().borrow()
    }

    // fn get_mut_node(&self) -> std::cell::RefMut<'_, Node<T>> {
    //     self.as_ref().borrow_mut()
    // }
}

struct Node<T> {
    pub next: Option<SharedNode<T>>,
    pub prev: Option<SharedNode<T>>,
    pub value: T,
    pub key: String,
}

impl<T> Node<T> {
    fn new(key: String, value: T) -> SharedNode<T> {
        Rc::new(RefCell::new(Self {
            next: None,
            prev: None,
            value,
            key,
        }))
    }

    fn from(node: Self) -> SharedNode<T> {
        Rc::new(RefCell::new(node))
    }
}

pub struct CacheLru {
    cache: HashMap<String, SharedNode<usize>>,
    dummy_head: SharedNode<usize>,
    dummy_tail: SharedNode<usize>,
    max_capacity: usize,
}

impl CacheLru {
    pub fn new(max_capacity: usize) -> Self {
        let mut dummy_head = Node::new("@dummy_head@".to_string(), 0);
        let mut dummy_tail = Node::new("@dummy_tail@".to_string(), 0);

        dummy_head.set_prev(dummy_tail.clone());
        dummy_tail.set_next(dummy_head.clone());

        Self {
            cache: HashMap::new(),
            dummy_head,
            dummy_tail,
            max_capacity,
        }
    }

    fn get_lru(&mut self) -> SharedNode<usize> {
        let lru_node = {
            let dummy_tail = self.dummy_tail.as_ref().borrow_mut();
            dummy_tail.next.as_ref().unwrap().clone()
        };
        lru_node
    }

    fn get_head(&self) -> SharedNode<usize> {
        let head_node = {
            let dummy_head = self.dummy_head.as_ref().borrow_mut();
            dummy_head.prev.as_ref().unwrap().clone()
        };
        head_node
    }

    fn remove_lru(&mut self) {
        if self.cache.len() <= self.max_capacity {
            return;
        }

        let lru = self.get_lru();
        self.remove(lru);
    }

    fn remove(&mut self, node: SharedNode<usize>) {
        let node = node.get_node();

        let key = &node.key;
        self.cache.remove(key);

        let mut prev_node = node.prev.clone().unwrap();
        let mut next_node = node.next.clone().unwrap();

        prev_node.set_next(next_node.clone());
        next_node.set_prev(prev_node.clone());
    }

    pub fn get(&mut self, key: &str) -> Option<usize> {
        if !self.cache.contains_key(key) {
            return None;
        }

        let node = self.cache.get(key).unwrap();
        let value = node.borrow().value;

        self.remove(node.clone());
        self.insert(key, value);

        Some(value)
    }

    pub fn insert(&mut self, key: &str, value: usize) {
        if let Some(node) = self.cache.get(key) {
            self.remove(node.clone());
        };

        let mut head_node = self.get_head();

        let new_head_node = Node::from(Node {
            next: Some(self.dummy_head.clone()),
            prev: Some(head_node.clone()),
            value,
            key: key.to_string(),
        });

        head_node.set_next(new_head_node.clone());
        self.dummy_head.set_prev(new_head_node.clone());

        self.cache.insert(key.to_string(), new_head_node);
        self.remove_lru();
    }
}

impl Debug for CacheLru {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "CacheLru {{")?;

        // Start with dummy_tail
        let mut node = Some(self.dummy_tail.clone());
        let mut i = 0;

        // Iterate through nodes
        while let Some(node_content) = node {
            let borrowed = node_content.borrow();
            writeln!(f, "\t{}), '{}': {}", i, borrowed.key, borrowed.value)?;

            node = borrowed.next.clone();
            i += 1;
        }

        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cache() {
        let mut cache = CacheLru::new(2);
        assert_eq!(cache.get("nonexistent"), None);
    }

    #[test]
    fn test_insert_and_get() {
        let mut cache = CacheLru::new(2);
        cache.insert("key1", 1);
        assert_eq!(cache.get("key1"), Some(1));
    }

    #[test]
    fn test_capacity_limit() {
        let mut cache = CacheLru::new(2);
        cache.insert("key1", 1);
        cache.insert("key2", 2);
        cache.insert("key3", 3); // This should evict key1
        assert_eq!(cache.get("key1"), None);
        assert_eq!(cache.get("key2"), Some(2));
        assert_eq!(cache.get("key3"), Some(3));
    }

    #[test]
    fn test_lru_get_eviction_order() {
        let mut cache = CacheLru::new(2);
        cache.insert("key1", 1);
        cache.insert("key2", 2);
        cache.get("key1"); // Makes key1 most recently used
        cache.insert("key3", 3); // Should evict key2
        assert_eq!(cache.get("key1"), Some(1));
        assert_eq!(cache.get("key2"), None);
        assert_eq!(cache.get("key3"), Some(3));
    }

    #[test]
    fn test_lru_insert_eviction_order() {
        let mut cache = CacheLru::new(2);
        cache.insert("key1", 1);
        cache.insert("key2", 2);
        cache.insert("key1", 50); // Makes key1 most recently used
        cache.insert("key3", 3); // Should evict key2
        assert_eq!(cache.get("key1"), Some(50));
        assert_eq!(cache.get("key2"), None);
        assert_eq!(cache.get("key3"), Some(3));
    }

    #[test]
    fn test_update_existing() {
        let mut cache = CacheLru::new(2);
        cache.insert("key1", 1);
        cache.insert("key1", 100);
        assert_eq!(cache.get("key1"), Some(100));
    }
}
