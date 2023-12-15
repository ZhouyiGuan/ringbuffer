/// generic ring buffer
/// the type T must implement Copy trait


use std::sync::atomic::{AtomicUsize, Ordering};
use std::cmp::min;


pub struct RingBuffer<T,const N: usize> {
    buffer: [T; N],
    head: AtomicUsize,
    tail: AtomicUsize,
    used_count: AtomicUsize,
}

impl<T, const N:usize> RingBuffer<T,N> 
where
    T: Copy + Default
{  
    pub fn new() -> Self {
        Self {
            buffer: [T::default(); N],
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            used_count: AtomicUsize::new(0),
        }
    }

    fn n_write(&mut self, data: &Vec<T>) -> usize {
        if N - self.used_count.load(Ordering::Relaxed) == 0 {
            println!("buffer full");
            return 0;
        }
        let write_count = min(data.len(),N - self.used_count.load(Ordering::Relaxed));

        let tail = self.tail.load(Ordering::Relaxed);
        let new_tail;
        if write_count <= (N - tail) {
            self.buffer[tail..tail + write_count].copy_from_slice(&data[..write_count]);

            new_tail = tail.wrapping_add(write_count);
        } else {
            new_tail = write_count - (N - tail);
            self.buffer[tail..].copy_from_slice(&data[..(N - tail)]);
            self.buffer[..new_tail].copy_from_slice(&data[(N - tail)..]);
        }

        self.used_count.fetch_add(write_count, Ordering::Release);
        self.tail.store(new_tail, Ordering::Release);
        write_count
    }
    fn n_read(&mut self,data: &mut Vec<T>) -> usize {
        data.clear();
        let read_count = self.used_count.load(Ordering::Relaxed);
        if read_count == 0 {
            println!("buffer empty");
            return 0;
        }

        let head = self.head.load(Ordering::Relaxed);
        let new_head;
        if read_count <= (N - head) {
            data.extend_from_slice(&self.buffer[head..head + read_count]);
            new_head = head.wrapping_add(read_count);
        } else {
            new_head = read_count - (N - head);
            data.extend_from_slice(&self.buffer[head..]);
            data.extend_from_slice(&self.buffer[..new_head]);
        }

        self.used_count.fetch_sub(read_count, Ordering::Release);
        self.head.store(new_head, Ordering::Release);
        read_count
    }
}


#[cfg(test)]
mod test {
    use super::RingBuffer;

    #[test]
    fn basics() {
        let mut ringbuffer = RingBuffer::<i32, 10>::new();

        // simple read/write
        let data = vec![0; 8];
        let mut result = vec![1; 8];

        assert_eq!(ringbuffer.n_write(&data), 8);
        assert_eq!(ringbuffer.n_read(&mut result), 8);

        assert_eq!(ringbuffer.n_write(&data), 8);
        assert_eq!(ringbuffer.n_write(&data), 2);
        assert_eq!(ringbuffer.n_read(&mut result), 10);
        assert_eq!(ringbuffer.n_read(&mut result), 0);
    }

    #[derive(Clone,Copy,PartialEq,Debug)]
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
    fn basics_with_struct() {
        let mut ringbuffer = RingBuffer::<TestStruct, 10>::new();
        let s1: TestStruct = TestStruct{a: [1; 36], b: 0};
        let s2: TestStruct = TestStruct{a: [2; 36], b: 0};

        let data = vec![s1; 8];
        let mut result = vec![s2; 8];

        assert_eq!(ringbuffer.n_write(&data), 8);
        assert_eq!(ringbuffer.n_read(&mut result), 8);
        assert_eq!(result, vec![s1; 8]);

        assert_eq!(ringbuffer.n_write(&data), 8);
        assert_eq!(ringbuffer.n_write(&data), 2);
        assert_eq!(ringbuffer.n_read(&mut result), 10);
        assert_eq!(result, vec![s1; 10]);

        assert_eq!(ringbuffer.n_read(&mut result), 0);
    }   

}
