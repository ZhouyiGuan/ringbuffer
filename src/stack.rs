pub struct Stack<T> {
    head: Node<T>,
}

type Node<T> = Option<Box<Content<T>>>;

struct Content<T> {
    elem: T,
    next: Node<T>,
}


impl<T> Stack<T> 
where
    T: std::fmt::Debug,
{
    pub fn new() -> Self {
        Stack { head: None }
    }
    pub fn push(&mut self, elem: T) {
        let new_node = Box::new(Content {
            elem: elem,
            next: self.head.take(),
        });
        self.head = Some(new_node);
    }
    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            self.head = node.next;
            node.elem
        })
    }
    // pub fn peek(&self) -> Option<&T> {
    //     self.head.map(|node| {
    //         &node.elem
    //     })
    // }
}

/// 由于Box<Content>的drop trait不是尾递归(如果是函数操作作为drop结尾就是尾递归,如果结尾是别的操作,
/// 那么这个递归的drop还会回反上来),会使得对于非常长的链表在递归drop的时候会导致栈溢出,所以需要自定义
/// 一下drop trait
impl<T> Drop for Stack<T> {
    fn drop(&mut self) {
        let mut cur_node = self.head.take();
        while let Some(mut content) = cur_node {
            cur_node = content.next.take();
        }
    }
}


#[cfg(test)]
mod test {
    use super::Stack;

    #[test]
    fn basics() {
        let mut Stack = Stack::new();

        // Check empty Stack behaves right
        assert_eq!(Stack.pop(), None);
        // Populate Stack
        Stack.push(1);
        Stack.push(2);
        Stack.push(3);
        // Check normal removal
        assert_eq!(Stack.pop(), Some(3));
        assert_eq!(Stack.pop(), Some(2));
        // Push some more just to make sure nothing's corrupted
        Stack.push(4);
        Stack.push(5);
        // Check normal removal
        assert_eq!(Stack.pop(), Some(5));
        assert_eq!(Stack.pop(), Some(4));
        // Check exhaustion
        assert_eq!(Stack.pop(), Some(1));
        assert_eq!(Stack.pop(), None);
    }

    /// 使用情况:1.当我们只有一个变量的可变引用的时候,我们想获得这个变量的所有权;2.我们的变量没有实
    /// 现copy trait;3.我们不想使用clone的方法(对于没有实现copy trait的变量,clone方法会进行深拷贝
    /// ,会进行内存分配,调用构造函数,开销非常大);
    /// core::ptr::read/write这两种方法都是直接操作内存的,所以都是unsafe,同样也不会进行深拷贝.
    /// core::ptr::read可以在不进行深拷贝(就是不新建一个对象)的情况下,对一个可变引用的内容直接拷贝
    /// 内存.这个原本可变引用的内容并没有被drop掉.
    /// core::ptr::write则是在不进行深拷贝的情况下,对一个可变引用的内容直接写入内存.新的值 src 被
    /// 写入 dest 指向的内存地址，覆盖了原来的数据，但同样不触发析构函数。这个过程不涉及到 src 的拷
    /// 贝构造或移动构造，因为它是一个直接的内存写入操作。
    #[test]
    fn ptr_readwrite(){
        let mut a = Some(String::from("a"));
        let src =  &mut a;
        unsafe{
            let result = core::ptr::read(src);
            assert_eq!(result,Some(String::from("a")));
            assert_eq!(src,&mut Some(String::from("a")));

            core::ptr::write(src,Some(String::from("b")));
            assert_eq!(src,&mut Some(String::from("b")));
        }   
    }

    #[test]
    fn drop_longstack() {
        let mut stack = Stack::new();
        for i in 0..100000 {
            stack.push(i);
        }
        drop(stack);
    }
}
