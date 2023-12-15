/// thread unsafe queue with Rc<RefCell>
/// todo!

use std::rc::Rc;
use std::cell::RefCell;

pub struct List<T> {
    head: Node<T>,
    tail: Node<T>,
}

type Node<T> = Option<Rc<RefCell<NodeContent<T>>>>;

struct NodeContent<T> {
    elem: T,
    next: Node<T>,
    prev: Node<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None, tail: None }
    }
}

impl<T> NodeContent<T> {
    fn new(elem: T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(NodeContent {
            elem: elem,
            prev: None,
            next: None,
        }))
    }
}