/// thread safe ring buffer
/// the type T is here for example limited to usize


use std::sync::atomic::{
    AtomicUsize, 
    Ordering,
};
use std::cmp::min;


pub struct RingBuffer<const N: usize> {
    buffer: [AtomicUsize; N],
    head: AtomicUsize,
    tail: AtomicUsize,
    used_count: AtomicUsize,
}


impl<const N:usize> RingBuffer<N> 
{  
    pub fn new() -> Self {
        let mut b = [(); N].map(|_| AtomicUsize::new(0));

        Self {
            buffer: b,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            used_count: AtomicUsize::new(0),
        }
    }

    fn n_write(&self, data: &Vec<AtomicUsize>) -> usize {
        if N - self.used_count.load(Ordering::Relaxed) == 0 {
            // println!("buffer full");
            return 0;
        }
        let write_count = min(data.len(),N - self.used_count.load(Ordering::Relaxed));
        let tail = self.tail.load(Ordering::Relaxed);
        let new_tail;
        if write_count <= (N - tail) {
            for i in 0..write_count {
                self.buffer[tail + i].store(data[i].load(Ordering::Relaxed), Ordering::Relaxed);
            }

            new_tail = tail.wrapping_add(write_count);
        } else {
            new_tail = write_count - (N - tail);

            for i in 0..(N - tail) {
                self.buffer[tail + i].store(data[i].load(Ordering::Relaxed), Ordering::Relaxed);               
            }
            for i in 0..new_tail {
                self.buffer[i].store(data[(N - tail) + i].load(Ordering::Relaxed), Ordering::Relaxed);
            }
        }

        self.used_count.fetch_add(write_count, Ordering::Release);
        self.tail.store(new_tail, Ordering::Release);
        write_count
    }
    fn n_read(&self,data: &mut Vec<usize>) -> usize {
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
                data.push(self.buffer[i].load(Ordering::Relaxed));
            }
            new_head = head.wrapping_add(read_count);
        } else {
            new_head = read_count - (N - head);
            for i in head..N {
                data.push(self.buffer[i].load(Ordering::Relaxed));
            }
            for i in 0..new_head {
                data.push(self.buffer[i].load(Ordering::Relaxed));
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

    #[test]
    fn basics() {
        let mut ringbuffer = RingBuffer::<10>::new();

        // simple read/write
        let mut data: Vec<AtomicUsize> = Vec::new();
        for i in 0..8 {
            data.push(AtomicUsize::new(1));
        }
        let mut result = vec![1; 8];

        assert_eq!(ringbuffer.n_write(&data), 8);
        assert_eq!(ringbuffer.n_read(&mut result), 8);
        assert_eq!(result, vec![1; 8]);

        assert_eq!(ringbuffer.n_write(&data), 8);
        assert_eq!(ringbuffer.n_write(&data), 2);
        assert_eq!(ringbuffer.n_read(&mut result), 10);
        assert_eq!(ringbuffer.n_read(&mut result), 0);
    }

    #[test]
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

    #[test]
    fn multi_thread_general(){
        let arc_ringbuffer1 = Arc::new(RingBuffer::<100>::new());
        let arc_ringbuffer2 = Arc::clone(&arc_ringbuffer1);
        
        thread::spawn(move || {
            let mut data: Vec<AtomicUsize> = Vec::new();
            for i in 0..10 {
                data.push(AtomicUsize::new(i));
            }
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
