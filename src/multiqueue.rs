use std::fmt::Debug;
use std::option::*;
use std::sync::*;
use std::thread::JoinHandle;

struct MQItem<T> {
    data: T,
    next: Arc<RwLock<Option<MQItem<T>>>>,
}
#[derive(Clone)]
pub struct MultiQueue<T> {
    head: Arc<RwLock<Option<MQItem<T>>>>,
}
impl<T> MQItem<T> {
    fn new(data: T) -> MQItem<T> {
        MQItem {
            data,
            next: Arc::new(RwLock::new(None)),
        }
    }
}
impl<T> MQItem<T> {
    fn push(&self, item: T) {
        push_helper(&self.next, item);
    }
}

// MultiQueue.push and MQItem.push are the same
fn push_helper<T>(this: &Arc<RwLock<Option<MQItem<T>>>>, item: T) {
    let mut n = this.write().unwrap();
    if n.is_some() {
        n.as_ref().unwrap().push(item)
    } else {
        *n = Some(MQItem::new(item));
    }
}

impl<T> MultiQueue<T>
where
    T: Clone + Debug + Sync + Send + 'static,
{
    pub fn new() -> MultiQueue<T> {
        MultiQueue {
            head: Arc::new(RwLock::new(None)),
        }
    }
    pub fn pull(&mut self) -> Option<T> {
        let o = self
            .head
            .read()
            .unwrap()
            .as_ref()
            .map(|i| (i.next.clone(), i.data.clone()));
        o.map(|i| {
            self.head = i.0;
            i.1
        })
    }
    pub fn push(&mut self, item: T) {
        push_helper(&self.head, item);
    }
    pub fn clone(&self) -> Self {
        MultiQueue {
            head: self.head.clone(),
        }
    }
    pub fn log(mut self) -> JoinHandle<()> {
        std::thread::spawn(move || loop {
            println!("{:?}", self.pull())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let mut q: MultiQueue<&str> = MultiQueue::new();
        q.push("one");
        // let mut q = q.clone();
        q.push("two");
        q.push("three");
        assert_eq!("two", q.pull().unwrap());
        assert_eq!(std::option::Option::None, q.pull());
    }
}
