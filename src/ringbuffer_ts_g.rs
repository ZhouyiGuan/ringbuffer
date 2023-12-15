/// thread safe and generic ring buffer
/// the type T must implement Copy trait

use std::{
    sync::atomic::{
        AtomicUsize, 
        Ordering, 
        AtomicPtr,
    },
    ptr,
    cmp::min,
};


pub struct RingBuffer<T, const N: usize> {
    buffer: [AtomicPtr<T>; N],
    head: AtomicUsize,
    tail: AtomicUsize,
    used_count: AtomicUsize,
}


impl<T, const N:usize> RingBuffer<T,N> 
where 
    T: Copy 
{  
    pub fn new() -> Self {
        Self {
            buffer: [(); N].map(|_| AtomicPtr::new(ptr::null_mut())),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            used_count: AtomicUsize::new(0),
        }
    }

    fn n_write(&self, data: &Vec<T>) -> usize {
        if N - self.used_count.load(Ordering::Relaxed) == 0 {
            // println!("buffer full");
            return 0;
        }
        let write_count = min(data.len(),N - self.used_count.load(Ordering::Relaxed));
        let tail = self.tail.load(Ordering::Relaxed);
        let new_tail;
        if write_count <= (N - tail) {
            for i in 0..write_count {
                let new_struct = Box::new(data[i]);
                let new_struct_ptr = Box::into_raw(new_struct);
                self.buffer[tail + i].store(new_struct_ptr, Ordering::Relaxed);
            }
            new_tail = tail.wrapping_add(write_count);
        } else {
            new_tail = write_count - (N - tail);
            for i in 0..(N - tail) {
                let new_struct = Box::new(data[i]);
                let new_struct_ptr = Box::into_raw(new_struct);
                self.buffer[tail + i].store(new_struct_ptr, Ordering::Relaxed);               
            }
            for i in 0..new_tail {
                let new_struct = Box::new(data[(N - tail) + i]);
                let new_struct_ptr = Box::into_raw(new_struct);
                self.buffer[i].store(new_struct_ptr, Ordering::Relaxed);
            }
        }

        self.used_count.fetch_add(write_count, Ordering::Release);
        self.tail.store(new_tail, Ordering::Release);
        write_count
    }
    fn n_read(&self,data: &mut Vec<T>) -> usize {
        data.clear();
        let read_count = self.used_count.load(Ordering::Relaxed);
        if read_count == 0 {
            // println!("buffer empty");
            return 0;
        }

        let head = self.head.load(Ordering::Relaxed);
        let new_head;
        if read_count <= (N - head) {
            for i in head..(head + read_count) {
                unsafe {
                    let struct_ptr = self.buffer[i].load(Ordering::Relaxed);
                    let elem = Box::from_raw(struct_ptr);
                    data.push(*elem);
                }
            }
            new_head = head.wrapping_add(read_count);
        } else {
            new_head = read_count - (N - head);
            for i in head..N {
                unsafe {
                    data.push(*self.buffer[i].load(Ordering::Relaxed));
                }
            }
            for i in 0..new_head {
                unsafe {
                    data.push(*self.buffer[i].load(Ordering::Relaxed));
                }
            }
        }

        self.used_count.fetch_sub(read_count, Ordering::Release);
        self.head.store(new_head, Ordering::Release);
        read_count
    }
}


#[cfg(test)]
mod test {
    use std::thread;
    use std::sync::Arc;
    use super::RingBuffer;
    use std::sync::atomic::AtomicUsize;

    #[derive(Copy, Clone, PartialEq, Debug)]
    struct TestStruct {
        a: [u8; 36],
        b: i32,
    }
    impl Default for TestStruct {
        fn default() -> Self {
            Self {
                a: [0; 36],
                b: 0,
            }
        }
    }

    #[test]
    fn basics() {
        let mut ringbuffer = RingBuffer::<TestStruct,10>::new();

        // simple read/write
        let mut data: Vec<TestStruct> = vec![TestStruct::default(); 8];
        let mut result = Vec::new();

        assert_eq!(ringbuffer.n_write(&data), 8);
        assert_eq!(ringbuffer.n_read(&mut result), 8);
        assert_eq!(result, vec![TestStruct::default(); 8]);

        assert_eq!(ringbuffer.n_write(&data), 8);
        assert_eq!(ringbuffer.n_write(&data), 2);
        assert_eq!(ringbuffer.n_read(&mut result), 10);
        assert_eq!(ringbuffer.n_read(&mut result), 0);
    }

/*     #[test]
    fn multi_thread_eq_slow(){
        let arc_ringbuffer1 = Arc::new(RingBuffer::<10>::new());
        let arc_ringbuffer2 = Arc::clone(&arc_ringbuffer1);
        
        thread::spawn(move || {
            let mut data: Vec<AtomicUsize> = Vec::new();
            for i in 0..=7 {
                data.push(AtomicUsize::new(i));
            }
            loop {
                arc_ringbuffer1.n_write(&data);
                println!("write data thread");
                thread::sleep(std::time::Duration::from_millis(100));
            }
        });
        thread::spawn(move || {
            let mut output = vec![100; 8];
            loop {
                arc_ringbuffer2.n_read(&mut output);
                println!("read data thread: {:?}", output);
                thread::sleep(std::time::Duration::from_millis(100));
            }
        });
        thread::sleep(std::time::Duration::from_millis(10000));
    }

    #[test]
    fn multi_thread_write_fast(){
        let arc_ringbuffer1 = Arc::new(RingBuffer::<10>::new());
        let arc_ringbuffer2 = Arc::clone(&arc_ringbuffer1);
        
        thread::spawn(move || {
            let mut data: Vec<AtomicUsize> = Vec::new();
            for i in 0..=7 {
                data.push(AtomicUsize::new(i));
            }
            loop {
                arc_ringbuffer1.n_write(&data);
                println!("write data thread");
                thread::sleep(std::time::Duration::from_millis(50));
            }
        });
        thread::spawn(move || {
            let mut output = vec![100; 8];
            loop {
                arc_ringbuffer2.n_read(&mut output);
                println!("read data thread: {:?}", output);
                thread::sleep(std::time::Duration::from_millis(100));
            }
        });
        thread::sleep(std::time::Duration::from_millis(10000));
    }

    #[test]
    fn multi_thread_read_fast(){
        let arc_ringbuffer1 = Arc::new(RingBuffer::<10>::new());
        let arc_ringbuffer2 = Arc::clone(&arc_ringbuffer1);
        
        thread::spawn(move || {
            let mut data: Vec<AtomicUsize> = Vec::new();
            for i in 0..=7 {
                data.push(AtomicUsize::new(i));
            }
            loop {
                arc_ringbuffer1.n_write(&data);
                println!("write data thread");
                thread::sleep(std::time::Duration::from_millis(100));
            }
        });
        thread::spawn(move || {
            let mut output = vec![100; 8];
            loop {
                arc_ringbuffer2.n_read(&mut output);
                println!("read data thread: {:?}", output);
                thread::sleep(std::time::Duration::from_millis(50));
            }
        });
        thread::sleep(std::time::Duration::from_millis(10000));
    }
 */

    #[test]
    fn multi_thread_general(){
        let arc_ringbuffer1 = Arc::new(RingBuffer::<TestStruct,20>::new());
        let arc_ringbuffer2 = Arc::clone(&arc_ringbuffer1);
        
        thread::spawn(move || {
            let mut data: Vec<TestStruct> = vec![TestStruct::default(); 8];
            loop {
                let n =arc_ringbuffer1.n_write(&data);
                println!("write data {}",n);
            }
        });
        thread::spawn(move || {
            let mut output = Vec::new();
            loop {
                let n = arc_ringbuffer2.n_read(&mut output);
                println!("read data {}: {:?}",n, output);
            }
        });
        thread::sleep(std::time::Duration::from_millis(10000));
    }
}
