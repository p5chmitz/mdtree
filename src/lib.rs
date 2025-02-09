// NOTE: All imports used by parse()
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use regex::Regex;

struct Heading {
    level: usize,
    title: String,
}
impl Heading {
    /** Just a humble Heading builder */
    fn new(title: String, level: usize) -> Heading {
        Heading { level, title }
    }
}

/** A position in the tree as raw pointer to a Node, generic over T */
type Pos<T> = Option<*mut Node<T>>;

/** Represents a general tree with a collection of children */
struct Node<T> {
    parent: Pos<T>,
    children: Vec<Pos<T>>,
    data: Option<T>,
}
impl<T> Node<T> {
    /** Builds a new Node and returns its position */
    fn build(data: Option<T>) -> Box<Node<T>> {
        Box::new(Node {
            parent: None,
            children: Vec::new(),
            data,
        })
    }

    /** Gets an immutable reference to the data at a position */
    fn get<'a>(position: Pos<T>) -> Option<&'a T> {
        if let Some(p) = position {
            unsafe { (*p).data.as_ref() }
        } else {
            None
        }
    }
}

/** The Tree struct represents a positional, linked-based general
tree structure with a root node that contains a single raw pointer
to the root node and the structure's size.
The genericity of the struct means you'll have to explicitly
type associated functions. */
struct GenTree<T> {
    root: Pos<T>,
    size: usize,
}
impl<T> GenTree<T> {

    /** Instantiates a new Tree with a default root */
    fn new() -> GenTree<Heading> {
        let data = Heading::new("ROOT".to_string(), 0);
        let root: Pos<Heading> = Some(Box::into_raw(Node::build(Some(data))));
        GenTree { root, size: 1 }
    }

    /** Returns an immutable reference to a position's data */
    fn get(&self, node: &Pos<T>) -> Option<&T> {
        // Imperative approach
        if let Some(n) = node {
            unsafe { (*(*n)).data.as_ref() } // Double de-ref for &*mut type
        } else {
            None
        }
        // Functional approach
        //node.as_ref().and_then(|n| unsafe { (*(*n)).data.as_ref() })
    }

    /** Returns the parent of a given node, if it exists */
    fn parent(&self, node: Pos<T>) -> Option<Pos<T>> {
        if let Some(n) = node {
            unsafe { Some((*n).parent) }
        } else {
            None
        }
    }

    /** Adds a child to a parent's children field represented as Vec<Pos<T>> */
    fn add_child(&mut self, ancestor: Pos<T>, node: Pos<T>) {
        unsafe {
            if let Some(p) = ancestor {
                // Adds the position to the parents arena
                (*p).children.push(node);

                // Links the node's parent Pos<T> to the correct ancestor
                if let Some(n) = node {
                    (*n).parent = ancestor;
                }
            }
            self.size += 1;
        }
    }

    // NOTE: The rest of the methods in this impl block are just used in testing

    /** Returns a reference to the collection of children for a given position, if any */
    fn _children(&self, node: Pos<T>) -> Option<&Vec<Pos<T>>> {
        if let Some(c) = node {
            Some(unsafe { (*c).children.as_ref() })
        } else {
            None
        }
    }

    /** Returns true if the given position is the tree's root */
    fn _is_root(&self, node: Pos<T>) -> bool {
        node == self.root
    }

    /** Returns the depth for a given node */
    fn _depth(&self, node: Pos<T>) -> Option<usize> {
        let mut d = 1;
        let mut cursor = node;
        while !self._is_root(cursor) {
            cursor = self.parent(cursor)?;
            d += 1;
        }
        Some(d)
    }

    /** Returns the height of a sub-tree at a given position */
    fn _height(&self, node: Pos<T>) -> Option<usize> {
        let mut h = 0;
        if let Some(n) = node {
            for e in unsafe { &(*n).children } {
                h = std::cmp::max(h, self._height(Some(e.expect("uh oh")))?)
            }
        }
        Some(h + 1)
    }

}

impl<T> Drop for GenTree<T> {
    fn drop(&mut self) {
        /** Recursive tree destructor */
        // TODO: Update implementation with NonNull
        // to avoid null pointer dereference check
        unsafe fn drop_node_recursive<T>(node_ptr: *mut Node<T>) {
            // Avoids a null pointer dereference
            if node_ptr.is_null() {
                return;
            }

            // Dereference the pointer and process its children
            let node = &mut *node_ptr;
            for &child_ptr in node.children.iter() {
                if let Some(child_ptr) = child_ptr {
                    drop_node_recursive(child_ptr);
                }
            }

            // Deallocate the current node
            let _ = Box::from_raw(node_ptr);
        }

        unsafe {
            if let Some(root_ptr) = self.root {
                drop_node_recursive(root_ptr);
            }
        }
    }
}

/** Takes a path to a Markdown file, parses it for title and headings,
and returns a tuple containing the document title and a vector of
headings.

Note: The document title portion of the tuple is specifically
designed for the Astro-formatted frontmatter of each MD document. */
fn parse(root: &Path) -> (String, Vec<Heading>) {
    // Regex for capturing the title from front matter
    let t = Regex::new(r"(?ms)^---.*?^title:\s*(.+?)\s*$.*?^---").unwrap();
    let mut doc_title = String::new();
    // Regex for capturing headings H1-H6 as #-######
    let h = Regex::new(r"^(#{1,6})\s+(.*)").unwrap();
    let mut headings: Vec<Heading> = Vec::new();

    // Read input
    let file_path = root;
    let file = File::open(file_path).unwrap(); // TODO: Fix lazy error handling
    let reader = BufReader::new(file);

    // Read the entire file into a single string
    // Imperative style
    let mut content = String::new();
    for line_result in reader.lines() {
        let line = line_result.unwrap();
        if !content.is_empty() {
            content.push('\n');
        }
        content.push_str(&line);
    }
    // Functional style
    //let content: String = reader
    //    .lines()
    //    .map(|l| l.unwrap())
    //    .collect::<Vec<_>>()
    //    .join("\n");

    // Extract the document title
    if let Some(captures) = t.captures(&content) {
        let title = captures.get(1).unwrap().as_str();
        doc_title.push_str(title);
    }

    // Parse headings line by line
    for line in content.lines() {
        if let Some(captures) = h.captures(line) {
            let level = captures.get(1).unwrap().as_str().len();
            let text = captures.get(2).unwrap().as_str().to_string();
            headings.push(Heading { level, title: text });
        }
    }

    (doc_title, headings)
}

/** Constructs a tree of Heading types */
fn construct(level: usize, data: Vec<Heading>) -> GenTree<Heading> {
    // Instantiates a Tree with a generic root and traversal positioning
    let mut tree = GenTree::<Heading>::new();
    let mut level_cursor = level;
    let mut position_cursor: Pos<Heading> = tree.root;

    // Constructs tree from Vec<T>
    for e in data {
        // Creates an owned position for each list entry
        let current_level = e.level; // Keep the borrow checker happy
        let node: Pos<Heading> = Some(Box::into_raw(Node::build(Some(e))));

        // Case: Adds a child to the current parent and sets level cursor
        if current_level == level_cursor + 1 {
            tree.add_child(position_cursor, node);
            let data = tree.get(&node).unwrap();
            level_cursor = data.level;
        }

        // Case: Adds a child with multi-generational skips with empty nodes
        else if current_level > level_cursor + 1 {
            let diff = current_level - level_cursor;
            for _ in 1..diff {
                let heading = Heading::new("[]".to_string(), 0);
                let placeholder: Pos<Heading> = Some(Box::into_raw(Node::build(Some(heading))));
                tree.add_child(position_cursor, placeholder);
                position_cursor = placeholder;
                level_cursor += 1;
            }
            tree.add_child(position_cursor, node);
            let data = tree.get(&node).unwrap();
            level_cursor = data.level;
        }

        // Case: Adds sibling to current parent
        else if current_level == level_cursor {
            tree.add_child(tree.parent(position_cursor).expect("No parent"), node);
        }

        // Case: Adds a child to the appropriate ancestor,
        // ensuring proper generational skips
        else {
            let diff = level_cursor - current_level;
            position_cursor = tree.parent(position_cursor).expect("No parent");
            for _ in 0..diff {
                position_cursor = tree.parent(position_cursor).expect("None parent");
                level_cursor -= 1;
            }
            tree.add_child(position_cursor, node);
            let data = tree.get(&node).unwrap();
            level_cursor = data.level;
        }

        // Updates the most recent addition
        position_cursor = node;
    }
    tree
}

/** A wrapper for a recursive preorder(ish) traversal function;
Contains logic to print [] on empty trees for more appealing presentation */
fn pretty_print(name: &str, position: &Pos<Heading>) {
    if let Some(p) = position {
        let children: &Vec<Pos<Heading>> = unsafe { (*(*p)).children.as_ref() };
        if children.len() == 0 {
            println!("📄 {}\n\t[]\n", name); // Empty trees
        } else {
            println!("📄 {}\n\t│", name);
            preorder(position, "");
            println!("");
        }
    }
}

/** Modified preorder traversal function that walks the tree recursively 
printing each node's title and children with appropriate box drawing components */
fn preorder(position: &Pos<Heading>, prefix: &str) {
    // Checks that the position (node) exists
    if let Some(p) = position {
        // Visit the node at the referenced position
        let children: &Vec<Pos<Heading>> = unsafe { (*(*p)).children.as_ref() };
        let mut index = children.len();

        // Recursively visit each child
        for e in children {
            let node = Node::get(*e).unwrap();
            index -= 1;
            if index == 0 {
                println!("\t{}└── {}", prefix, node.title);
                preorder(e, &format!("{}    ", prefix));
            } else {
                println!("\t{}├── {}", prefix, node.title);
                preorder(e, &format!("{}│   ", prefix));
            }
        }
    } else {
        println!("Not a valid position")
    }
}

/** A recursive function that chains the module's utility functions to 
pretty-print a table of contents for each Markdown file in the specified 
directory; The is_file() path contains logic to build a tree from filtered 
values, skipping headers above the user-supplied level argument;
The function also substitues the file name (if any) for all MD files
not formatted with Astro's frontmatter */
pub fn navigator(level: usize, path: &Path) {
    if path.is_dir() {
        for e in path.read_dir().expect("read_dir call failed") {
            let entry = e.expect("failure to deconstruct value");
            navigator(level, &entry.path()); // Recursive call
        }
    } else if path.is_file() {
        if let Some(ext) = path.extension() {
            match ext.to_str() { 
                Some("md") | Some("mdx") => {
                    println!("{}", path.display());
                    let parsed = parse(path);
                    let mut name: String = parsed.0;
                    if name == "" {
                        if let Some(n) = path
                            .file_name()
                            .expect("Error extracting file name")
                            .to_str()
                        {
                            name = n.to_string()
                        }
                    }
                    let filtered = parsed.1.into_iter().filter(|h| h.level > level).collect();
                    let tree = construct(level, filtered);
                    pretty_print(&name, &tree.root);
                }
                _ => ()
            }
        }
    }
}

#[cfg(test)]
mod tests{

    use super::*;

    #[test]
    fn basic_function_test() {
        use std::ptr; // Used by test
    
        // Creates a tree with a default ROOT node
        let mut tree = GenTree::<Heading>::new();
        if let Some(r) = tree.root {
            if let Some(h) = unsafe { (*r).data.as_ref() } {
                assert_eq!(&h.title, "ROOT");
            } else {
                panic!("Data is None!");
            }
        }
    
        // Builds a Heading that simulates an H2, converts it to a Node,
        // and finally converts it to a position Pos<Heading> as raw pointer "a"
        let h2 = Heading::new("H2".to_string(), 2);
        let node_a: Box<Node<Heading>> = Node::build(Some(h2));
        let node_a_ptr: Pos<Heading> = Some(Box::into_raw(node_a));
    
        // Adds a to root
        tree.add_child(tree.root, node_a_ptr);
    
        // Checks that add_child() assigns correct parent for the node
        assert_eq!(tree.root, tree.parent(node_a_ptr).expect("No parent"));
        // Checks that the parent (ROOT) has exactly one child as the "a" node
        assert_eq!(tree._children(tree.root), Some(&vec![node_a_ptr]));
        // Checks that the ROOT's children list _contains_ the "a" node
        assert!(tree._children(tree.root).unwrap().iter().any(|&item| {
            if let Some(ptr) = item {
                ptr::eq(ptr, node_a_ptr.unwrap())
            } else {
                false
            }
        }));
    
        // At this point there should be one node with one default ROOT node
        assert_eq!(tree.size, 2);
    
        // Builds a Heading that simulates an H3, converts it to a Node,
        // and finally converts it to a position Pos<Heading> as raw pointer "b"
        let h3 = Heading::new("H3".to_string(), 3);
        let node_b: Box<Node<Heading>> = Node::build(Some(h3));
        let node_b_ptr: Pos<Heading> = Some(Box::into_raw(node_b));
    
        // Adds "b" to "a"
        tree.add_child(node_a_ptr, node_b_ptr);
    
        // Checks the tree's size, height, and depth of "b"
        // NOTE: size, height, and depth include the ROOT node
        assert_eq!(tree.size, 3);
        assert_eq!(tree._height(tree.root), Some(3));
        assert_eq!(tree._depth(node_b_ptr), Some(3));
    }
    
    #[test]
    /** Creates this tree to test properties
        [] Lorem Ipsum Test
        │    An ordered look at MD parsing
        │
        ├── Landlocked
        │   ├── Switzerland
        │   │   └── Geneva
        │   │       └── Old Town
        │   │           └── Cathédrale Saint-Pierre
        │   └── Bolivia
        └── Island
          ├── Marine
          │   └── Australiae
          └── Fresh Water
    */
    fn n_ary_algorithm_test() {
    
        // Checks that the height is 4
    
        // Checks that the depth of the H5 is 4
    
        // Empty doc test
    }
}
